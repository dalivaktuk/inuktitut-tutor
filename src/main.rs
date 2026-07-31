mod app;
mod layout;
mod lessons;
mod save;
mod ui;

use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};

use app::App;

fn main() -> std::io::Result<()> {
    // ratatui::init() enables raw mode + the alternate screen and installs a
    // panic hook that restores the terminal before the default hook runs, so
    // a panic never leaves the shell in a garbled state.
    let mut terminal = ratatui::init();
    let result = run(&mut terminal);
    ratatui::restore();

    let app = result?;
    if app.save_and_quit {
        if let Err(e) = save::save(&app) {
            eprintln!("failed to save progress: {e}");
        }
    }
    Ok(())
}

fn run(terminal: &mut ratatui::DefaultTerminal) -> std::io::Result<App> {
    let mut app = App::new();
    if let Some(data) = save::load(app.lessons.len()) {
        app.apply_save(data);
    }

    while !app.quit {
        terminal.draw(|f| ui::draw(f, &app))?;

        // Poll with a short timeout so error flashes and the completion
        // toast expire on screen even without new keystrokes.
        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => handle_key(&mut app, key),
                _ => {}
            }
        }
        app.tick();
    }

    Ok(app)
}

fn handle_key(app: &mut App, key: crossterm::event::KeyEvent) {
    let is_ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    match key.code {
        KeyCode::Char('s') if is_ctrl => app.request_save_and_quit(),
        KeyCode::Char('q') | KeyCode::Esc => app.quit = true,
        KeyCode::Char('[') | KeyCode::Left => app.prev_step(),
        KeyCode::Char(']') | KeyCode::Right => app.next_step(),
        KeyCode::Char('r') => app.restart_step(),
        KeyCode::Char('h') => app.show_key_labels = !app.show_key_labels,
        _ => app.handle_key(key),
    }
}
