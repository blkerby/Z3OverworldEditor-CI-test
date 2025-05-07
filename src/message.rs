use std::path::PathBuf;

use crate::state::{ColorIdx, ColorValue};

#[derive(Debug, Clone)]
pub enum Message {
    Event(iced::Event),
    SaveProject,
    WindowClose(iced::window::Id),
    ProjectOpened(Option<PathBuf>),
    SelectPalette(String),
    AddPaletteDialogue,
    SetAddPaletteName(String),
    AddPalette,
    DeletePaletteDialogue,
    DeletePalette,
    RenamePaletteDialogue,
    SetRenamePaletteName(String),
    RenamePalette,
    HideModal,
    ColorSelectMode,
    ColorBrushMode,
    ClickColor(ColorIdx),
    ChangeRed(ColorValue),
    ChangeGreen(ColorValue),
    ChangeBlue(ColorValue),
}
