use super::app::{App, AppMode, FormField, Tab};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Tabs, Wrap},
    Frame,
};

pub fn render_ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header with tabs
            Constraint::Min(0),     // Main content
            Constraint::Length(3),  // Status bar
        ])
        .split(f.area());

    render_header(f, chunks[0], app);

    match app.mode {
        AppMode::Normal => render_provider_list(f, chunks[1], app),
        AppMode::ProviderForm => render_provider_form(f, chunks[1], app),
        AppMode::Confirmation => {
            render_provider_list(f, chunks[1], app);
            render_confirmation_dialog(f, app);
        }
        AppMode::McpManagement => render_mcp_list(f, chunks[1], app),
        AppMode::Help => render_help_screen(f, chunks[1]),
    }

    render_status_bar(f, chunks[2], app);
}

fn render_header(f: &mut Frame, area: Rect, app: &App) {
    let tabs = vec![
        Span::styled(" Claude ", Style::default()),
        Span::styled(" Codex ", Style::default()),
    ];

    let selected = match app.current_tab {
        Tab::Claude => 0,
        Tab::Codex => 1,
    };

    let tabs = Tabs::new(tabs)
        .block(Block::default().borders(Borders::ALL).title(" CC-Switch CLI "))
        .select(selected)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));

    f.render_widget(tabs, area);
}

fn render_provider_list(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),     // Provider list
            Constraint::Length(4),  // Help text
        ])
        .split(area);

    // Render provider list
    let items: Vec<ListItem> = app.providers
        .iter()
        .map(|(id, provider)| {
            let is_current = id == &app.current_provider_id;
            let prefix = if is_current { "● " } else { "  " };
            let style = if is_current {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let content = format!("{}{}", prefix, provider.name);
            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} Providers ", app.current_tab.title())))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("► ");

    f.render_stateful_widget(list, chunks[0], &mut app.list_state.clone());

    // Render help text
    let help_text = vec![
        Line::from(vec![
            Span::styled("↑/k", Style::default().fg(Color::Cyan)),
            Span::raw(" Up  "),
            Span::styled("↓/j", Style::default().fg(Color::Cyan)),
            Span::raw(" Down  "),
            Span::styled("Enter", Style::default().fg(Color::Cyan)),
            Span::raw(" Switch  "),
            Span::styled("a", Style::default().fg(Color::Cyan)),
            Span::raw(" Add  "),
            Span::styled("e", Style::default().fg(Color::Cyan)),
            Span::raw(" Edit  "),
            Span::styled("d", Style::default().fg(Color::Cyan)),
            Span::raw(" Delete"),
        ]),
        Line::from(vec![
            Span::styled("m", Style::default().fg(Color::Cyan)),
            Span::raw(" MCP  "),
            Span::styled("Tab", Style::default().fg(Color::Cyan)),
            Span::raw(" Switch Tab  "),
            Span::styled("r", Style::default().fg(Color::Cyan)),
            Span::raw(" Refresh  "),
            Span::styled("?", Style::default().fg(Color::Cyan)),
            Span::raw(" Help  "),
            Span::styled("q", Style::default().fg(Color::Cyan)),
            Span::raw(" Quit"),
        ]),
    ];

    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title(" Commands "));

    f.render_widget(help, chunks[1]);
}

fn render_provider_form(f: &mut Frame, area: Rect, app: &App) {
    if let Some(form) = &app.form_data {
        let title = if form.is_edit { " Edit Provider " } else { " Add Provider " };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .margin(2)
            .split(area);

        // Name field
        let name_style = if matches!(form.current_field, FormField::Name) {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        let name = Paragraph::new(form.name.as_str())
            .block(Block::default().borders(Borders::ALL).title(" Name ").border_style(name_style));
        f.render_widget(name, chunks[0]);

        // API Key field
        let key_style = if matches!(form.current_field, FormField::ApiKey) {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        let masked_key = if form.api_key.is_empty() {
            String::new()
        } else {
            "*".repeat(form.api_key.len())
        };
        let api_key = Paragraph::new(masked_key)
            .block(Block::default().borders(Borders::ALL).title(" API Key ").border_style(key_style));
        f.render_widget(api_key, chunks[1]);

        // Base URL field
        let url_style = if matches!(form.current_field, FormField::BaseUrl) {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        let base_url = Paragraph::new(form.base_url.as_str())
            .block(Block::default().borders(Borders::ALL).title(" Base URL (optional) ").border_style(url_style));
        f.render_widget(base_url, chunks[2]);

        // Help text
        let help_text = vec![
            Line::from(vec![
                Span::styled("Tab/↓", Style::default().fg(Color::Cyan)),
                Span::raw(" Next  "),
                Span::styled("Shift+Tab/↑", Style::default().fg(Color::Cyan)),
                Span::raw(" Previous  "),
                Span::styled("Enter", Style::default().fg(Color::Cyan)),
                Span::raw(" Submit  "),
                Span::styled("Esc", Style::default().fg(Color::Cyan)),
                Span::raw(" Cancel"),
            ]),
        ];

        let help = Paragraph::new(help_text)
            .block(Block::default().borders(Borders::ALL).title(" Commands "));
        f.render_widget(help, chunks[4]);

        // Outer block
        let outer_block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(Color::Green));
        f.render_widget(outer_block, area);
    }
}

fn render_confirmation_dialog(f: &mut Frame, app: &App) {
    if let Some((_, name)) = &app.delete_target {
        let area = centered_rect(60, 20, f.area());

        f.render_widget(Clear, area);

        let text = vec![
            Line::from(""),
            Line::from(Span::styled(
                format!("Delete provider '{}'?", name),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "This action cannot be undone.",
                Style::default().fg(Color::Red),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("y", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw(" Yes  "),
                Span::styled("n", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::raw(" No"),
            ]),
        ];

        let paragraph = Paragraph::new(text)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(" Confirm Delete ")
                .border_style(Style::default().fg(Color::Red)))
            .alignment(Alignment::Center);

        f.render_widget(paragraph, area);
    }
}

fn render_mcp_list(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(4),
        ])
        .split(area);

    // Render MCP server list
    let items: Vec<ListItem> = app.mcp_servers
        .iter()
        .map(|server| {
            let checkbox = if server.enabled { "[✓] " } else { "[ ] " };
            let transport = if server.command.is_some() {
                "stdio"
            } else if server.url.is_some() {
                "http"
            } else {
                "unknown"
            };

            let content = format!("{}{} ({})", checkbox, server.name, transport);
            let style = if server.enabled {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} MCP Servers ", app.current_tab.title())))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("► ");

    f.render_stateful_widget(list, chunks[0], &mut app.mcp_list_state.clone());

    // Help text
    let help_text = vec![
        Line::from(vec![
            Span::styled("↑/k", Style::default().fg(Color::Cyan)),
            Span::raw(" Up  "),
            Span::styled("↓/j", Style::default().fg(Color::Cyan)),
            Span::raw(" Down  "),
            Span::styled("Space", Style::default().fg(Color::Cyan)),
            Span::raw(" Toggle  "),
            Span::styled("d", Style::default().fg(Color::Cyan)),
            Span::raw(" Delete"),
        ]),
        Line::from(vec![
            Span::styled("i", Style::default().fg(Color::Cyan)),
            Span::raw(" Import from Live  "),
            Span::styled("s", Style::default().fg(Color::Cyan)),
            Span::raw(" Sync to Live  "),
            Span::styled("Esc/q", Style::default().fg(Color::Cyan)),
            Span::raw(" Back"),
        ]),
    ];

    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title(" Commands "));

    f.render_widget(help, chunks[1]);
}

fn render_help_screen(f: &mut Frame, area: Rect) {
    let help_text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "CC-Switch CLI - Keyboard Shortcuts",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled("Navigation:", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from("  ↑/k          - Move up"),
        Line::from("  ↓/j          - Move down"),
        Line::from("  Tab          - Next tab"),
        Line::from("  Shift+Tab    - Previous tab"),
        Line::from("  1            - Switch to Claude tab"),
        Line::from("  2            - Switch to Codex tab"),
        Line::from(""),
        Line::from(Span::styled("Provider Management:", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from("  Enter        - Switch to selected provider"),
        Line::from("  a            - Add new provider"),
        Line::from("  e            - Edit selected provider"),
        Line::from("  d            - Delete selected provider"),
        Line::from("  r            - Refresh provider list"),
        Line::from(""),
        Line::from(Span::styled("MCP Management:", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from("  m            - Enter MCP management mode"),
        Line::from("  Space        - Toggle MCP server (in MCP mode)"),
        Line::from("  i            - Import MCP servers from live config (in MCP mode)"),
        Line::from("  s            - Sync enabled MCP servers to live config (in MCP mode)"),
        Line::from("  d            - Delete MCP server (in MCP mode)"),
        Line::from(""),
        Line::from(Span::styled("General:", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from("  ?            - Show this help screen"),
        Line::from("  q            - Quit application"),
        Line::from("  Esc          - Cancel/Go back"),
        Line::from(""),
        Line::from(Span::raw("Press any key to close...")),
    ];

    let paragraph = Paragraph::new(help_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(" Help ")
            .border_style(Style::default().fg(Color::Yellow)))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

fn render_status_bar(f: &mut Frame, area: Rect, app: &App) {
    let status_text = if let Some(error) = &app.error_message {
        Line::from(Span::styled(
            format!(" ✗ {}", error),
            Style::default().fg(Color::Red),
        ))
    } else if let Some(msg) = &app.status_message {
        Line::from(Span::styled(
            format!(" {}", msg),
            Style::default().fg(Color::Green),
        ))
    } else {
        Line::from(Span::styled(
            " Ready",
            Style::default().fg(Color::White),
        ))
    };

    let status = Paragraph::new(status_text)
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(status, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
