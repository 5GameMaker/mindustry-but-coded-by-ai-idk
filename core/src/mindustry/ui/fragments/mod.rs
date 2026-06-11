//! Frontend fragment state models mirroring upstream `mindustry.ui.fragments`.

pub mod block_config_fragment;
pub mod block_inventory_fragment;
pub mod chat_fragment;
pub mod console_fragment;
pub mod fade_in_fragment;
pub mod hints_fragment;
pub mod hud_fragment;
pub mod loading_fragment;
pub mod menu_fragment;
pub mod minimap_fragment;
pub mod placement_fragment;
pub mod plan_config_fragment;
pub mod player_list_fragment;

pub use block_config_fragment::{
    BlockConfigFragment, BlockConfigShowResult, BlockConfigTableAction, ConfigBuildingRef,
};
pub use block_inventory_fragment::{
    round_amount, BlockInventoryAction, BlockInventoryBuildingRef, BlockInventoryFragment,
    BlockInventoryItemCell, BlockInventoryModel, BlockInventoryUpdateContext,
    BLOCK_INVENTORY_CELL_PAD, BLOCK_INVENTORY_CELL_SIZE, BLOCK_INVENTORY_COLUMNS,
    BLOCK_INVENTORY_HOLD_SHRINK, BLOCK_INVENTORY_HOLD_WITHDRAW, BLOCK_INVENTORY_MARGIN,
};
pub use chat_fragment::{ChatAction, ChatDrawMessage, ChatFragment, ChatMode, CHAT_MESSAGES_SHOWN};
pub use console_fragment::{
    ConsoleAction, ConsoleFragment, ConsoleMobileButtonAction, ConsoleMobileButtonKind,
    ConsoleMobileButtonModel, ConsoleMobileToolbarModel, CONSOLE_INJECT_VARIABLES,
    CONSOLE_MESSAGES_SHOWN, CONSOLE_MOBILE_BUTTON_PAD_LEFT, CONSOLE_MOBILE_BUTTON_SIZE,
};
pub use fade_in_fragment::{FadeInDrawPlan, FadeInFragment};
pub use hints_fragment::{
    DefaultHint, HintState, HintsAction, HintsFragment, HintsUpdateContext, HINT_DESKTOP_WIDTH,
    HINT_FADE_OUT_TIME, HINT_MOBILE_WIDTH, HINT_VISIBLE_ALL, HINT_VISIBLE_DESKTOP,
    HINT_VISIBLE_MOBILE,
};
pub use hud_fragment::{
    can_skip_wave, format_waiting_seconds, status_text, HudContext, HudFragment, HudFragmentAction,
    HudMobileButtonAction, HudMobileButtonKind, HudMobileButtonModel, HudMobileButtonsModel,
    HudObjective, HudOverlayModel, HudRulesSnapshot, HudStatusModel, HudToastAction,
    HudUnlockAction, HudWaveSkipAction, HUD_DSIZE, HUD_PAUSE_DISABLED_DURATION, HUD_PAUSE_HEIGHT,
    HUD_STATUS_TABLE_WIDTH, HUD_TOAST_HOLD_DURATION, HUD_TOAST_INTERVAL_MILLIS, HUD_TOAST_MARGIN,
    HUD_TOAST_TEXT_WIDTH, HUD_TOAST_TRANSLATE_DURATION, HUD_UNLOCK_COLUMNS, HUD_UNLOCK_ICON_CAP,
};
pub use loading_fragment::{
    LoadingFragment, LoadingFragmentAction, LoadingFragmentModel, LOADING_FRAGMENT_BAR_SIZE,
    LOADING_FRAGMENT_CANCEL_BUTTON_SIZE, LOADING_FRAGMENT_FADE_OUT_DURATION,
};
pub use menu_fragment::{
    MenuButton, MenuButtonAction, MenuFragment, MenuFragmentAction, MenuLayoutModel,
    MENU_DESKTOP_BUTTON_HEIGHT, MENU_DESKTOP_BUTTON_WIDTH, MENU_MOBILE_BUTTON_SIZE,
    MENU_SUBMENU_FADE_IN, MENU_SUBMENU_FADE_OUT,
};
pub use minimap_fragment::{
    MinimapAction, MinimapConvertEnv, MinimapDrawPlan, MinimapFragment, MinimapGraphics,
    MinimapSceneMargins, MinimapTexture, MinimapToggleFocus, MinimapUpdateContext, MinimapWorld,
};
pub use placement_fragment::{
    PlacementAction, PlacementBlock, PlacementDisplayable, PlacementFragment,
    PlacementHoverContext, PlacementInfoBoxModel, PlacementInfoDisplay, PlacementInputContext,
    PlacementKey, PlacementPaletteCell, PlacementPaletteModel, PlacementRequirement,
    PLACEMENT_BLOCK_SELECT_TIMEOUT_MILLIS, PLACEMENT_ROW_WIDTH,
};
pub use plan_config_fragment::{
    BuildPlanUiRef, PlanConfigBlock, PlanConfigFragment, PlanConfigShowResult,
    PlanConfigTableAction, PlanConfigTableModel,
};
pub use player_list_fragment::{
    PlayerListContext, PlayerListFooterButtonAction, PlayerListFooterButtonModel,
    PlayerListFragment, PlayerListModel, PlayerListPlayer, PlayerListPlayerMenuAction,
    PlayerListPlayerMenuButtonModel, PlayerListPlayerMenuModel, PlayerListRow, PlayerListRowAction,
    PLAYER_LIST_CONTENT_MARGIN_HORIZONTAL, PLAYER_LIST_DIALOG_MIN_WIDTH, PLAYER_LIST_ICON_SIZE,
    PLAYER_LIST_MENU_BUTTON_HEIGHT, PLAYER_LIST_MENU_DIALOG_BUTTON_HEIGHT,
    PLAYER_LIST_MENU_DIALOG_BUTTON_WIDTH, PLAYER_LIST_ROW_HEIGHT, PLAYER_LIST_TEAM_BUTTON_SIZE,
    PLAYER_LIST_WIDTH,
};
