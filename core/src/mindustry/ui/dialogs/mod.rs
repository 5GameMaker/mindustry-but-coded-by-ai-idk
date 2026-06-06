//! Dialog abstractions.

pub mod base_dialog;
pub mod full_text_dialog;
pub mod keybind_dialog;

pub use base_dialog::{
    BaseDialog, DialogAlignment, DialogResizeContext, DialogRuntime, DialogShellLayout,
    DialogState, DialogStyle,
};
pub use full_text_dialog::FullTextDialog;
pub use keybind_dialog::{KeybindDialog, KeybindDialogRow};
