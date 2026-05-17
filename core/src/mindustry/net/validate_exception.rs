use thiserror::Error;

#[derive(Debug, Error)]
#[error("validation failed for player {player:?}: {message}")]
pub struct ValidateException {
    pub player: Option<String>,
    pub message: String,
}

impl ValidateException {
    pub fn new(player: Option<String>, message: impl Into<String>) -> Self {
        Self {
            player,
            message: message.into(),
        }
    }
}
