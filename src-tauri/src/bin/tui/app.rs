use cc_switch_lib::{
    AppState, AppType, Provider, ProviderService, McpService,
};
use ratatui::widgets::ListState;
use serde_json::json;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Claude,
    Codex,
}

impl Tab {
    pub fn as_app_type(&self) -> AppType {
        match self {
            Tab::Claude => AppType::Claude,
            Tab::Codex => AppType::Codex,
        }
    }

    pub fn title(&self) -> &str {
        match self {
            Tab::Claude => "Claude",
            Tab::Codex => "Codex",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Normal,
    ProviderForm,
    Confirmation,
    McpManagement,
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormField {
    Name,
    ApiKey,
    BaseUrl,
}

pub struct ProviderFormData {
    pub name: String,
    pub api_key: String,
    pub base_url: String,
    pub current_field: FormField,
    pub is_edit: bool,
    pub edit_id: Option<String>,
}

impl ProviderFormData {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            api_key: String::new(),
            base_url: String::new(),
            current_field: FormField::Name,
            is_edit: false,
            edit_id: None,
        }
    }

    pub fn for_edit(provider: &Provider) -> Self {
        // Extract API key and base URL from settingsConfig
        let api_key = provider.settings_config
            .get("env")
            .and_then(|env| env.get("ANTHROPIC_API_KEY")
                .or_else(|| env.get("ANTHROPIC_AUTH_TOKEN"))
                .or_else(|| env.get("OPENAI_API_KEY")))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let base_url = provider.settings_config
            .get("env")
            .and_then(|env| env.get("ANTHROPIC_BASE_URL")
                .or_else(|| env.get("OPENAI_BASE_URL")))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        Self {
            name: provider.name.clone(),
            api_key,
            base_url,
            current_field: FormField::Name,
            is_edit: true,
            edit_id: Some(provider.id.clone()),
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.trim().is_empty() && !self.api_key.trim().is_empty()
    }
}

pub struct McpServerItem {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub command: Option<String>,
    pub url: Option<String>,
}

pub struct App {
    pub should_quit: bool,
    pub mode: AppMode,
    pub current_tab: Tab,
    pub state: AppState,

    // Provider list
    pub providers: Vec<(String, Provider)>,
    pub list_state: ListState,
    pub current_provider_id: String,

    // Provider form
    pub form_data: Option<ProviderFormData>,

    // Deletion confirmation
    pub delete_target: Option<(String, String)>, // (id, name)

    // MCP management
    pub mcp_servers: Vec<McpServerItem>,
    pub mcp_list_state: ListState,

    // Status messages
    pub status_message: Option<String>,
    pub error_message: Option<String>,
}

impl App {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let state = AppState::try_new()
            .map_err(|e| format!("Failed to load config: {}", e))?;

        let mut app = Self {
            should_quit: false,
            mode: AppMode::Normal,
            current_tab: Tab::Claude,
            state,
            providers: Vec::new(),
            list_state: ListState::default(),
            current_provider_id: String::new(),
            form_data: None,
            delete_target: None,
            mcp_servers: Vec::new(),
            mcp_list_state: ListState::default(),
            status_message: None,
            error_message: None,
        };

        app.refresh_provider_list()?;
        Ok(app)
    }

    pub fn refresh_provider_list(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let config = self.state.config.read().map_err(|e| format!("Lock error: {}", e))?;
        let app_type = self.current_tab.as_app_type();

        if let Some(manager) = config.get_manager(&app_type) {
            self.current_provider_id = manager.current.clone();

            // Sort providers
            let mut providers: Vec<_> = manager.providers.iter()
                .map(|(id, p)| (id.clone(), p.clone()))
                .collect();

            providers.sort_by(|(_, a), (_, b)| {
                match (a.sort_index, b.sort_index) {
                    (Some(idx_a), Some(idx_b)) => idx_a.cmp(&idx_b),
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    _ => {
                        match (a.created_at, b.created_at) {
                            (Some(time_a), Some(time_b)) => time_a.cmp(&time_b),
                            (Some(_), None) => std::cmp::Ordering::Greater,
                            (None, Some(_)) => std::cmp::Ordering::Less,
                            _ => a.name.cmp(&b.name),
                        }
                    }
                }
            });

            self.providers = providers;

            // Set initial selection
            if !self.providers.is_empty() {
                let current_index = self.providers.iter()
                    .position(|(id, _)| id == &self.current_provider_id)
                    .unwrap_or(0);
                self.list_state.select(Some(current_index));
            }
        } else {
            self.providers.clear();
        }

        Ok(())
    }

    pub fn next_tab(&mut self) {
        self.current_tab = match self.current_tab {
            Tab::Claude => Tab::Codex,
            Tab::Codex => Tab::Claude,
        };
        let _ = self.refresh_provider_list();
    }

    pub fn previous_tab(&mut self) {
        self.next_tab();
    }

    pub fn next_provider(&mut self) {
        if self.providers.is_empty() {
            return;
        }

        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.providers.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn previous_provider(&mut self) {
        if self.providers.is_empty() {
            return;
        }

        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.providers.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn switch_selected_provider(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(i) = self.list_state.selected() {
            if let Some((id, provider)) = self.providers.get(i) {
                let app_type = self.current_tab.as_app_type();

                ProviderService::switch(
                    &self.state,
                    app_type,
                    id,
                )?;

                self.current_provider_id = id.clone();
                self.status_message = Some(format!("✓ Switched to '{}'", provider.name));
                self.error_message = None;
            }
        }
        Ok(())
    }

    pub fn start_add_provider(&mut self) {
        self.form_data = Some(ProviderFormData::new());
        self.mode = AppMode::ProviderForm;
        self.error_message = None;
    }

    pub fn start_edit_provider(&mut self) {
        if let Some(i) = self.list_state.selected() {
            if let Some((_, provider)) = self.providers.get(i) {
                self.form_data = Some(ProviderFormData::for_edit(provider));
                self.mode = AppMode::ProviderForm;
                self.error_message = None;
            }
        }
    }

    pub fn start_delete_confirmation(&mut self) {
        if let Some(i) = self.list_state.selected() {
            if let Some((id, provider)) = self.providers.get(i) {
                self.delete_target = Some((id.clone(), provider.name.clone()));
                self.mode = AppMode::Confirmation;
            }
        }
    }

    pub fn cancel_form(&mut self) {
        self.form_data = None;
        self.mode = AppMode::Normal;
    }

    pub fn cancel_confirmation(&mut self) {
        self.delete_target = None;
        self.mode = AppMode::Normal;
    }

    pub fn next_form_field(&mut self) {
        if let Some(form) = &mut self.form_data {
            form.current_field = match form.current_field {
                FormField::Name => FormField::ApiKey,
                FormField::ApiKey => FormField::BaseUrl,
                FormField::BaseUrl => FormField::Name,
            };
        }
    }

    pub fn previous_form_field(&mut self) {
        if let Some(form) = &mut self.form_data {
            form.current_field = match form.current_field {
                FormField::Name => FormField::BaseUrl,
                FormField::ApiKey => FormField::Name,
                FormField::BaseUrl => FormField::ApiKey,
            };
        }
    }

    pub fn input_char(&mut self, c: char) {
        if let Some(form) = &mut self.form_data {
            match form.current_field {
                FormField::Name => form.name.push(c),
                FormField::ApiKey => form.api_key.push(c),
                FormField::BaseUrl => form.base_url.push(c),
            }
        }
    }

    pub fn delete_char(&mut self) {
        if let Some(form) = &mut self.form_data {
            match form.current_field {
                FormField::Name => { form.name.pop(); },
                FormField::ApiKey => { form.api_key.pop(); },
                FormField::BaseUrl => { form.base_url.pop(); },
            }
        }
    }

    pub fn is_form_valid(&self) -> bool {
        self.form_data.as_ref().map(|f| f.is_valid()).unwrap_or(false)
    }

    pub fn submit_form(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(form) = self.form_data.take() {
            let app_type = self.current_tab.as_app_type();

            // Build settingsConfig based on app type
            let settings_config = match app_type {
                AppType::Claude => {
                    let mut env = serde_json::Map::new();
                    env.insert("ANTHROPIC_API_KEY".to_string(), json!(form.api_key));
                    if !form.base_url.is_empty() {
                        env.insert("ANTHROPIC_BASE_URL".to_string(), json!(form.base_url));
                    }
                    json!({ "env": env })
                }
                AppType::Codex => {
                    let mut env = serde_json::Map::new();
                    env.insert("OPENAI_API_KEY".to_string(), json!(form.api_key));
                    if !form.base_url.is_empty() {
                        env.insert("OPENAI_BASE_URL".to_string(), json!(form.base_url));
                    }
                    json!({ "env": env })
                }
            };

            if form.is_edit {
                // Update existing provider
                if let Some(id) = form.edit_id {
                    let provider = Provider::with_id(
                        id,
                        form.name.clone(),
                        settings_config,
                        None,
                    );
                    ProviderService::update(&self.state, app_type, provider)?;
                    self.status_message = Some(format!("✓ Updated '{}'", form.name));
                }
            } else {
                // Add new provider
                let provider = Provider::with_id(
                    uuid::Uuid::new_v4().to_string(),
                    form.name.clone(),
                    settings_config,
                    None,
                );
                ProviderService::add(&self.state, app_type, provider)?;
                self.status_message = Some(format!("✓ Added '{}'", form.name));
            }

            self.error_message = None;
            self.mode = AppMode::Normal;
            self.refresh_provider_list()?;
        }
        Ok(())
    }

    pub fn confirm_delete(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some((id, name)) = self.delete_target.take() {
            let app_type = self.current_tab.as_app_type();

            ProviderService::delete(&self.state, app_type, &id)?;

            self.status_message = Some(format!("✓ Deleted '{}'", name));
            self.error_message = None;
            self.mode = AppMode::Normal;
            self.refresh_provider_list()?;
        }
        Ok(())
    }

    // MCP Management methods
    pub fn enter_mcp_mode(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.refresh_mcp_list()?;
        self.mode = AppMode::McpManagement;
        Ok(())
    }

    pub fn exit_mcp_mode(&mut self) {
        self.mode = AppMode::Normal;
    }

    pub fn refresh_mcp_list(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let config = self.state.config.read().map_err(|e| format!("Lock error: {}", e))?;
        let app_type = self.current_tab.as_app_type();

        let servers = match app_type {
            AppType::Claude => &config.mcp.claude.servers,
            AppType::Codex => &config.mcp.codex.servers,
        };

        self.mcp_servers = servers.iter().map(|(id, server)| {
            let enabled = server.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false);
            let command = server.get("command").and_then(|v| v.as_str()).map(String::from);
            let url = server.get("url").and_then(|v| v.as_str()).map(String::from);

            McpServerItem {
                id: id.clone(),
                name: id.clone(),
                enabled,
                command,
                url,
            }
        }).collect();

        if !self.mcp_servers.is_empty() && self.mcp_list_state.selected().is_none() {
            self.mcp_list_state.select(Some(0));
        }

        Ok(())
    }

    pub fn next_mcp_server(&mut self) {
        if self.mcp_servers.is_empty() {
            return;
        }

        let i = match self.mcp_list_state.selected() {
            Some(i) => (i + 1) % self.mcp_servers.len(),
            None => 0,
        };
        self.mcp_list_state.select(Some(i));
    }

    pub fn previous_mcp_server(&mut self) {
        if self.mcp_servers.is_empty() {
            return;
        }

        let i = match self.mcp_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.mcp_servers.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.mcp_list_state.select(Some(i));
    }

    pub fn toggle_mcp_server(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(i) = self.mcp_list_state.selected() {
            if let Some(server) = self.mcp_servers.get_mut(i) {
                server.enabled = !server.enabled;

                let app_type = self.current_tab.as_app_type();
                McpService::set_enabled(&self.state, app_type, &server.id, server.enabled)?;

                self.status_message = Some(format!(
                    "✓ {} '{}'",
                    if server.enabled { "Enabled" } else { "Disabled" },
                    server.name
                ));
            }
        }
        Ok(())
    }

    pub fn delete_mcp_server(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(i) = self.mcp_list_state.selected() {
            if let Some(server) = self.mcp_servers.get(i) {
                let app_type = self.current_tab.as_app_type();
                let name = server.name.clone();

                McpService::delete_server(&self.state, app_type, &server.id)?;

                self.status_message = Some(format!("✓ Deleted MCP server '{}'", name));
                self.refresh_mcp_list()?;
            }
        }
        Ok(())
    }

    pub fn import_mcp_from_live(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let app_type = self.current_tab.as_app_type();

        match app_type {
            AppType::Claude => {
                McpService::import_from_claude(&self.state)?;
                self.status_message = Some("✓ Imported MCP servers from Claude".to_string());
            }
            AppType::Codex => {
                McpService::import_from_codex(&self.state)?;
                self.status_message = Some("✓ Imported MCP servers from Codex".to_string());
            }
        }

        self.refresh_mcp_list()?;
        Ok(())
    }

    pub fn sync_mcp_to_live(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let app_type = self.current_tab.as_app_type();
        let app_type_str = app_type.as_str().to_string();

        McpService::sync_enabled(&self.state, app_type)?;
        self.status_message = Some(format!("✓ Synced enabled MCP servers to {}", app_type_str));

        Ok(())
    }
}
