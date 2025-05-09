use anyhow::{Context, Result};
use hashbrown::HashMap;
use log::info;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::persist;

pub type ColorValue = u8; // Color value (0-31)
pub type ColorIdx = u8; // Index into 4bpp palette (0-15)
pub type PaletteId = u8; // ID of the palette
pub type TileIdx = u16; // Index into palette's tile list
pub type PixelCoord = u8; // Index into 8x8 row or column (0-7)
pub type TileCoord = u16; // Index into screen: number of 8x8 tiles from top-left corner
pub type ColorRGB = (ColorValue, ColorValue, ColorValue);
pub type Tile = [[ColorIdx; 8]; 8];

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct Palette {
    #[serde(skip_serializing, skip_deserializing)]
    pub modified: bool,
    #[serde(skip_serializing, skip_deserializing)]
    pub name: String,
    pub id: PaletteId,
    pub colors: [ColorRGB; 16],
    pub tiles: Vec<Tile>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct GlobalConfig {
    #[serde(skip_serializing, skip_deserializing)]
    pub modified: bool,
    pub project_dir: Option<PathBuf>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Subscreen {
    // X and Y position of the subscreen within the screen, in subscreen counts
    // The subscreens are always listed in row-major order, so `position` is
    // redundant; its onlu purpose is to improve readability of the JSON.
    pub position: (u8, u8),
    pub palettes: [[PaletteId; 32]; 32],
    pub tiles: [[TileIdx; 32]; 32],
}

#[derive(Serialize, Deserialize, Default)]
pub struct Screen {
    #[serde(skip_serializing, skip_deserializing)]
    pub modified: bool,
    #[serde(skip_serializing, skip_deserializing)]
    pub name: String,
    #[serde(skip_serializing, skip_deserializing)]
    pub theme: String,
    // X and Y dimensions, measured in number of subscreens:
    pub size: (u8, u8),
    // A 'subscreen' is a 256x256 pixel section, roughly the size that fits on camera at once.
    // Splitting it up like this helps with formatting of the JSON, e.g. for viewing git diffs.
    pub subscreens: Vec<Subscreen>,
}

impl Screen {
    pub fn get_subscreen(&self, x: TileCoord, y: TileCoord) -> usize {
        let subscreen_x = (x / 32) as usize;
        let subscreen_y = (y / 32) as usize;
        let subscreen_i = subscreen_y * self.size.0 as usize + subscreen_x;
        subscreen_i
    }

    pub fn get_tile(&self, x: TileCoord, y: TileCoord) -> TileIdx {
        let subscreen_i = self.get_subscreen(x, y);
        self.subscreens[subscreen_i as usize].tiles[(y % 32) as usize][(x % 32) as usize]
    }

    pub fn get_palette(&self, x: TileCoord, y: TileCoord) -> PaletteId {
        let subscreen_i = self.get_subscreen(x, y);
        self.subscreens[subscreen_i as usize].palettes[(y % 32) as usize][(x % 32) as usize]
    }

    pub fn set_tile(&mut self, x: TileCoord, y: TileCoord, tile_idx: TileIdx) {
        let subscreen_i = self.get_subscreen(x, y);
        self.subscreens[subscreen_i as usize].tiles[(y % 32) as usize][(x % 32) as usize] =
            tile_idx;
    }

    pub fn set_palette(&mut self, x: TileCoord, y: TileCoord, palette_id: PaletteId) {
        let subscreen_i = self.get_subscreen(x, y);
        self.subscreens[subscreen_i as usize].palettes[(y % 32) as usize][(x % 32) as usize] =
            palette_id;
    }
}

pub enum Dialogue {
    AddPalette { name: String, id: u8 },
    RenamePalette { name: String },
    DeletePalette,
    AddScreen { name: String, size: (u8, u8) },
    RenameScreen { name: String },
    DeleteScreen,
    AddTheme { name: String },
    RenameTheme { name: String },
    DeleteTheme,
}

#[derive(Default, Debug)]
pub struct TileBlock {
    pub size: (TileCoord, TileCoord),
    pub palettes: Vec<Vec<PaletteId>>,
    pub tiles: Vec<Vec<TileIdx>>,
}

pub struct EditorState {
    pub global_config_path: PathBuf,
    pub global_config: GlobalConfig,

    // Project data:
    pub palettes: Vec<Palette>,
    pub screen: Screen,
    pub screen_names: Vec<String>,
    pub theme_names: Vec<String>,

    // General editing state:
    pub brush_mode: bool,

    // Palette editing state:
    pub palette_idx: usize,
    pub color_idx: Option<ColorIdx>,
    pub selected_color: ColorRGB,

    // Tile editing state:
    pub tile_idx: Option<TileIdx>,
    pub selected_tile: Tile,

    // Graphics editing state:
    pub pixel_coords: Option<(PixelCoord, PixelCoord)>,

    // Screen editing state:
    pub start_coords: Option<(TileCoord, TileCoord)>,
    pub end_coords: Option<(TileCoord, TileCoord)>,
    pub selected_tile_block: TileBlock,

    // Other editor state:
    pub dialogue: Option<Dialogue>,

    // Cached data:
    pub palettes_id_idx_map: HashMap<u8, usize>,
}

fn get_global_config_path() -> Result<PathBuf> {
    let project_dirs = directories::ProjectDirs::from("", "", "Z3OverworldEditor")
        .context("Unable to open global config directory.")?;
    let config_dir = project_dirs.config_dir();
    let config_path = config_dir.join("config.json");
    Ok(config_path)
}

pub fn get_initial_state() -> Result<EditorState> {
    let mut editor_state = EditorState {
        global_config_path: get_global_config_path()?,
        global_config: GlobalConfig::default(),
        palettes: vec![],
        screen: Screen::default(),
        screen_names: vec![],
        theme_names: vec![],
        brush_mode: false,
        palette_idx: 0,
        color_idx: None,
        selected_color: (0, 0, 0),
        tile_idx: None,
        selected_tile: [[0; 8]; 8],
        start_coords: None,
        end_coords: None,
        selected_tile_block: TileBlock::default(),
        pixel_coords: None,
        dialogue: None,
        palettes_id_idx_map: HashMap::new(),
    };
    match persist::load_global_config(&mut editor_state) {
        Ok(_) => {
            persist::load_project(&mut editor_state)?;
        }
        Err(err) => {
            info!("Unable to load global config, using default: {}", err);
        }
    }
    Ok(editor_state)
}

pub fn scale_color(c: u8) -> u8 {
    ((c as u16) * 255 / 31) as u8
}
