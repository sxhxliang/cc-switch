use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;

mod tui;
use tui::{
    app::{App, AppMode, Tab},
    ui::render_ui,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run
    let mut app = App::new()?;
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        terminal.draw(|f| render_ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match app.mode {
                    AppMode::Normal => handle_normal_mode(app, key.code)?,
                    AppMode::ProviderForm => handle_form_mode(app, key.code)?,
                    AppMode::Confirmation => handle_confirmation_mode(app, key.code)?,
                    AppMode::McpManagement => handle_mcp_mode(app, key.code)?,
                    AppMode::Help => {
                        if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
                            app.mode = AppMode::Normal;
                        }
                    }
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn handle_normal_mode(app: &mut App, key: KeyCode) -> Result<(), Box<dyn std::error::Error>> {
    match key {
        KeyCode::Char('q') => {
            app.should_quit = true;
        }
        KeyCode::Char('?') => {
            app.mode = AppMode::Help;
        }
        KeyCode::Tab => {
            app.next_tab();
        }
        KeyCode::BackTab => {
            app.previous_tab();
        }
        KeyCode::Char('1') => {
            app.current_tab = Tab::Claude;
            app.refresh_provider_list()?;
        }
        KeyCode::Char('2') => {
            app.current_tab = Tab::Codex;
            app.refresh_provider_list()?;
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.next_provider();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.previous_provider();
        }
        KeyCode::Enter => {
            app.switch_selected_provider()?;
        }
        KeyCode::Char('a') => {
            app.start_add_provider();
        }
        KeyCode::Char('e') => {
            app.start_edit_provider();
        }
        KeyCode::Char('d') => {
            app.start_delete_confirmation();
        }
        KeyCode::Char('m') => {
            app.enter_mcp_mode()?;
        }
        KeyCode::Char('r') => {
            app.refresh_provider_list()?;
        }
        _ => {}
    }
    Ok(())
}

fn handle_form_mode(app: &mut App, key: KeyCode) -> Result<(), Box<dyn std::error::Error>> {
    match key {
        KeyCode::Esc => {
            app.cancel_form();
        }
        KeyCode::Tab | KeyCode::Down => {
            app.next_form_field();
        }
        KeyCode::BackTab | KeyCode::Up => {
            app.previous_form_field();
        }
        KeyCode::Enter => {
            if app.is_form_valid() {
                app.submit_form()?;
            }
        }
        KeyCode::Char(c) => {
            app.input_char(c);
        }
        KeyCode::Backspace => {
            app.delete_char();
        }
        _ => {}
    }
    Ok(())
}

fn handle_confirmation_mode(app: &mut App, key: KeyCode) -> Result<(), Box<dyn std::error::Error>> {
    match key {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            app.confirm_delete()?;
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.cancel_confirmation();
        }
        _ => {}
    }
    Ok(())
}

fn handle_mcp_mode(app: &mut App, key: KeyCode) -> Result<(), Box<dyn std::error::Error>> {
    match key {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.exit_mcp_mode();
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.next_mcp_server();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.previous_mcp_server();
        }
        KeyCode::Char(' ') => {
            app.toggle_mcp_server()?;
        }
        KeyCode::Char('a') => {
            // TODO: Add MCP server form
        }
        KeyCode::Char('d') => {
            app.delete_mcp_server()?;
        }
        KeyCode::Char('i') => {
            app.import_mcp_from_live()?;
        }
        KeyCode::Char('s') => {
            app.sync_mcp_to_live()?;
        }
        _ => {}
    }
    Ok(())
}
