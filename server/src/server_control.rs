use mindustry_core::mindustry::game::Gamemode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerCommandResult {
    Valid {
        command: String,
    },
    Unknown {
        run_command: String,
        suggestion: Option<String>,
    },
    NoSuggestion,
    Empty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerControl {
    pub startup_commands: Vec<String>,
    pub registered_commands: Vec<String>,
    pub command_history: Vec<String>,
    pub stdin_enabled: bool,
    pub socket_enabled: bool,
    pub autosave_enabled: bool,
    pub logging_enabled: bool,
    pub last_mode: Gamemode,
    pub suggested: Option<String>,
    pub last_response: Option<ServerCommandResult>,
}

impl ServerControl {
    pub fn new(startup_commands: Vec<String>) -> Self {
        Self {
            startup_commands,
            registered_commands: default_server_commands(),
            command_history: Vec::new(),
            stdin_enabled: false,
            socket_enabled: false,
            autosave_enabled: false,
            logging_enabled: false,
            last_mode: Gamemode::Survival,
            suggested: None,
            last_response: None,
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

    pub fn register_command(&mut self, command: impl Into<String>) {
        let command = command.into();
        if !self
            .registered_commands
            .iter()
            .any(|known| known.eq_ignore_ascii_case(&command))
        {
            self.registered_commands.push(command);
        }
    }

    pub fn handle_command_string(&mut self, line: &str) -> ServerCommandResult {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            let response = ServerCommandResult::Empty;
            self.last_response = Some(response.clone());
            return response;
        }

        let (run_command, rest) = split_command(trimmed);
        if run_command.eq_ignore_ascii_case("yes") {
            let Some(suggested) = self.suggested.clone() else {
                let response = ServerCommandResult::NoSuggestion;
                self.last_response = Some(response.clone());
                return response;
            };

            return self.handle_command_string(&suggested);
        }

        if let Some(command) = self
            .registered_commands
            .iter()
            .find(|known| known.eq_ignore_ascii_case(run_command))
            .cloned()
        {
            self.suggested = None;
            self.command_history.push(trimmed.to_string());
            let response = ServerCommandResult::Valid { command };
            self.last_response = Some(response.clone());
            return response;
        }

        let suggestion = self.closest_command(run_command).map(|closest| {
            let replacement = format!("{closest}{rest}");
            self.suggested = Some(replacement.clone());
            replacement
        });

        if suggestion.is_none() {
            self.suggested = None;
        }

        let response = ServerCommandResult::Unknown {
            run_command: run_command.to_string(),
            suggestion,
        };
        self.last_response = Some(response.clone());
        response
    }

    fn closest_command(&self, run_command: &str) -> Option<String> {
        self.registered_commands
            .iter()
            .filter(|command| !command.eq_ignore_ascii_case("yes"))
            .filter_map(|command| {
                let distance = levenshtein(
                    &command.to_ascii_lowercase(),
                    &run_command.to_ascii_lowercase(),
                );
                (distance < 3).then_some((distance, command))
            })
            .min_by_key(|(distance, _)| *distance)
            .map(|(_, command)| command.clone())
    }
}

pub fn default_server_commands() -> Vec<String> {
    [
        "help",
        "version",
        "exit",
        "stop",
        "host",
        "maps",
        "reloadpatches",
        "reloadmaps",
        "status",
        "yes",
        "gc",
        "dos-ban",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

fn split_command(line: &str) -> (&str, &str) {
    match line.find(char::is_whitespace) {
        Some(index) => (&line[..index], &line[index..]),
        None => (line, ""),
    }
}

fn levenshtein(a: &str, b: &str) -> usize {
    if a.is_empty() {
        return b.chars().count();
    }
    if b.is_empty() {
        return a.chars().count();
    }

    let b_chars: Vec<char> = b.chars().collect();
    let mut costs: Vec<usize> = (0..=b_chars.len()).collect();

    for (i, ca) in a.chars().enumerate() {
        let mut previous = costs[0];
        costs[0] = i + 1;
        for (j, &cb) in b_chars.iter().enumerate() {
            let insertion = costs[j + 1] + 1;
            let deletion = costs[j] + 1;
            let substitution = previous + usize::from(ca != cb);
            previous = costs[j + 1];
            costs[j + 1] = insertion.min(deletion).min(substitution);
        }
    }

    costs[b_chars.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_handler_accepts_registered_commands_and_records_history() {
        let mut control = ServerControl::new(Vec::new());

        assert_eq!(
            control.handle_command_string("status"),
            ServerCommandResult::Valid {
                command: "status".into()
            }
        );
        assert_eq!(control.command_history, vec!["status"]);
        assert!(control.suggested.is_none());
    }

    #[test]
    fn command_handler_suggests_close_unknown_commands_like_java() {
        let mut control = ServerControl::new(Vec::new());

        assert_eq!(
            control.handle_command_string("statu"),
            ServerCommandResult::Unknown {
                run_command: "statu".into(),
                suggestion: Some("status".into())
            }
        );
        assert_eq!(control.suggested.as_deref(), Some("status"));
    }

    #[test]
    fn yes_replays_last_suggested_command_and_clears_suggestion() {
        let mut control = ServerControl::new(Vec::new());
        control.handle_command_string("statu");

        assert_eq!(
            control.handle_command_string("yes"),
            ServerCommandResult::Valid {
                command: "status".into()
            }
        );
        assert_eq!(control.command_history, vec!["status"]);
        assert!(control.suggested.is_none());
    }

    #[test]
    fn command_handler_reports_missing_suggestion_for_yes() {
        let mut control = ServerControl::new(Vec::new());

        assert_eq!(
            control.handle_command_string("yes"),
            ServerCommandResult::NoSuggestion
        );
    }

    #[test]
    fn command_handler_preserves_arguments_when_suggesting() {
        let mut control = ServerControl::new(Vec::new());

        assert_eq!(
            control.handle_command_string("hst map survival"),
            ServerCommandResult::Unknown {
                run_command: "hst".into(),
                suggestion: Some("host map survival".into())
            }
        );
    }
}
