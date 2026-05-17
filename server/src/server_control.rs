use mindustry_core::mindustry::game::Gamemode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerControl {
    pub startup_commands: Vec<String>,
    pub stdin_enabled: bool,
    pub socket_enabled: bool,
    pub autosave_enabled: bool,
    pub logging_enabled: bool,
    pub last_mode: Gamemode,
}

impl ServerControl {
    pub fn new(startup_commands: Vec<String>) -> Self {
        Self {
            startup_commands,
            stdin_enabled: false,
            socket_enabled: false,
            autosave_enabled: false,
            logging_enabled: false,
            last_mode: Gamemode::Survival,
        }
    }

    pub fn enable_stdin(&mut self) {
        self.stdin_enabled = true;
    }

    pub fn enable_socket(&mut self) {
        self.socket_enabled = true;
    }

    pub fn enable_autosave(&mut self) {
        self.autosave_enabled = true;
    }

    pub fn enable_logging(&mut self) {
        self.logging_enabled = true;
    }

    pub fn set_last_mode(&mut self, mode: Gamemode) {
        self.last_mode = mode;
    }

    pub fn handle_command_string(&mut self, _line: &str) {}
}
