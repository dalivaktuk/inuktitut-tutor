//! Rendering. Three panels top to bottom: header, on-screen keyboard,
//! exercise strip, footer -- drawn fresh every frame from `App` state.

use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::{App, CellStatus, ToastKind};
use crate::layout::{locate_glyph, KEYS};

const ACCENT: Color = Color::Cyan;
const GOOD: Color = Color::Green;
const BAD: Color = Color::Red;
const DIM: Color = Color::DarkGray;
const SHIFT_COLOR: Color = Color::Yellow;

const MIN_WIDTH: u16 = 66;
const HEADER_HEIGHT: u16 = 4;
const KEYBOARD_HEIGHT: u16 = 19;
const FOOTER_HEIGHT: u16 = 5;
/// Cell width in the exercise strip: key label / glyph plus a 1-col border on
/// each side.
const CELL_W: u16 = 5;

/// How many rows the exercise strip needs to wrap `num_cells` cells into,
/// given the width available *inside* its border.
fn exercise_row_count(num_cells: usize, inner_width: u16) -> usize {
    let per_row = (inner_width / CELL_W).max(1) as usize;
    num_cells.div_ceil(per_row).max(1)
}

pub fn draw(f: &mut Frame, app: &App) {
    let area = f.area();

    if app.course_complete {
        let required_height = complete_screen_height(app);
        if area.width < MIN_WIDTH || area.height < required_height {
            draw_too_small(f, area, required_height);
            return;
        }
        draw_course_complete(f, area, app);
        return;
    }

    let exercise_inner_width = area.width.saturating_sub(2);
    let exercise_rows = exercise_row_count(app.current_line().len(), exercise_inner_width);
    let exercise_height = exercise_rows as u16 * 3 + 2;
    let required_height = HEADER_HEIGHT + KEYBOARD_HEIGHT + exercise_height + FOOTER_HEIGHT;

    if area.width < MIN_WIDTH || area.height < required_height {
        draw_too_small(f, area, required_height);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(HEADER_HEIGHT),
            Constraint::Length(KEYBOARD_HEIGHT),
            Constraint::Length(exercise_height),
            Constraint::Min(FOOTER_HEIGHT),
        ])
        .split(area);

    draw_header(f, chunks[0], app);
    draw_keyboard(f, chunks[1], app);
    draw_exercise(f, chunks[2], app);
    draw_footer(f, chunks[3], app);
}

fn target_glyph(app: &App) -> char {
    app.current_cell().glyph
}

fn draw_too_small(f: &mut Frame, area: Rect, required_height: u16) {
    let msg = Paragraph::new(format!(
        "terminal too small — resize (need at least {MIN_WIDTH}x{required_height}, have {}x{})",
        area.width, area.height
    ))
    .alignment(Alignment::Center)
    .wrap(Wrap { trim: true });
    f.render_widget(msg, area);
}

// ---------------------------------------------------------------- header ---

fn draw_header(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default().borders(Borders::ALL).border_style(Style::default().fg(DIM));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let stats = app.stats();
    let lesson = app.lesson();

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(inner);

    let title_width = rows[0].width as usize;
    let stats_text = format!(
        "wpm {:>5.1}  ·  acc {:>5.1}%  ·  done {:>5.1}%",
        stats.wpm, stats.accuracy, stats.progress * 100.0
    );
    let title = format!("{}  [{}/{}]", lesson.title, app.step + 1, app.lessons.len());
    let pad = title_width.saturating_sub(title.len() + stats_text.len()).max(1);
    let line1 = Line::from(vec![
        Span::styled(title, Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" ".repeat(pad)),
        Span::styled(stats_text, Style::default().fg(ACCENT)),
    ]);
    f.render_widget(Paragraph::new(line1), rows[0]);

    let line2 = Line::from(Span::styled(lesson.hint, Style::default().fg(DIM)));
    f.render_widget(Paragraph::new(line2), rows[1]);
}

/// Key cell width in the keyboard panel: QWERTY label / glyph plus a 1-col
/// border on each side.
const KB_CELL_W: u16 = 5;
/// Leading stagger offset (in columns) for each physical row, mimicking a
/// real QWERTY's rightward creep from the number row down to home row.
const NUMBER_ROW_OFFSET: u16 = 0;
const TOP_ROW_OFFSET: u16 = 2;
const HOME_ROW_OFFSET: u16 = 4;
const BOTTOM_ROW_OFFSET: u16 = 6;
const SPACEBAR_OFFSET: u16 = 8;
const SPACEBAR_W: u16 = 30;

/// The widest row (top row: 11 keys plus its stagger offset) sets the width
/// of the whole keyboard block, so it can be centered as a unit within the
/// panel instead of sitting flush against the left border.
fn keyboard_content_width() -> u16 {
    [
        NUMBER_ROW_OFFSET + 10 * KB_CELL_W,
        TOP_ROW_OFFSET + 11 * KB_CELL_W,
        HOME_ROW_OFFSET + 9 * KB_CELL_W,
        BOTTOM_ROW_OFFSET + 10 * KB_CELL_W,
        SPACEBAR_OFFSET + SPACEBAR_W,
    ]
    .into_iter()
    .max()
    .unwrap()
}

fn draw_keyboard(f: &mut Frame, area: Rect, app: &App) {
    let target = target_glyph(app);
    let loc = locate_glyph(target);
    // The whole board flips to its shift layer together, the way a real
    // keyboard's printed alternate characters read once you hold Shift --
    // not two glyphs stacked in every cell at once.
    let shift_active = loc.map(|l| l.shifted).unwrap_or(false);

    let title = if shift_active {
        " keyboard — ca(ike)  ⇧ shift "
    } else {
        " keyboard — ca(ike) "
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(DIM))
        .title(Span::styled(
            title,
            if shift_active {
                Style::default().fg(SHIFT_COLOR).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(DIM)
            },
        ));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // number row
            Constraint::Length(3), // top row
            Constraint::Length(3), // home row
            Constraint::Length(3), // bottom row
            Constraint::Length(3), // space bar
            Constraint::Min(1),    // legend
        ])
        .split(inner);

    // Center the key grid as a single block within the panel; the legend
    // stays full-width since it's a short text line, not a grid.
    let content_w = keyboard_content_width().min(inner.width);
    let pad = (inner.width - content_w) / 2;
    let centered = |rect: Rect| Rect {
        x: rect.x + pad,
        width: content_w,
        ..rect
    };

    draw_key_row(f, centered(rows[0]), &KEYS[0..10], NUMBER_ROW_OFFSET, loc, shift_active);
    draw_key_row(f, centered(rows[1]), &KEYS[10..21], TOP_ROW_OFFSET, loc, shift_active);
    draw_key_row(f, centered(rows[2]), &KEYS[21..30], HOME_ROW_OFFSET, loc, shift_active);
    draw_key_row(f, centered(rows[3]), &KEYS[30..40], BOTTOM_ROW_OFFSET, loc, shift_active);
    draw_spacebar(f, centered(rows[4]), target == ' ');
    draw_legend(f, rows[5], app, loc);
}

fn draw_key_row(
    f: &mut Frame,
    area: Rect,
    keys: &[crate::layout::KeyDef],
    offset: u16,
    loc: Option<crate::layout::GlyphLocation>,
    shift_active: bool,
) {
    let mut constraints = vec![Constraint::Length(offset)];
    constraints.extend(keys.iter().map(|_| Constraint::Length(KB_CELL_W)));
    constraints.push(Constraint::Min(0));
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(area);

    for (i, &(key, base, shift, _finger, is_home)) in keys.iter().enumerate() {
        let rect = cols[i + 1];
        let is_current = loc.map(|l| l.key == key).unwrap_or(false);
        let is_bump = key == 'f' || key == 'j';

        let mut border_style = if is_current {
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
        } else if is_bump {
            Style::default().fg(SHIFT_COLOR)
        } else if is_home {
            Style::default().fg(Color::Blue)
        } else {
            Style::default().fg(DIM)
        };
        if is_current {
            border_style = border_style.add_modifier(Modifier::REVERSED);
        }

        let border_type = if is_current {
            BorderType::Thick
        } else {
            BorderType::Plain
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(border_type)
            .border_style(border_style)
            .title(Span::styled(key.to_string(), Style::default().fg(DIM)));
        let content_area = block.inner(rect);
        f.render_widget(block, rect);

        // If Shift is "held" (the current target needs it), every key shows
        // its shifted glyph instead of its base one -- a key with no shift
        // glyph just keeps showing its base, same as a real keyboard.
        let displayed = if shift_active {
            shift.unwrap_or(base)
        } else {
            base
        };
        let glyph_style = if is_current {
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        f.render_widget(
            Paragraph::new(displayed.to_string())
                .alignment(Alignment::Center)
                .style(glyph_style),
            content_area,
        );
    }
}

fn draw_spacebar(f: &mut Frame, area: Rect, is_current: bool) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(SPACEBAR_OFFSET),
            Constraint::Length(SPACEBAR_W),
            Constraint::Min(0),
        ])
        .split(area);

    let style = if is_current {
        Style::default().fg(ACCENT).add_modifier(Modifier::BOLD | Modifier::REVERSED)
    } else {
        Style::default().fg(DIM)
    };
    let border_type = if is_current { BorderType::Thick } else { BorderType::Plain };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(border_type)
        .border_style(style)
        .title(Span::styled("space", Style::default().fg(DIM)));
    let inner = block.inner(cols[1]);
    f.render_widget(block, cols[1]);
    f.render_widget(Paragraph::new("␣").alignment(Alignment::Center).style(style), inner);
}

fn draw_legend(f: &mut Frame, area: Rect, app: &App, loc: Option<crate::layout::GlyphLocation>) {
    let finger_hint = match loc {
        Some(l) => {
            if l.shifted {
                format!("Shift + {}", l.finger.label())
            } else {
                l.finger.label().to_string()
            }
        }
        None => "thumb (space bar)".to_string(),
    };

    let mut spans = vec![
        Span::styled("▮ next key", Style::default().fg(ACCENT)),
        Span::raw("   "),
        Span::styled("▪ home row", Style::default().fg(Color::Blue)),
        Span::raw("   "),
        Span::styled("F/J", Style::default().fg(SHIFT_COLOR)),
        Span::raw(" bumps"),
        Span::raw("      →  press: "),
    ];
    if let Some(l) = loc {
        spans.push(Span::styled(
            l.key.to_string(),
            Style::default().add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw("  "));
    }
    spans.push(Span::styled(
        finger_hint,
        Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
    ));
    let _ = app;
    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

// --------------------------------------------------------- exercise strip ---

fn draw_exercise(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(DIM))
        .title(" type this line ");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let line = app.current_line();
    let per_row = (inner.width / CELL_W).max(1) as usize;
    let row_count = exercise_row_count(line.len(), inner.width);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Length(3); row_count])
        .split(inner);

    for (row_idx, chunk) in line.chunks(per_row).enumerate() {
        // Center each row's cells within the panel rather than packing them
        // flush against the left border.
        let row_width = chunk.len() as u16 * CELL_W;
        let pad = inner.width.saturating_sub(row_width) / 2;

        let mut constraints = vec![Constraint::Length(pad)];
        constraints.extend(chunk.iter().map(|_| Constraint::Length(CELL_W)));
        constraints.push(Constraint::Min(0));
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(rows[row_idx]);

        for (i, cell) in chunk.iter().enumerate() {
            let idx = row_idx * per_row + i;
            let status = app.cell_status.get(idx).copied().unwrap_or(CellStatus::Pending);
            let is_current = idx == app.cell_idx;
            draw_exercise_cell(f, cols[i + 1], cell, status, is_current, app.show_key_labels);
        }
    }
}

fn draw_exercise_cell(
    f: &mut Frame,
    rect: Rect,
    cell: &crate::lessons::Cell,
    status: CellStatus,
    is_current: bool,
    show_key_labels: bool,
) {
    let is_space = cell.glyph == ' ';

    let (border_style, border_type) = match (status, is_current) {
        (CellStatus::Error, _) => (
            Style::default().fg(BAD).add_modifier(Modifier::BOLD | Modifier::REVERSED),
            BorderType::Thick,
        ),
        (CellStatus::Done, _) => (Style::default().fg(GOOD), BorderType::Plain),
        (_, true) => (
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            BorderType::Thick,
        ),
        _ if is_space => (Style::default().fg(DIM).add_modifier(Modifier::DIM), BorderType::Rounded),
        _ => (Style::default().fg(DIM), BorderType::Plain),
    };

    let label = if show_key_labels {
        cell.key_label.to_string()
    } else {
        String::new()
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(border_type)
        .border_style(border_style)
        .title(Span::styled(label, Style::default().fg(DIM)));
    let inner = block.inner(rect);
    f.render_widget(block, rect);

    let glyph_str = if is_space { "␣".to_string() } else { cell.glyph.to_string() };
    let text_style = match status {
        CellStatus::Done => Style::default().fg(GOOD),
        CellStatus::Error => Style::default().fg(BAD).add_modifier(Modifier::BOLD),
        CellStatus::Pending if is_current => Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        _ => Style::default(),
    };
    f.render_widget(
        Paragraph::new(glyph_str).alignment(Alignment::Center).style(text_style),
        inner,
    );
}

// -------------------------------------------------------------- footer ---

fn draw_footer(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default().borders(Borders::ALL).border_style(Style::default().fg(DIM));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1), Constraint::Length(1)])
        .split(inner);

    let mut dots = Vec::with_capacity(app.lessons.len() * 2);
    for (i, &done) in app.lesson_done.iter().enumerate() {
        let is_current = i == app.step;
        let (glyph, style) = if is_current {
            ("●", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD))
        } else if done {
            ("●", Style::default().fg(GOOD))
        } else {
            ("○", Style::default().fg(DIM))
        };
        dots.push(Span::styled(glyph, style));
        dots.push(Span::raw(" "));
    }
    f.render_widget(Paragraph::new(Line::from(dots)), rows[0]);

    // Kept short enough to fit the MIN_WIDTH floor (64 usable columns) without
    // truncating; .wrap() is a belt-and-suspenders fallback if it ever grows.
    let hints = "[ ] step · r restart · h labels · ctrl+s save & quit · q quit";
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(hints, Style::default().fg(DIM))))
            .wrap(Wrap { trim: true }),
        rows[1],
    );

    let status_line = if let Some((msg, kind, _)) = &app.toast {
        let color = match kind {
            ToastKind::Success => GOOD,
            ToastKind::Retry => SHIFT_COLOR,
        };
        Line::from(Span::styled(
            msg.clone(),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ))
    } else if app.layout_warning {
        Line::from(Span::styled(
            "layout not active? run: setxkbmap ca ike",
            Style::default().fg(BAD).add_modifier(Modifier::BOLD),
        ))
    } else {
        Line::from("")
    };
    f.render_widget(
        Paragraph::new(status_line).wrap(Wrap { trim: true }),
        rows[2],
    );
}

// -------------------------------------------------------- course complete ---

fn complete_screen_height(app: &App) -> u16 {
    // border(2) + title + blank + subtitle + blank + one line per step + blank + controls
    2 + 1 + 1 + 1 + 1 + app.lessons.len() as u16 + 1 + 1
}

/// Shown instead of the normal three panels once the last step has been
/// typed correctly: a recap of every step's last completion stats, in place
/// of leaving the learner staring at a frozen exercise strip.
fn draw_course_complete(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ACCENT))
        .title(" course complete ");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut lines = vec![
        Line::from(Span::styled(
            "COURSE COMPLETE",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Center),
        Line::from(""),
        Line::from(Span::styled(
            "You've worked through all steps of the ca(ike) layout.",
            Style::default().fg(DIM),
        ))
        .alignment(Alignment::Center),
        Line::from(""),
    ];

    for (i, lesson) in app.lessons.iter().enumerate() {
        let stats_text = match app.best_stats[i] {
            Some((wpm, acc)) => format!("{wpm:>5.1} wpm · {acc:>5.1}% acc"),
            None => "  —  not completed".to_string(),
        };
        let number = format!("{:>2}", i + 1);
        let title = format!("{:<32}", lesson.title);
        lines.push(Line::from(vec![
            Span::styled(format!("{number}  "), Style::default().fg(DIM)),
            Span::raw(title),
            Span::styled(stats_text, Style::default().fg(GOOD)),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(
        // Kept short enough to fit the MIN_WIDTH floor without truncating;
        // see the matching comment on the footer hints line.
        Line::from(Span::styled(
            "r restart · ctrl+s save & quit · [ ] browse steps · q quit",
            Style::default().fg(DIM),
        ))
        .alignment(Alignment::Center),
    );

    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), inner);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;
    use crossterm::event::{KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    fn press(app: &mut App, c: char) {
        app.handle_key(KeyEvent {
            code: crossterm::event::KeyCode::Char(c),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        });
    }

    fn buffer_text(terminal: &Terminal<TestBackend>) -> String {
        terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect()
    }

    #[test]
    fn normal_screen_renders_without_panicking() {
        let app = App::new();
        let mut terminal = Terminal::new(TestBackend::new(100, 45)).unwrap();
        terminal.draw(|f| draw(f, &app)).unwrap();
    }

    #[test]
    fn too_small_terminal_shows_resize_message_without_panicking() {
        let app = App::new();
        let mut terminal = Terminal::new(TestBackend::new(40, 15)).unwrap();
        terminal.draw(|f| draw(f, &app)).unwrap();
        assert!(buffer_text(&terminal).contains("too small"));
    }

    /// The save-and-quit binding is only useful if it's actually visible.
    /// Regression test for a truncated (not wrapped) footer hint line at
    /// common terminal widths -- everything from the declared `MIN_WIDTH`
    /// floor up must still show the full hint.
    #[test]
    fn footer_hint_not_truncated_near_min_width() {
        let app = App::new();
        for width in [MIN_WIDTH, 80] {
            let mut terminal = Terminal::new(TestBackend::new(width, 45)).unwrap();
            terminal.draw(|f| draw(f, &app)).unwrap();
            let text = buffer_text(&terminal);
            assert!(
                text.contains("ctrl+s save & quit"),
                "footer hint truncated at width {width}: {text}"
            );
        }
    }

    #[test]
    fn course_complete_controls_not_truncated_near_min_width() {
        let mut app = App::new();
        for _ in 0..100_000 {
            if app.course_complete {
                break;
            }
            let target = app.current_cell().glyph;
            press(&mut app, target);
        }
        assert!(app.course_complete);

        for width in [MIN_WIDTH, 80] {
            let mut terminal = Terminal::new(TestBackend::new(width, 45)).unwrap();
            terminal.draw(|f| draw(f, &app)).unwrap();
            let text = buffer_text(&terminal);
            assert!(
                text.contains("ctrl+s save & quit"),
                "course-complete controls truncated at width {width}: {text}"
            );
        }
    }

    #[test]
    fn course_complete_screen_renders_and_lists_every_step() {
        let mut app = App::new();
        for _ in 0..100_000 {
            if app.course_complete {
                break;
            }
            let target = app.current_cell().glyph;
            press(&mut app, target);
        }
        assert!(app.course_complete);

        let mut terminal = Terminal::new(TestBackend::new(100, 45)).unwrap();
        terminal.draw(|f| draw(f, &app)).unwrap();

        let text = buffer_text(&terminal);
        assert!(text.contains("COURSE COMPLETE"));
        assert!(text.contains("save & quit"));
        for lesson in &app.lessons {
            assert!(
                text.contains(lesson.title),
                "recap screen missing {}",
                lesson.title
            );
        }
    }

    #[test]
    fn course_complete_screen_handles_small_terminal_without_panicking() {
        let mut app = App::new();
        for _ in 0..100_000 {
            if app.course_complete {
                break;
            }
            let target = app.current_cell().glyph;
            press(&mut app, target);
        }
        let mut terminal = Terminal::new(TestBackend::new(40, 15)).unwrap();
        terminal.draw(|f| draw(f, &app)).unwrap();
    }
}
