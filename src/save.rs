//! Minimal, dependency-free persistence for progress: which steps are done,
//! each step's last completion stats, and which step to resume on next
//! launch. Stored as plain text under each platform's idiomatic per-user
//! data directory -- three small fields don't need a serialization crate
//! (or a `dirs`-style dependency) to get right.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::app::App;

pub struct SaveData {
    pub step: usize,
    pub lesson_done: Vec<bool>,
    pub best_stats: Vec<Option<(f64, f64)>>,
}

fn save_path() -> Option<PathBuf> {
    Some(data_dir()?.join("inuktitut-tutor").join("progress.txt"))
}

/// The per-user application-data directory, platform-appropriate:
/// `%APPDATA%` on Windows, `~/Library/Application Support` on macOS, and
/// `$XDG_DATA_HOME` (falling back to `~/.local/share`) everywhere else.
#[cfg(target_os = "windows")]
fn data_dir() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(PathBuf::from)
}

#[cfg(target_os = "macos")]
fn data_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|home| PathBuf::from(home).join("Library/Application Support"))
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn data_dir() -> Option<PathBuf> {
    std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share")))
}

pub fn save(app: &App) -> io::Result<()> {
    match save_path() {
        Some(path) => save_to(&path, app),
        None => Ok(()),
    }
}

pub fn load(num_lessons: usize) -> Option<SaveData> {
    load_from(&save_path()?, num_lessons)
}

fn save_to(path: &Path, app: &App) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let done: String = app
        .lesson_done
        .iter()
        .map(|&d| if d { '1' } else { '0' })
        .collect();
    let best: String = app
        .best_stats
        .iter()
        .map(|entry| match entry {
            Some((wpm, acc)) => format!("{wpm:.2}:{acc:.2}"),
            None => "-".to_string(),
        })
        .collect::<Vec<_>>()
        .join(",");

    fs::write(path, format!("step={}\ndone={done}\nbest={best}\n", app.step))
}

fn load_from(path: &Path, num_lessons: usize) -> Option<SaveData> {
    let text = fs::read_to_string(path).ok()?;

    let mut step = 0usize;
    let mut lesson_done = vec![false; num_lessons];
    let mut best_stats: Vec<Option<(f64, f64)>> = vec![None; num_lessons];

    for line in text.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key {
            "step" => step = value.parse().unwrap_or(0),
            "done" => {
                for (i, ch) in value.chars().enumerate().take(num_lessons) {
                    lesson_done[i] = ch == '1';
                }
            }
            "best" => {
                for (i, entry) in value.split(',').enumerate().take(num_lessons) {
                    if let Some((w, a)) = entry.split_once(':') {
                        if let (Ok(w), Ok(a)) = (w.parse(), a.parse()) {
                            best_stats[i] = Some((w, a));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    Some(SaveData {
        step: step.min(num_lessons.saturating_sub(1)),
        lesson_done,
        best_stats,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    fn press(app: &mut App, c: char) {
        app.handle_key(KeyEvent {
            code: crossterm::event::KeyCode::Char(c),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        });
    }

    /// Complete step 1 for real (typing its actual glyph sequence, not
    /// synthetic data), save, and load back into a fresh `App` -- exercising
    /// both the `done=` and `best=` parse branches with real values instead
    /// of the all-default line this module started with.
    #[test]
    fn round_trip_preserves_completed_step_and_stats() {
        let mut app = App::new();
        let first_lesson_len = app.lesson().total_chars();

        // Finish step 1, then move into step 2 so `app.step` (the resume
        // point) differs from the completed step being checked.
        while app.step == 0 {
            let target = app.current_cell().glyph;
            press(&mut app, target);
        }
        assert!(app.lesson_done[0]);
        let (saved_wpm, saved_acc) = app.best_stats[0].expect("step 1 should have recorded stats");
        assert_eq!(app.step, 1);

        let path = std::env::temp_dir().join(format!(
            "inuktitut-tutor-test-{}-{}.txt",
            std::process::id(),
            first_lesson_len // cheap per-test uniqueness without extra deps
        ));
        save_to(&path, &app).expect("save should succeed");

        let loaded = load_from(&path, app.lessons.len()).expect("load should find the file");
        std::fs::remove_file(&path).ok();

        assert_eq!(loaded.step, 1);
        assert!(loaded.lesson_done[0], "step 1 should be marked done");
        assert!(!loaded.lesson_done[1], "step 2 shouldn't be marked done yet");
        let (loaded_wpm, loaded_acc) = loaded.best_stats[0].expect("step 1 stats should round-trip");
        assert!((loaded_wpm - saved_wpm).abs() < 0.01);
        assert!((loaded_acc - saved_acc).abs() < 0.01);
        assert!(loaded.best_stats[1].is_none());

        // And apply_save actually resumes on the loaded step with the loaded
        // flags, not just parses them.
        let mut fresh = App::new();
        fresh.apply_save(loaded);
        assert_eq!(fresh.step, 1);
        assert!(fresh.lesson_done[0]);
    }

    #[test]
    fn load_from_missing_file_returns_none() {
        let path = std::env::temp_dir().join("inuktitut-tutor-test-does-not-exist.txt");
        assert!(load_from(&path, 13).is_none());
    }
}
