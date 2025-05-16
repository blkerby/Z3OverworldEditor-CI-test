use std::path::PathBuf;

use iced::Point;

use crate::state::{
    AreaId, AreaPosition, CollisionType, ColorIdx, ColorRGB, ColorValue, Focus, Palette, PaletteId,
    PaletteIdx, PixelCoord, Tile, TileBlock, TileCoord, TileIdx,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SelectionSource {
    MainArea,
    SideArea,
    Tileset,
}

#[derive(Debug, Clone)]
pub enum Message {
    Event(iced::Event),
    Focus(Focus),
    WindowClose(iced::window::Id),
    SaveProject,
    OpenProject,
    ProjectOpened(Option<PathBuf>),
    SettingsDialogue,
    SetPixelSize(f32),
    CloseDialogue,
    ImportDialogue,
    ImportConfirm(Option<PathBuf>),
    ImportROMProgress,
    ImportROM,
    SelectPalette(String),
    AddPaletteDialogue,
    SetAddPaletteName(String),
    SetAddPaletteID(PaletteId),
    AddPalette {
        name: String,
        id: PaletteId,
    },
    DeletePaletteDialogue,
    DeletePalette(PaletteId),
    RestorePalette(Palette),
    RenamePaletteDialogue,
    SetRenamePaletteName(String),
    RenamePalette {
        id: PaletteId,
        name: String,
    },
    HideModal,
    SelectColor(PaletteIdx, ColorIdx),
    BrushColor {
        palette_id: PaletteId,
        color_idx: ColorIdx,
        color: ColorRGB,
    },
    ChangeRed(ColorValue),
    ChangeGreen(ColorValue),
    ChangeBlue(ColorValue),
    AddTileRow(PaletteId),
    DeleteTileRow(PaletteId),
    RestoreTileRow(PaletteId, Vec<Tile>),
    SetTilePriority {
        palette_id: PaletteId,
        tile_idx: TileIdx,
        priority: bool,
    },
    SetTileCollision {
        palette_id: PaletteId,
        tile_idx: TileIdx,
        collision: CollisionType,
    },
    SetTileHFlippable {
        palette_id: PaletteId,
        tile_idx: TileIdx,
        h_flippable: bool,
    },
    SetTileVFlippable {
        palette_id: PaletteId,
        tile_idx: TileIdx,
        v_flippable: bool,
    },
    TilesetBrush {
        palette_id: PaletteId,
        coords: Point<TileCoord>,
        selected_gfx: Vec<Vec<Tile>>,
    },
    SelectPixel(PixelCoord, PixelCoord),
    BrushPixel {
        palette_id: PaletteId,
        tile_idx: TileIdx,
        coords: Point<PixelCoord>,
        color_idx: ColorIdx,
    },
    SelectMainArea(String),
    AddAreaDialogue,
    SetAddAreaName(String),
    SetAddAreaSizeX(u8),
    SetAddAreaSizeY(u8),
    AddArea {
        name: String,
        size: (u8, u8),
    },
    EditAreaDialogue,
    SetEditAreaName(String),
    EditArea {
        old_name: String,
        new_name: String,
    },
    EditAreaBGRed(ColorValue),
    EditAreaBGGreen(ColorValue),
    EditAreaBGBlue(ColorValue),
    EditAreaBGColor {
        area_id: AreaId,
        color: ColorRGB,
    },
    DeleteAreaDialogue,
    DeleteArea(String),
    SelectTheme(AreaPosition, String),
    AddThemeDialogue,
    SetAddThemeName(String),
    AddTheme(String),
    RenameThemeDialogue,
    SetRenameThemeName(String),
    RenameTheme {
        old_name: String,
        new_name: String,
    },
    DeleteThemeDialogue,
    DeleteTheme(String),
    StartTileSelection(Point<TileCoord>, SelectionSource),
    ProgressTileSelection(Point<TileCoord>),
    EndTileSelection(Point<TileCoord>),
    AreaBrush {
        position: AreaPosition,
        area_id: AreaId,
        coords: Point<TileCoord>,
        selection: TileBlock,
    },
    OpenTile {
        palette_id: PaletteId,
        tile_idx: TileIdx,
    },
}
