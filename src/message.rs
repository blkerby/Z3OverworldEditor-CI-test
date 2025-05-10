use std::path::PathBuf;

use iced::Point;

use crate::state::{ColorIdx, ColorValue, PixelCoord, TileCoord, TileIdx};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectionSource {
    MainScreen,
    Tileset,
}

#[derive(Debug, Clone)]
pub enum Message {
    Event(iced::Event),
    WindowClose(iced::window::Id),
    SaveProject,
    OpenProject,
    ProjectOpened(Option<PathBuf>),
    SettingsDialogue,
    SetPixelSize(f32),
    CloseDialogue,
    SelectPalette(String),
    AddPaletteDialogue,
    SetAddPaletteName(String),
    SetAddPaletteID(u8),
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
    // ClickTile(TileIdx),
    TilesetBrush(Point<TileCoord>),
    ClickPixel(PixelCoord, PixelCoord),
    SelectScreen(String),
    AddScreenDialogue,
    SetAddScreenName(String),
    SetAddScreenSizeX(u8),
    SetAddScreenSizeY(u8),
    AddScreen,
    RenameScreenDialogue,
    SetRenameScreenName(String),
    RenameScreen,
    DeleteScreenDialogue,
    DeleteScreen,
    SelectTheme(String),
    AddThemeDialogue,
    SetAddThemeName(String),
    AddTheme,
    RenameThemeDialogue,
    SetRenameThemeName(String),
    RenameTheme,
    DeleteThemeDialogue,
    DeleteTheme,
    StartScreenSelection(Point<TileCoord>, SelectionSource),
    ProgressScreenSelection(Point<TileCoord>),
    EndScreenSelection(Point<TileCoord>),
    ScreenBrush(Point<TileCoord>),
}
