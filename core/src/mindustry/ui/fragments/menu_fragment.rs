//! Main menu fragment state model mirroring upstream `mindustry.ui.fragments.MenuFragment`.

pub const MENU_DESKTOP_BUTTON_WIDTH: f32 = 230.0;
pub const MENU_DESKTOP_BUTTON_HEIGHT: f32 = 70.0;
pub const MENU_MOBILE_BUTTON_SIZE: f32 = 120.0;
pub const MENU_SUBMENU_FADE_IN: f32 = 0.15;
pub const MENU_SUBMENU_FADE_OUT: f32 = 0.2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuButton {
    pub text: String,
    pub icon_name: String,
    pub action: MenuButtonAction,
    pub submenu: Vec<MenuButton>,
    pub requires_no_content_errors: bool,
}

impl MenuButton {
    pub fn new(
        text: impl Into<String>,
        icon_name: impl Into<String>,
        action: MenuButtonAction,
    ) -> Self {
        Self {
            text: text.into(),
            icon_name: icon_name.into(),
            action,
            submenu: Vec::new(),
            requires_no_content_errors: false,
        }
    }

    pub fn gated(mut self) -> Self {
        self.requires_no_content_errors = true;
        self
    }

    pub fn with_submenu(
        text: impl Into<String>,
        icon_name: impl Into<String>,
        submenu: Vec<MenuButton>,
    ) -> Self {
        Self {
            text: text.into(),
            icon_name: icon_name.into(),
            action: MenuButtonAction::None,
            submenu,
            requires_no_content_errors: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuButtonAction {
    None,
    Campaign,
    JoinGame,
    CustomGame,
    LoadGame,
    Schematics,
    Database,
    About,
    Editor,
    Workshop,
    Mods,
    Settings,
    Quit,
    ConsoleMobile,
    Discord,
    BeCheck,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum MenuFragmentAction {
    Run(MenuButtonAction),
    ShowInfo(&'static str),
    FadeInSubmenu { duration: f32 },
    FadeOutSubmenu { duration: f32 },
    ClearSubmenu,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuLayoutModel {
    pub mobile: bool,
    pub portrait: bool,
    pub rows: Vec<Vec<String>>,
    pub discord_visible: bool,
    pub info_visible: bool,
    pub console_visible: bool,
    pub be_check_visible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MenuFragment {
    custom_buttons: Vec<MenuButton>,
    desktop_buttons: Option<Vec<MenuButton>>,
    current_menu: Option<String>,
    submenu: Vec<MenuButton>,
}

impl MenuFragment {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_button(&mut self, button: MenuButton) {
        self.custom_buttons.push(button);
    }

    pub fn current_menu(&self) -> Option<&str> {
        self.current_menu.as_deref()
    }

    pub fn submenu(&self) -> &[MenuButton] {
        &self.submenu
    }

    pub fn build_mobile(
        &self,
        portrait: bool,
        ios: bool,
        console_enabled: bool,
        console_shown: bool,
    ) -> MenuLayoutModel {
        let customs = self.custom_buttons.clone();
        let final_button = if ios {
            mobile_button("@about.button", "info", MenuButtonAction::About)
        } else {
            mobile_button("@quit", "exit", MenuButtonAction::Quit)
        };

        let mut rows = Vec::new();
        if !portrait {
            let mut first = vec![
                "@campaign".into(),
                "@joingame".into(),
                "@customgame".into(),
                "@loadgame".into(),
            ];
            for i in (1..customs.len()).step_by(2) {
                first.push(customs[i].text.clone());
            }
            let mut second = vec!["@editor".into(), "@settings".into(), "@mods".into()];
            for i in (0..customs.len()).step_by(2) {
                second.push(customs[i].text.clone());
            }
            second.push(final_button.text);
            rows.push(first);
            rows.push(second);
        } else {
            rows.push(vec!["@campaign".into(), "@loadgame".into()]);
            rows.push(vec!["@customgame".into(), "@joingame".into()]);
            rows.push(vec!["@editor".into(), "@settings".into()]);
            let mut row = vec!["@mods".into()];
            for (index, button) in customs.iter().enumerate() {
                row.push(button.text.clone());
                if index % 2 == 0 {
                    rows.push(row);
                    row = Vec::new();
                }
            }
            row.push(final_button.text);
            rows.push(row);
        }

        MenuLayoutModel {
            mobile: true,
            portrait,
            rows,
            discord_visible: !console_shown,
            info_visible: !console_shown,
            console_visible: !console_shown && console_enabled,
            be_check_visible: false,
        }
    }

    pub fn build_desktop(
        &mut self,
        steam: bool,
        be_control_active: bool,
        console_shown: bool,
    ) -> MenuLayoutModel {
        if self.desktop_buttons.is_none() {
            self.desktop_buttons = Some(default_desktop_buttons(steam));
        }
        let mut rows = Vec::new();
        for button in self.desktop_buttons.as_ref().unwrap() {
            rows.push(vec![button.text.clone()]);
        }
        for button in &self.custom_buttons {
            rows.push(vec![button.text.clone()]);
        }
        rows.push(vec!["@quit".into()]);

        MenuLayoutModel {
            mobile: false,
            portrait: false,
            rows,
            discord_visible: !console_shown,
            info_visible: false,
            console_visible: false,
            be_check_visible: be_control_active,
        }
    }

    pub fn desktop_buttons(&mut self, steam: bool) -> &[MenuButton] {
        if self.desktop_buttons.is_none() {
            self.desktop_buttons = Some(default_desktop_buttons(steam));
        }
        self.desktop_buttons.as_deref().unwrap()
    }

    pub fn click_desktop_button(
        &mut self,
        button: &MenuButton,
        content_errors: bool,
    ) -> Vec<MenuFragmentAction> {
        if self.current_menu.as_deref() == Some(&button.text) {
            self.current_menu = None;
            self.submenu.clear();
            return vec![
                MenuFragmentAction::FadeOutSubmenu {
                    duration: MENU_SUBMENU_FADE_OUT,
                },
                MenuFragmentAction::ClearSubmenu,
            ];
        }

        if !button.submenu.is_empty() {
            self.current_menu = Some(button.text.clone());
            self.submenu = button.submenu.clone();
            return vec![MenuFragmentAction::FadeInSubmenu {
                duration: MENU_SUBMENU_FADE_IN,
            }];
        }

        self.current_menu = None;
        self.submenu.clear();
        let mut actions = vec![
            MenuFragmentAction::FadeOutSubmenu {
                duration: MENU_SUBMENU_FADE_OUT,
            },
            MenuFragmentAction::ClearSubmenu,
        ];
        actions.extend(run_button(button, content_errors));
        actions
    }

    pub fn click_mobile_button(
        &self,
        button: &MenuButton,
        content_errors: bool,
    ) -> Vec<MenuFragmentAction> {
        run_button(button, content_errors)
    }
}

fn run_button(button: &MenuButton, content_errors: bool) -> Vec<MenuFragmentAction> {
    if button.requires_no_content_errors && content_errors {
        vec![MenuFragmentAction::ShowInfo("@mod.noerrorplay")]
    } else {
        vec![MenuFragmentAction::Run(button.action.clone())]
    }
}

fn mobile_button(text: &str, icon: &str, action: MenuButtonAction) -> MenuButton {
    MenuButton::new(text, icon, action).gated()
}

fn default_desktop_buttons(steam: bool) -> Vec<MenuButton> {
    let mut out = vec![
        MenuButton::with_submenu(
            "@play",
            "play",
            vec![
                MenuButton::new("@campaign", "play", MenuButtonAction::Campaign).gated(),
                MenuButton::new("@joingame", "add", MenuButtonAction::JoinGame).gated(),
                MenuButton::new("@customgame", "terrain", MenuButtonAction::CustomGame).gated(),
                MenuButton::new("@loadgame", "download", MenuButtonAction::LoadGame).gated(),
            ],
        ),
        MenuButton::with_submenu(
            "@database.button",
            "menu",
            vec![
                MenuButton::new("@schematics", "paste", MenuButtonAction::Schematics),
                MenuButton::new("@database", "book", MenuButtonAction::Database),
                MenuButton::new("@about.button", "info", MenuButtonAction::About),
            ],
        ),
        MenuButton::new("@editor", "terrain", MenuButtonAction::Editor).gated(),
    ];
    if steam {
        out.push(MenuButton::new(
            "@workshop",
            "steam",
            MenuButtonAction::Workshop,
        ));
    }
    out.extend([
        MenuButton::new("@mods", "book", MenuButtonAction::Mods),
        MenuButton::new("@settings", "settings", MenuButtonAction::Settings),
    ]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mobile_landscape_layout_interleaves_custom_buttons_like_java() {
        let mut menu = MenuFragment::new();
        menu.add_button(MenuButton::new(
            "@custom0",
            "none",
            MenuButtonAction::Custom("c0".into()),
        ));
        menu.add_button(MenuButton::new(
            "@custom1",
            "none",
            MenuButtonAction::Custom("c1".into()),
        ));

        let model = menu.build_mobile(false, false, true, false);

        assert_eq!(
            model.rows,
            vec![
                vec![
                    "@campaign",
                    "@joingame",
                    "@customgame",
                    "@loadgame",
                    "@custom1"
                ],
                vec!["@editor", "@settings", "@mods", "@custom0", "@quit"],
            ]
        );
        assert!(model.console_visible);
        assert!(model.discord_visible);
    }

    #[test]
    fn mobile_portrait_layout_uses_about_on_ios_and_custom_row_breaks() {
        let mut menu = MenuFragment::new();
        menu.add_button(MenuButton::new(
            "@custom0",
            "none",
            MenuButtonAction::Custom("c0".into()),
        ));
        menu.add_button(MenuButton::new(
            "@custom1",
            "none",
            MenuButtonAction::Custom("c1".into()),
        ));

        let model = menu.build_mobile(true, true, false, true);

        assert_eq!(model.rows[0], vec!["@campaign", "@loadgame"]);
        assert_eq!(model.rows[3], vec!["@mods", "@custom0"]);
        assert_eq!(model.rows[4], vec!["@custom1", "@about.button"]);
        assert!(!model.discord_visible);
        assert!(!model.console_visible);
    }

    #[test]
    fn desktop_default_buttons_match_upstream_groups_and_steam_workshop_slot() {
        let mut menu = MenuFragment::new();
        let buttons = menu.desktop_buttons(true);

        assert_eq!(buttons[0].text, "@play");
        assert_eq!(buttons[0].submenu[0].text, "@campaign");
        assert_eq!(buttons[1].text, "@database.button");
        assert_eq!(buttons[2].text, "@editor");
        assert_eq!(buttons[3].text, "@workshop");
        assert_eq!(buttons[4].text, "@mods");
        assert_eq!(buttons[5].text, "@settings");
    }

    #[test]
    fn desktop_click_toggles_submenu_and_runs_leaf_with_content_error_gate() {
        let mut menu = MenuFragment::new();
        let play = menu.desktop_buttons(false)[0].clone();

        assert_eq!(
            menu.click_desktop_button(&play, false),
            vec![MenuFragmentAction::FadeInSubmenu { duration: 0.15 }]
        );
        assert_eq!(menu.current_menu(), Some("@play"));
        assert_eq!(menu.submenu().len(), 4);

        assert_eq!(
            menu.click_desktop_button(&play, false),
            vec![
                MenuFragmentAction::FadeOutSubmenu { duration: 0.2 },
                MenuFragmentAction::ClearSubmenu
            ]
        );
        assert_eq!(menu.current_menu(), None);

        let campaign = play.submenu[0].clone();
        assert_eq!(
            menu.click_desktop_button(&campaign, true),
            vec![
                MenuFragmentAction::FadeOutSubmenu { duration: 0.2 },
                MenuFragmentAction::ClearSubmenu,
                MenuFragmentAction::ShowInfo("@mod.noerrorplay")
            ]
        );
    }
}
