use crate::mindustry::service::{
    AchievementState, DefaultGameService, GameService, GameServiceInitAction,
};
use crate::mindustry::vars::AppContext;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientLauncher {
    pub context: AppContext,
    pub loaded: bool,
    pub service: DefaultGameService,
    pub achievement_state: AchievementState,
    service_waiting_for_client_load: bool,
}

impl ClientLauncher {
    pub fn new(context: AppContext) -> Self {
        Self {
            context,
            loaded: false,
            service: DefaultGameService::new(),
            achievement_state: AchievementState::new(),
            service_waiting_for_client_load: false,
        }
    }

    pub fn setup(&mut self) {
        self.loaded = false;
        self.init_game_service();
    }

    pub fn update(&mut self) {
        self.loaded = true;
        if self.service_waiting_for_client_load || !self.service.events_registered() {
            self.init_game_service();
        }
    }

    pub fn init_game_service(&mut self) -> GameServiceInitAction {
        if self.service.events_registered() {
            self.service_waiting_for_client_load = false;
            return GameServiceInitAction::RegisterEventsNow;
        }

        let action = self.service.init(self.loaded);
        self.service_waiting_for_client_load =
            matches!(action, GameServiceInitAction::WaitForClientLoad);
        action
    }

    pub fn service_waiting_for_client_load(&self) -> bool {
        self.service_waiting_for_client_load
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::service::game_service::java_t5_units;

    #[test]
    fn setup_defers_game_service_events_until_client_load_completes() {
        let mut launcher = ClientLauncher::new(AppContext::new("data"));

        launcher.setup();

        assert!(!launcher.loaded);
        assert!(launcher.service_waiting_for_client_load());
        assert!(!launcher.service.events_registered());

        launcher.update();

        assert!(launcher.loaded);
        assert!(!launcher.service_waiting_for_client_load());
        assert!(launcher.service.events_registered());
        assert_eq!(launcher.service.state().t5s, java_t5_units());
    }
}
