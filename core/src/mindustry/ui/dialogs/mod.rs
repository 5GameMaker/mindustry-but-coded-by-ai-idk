//! Dialog abstractions.

pub mod base_dialog;
pub mod full_text_dialog;

pub use base_dialog::{
    BaseDialog, DialogAlignment, DialogResizeContext, DialogRuntime, DialogState, DialogStyle,
};
pub use full_text_dialog::FullTextDialog;
