//! Dialog abstractions.

pub mod base_dialog;
pub mod full_text_dialog;
pub mod keybind_dialog;
pub mod map_locales_dialog;

pub use base_dialog::{
    BaseDialog, DialogAlignment, DialogResizeContext, DialogRuntime, DialogShellLayout,
    DialogState, DialogStyle,
};
pub use full_text_dialog::FullTextDialog;
pub use keybind_dialog::{KeybindDialog, KeybindDialogRow};
pub use map_locales_dialog::{
    MapLocalesDialog, MapLocalesDialogLocaleAddRow, MapLocalesDialogLocaleEntry,
    MapLocalesDialogLocaleRow, MapLocalesDialogMainCard, MapLocalesDialogPropertyStatus,
    MapLocalesDialogPropertyViewCard, MAP_LOCALES_CARD_WIDTH, MAP_LOCALES_LOCALE_ADD_BUTTON_HEIGHT,
    MAP_LOCALES_LOCALE_ADD_BUTTON_WIDTH, MAP_LOCALES_LOCALE_DELETE_BUTTON_WIDTH,
    MAP_LOCALES_LOCALE_EDIT_BUTTON_WIDTH, MAP_LOCALES_LOCALE_ITEM_WIDTH,
    MAP_LOCALES_MAIN_PROPERTY_COLLAPSE_BUTTON_SIZE, MAP_LOCALES_MAIN_PROPERTY_EDIT_BUTTON_SIZE,
    MAP_LOCALES_MAIN_PROPERTY_FIELD_WIDTH, MAP_LOCALES_MAIN_PROPERTY_REMOVE_BUTTON_SIZE,
    MAP_LOCALES_MAIN_PROPERTY_VALUE_HEIGHT, MAP_LOCALES_MISSING_PLACEHOLDER,
    MAP_LOCALES_PROPERTY_VIEW_ADD_BUTTON_HEIGHT, MAP_LOCALES_PROPERTY_VIEW_ADD_BUTTON_WIDTH,
    MAP_LOCALES_PROPERTY_VIEW_VALUE_HEIGHT,
};
