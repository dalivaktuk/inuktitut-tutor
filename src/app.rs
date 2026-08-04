//! Application state machine: current step/line/cell, live stats, and input
//! handling. Rendering lives in `ui.rs`; layout/lesson data live in
//! `layout.rs` / `lessons.rs`.

use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

use crate::layout::locate_glyph;
use crate::lessons::{all_lessons, Cell, Lesson};
use crate::save::SaveData;

/// How long a wrong keystroke stays flashed red before reverting to pending.
const ERROR_FLASH: Duration = Duration::from_millis(200);
/// How long a toast (step-done or stage-retry) stays on screen.
const TOAST_DURATION: Duration = Duration::from_millis(2500);

/// Minimum live accuracy (0-100) required on a stage's final lesson to
/// auto-advance into the next stage. Deliberately lenient on speed, strict
/// on correctness: this app teaches an unfamiliar script, not a race.
const STAGE_GATE_MIN_ACCURACY: f64 = 90.0;
/// Minimum live wpm required alongside the accuracy floor. Low on purpose --
/// a sanity floor against "correct but agonizingly slow," not a speed test.
const STAGE_GATE_MIN_WPM: f64 = 5.0;

/// A group of lessons the learner must clear together before unlocking the
/// next group. Derived purely from step index -- nothing about this is
/// persisted, so lessons can be appended later with zero migration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stage {
    Home,
    Top,
    Bottom,
    Advanced,
}

impl Stage {
    fn label(self) -> &'static str {
        match self {
            Stage::Home => "Home Row",
            Stage::Top => "Top Row",
            Stage::Bottom => "Bottom Row",
            Stage::Advanced => "Advanced",
        }
    }
}

/// Keep in sync with `stage_start_step` below -- these are the same boundary
/// ranges expressed two ways (which stage a step is in / where that stage
/// begins).
fn stage_of(step: usize) -> Stage {
    match step {
        0..=5 => Stage::Home,   // STEP 1-6
        6 => Stage::Top,        // STEP 7
        7..=9 => Stage::Bottom, // STEP 8-10
        _ => Stage::Advanced,   // STEP 11-14
    }
}

/// The first step index of the stage `step` belongs to.
fn stage_start_step(step: usize) -> usize {
    match stage_of(step) {
        Stage::Home => 0,
        Stage::Top => 6,
        Stage::Bottom => 7,
        Stage::Advanced => 10,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    /// A lesson was completed and the stage's proficiency bar was cleared.
    Success,
    /// A stage's exit lesson was completed but proficiency wasn't met --
    /// bounced back to the start of the stage for another pass.
    Retry,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellStatus {
    Pending,
    Done,
    Error,
}

pub struct Stats {
    pub wpm: f64,
    pub accuracy: f64,
    pub progress: f64,
}

pub struct App {
    pub lessons: Vec<Lesson>,
    pub step: usize,
    pub line_idx: usize,
    pub cell_idx: usize,
    pub cell_status: Vec<CellStatus>,
    error_since: Option<Instant>,

    pub lesson_done: Vec<bool>,
    /// Wpm/accuracy recorded the last time each step was completed. `None`
    /// for a step never finished (e.g. skipped over with `]`). Shown on the
    /// course-complete recap screen and persisted across runs.
    pub best_stats: Vec<Option<(f64, f64)>>,
    pub show_key_labels: bool,
    pub quit: bool,
    /// Set by the "save & quit" keybinding; `main` checks this after the
    /// event loop exits to decide whether to persist progress to disk.
    pub save_and_quit: bool,
    /// Set once the very last cell of the very last step has been typed
    /// correctly. `cell_idx` is clamped back to the last valid index at that
    /// point (there is no "next" cell to point at), so further input is
    /// ignored until a restart or step change. The UI shows a dedicated
    /// recap screen instead of the normal panels while this is set.
    pub course_complete: bool,

    start_time: Option<Instant>,
    correct_keystrokes: u32,
    total_keystrokes: u32,
    /// Cells completed in lines *before* the current one, for this step.
    chars_done_prior_lines: usize,

    pub layout_warning: bool,
    pub toast: Option<(String, ToastKind, Instant)>,
}

impl App {
    pub fn new() -> Self {
        let lessons = all_lessons();
        let lesson_done = vec![false; lessons.len()];
        let best_stats = vec![None; lessons.len()];
        let mut app = App {
            lessons,
            step: 0,
            line_idx: 0,
            cell_idx: 0,
            cell_status: Vec::new(),
            error_since: None,
            lesson_done,
            best_stats,
            show_key_labels: true,
            quit: false,
            save_and_quit: false,
            course_complete: false,
            start_time: None,
            correct_keystrokes: 0,
            total_keystrokes: 0,
            chars_done_prior_lines: 0,
            layout_warning: false,
            toast: None,
        };
        app.reset_line_status();
        app
    }

    /// Restore progress loaded from disk: which steps are done, their last
    /// stats, and which step to resume on. Call right after `new()`, before
    /// the first draw.
    pub fn apply_save(&mut self, data: SaveData) {
        self.lesson_done = data.lesson_done;
        self.best_stats = data.best_stats;
        self.step = data.step.min(self.lessons.len() - 1);
        self.reset_step();
        self.toast = None;
    }

    /// Save-and-quit keybinding: persist progress on the next iteration of
    /// the event loop and stop.
    pub fn request_save_and_quit(&mut self) {
        self.save_and_quit = true;
        self.quit = true;
    }

    pub fn lesson(&self) -> &Lesson {
        &self.lessons[self.step]
    }

    pub fn current_line(&self) -> &[Cell] {
        &self.lesson().lines[self.line_idx]
    }

    pub fn current_cell(&self) -> Cell {
        self.current_line()[self.cell_idx]
    }

    fn reset_line_status(&mut self) {
        self.cell_status = vec![CellStatus::Pending; self.current_line().len()];
        self.cell_idx = 0;
        self.error_since = None;
    }

    /// Reset all progress and stats for the current step (used on entry,
    /// restart, and step navigation).
    fn reset_step(&mut self) {
        self.line_idx = 0;
        self.chars_done_prior_lines = 0;
        self.start_time = None;
        self.correct_keystrokes = 0;
        self.total_keystrokes = 0;
        self.layout_warning = false;
        self.course_complete = false;
        self.reset_line_status();
    }

    pub fn restart_step(&mut self) {
        self.reset_step();
        self.toast = None;
    }

    pub fn goto_step(&mut self, step: usize) {
        self.step = step.clamp(0, self.lessons.len() - 1);
        self.reset_step();
        self.toast = None;
    }

    pub fn next_step(&mut self) {
        if self.step + 1 < self.lessons.len() {
            self.goto_step(self.step + 1);
        }
    }

    pub fn prev_step(&mut self) {
        if self.step > 0 {
            self.goto_step(self.step - 1);
        }
    }

    /// Periodic tick, independent of key events: expire error flashes and the
    /// completion toast so the UI updates even without new input.
    pub fn tick(&mut self) {
        if let Some(since) = self.error_since {
            if since.elapsed() >= ERROR_FLASH {
                if let Some(status) = self.cell_status.get_mut(self.cell_idx) {
                    *status = CellStatus::Pending;
                }
                self.error_since = None;
            }
        }
        if let Some((_, _, at)) = &self.toast {
            if at.elapsed() >= TOAST_DURATION {
                self.toast = None;
            }
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        match key.code {
            KeyCode::Char(c) => self.handle_char(c),
            KeyCode::Backspace => self.handle_backspace(),
            _ => {}
        }
    }

    fn handle_backspace(&mut self) {
        if self.cell_idx > 0 {
            self.cell_idx -= 1;
            self.cell_status[self.cell_idx] = CellStatus::Pending;
            self.error_since = None;
        }
    }

    fn handle_char(&mut self, c: char) {
        if self.course_complete {
            return;
        }
        // Every real key on this layout produces either a syllabic, one of
        // the plain-ASCII punctuation marks it also emits (`,` `.` `?`), or a
        // literal space; anything else means the OS layout probably isn't
        // active.
        if c != ' ' && locate_glyph(c).is_none() {
            self.layout_warning = true;
            self.total_keystrokes += 1;
            self.mark_error();
            return;
        }
        self.layout_warning = false;

        if self.start_time.is_none() {
            self.start_time = Some(Instant::now());
        }

        let target = self.current_cell().glyph;
        self.total_keystrokes += 1;

        if c == target {
            self.correct_keystrokes += 1;
            self.cell_status[self.cell_idx] = CellStatus::Done;
            self.error_since = None;
            self.advance();
        } else {
            self.mark_error();
        }
    }

    fn mark_error(&mut self) {
        if let Some(status) = self.cell_status.get_mut(self.cell_idx) {
            *status = CellStatus::Error;
        }
        self.error_since = Some(Instant::now());
    }

    fn advance(&mut self) {
        self.cell_idx += 1;
        if self.cell_idx < self.current_line().len() {
            return;
        }
        // Line finished.
        self.chars_done_prior_lines += self.current_line().len();
        if self.line_idx + 1 < self.lesson().lines.len() {
            self.line_idx += 1;
            self.reset_line_status();
        } else {
            self.finish_lesson();
        }
    }

    fn finish_lesson(&mut self) {
        // At this point `cell_idx` has already run past the last valid index
        // of the last line (there is no next cell), so grab the WPM/accuracy
        // numbers -- which don't depend on `cell_idx` -- before touching
        // anything else.
        let stats = self.stats();
        self.lesson_done[self.step] = true;
        self.best_stats[self.step] = Some((stats.wpm, stats.accuracy));

        let has_next = self.step + 1 < self.lessons.len();
        let crossing_stage = has_next && stage_of(self.step) != stage_of(self.step + 1);
        let gate_met =
            stats.accuracy >= STAGE_GATE_MIN_ACCURACY && stats.wpm >= STAGE_GATE_MIN_WPM;

        if crossing_stage && !gate_met {
            // Finished the stage's exit lesson correctly, but not proficiently
            // enough to unlock the next stage: bounce back to the start of
            // this stage for another pass rather than pressing forward.
            // `goto_step` is the same call `]`/`[` make, so manual navigation
            // is never affected by this.
            let stage_name = stage_of(self.step).label();
            let restart_to = stage_start_step(self.step);
            self.goto_step(restart_to);
            self.toast = Some((
                format!(
                    "not quite yet — need {:.0}%/{:.0}wpm, got {:.0}%/{:.0} — retry {stage_name}",
                    STAGE_GATE_MIN_ACCURACY, STAGE_GATE_MIN_WPM, stats.accuracy, stats.wpm,
                ),
                ToastKind::Retry,
                Instant::now(),
            ));
            return;
        }

        self.toast = Some((
            format!(
                "step done — {:.0} wpm · {:.0}% acc",
                stats.wpm, stats.accuracy
            ),
            ToastKind::Success,
            Instant::now(),
        ));
        if has_next {
            let toast = self.toast.take();
            self.goto_step(self.step + 1);
            self.toast = toast;
        } else {
            // No next step to move to: clamp back onto the last cell (still
            // marked Done) instead of leaving an out-of-range index, and stop
            // accepting input. `r` restarts the step from here.
            self.cell_idx -= 1;
            self.course_complete = true;
        }
    }

    pub fn stats(&self) -> Stats {
        let total_chars = self.lesson().total_chars().max(1);
        let done_chars = if self.course_complete {
            total_chars
        } else {
            self.chars_done_prior_lines + self.cell_idx
        };
        let progress = done_chars as f64 / total_chars as f64;

        let accuracy = if self.total_keystrokes > 0 {
            self.correct_keystrokes as f64 / self.total_keystrokes as f64 * 100.0
        } else {
            100.0
        };

        let wpm = match self.start_time {
            Some(start) => {
                let minutes = start.elapsed().as_secs_f64() / 60.0;
                if minutes > 0.0 {
                    (self.correct_keystrokes as f64 / 5.0) / minutes
                } else {
                    0.0
                }
            }
            None => 0.0,
        };

        Stats {
            wpm,
            accuracy,
            progress,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Types the current lesson correctly, cell by cell, until the step
    /// index changes (advance or gate-bounce) or the course completes.
    fn type_lesson_correctly(app: &mut App) {
        let starting_step = app.step;
        while app.step == starting_step && !app.course_complete {
            let target = app.current_cell().glyph;
            app.handle_char(target);
        }
    }

    /// Types the current lesson with one guaranteed-wrong keystroke ('z' is
    /// never a valid target for any cell) before every correct one, tanking
    /// accuracy well under any reasonable gate threshold.
    fn type_lesson_with_errors(app: &mut App) {
        let starting_step = app.step;
        while app.step == starting_step && !app.course_complete {
            app.handle_char('z');
            let target = app.current_cell().glyph;
            app.handle_char(target);
        }
    }

    /// Types every cell of every line of every step correctly, in order, and
    /// checks the state machine never leaves `cell_idx` pointing past the end
    /// of the current line -- the bug class that produces an index-out-of-
    /// bounds panic on `current_cell()`/`current_line()` right after the last
    /// step's last keystroke.
    #[test]
    fn plays_through_entire_course_without_panicking() {
        let mut app = App::new();
        let last_step = app.lessons.len() - 1;

        for _ in 0..100_000 {
            if app.course_complete {
                break;
            }
            let target = app.current_cell().glyph;
            app.handle_char(target);
            assert!(
                app.cell_idx < app.current_line().len(),
                "cell_idx {} out of range for a line of length {}",
                app.cell_idx,
                app.current_line().len()
            );
        }

        assert!(app.course_complete, "did not reach the end of the course");
        assert_eq!(app.step, last_step);
        assert!(app.stats().progress >= 1.0 && app.stats().progress <= 1.0001);
    }

    #[test]
    fn restart_after_course_complete_accepts_input_again() {
        let mut app = App::new();
        for _ in 0..100_000 {
            if app.course_complete {
                break;
            }
            let target = app.current_cell().glyph;
            app.handle_char(target);
        }
        assert!(app.course_complete);

        app.restart_step();
        assert!(!app.course_complete);
        let target = app.current_cell().glyph;
        app.handle_char(target);
        assert_eq!(app.cell_idx, 1);
    }

    /// The Home stage (steps 0-5) gates entry into Top Row (step 6) on
    /// clearing STAGE_GATE_MIN_ACCURACY/WPM on its exit lesson (step 5).
    #[test]
    fn gate_blocks_low_proficiency_then_allows_high_proficiency() {
        let mut app = App::new();

        // Breeze through the first five Home-stage lessons cleanly.
        for _ in 0..5 {
            type_lesson_correctly(&mut app);
        }
        assert_eq!(app.step, 5, "should be on the Home Row Shift exit lesson");

        // Fail the exit lesson on purpose: accuracy well under the floor.
        type_lesson_with_errors(&mut app);
        assert_eq!(app.step, 0, "should bounce back to the start of Home stage");
        assert!(
            app.lesson_done[5],
            "finishing the lesson should still mark it done, even though the gate wasn't cleared"
        );
        assert!(!app.course_complete);

        // Clear the whole Home stage cleanly this time, including the exit
        // lesson -- should now unlock Top Row.
        for _ in 0..6 {
            type_lesson_correctly(&mut app);
        }
        assert_eq!(app.step, 6, "should advance into Top Row now the bar is cleared");
    }

    /// Manual `]`/`[` navigation (`next_step`/`prev_step`) must never be
    /// gated -- it's the user's escape hatch regardless of proficiency.
    #[test]
    fn manual_navigation_ignores_the_gate() {
        let mut app = App::new();
        for _ in 0..5 {
            type_lesson_correctly(&mut app);
        }
        type_lesson_with_errors(&mut app);
        assert_eq!(app.step, 0, "gate should have bounced back to Home stage start");

        app.next_step();
        assert_eq!(app.step, 1, "manual next_step must work regardless of the gate");
    }
}
