//! File chooser dialog model mirroring upstream `mindustry.ui.dialogs.FileChooser`.

use crate::mindustry::core::FileChooserRequest;

pub const FILE_CHOOSER_HOME_SETTING: &str = "lastDirectory";
pub const FILE_CHOOSER_FILL_PARENT: bool = true;
pub const FILE_CHOOSER_CONT_MARGIN: f32 = -10.0;
pub const FILE_CHOOSER_FILEFIELD_ONLY_FONT_CHARS: bool = false;
pub const FILE_CHOOSER_LOAD_TEXT: &str = "@load";
pub const FILE_CHOOSER_SAVE_TEXT: &str = "@save";
pub const FILE_CHOOSER_CANCEL_TEXT: &str = "@cancel";
pub const FILE_CHOOSER_FILENAME_LABEL: &str = "@filename";
pub const FILE_CHOOSER_NAVIGATION_TOUCHABLE: &str = "Touchable.disabled";
pub const FILE_CHOOSER_FILES_MARGIN_RIGHT: f32 = 10.0;
pub const FILE_CHOOSER_FILES_MARGIN_LEFT: f32 = 3.0;
pub const FILE_CHOOSER_PANE_OVER_SCROLL: (bool, bool) = (false, false);
pub const FILE_CHOOSER_PANE_FADE_SCROLL_BARS: bool = false;
pub const FILE_CHOOSER_NAV_BUTTON_HEIGHT: f32 = 60.0;
pub const FILE_CHOOSER_NAV_BUTTON_PAD_TOP: f32 = 5.0;
pub const FILE_CHOOSER_NAV_BUTTON_UNIFORM: bool = true;
pub const FILE_CHOOSER_NAV_HOME_ICON: &str = "home";
pub const FILE_CHOOSER_NAV_BACK_ICON: &str = "left";
pub const FILE_CHOOSER_NAV_FORWARD_ICON: &str = "right";
pub const FILE_CHOOSER_NAV_UP_ICON: &str = "upOpen";
pub const FILE_CHOOSER_FIELD_HEIGHT: f32 = 40.0;
pub const FILE_CHOOSER_FIELD_PAD_LEFT: f32 = 10.0;
pub const FILE_CHOOSER_FIELD_PAD_TOP: f32 = -2.0;
pub const FILE_CHOOSER_FIELD_PAD_BOTTOM: f32 = 2.0;
pub const FILE_CHOOSER_BOTTOM_BUTTON_HEIGHT: f32 = 60.0;
pub const FILE_CHOOSER_FILE_BUTTON_STYLE: &str = "Styles.flatTogglet";
pub const FILE_CHOOSER_FILE_BUTTON_HEIGHT: f32 = 50.0;
pub const FILE_CHOOSER_FILE_BUTTON_PAD: f32 = 2.0;
pub const FILE_CHOOSER_FILE_BUTTON_PAD_TOP: f32 = 0.0;
pub const FILE_CHOOSER_FILE_BUTTON_PAD_BOTTOM: f32 = 0.0;
pub const FILE_CHOOSER_FILE_ICON_PAD_RIGHT: f32 = 4.0;
pub const FILE_CHOOSER_FILE_ICON_PAD_LEFT: f32 = 4.0;
pub const FILE_CHOOSER_UP_ROW_ICON: &str = "upOpen";
pub const FILE_CHOOSER_FOLDER_ICON: &str = "folder";
pub const FILE_CHOOSER_FILE_ICON: &str = "fileText";
pub const FILE_CHOOSER_LABEL_ALIGNMENT: &str = "Align.left";
pub const FILE_CHOOSER_UP_ROW_COLSPAN: usize = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileChooserEntry {
    pub name: String,
    pub path: String,
    pub directory: bool,
    pub hidden: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FileChooserModel {
    pub title: String,
    pub fill_parent: bool,
    pub cont_margin: f32,
    pub open: bool,
    pub directory: String,
    pub navigation: FileChooserNavigationModel,
    pub file_field: FileChooserFileFieldModel,
    pub file_list: FileChooserFileListModel,
    pub nav_buttons: Vec<FileChooserNavButton>,
    pub bottom_buttons: Vec<FileChooserBottomButton>,
    pub save_filename_row: Option<FileChooserFilenameRow>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FileChooserNavigationModel {
    pub text: String,
    pub touchable: &'static str,
    pub cursor_position: FileChooserNavigationCursor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileChooserNavigationCursor {
    Start,
    End,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FileChooserFileFieldModel {
    pub text: String,
    pub disabled: bool,
    pub only_font_chars: bool,
    pub input_dialog: bool,
    pub height: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FileChooserFileListModel {
    pub margin_right: f32,
    pub margin_left: f32,
    pub pane_overscroll: (bool, bool),
    pub fade_scroll_bars: bool,
    pub scroll_y: f32,
    pub up_row: FileChooserFileRow,
    pub rows: Vec<FileChooserFileRow>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FileChooserFileRow {
    pub name: String,
    pub text: String,
    pub path: String,
    pub icon: &'static str,
    pub directory: bool,
    pub style: &'static str,
    pub height: f32,
    pub pad: f32,
    pub pad_top: f32,
    pub pad_bottom: f32,
    pub icon_pad_right: f32,
    pub icon_pad_left: f32,
    pub label_alignment: &'static str,
    pub colspan: usize,
    pub checked: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FileChooserNavButton {
    pub kind: FileChooserNavButtonKind,
    pub icon: &'static str,
    pub height: f32,
    pub grow_x: bool,
    pub pad_top: f32,
    pub uniform: bool,
    pub disabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileChooserNavButtonKind {
    Home,
    Back,
    Forward,
    Up,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FileChooserBottomButton {
    pub kind: FileChooserBottomButtonKind,
    pub text: &'static str,
    pub grow_x: bool,
    pub height: f32,
    pub disabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileChooserBottomButtonKind {
    Cancel,
    Ok,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FileChooserFilenameRow {
    pub label: &'static str,
    pub field_height: f32,
    pub field_pad_left: f32,
    pub pad_top: f32,
    pub pad_bottom: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileChooserAction {
    HideDialog,
    SelectFile { path: String },
    SetDirectory { path: String },
    SetLastDirectory { path: String },
    PushHistory { path: String },
    UpdateFiles { push: bool },
    SetFileField { text: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileHistory {
    history: Vec<String>,
    index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileChooser {
    pub title: String,
    pub open: bool,
    pub home_directory: String,
    pub directory: String,
    pub last_directory: String,
    pub extension: Option<String>,
    pub filefield: String,
    pub history: FileHistory,
}

impl FileChooserEntry {
    pub fn file(path: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            path: normalize_dialog_path(path.into()),
            name: name.into(),
            directory: false,
            hidden: false,
        }
    }

    pub fn directory(path: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            path: normalize_dialog_path(path.into()),
            name: name.into(),
            directory: true,
            hidden: false,
        }
    }
}

impl Default for FileHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl FileHistory {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            index: 0,
        }
    }

    pub fn push(&mut self, file: impl Into<String>) {
        if self.index != self.history.len() {
            self.history.truncate(self.index);
        }
        self.history.push(normalize_dialog_path(file.into()));
        self.index += 1;
    }

    pub fn back(&mut self) -> Option<String> {
        if !self.can_back() {
            return None;
        }
        self.index -= 1;
        self.history.get(self.index - 1).cloned()
    }

    pub fn forward(&mut self) -> Option<String> {
        if !self.can_forward() {
            return None;
        }
        let file = self.history.get(self.index).cloned();
        self.index += 1;
        file
    }

    pub fn can_forward(&self) -> bool {
        self.index < self.history.len()
    }

    pub fn can_back(&self) -> bool {
        self.index != 1 && self.index > 0
    }

    pub fn entries(&self) -> &[String] {
        &self.history
    }

    pub fn index(&self) -> usize {
        self.index
    }
}

impl FileChooser {
    pub fn new(
        title: impl Into<String>,
        open: bool,
        home_directory: impl Into<String>,
        last_directory: impl Into<String>,
        last_directory_exists: bool,
        extension: Option<impl Into<String>>,
    ) -> Self {
        let home_directory = normalize_dialog_path(home_directory.into());
        let last_directory = normalize_dialog_path(last_directory.into());
        let directory = if last_directory_exists {
            last_directory.clone()
        } else {
            home_directory.clone()
        };
        Self {
            title: title.into(),
            open,
            home_directory,
            directory: directory.clone(),
            last_directory: directory,
            extension: extension.map(|value| normalize_extension(value.into())),
            filefield: String::new(),
            history: FileHistory::new(),
        }
    }

    pub fn from_request(
        request: &FileChooserRequest,
        home_directory: impl Into<String>,
        last_directory: impl Into<String>,
        last_directory_exists: bool,
    ) -> Self {
        Self::new(
            request.title.clone(),
            request.open,
            home_directory,
            last_directory,
            last_directory_exists,
            Some(request.extension.clone()),
        )
    }

    pub fn setup_widgets(
        &mut self,
        entries: &[FileChooserEntry],
        navigation_width: f32,
    ) -> FileChooserModel {
        self.update_files(entries, true, navigation_width)
    }

    pub fn update_files(
        &mut self,
        entries: &[FileChooserEntry],
        push: bool,
        navigation_width: f32,
    ) -> FileChooserModel {
        if push {
            self.history.push(self.directory.clone());
        }
        if self.open {
            self.filefield.clear();
        }
        self.model(entries, navigation_width)
    }

    pub fn click_file(
        &mut self,
        entry: &FileChooserEntry,
        entries_in_new_directory: &[FileChooserEntry],
        navigation_width: f32,
    ) -> (Vec<FileChooserAction>, Option<FileChooserModel>) {
        if entry.directory {
            self.directory = child_path(&self.directory, &entry.name);
            self.last_directory = self.directory.clone();
            let model = self.update_files(entries_in_new_directory, true, navigation_width);
            (
                vec![
                    FileChooserAction::SetDirectory {
                        path: self.directory.clone(),
                    },
                    FileChooserAction::SetLastDirectory {
                        path: self.directory.clone(),
                    },
                    FileChooserAction::UpdateFiles { push: true },
                ],
                Some(model),
            )
        } else {
            self.filefield = entry.name.clone();
            (
                vec![FileChooserAction::SetFileField {
                    text: entry.name.clone(),
                }],
                None,
            )
        }
    }

    pub fn nav_button_plan(
        &mut self,
        kind: FileChooserNavButtonKind,
        entries: &[FileChooserEntry],
        navigation_width: f32,
    ) -> (Vec<FileChooserAction>, Option<FileChooserModel>) {
        match kind {
            FileChooserNavButtonKind::Home => {
                self.directory = self.home_directory.clone();
                self.last_directory = self.directory.clone();
                let model = self.update_files(entries, true, navigation_width);
                (
                    vec![
                        FileChooserAction::SetDirectory {
                            path: self.directory.clone(),
                        },
                        FileChooserAction::SetLastDirectory {
                            path: self.directory.clone(),
                        },
                        FileChooserAction::UpdateFiles { push: true },
                    ],
                    Some(model),
                )
            }
            FileChooserNavButtonKind::Up => {
                self.directory = parent_path(&self.directory);
                let model = self.update_files(entries, true, navigation_width);
                (
                    vec![
                        FileChooserAction::SetDirectory {
                            path: self.directory.clone(),
                        },
                        FileChooserAction::UpdateFiles { push: true },
                    ],
                    Some(model),
                )
            }
            FileChooserNavButtonKind::Back => {
                if let Some(directory) = self.history.back() {
                    self.directory = directory;
                    self.last_directory = self.directory.clone();
                    let model = self.update_files(entries, false, navigation_width);
                    (
                        vec![
                            FileChooserAction::SetDirectory {
                                path: self.directory.clone(),
                            },
                            FileChooserAction::SetLastDirectory {
                                path: self.directory.clone(),
                            },
                            FileChooserAction::UpdateFiles { push: false },
                        ],
                        Some(model),
                    )
                } else {
                    (Vec::new(), None)
                }
            }
            FileChooserNavButtonKind::Forward => {
                if let Some(directory) = self.history.forward() {
                    self.directory = directory;
                    self.last_directory = self.directory.clone();
                    let model = self.update_files(entries, false, navigation_width);
                    (
                        vec![
                            FileChooserAction::SetDirectory {
                                path: self.directory.clone(),
                            },
                            FileChooserAction::SetLastDirectory {
                                path: self.directory.clone(),
                            },
                            FileChooserAction::UpdateFiles { push: false },
                        ],
                        Some(model),
                    )
                } else {
                    (Vec::new(), None)
                }
            }
        }
    }

    pub fn up_row_plan(
        &mut self,
        entries: &[FileChooserEntry],
        navigation_width: f32,
    ) -> (Vec<FileChooserAction>, FileChooserModel) {
        self.directory = parent_path(&self.directory);
        self.last_directory = self.directory.clone();
        let model = self.update_files(entries, true, navigation_width);
        (
            vec![
                FileChooserAction::SetDirectory {
                    path: self.directory.clone(),
                },
                FileChooserAction::SetLastDirectory {
                    path: self.directory.clone(),
                },
                FileChooserAction::UpdateFiles { push: true },
            ],
            model,
        )
    }

    pub fn ok_plan(&self, entries: &[FileChooserEntry]) -> Vec<FileChooserAction> {
        if self.ok_disabled(entries) {
            return Vec::new();
        }
        vec![
            FileChooserAction::SelectFile {
                path: child_path(&self.directory, &self.filefield),
            },
            FileChooserAction::HideDialog,
        ]
    }

    pub fn enter_key_plan(&self, entries: &[FileChooserEntry]) -> Vec<FileChooserAction> {
        self.ok_plan(entries)
    }

    pub fn cancel_plan() -> Vec<FileChooserAction> {
        vec![FileChooserAction::HideDialog]
    }

    fn model(&self, entries: &[FileChooserEntry], navigation_width: f32) -> FileChooserModel {
        let sorted = sorted_file_entries(entries, self.extension.as_deref());
        let ok_disabled = self.ok_disabled(entries);
        FileChooserModel {
            title: self.title.clone(),
            fill_parent: FILE_CHOOSER_FILL_PARENT,
            cont_margin: FILE_CHOOSER_CONT_MARGIN,
            open: self.open,
            directory: self.directory.clone(),
            navigation: FileChooserNavigationModel {
                text: self.directory.clone(),
                touchable: FILE_CHOOSER_NAVIGATION_TOUCHABLE,
                cursor_position: navigation_cursor(&self.directory, navigation_width),
            },
            file_field: FileChooserFileFieldModel {
                text: self.filefield.clone(),
                disabled: self.open,
                only_font_chars: FILE_CHOOSER_FILEFIELD_ONLY_FONT_CHARS,
                input_dialog: !self.open,
                height: FILE_CHOOSER_FIELD_HEIGHT,
            },
            file_list: FileChooserFileListModel {
                margin_right: FILE_CHOOSER_FILES_MARGIN_RIGHT,
                margin_left: FILE_CHOOSER_FILES_MARGIN_LEFT,
                pane_overscroll: FILE_CHOOSER_PANE_OVER_SCROLL,
                fade_scroll_bars: FILE_CHOOSER_PANE_FADE_SCROLL_BARS,
                scroll_y: 0.0,
                up_row: up_row(&self.directory),
                rows: sorted
                    .iter()
                    .map(|entry| file_row(entry, &self.filefield))
                    .collect(),
            },
            nav_buttons: vec![
                nav_button(
                    FileChooserNavButtonKind::Home,
                    FILE_CHOOSER_NAV_HOME_ICON,
                    false,
                ),
                nav_button(
                    FileChooserNavButtonKind::Back,
                    FILE_CHOOSER_NAV_BACK_ICON,
                    !self.history.can_back(),
                ),
                nav_button(
                    FileChooserNavButtonKind::Forward,
                    FILE_CHOOSER_NAV_FORWARD_ICON,
                    !self.history.can_forward(),
                ),
                nav_button(
                    FileChooserNavButtonKind::Up,
                    FILE_CHOOSER_NAV_UP_ICON,
                    false,
                ),
            ],
            bottom_buttons: vec![
                FileChooserBottomButton {
                    kind: FileChooserBottomButtonKind::Cancel,
                    text: FILE_CHOOSER_CANCEL_TEXT,
                    grow_x: true,
                    height: FILE_CHOOSER_BOTTOM_BUTTON_HEIGHT,
                    disabled: false,
                },
                FileChooserBottomButton {
                    kind: FileChooserBottomButtonKind::Ok,
                    text: if self.open {
                        FILE_CHOOSER_LOAD_TEXT
                    } else {
                        FILE_CHOOSER_SAVE_TEXT
                    },
                    grow_x: true,
                    height: FILE_CHOOSER_BOTTOM_BUTTON_HEIGHT,
                    disabled: ok_disabled,
                },
            ],
            save_filename_row: (!self.open).then_some(FileChooserFilenameRow {
                label: FILE_CHOOSER_FILENAME_LABEL,
                field_height: FILE_CHOOSER_FIELD_HEIGHT,
                field_pad_left: FILE_CHOOSER_FIELD_PAD_LEFT,
                pad_top: FILE_CHOOSER_FIELD_PAD_TOP,
                pad_bottom: FILE_CHOOSER_FIELD_PAD_BOTTOM,
            }),
        }
    }

    fn ok_disabled(&self, entries: &[FileChooserEntry]) -> bool {
        if !self.open {
            self.filefield.replace(' ', "").is_empty()
        } else {
            !entries
                .iter()
                .any(|entry| entry.name == self.filefield && !entry.directory)
        }
    }
}

fn nav_button(
    kind: FileChooserNavButtonKind,
    icon: &'static str,
    disabled: bool,
) -> FileChooserNavButton {
    FileChooserNavButton {
        kind,
        icon,
        height: FILE_CHOOSER_NAV_BUTTON_HEIGHT,
        grow_x: true,
        pad_top: FILE_CHOOSER_NAV_BUTTON_PAD_TOP,
        uniform: FILE_CHOOSER_NAV_BUTTON_UNIFORM,
        disabled,
    }
}

fn up_row(directory: &str) -> FileChooserFileRow {
    FileChooserFileRow {
        name: "..".to_string(),
        text: format!("..{directory}"),
        path: parent_path(directory),
        icon: FILE_CHOOSER_UP_ROW_ICON,
        directory: true,
        style: FILE_CHOOSER_FILE_BUTTON_STYLE,
        height: FILE_CHOOSER_FILE_BUTTON_HEIGHT,
        pad: FILE_CHOOSER_FILE_BUTTON_PAD,
        pad_top: FILE_CHOOSER_FILE_BUTTON_PAD_TOP,
        pad_bottom: FILE_CHOOSER_FILE_BUTTON_PAD_BOTTOM,
        icon_pad_right: FILE_CHOOSER_FILE_ICON_PAD_RIGHT,
        icon_pad_left: FILE_CHOOSER_FILE_ICON_PAD_LEFT,
        label_alignment: FILE_CHOOSER_LABEL_ALIGNMENT,
        colspan: FILE_CHOOSER_UP_ROW_COLSPAN,
        checked: false,
    }
}

fn file_row(entry: &FileChooserEntry, filefield: &str) -> FileChooserFileRow {
    FileChooserFileRow {
        name: entry.name.clone(),
        text: escape_file_button_text(&entry.name),
        path: entry.path.clone(),
        icon: if entry.directory {
            FILE_CHOOSER_FOLDER_ICON
        } else {
            FILE_CHOOSER_FILE_ICON
        },
        directory: entry.directory,
        style: FILE_CHOOSER_FILE_BUTTON_STYLE,
        height: FILE_CHOOSER_FILE_BUTTON_HEIGHT,
        pad: FILE_CHOOSER_FILE_BUTTON_PAD,
        pad_top: FILE_CHOOSER_FILE_BUTTON_PAD_TOP,
        pad_bottom: FILE_CHOOSER_FILE_BUTTON_PAD_BOTTOM,
        icon_pad_right: FILE_CHOOSER_FILE_ICON_PAD_RIGHT,
        icon_pad_left: FILE_CHOOSER_FILE_ICON_PAD_LEFT,
        label_alignment: FILE_CHOOSER_LABEL_ALIGNMENT,
        colspan: 2,
        checked: entry.name == filefield,
    }
}

pub fn sorted_file_entries(
    entries: &[FileChooserEntry],
    extension: Option<&str>,
) -> Vec<FileChooserEntry> {
    let mut handles = entries
        .iter()
        .filter(|entry| !entry.name.starts_with('.'))
        .filter(|entry| entry.directory || accepts_extension(&entry.name, extension))
        .cloned()
        .collect::<Vec<_>>();
    handles.sort_by(|left, right| match (left.directory, right.directory) {
        (true, false) => core::cmp::Ordering::Less,
        (false, true) => core::cmp::Ordering::Greater,
        _ => left
            .name
            .to_ascii_lowercase()
            .cmp(&right.name.to_ascii_lowercase())
            .then_with(|| left.name.cmp(&right.name)),
    });
    handles
}

pub fn escape_file_button_text(name: &str) -> String {
    name.replace('[', "[[")
}

fn accepts_extension(name: &str, extension: Option<&str>) -> bool {
    match extension {
        Some(extension) => {
            file_extension(name).is_some_and(|found| found.eq_ignore_ascii_case(extension))
        }
        None => true,
    }
}

fn file_extension(name: &str) -> Option<&str> {
    name.rsplit_once('.')
        .map(|(_, extension)| extension)
        .filter(|extension| !extension.is_empty())
}

fn normalize_extension(extension: String) -> String {
    extension.trim_start_matches('.').to_ascii_lowercase()
}

fn navigation_cursor(text: &str, navigation_width: f32) -> FileChooserNavigationCursor {
    let estimated_width = text.chars().count() as f32 * 8.0;
    if estimated_width < navigation_width {
        FileChooserNavigationCursor::Start
    } else {
        FileChooserNavigationCursor::End
    }
}

fn child_path(directory: &str, child: &str) -> String {
    if directory == "/" {
        format!("/{child}")
    } else {
        format!("{}/{}", directory.trim_end_matches('/'), child)
    }
}

fn parent_path(path: &str) -> String {
    let normalized = normalize_dialog_path(path.to_string());
    if normalized == "/" {
        return normalized;
    }
    normalized
        .rsplit_once('/')
        .map(|(parent, _)| {
            if parent.is_empty() {
                "/".to_string()
            } else {
                parent.to_string()
            }
        })
        .unwrap_or_else(|| "/".to_string())
}

fn normalize_dialog_path(path: String) -> String {
    let path = path.replace('\\', "/");
    if path.is_empty() {
        "/".to_string()
    } else {
        path.trim_end_matches('/').to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entries() -> Vec<FileChooserEntry> {
        vec![
            FileChooserEntry::directory("/home/user/Zeta", "Zeta"),
            FileChooserEntry::directory("/home/user/alpha", "alpha"),
            FileChooserEntry::file("/home/user/beta.txt", "beta.txt"),
            FileChooserEntry::file("/home/user/alpha.msav", "alpha.msav"),
            FileChooserEntry::file("/home/user/Map[1].MSAV", "Map[1].MSAV"),
            FileChooserEntry {
                hidden: true,
                ..FileChooserEntry::file("/home/user/.secret.msav", ".secret.msav")
            },
        ]
    }

    #[test]
    fn setup_widgets_matches_java_constructor_layout_and_open_mode() {
        let request = FileChooserRequest::new(true, "@open", "msav");
        let mut chooser = FileChooser::from_request(&request, "/home/user", "/missing", false);
        let model = chooser.setup_widgets(&entries(), 1000.0);

        assert_eq!(chooser.directory, "/home/user");
        assert_eq!(model.title, "@open");
        assert!(model.fill_parent);
        assert_eq!(model.cont_margin, -10.0);
        assert!(model.open);
        assert_eq!(model.file_field.text, "");
        assert!(model.file_field.disabled);
        assert!(!model.file_field.input_dialog);
        assert!(!model.file_field.only_font_chars);
        assert_eq!(model.navigation.text, "/home/user");
        assert_eq!(model.navigation.touchable, "Touchable.disabled");
        assert_eq!(
            model.navigation.cursor_position,
            FileChooserNavigationCursor::Start
        );
        assert_eq!(
            model
                .nav_buttons
                .iter()
                .map(|button| (button.kind, button.icon, button.height, button.disabled))
                .collect::<Vec<_>>(),
            vec![
                (FileChooserNavButtonKind::Home, "home", 60.0, false),
                (FileChooserNavButtonKind::Back, "left", 60.0, true),
                (FileChooserNavButtonKind::Forward, "right", 60.0, true),
                (FileChooserNavButtonKind::Up, "upOpen", 60.0, false),
            ]
        );
        assert_eq!(model.bottom_buttons[0].text, "@cancel");
        assert_eq!(model.bottom_buttons[1].text, "@load");
        assert!(model.bottom_buttons[1].disabled);
        assert_eq!(model.save_filename_row, None);
        assert_eq!(model.file_list.up_row.text, "../home/user");
        assert_eq!(model.file_list.up_row.icon, "upOpen");
        assert_eq!(model.file_list.up_row.height, 50.0);
    }

    #[test]
    fn update_files_sorts_directories_first_filters_hidden_and_extensions() {
        let mut chooser = FileChooser::new(
            "@save",
            false,
            "/home/user",
            "/home/user",
            true,
            Some("msav"),
        );
        let model = chooser.setup_widgets(&entries(), 10.0);

        assert_eq!(
            model.navigation.cursor_position,
            FileChooserNavigationCursor::End
        );
        assert_eq!(
            model
                .file_list
                .rows
                .iter()
                .map(|row| (row.text.as_str(), row.icon, row.directory))
                .collect::<Vec<_>>(),
            vec![
                ("alpha", "folder", true),
                ("Zeta", "folder", true),
                ("alpha.msav", "fileText", false),
                ("Map[[1].MSAV", "fileText", false),
            ]
        );
        assert_eq!(model.file_list.margin_right, 10.0);
        assert_eq!(model.file_list.margin_left, 3.0);
        assert_eq!(model.file_list.pane_overscroll, (false, false));
        assert!(!model.file_list.fade_scroll_bars);
        assert_eq!(model.save_filename_row.unwrap().label, "@filename");
        assert_eq!(model.bottom_buttons[1].text, "@save");
        assert!(model.bottom_buttons[1].disabled);
    }

    #[test]
    fn file_click_and_ok_plan_follow_open_and_save_status_rules() {
        let mut open = FileChooser::new(
            "@open",
            true,
            "/home/user",
            "/home/user",
            true,
            Some("msav"),
        );
        open.setup_widgets(&entries(), 1000.0);
        let file = entries()
            .into_iter()
            .find(|entry| entry.name == "alpha.msav")
            .unwrap();
        let (actions, model) = open.click_file(&file, &[], 1000.0);

        assert_eq!(
            actions,
            vec![FileChooserAction::SetFileField {
                text: "alpha.msav".into()
            }]
        );
        assert_eq!(model, None);
        assert_eq!(
            open.ok_plan(&[file.clone()]),
            vec![
                FileChooserAction::SelectFile {
                    path: "/home/user/alpha.msav".into()
                },
                FileChooserAction::HideDialog
            ]
        );
        assert_eq!(
            open.enter_key_plan(&[file]),
            open.ok_plan(&[FileChooserEntry::file(
                "/home/user/alpha.msav",
                "alpha.msav"
            )])
        );

        let mut save = FileChooser::new(
            "@save",
            false,
            "/home/user",
            "/home/user",
            true,
            Some("msav"),
        );
        save.filefield = "   ".into();
        assert_eq!(save.ok_plan(&[]), Vec::<FileChooserAction>::new());
        save.filefield = "new-map".into();
        assert_eq!(
            save.ok_plan(&[]),
            vec![
                FileChooserAction::SelectFile {
                    path: "/home/user/new-map".into()
                },
                FileChooserAction::HideDialog
            ]
        );
        assert_eq!(
            FileChooser::cancel_plan(),
            vec![FileChooserAction::HideDialog]
        );
    }

    #[test]
    fn directory_navigation_home_up_row_back_and_forward_match_file_history() {
        let mut chooser = FileChooser::new(
            "@open",
            true,
            "/home/user",
            "/home/user",
            true,
            Some("msav"),
        );
        chooser.setup_widgets(&entries(), 1000.0);

        let alpha = FileChooserEntry::directory("/home/user/alpha", "alpha");
        let (actions, _) = chooser.click_file(&alpha, &[], 1000.0);
        assert_eq!(chooser.directory, "/home/user/alpha");
        assert_eq!(
            actions,
            vec![
                FileChooserAction::SetDirectory {
                    path: "/home/user/alpha".into()
                },
                FileChooserAction::SetLastDirectory {
                    path: "/home/user/alpha".into()
                },
                FileChooserAction::UpdateFiles { push: true }
            ]
        );

        let (actions, _) = chooser.nav_button_plan(FileChooserNavButtonKind::Back, &[], 1000.0);
        assert_eq!(chooser.directory, "/home/user");
        assert_eq!(actions[2], FileChooserAction::UpdateFiles { push: false });

        let (actions, _) = chooser.nav_button_plan(FileChooserNavButtonKind::Forward, &[], 1000.0);
        assert_eq!(chooser.directory, "/home/user/alpha");
        assert_eq!(actions[2], FileChooserAction::UpdateFiles { push: false });

        let (actions, _) = chooser.nav_button_plan(FileChooserNavButtonKind::Up, &[], 1000.0);
        assert_eq!(chooser.directory, "/home/user");
        assert!(
            !actions
                .iter()
                .any(|action| matches!(action, FileChooserAction::SetLastDirectory { .. })),
            "top up button does not call setLastDirectory in Java"
        );

        let (actions, _) = chooser.up_row_plan(&[], 1000.0);
        assert_eq!(chooser.directory, "/home");
        assert!(
            actions
                .iter()
                .any(|action| matches!(action, FileChooserAction::SetLastDirectory { .. })),
            "the '..' row does call setLastDirectory in Java"
        );

        let (actions, _) = chooser.nav_button_plan(FileChooserNavButtonKind::Home, &[], 1000.0);
        assert_eq!(chooser.directory, "/home/user");
        assert_eq!(
            actions[1],
            FileChooserAction::SetLastDirectory {
                path: "/home/user".into()
            }
        );
    }

    #[test]
    fn file_history_truncates_forward_stack_and_uses_java_index_rules() {
        let mut history = FileHistory::new();
        assert!(!history.can_back());
        assert!(!history.can_forward());

        history.push("/a");
        history.push("/b");
        history.push("/c");
        assert!(history.can_back());
        assert!(!history.can_forward());
        assert_eq!(history.back().as_deref(), Some("/b"));
        assert_eq!(history.index(), 2);
        assert!(history.can_forward());
        history.push("/d");
        assert_eq!(
            history.entries(),
            &["/a".to_string(), "/b".to_string(), "/d".to_string()]
        );
        assert!(!history.can_forward());
        assert_eq!(history.back().as_deref(), Some("/b"));
        assert_eq!(history.back().as_deref(), Some("/a"));
        assert!(!history.can_back());
        assert_eq!(history.forward().as_deref(), Some("/b"));
    }
}
