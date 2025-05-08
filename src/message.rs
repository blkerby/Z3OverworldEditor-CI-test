use std::path::PathBuf;

use crate::state::{ColorIdx, ColorValue, PixelCoord, TileIdx};

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
    ClickColor(ColorIdx),
    ChangeRed(ColorValue),
    ChangeGreen(ColorValue),
    ChangeBlue(ColorValue),
    AddTileRow,
    DeleteTileRow,
    ClickTile(TileIdx),
    ClickPixel(PixelCoord, PixelCoord),
    SelectScreen(String),
    AddScreenDialogue,
    RenameScreenDialogue,
}
