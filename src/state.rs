use anyhow::{Context, Result};
use hashbrown::HashMap;
use log::info;
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{message::SelectionSource, persist};

pub type ColorValue = u8; // Color value (0-31)
pub type ColorIdx = u8; // Index into 4bpp palette (0-15)
pub type PaletteId = u16; // ID of the palette
pub type TileIdx = u16; // Index into palette's tile list
pub type PixelCoord = u8; // Index into 8x8 row or column (0-7)
pub type TileCoord = u16; // Index into area: number of 8x8 tiles from top-left corner
pub type CollisionType = u8;
pub type ColorRGB = [ColorValue; 3];

#[derive(Copy, Clone, Serialize, Deserialize, Default, Debug, PartialEq, Eq, Hash)]
pub struct Tile {
    pub priority: bool,
    pub collision: CollisionType,
    pub h_flippable: bool,
    pub v_flippable: bool,
    pub pixels: [[ColorIdx; 8]; 8],
}

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
    #[serde(default = "default_pixel_size")]
    pub pixel_size: f32,
}

pub const MIN_PIXEL_SIZE: f32 = 1.0;
pub const MAX_PIXEL_SIZE: f32 = 8.0;

fn default_pixel_size() -> f32 {
    3.0
}

#[derive(Clone, Copy, Serialize_repr, Deserialize_repr, Default, Debug)]
#[repr(u8)]
pub enum Flip {
    #[default]
    None = 0,
    Horizontal = 1,
    Vertical = 2,
    Both = 3,
}

impl Flip {
    pub fn flip_horizontally(self) -> Self {
        match self {
            Flip::None => Flip::Horizontal,
            Flip::Horizontal => Flip::None,
            Flip::Vertical => Flip::Both,
            Flip::Both => Flip::Vertical,
        }
    }

    pub fn flip_vertically(self) -> Self {
        match self {
            Flip::None => Flip::Vertical,
            Flip::Horizontal => Flip::Both,
            Flip::Vertical => Flip::None,
            Flip::Both => Flip::Horizontal,
        }
    }

    pub fn apply_to_pixels(self, mut pixels: [[ColorIdx; 8]; 8]) -> [[ColorIdx; 8]; 8] {
        match self {
            Flip::None => {}
            Flip::Horizontal => {
                for row in pixels.iter_mut() {
                    row.reverse();
                }
            }
            Flip::Vertical => {
                pixels.reverse();
            }
            Flip::Both => {
                for row in pixels.iter_mut() {
                    row.reverse();
                }
                pixels.reverse();
            }
        }
        pixels
    }

    pub fn apply_to_tile(self, mut tile: Tile) -> Tile {
        // TODO: also apply flips to slope collisions
        tile.pixels = self.apply_to_pixels(tile.pixels);
        tile
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct Screen {
    // X and Y position of the screen (256 x 256 block) within the area, in screen counts:
    // The screens are always listed in row-major order, so `position` is
    // redundant; its only purpose is to improve readability of the JSON.
    pub position: (u8, u8),
    pub palettes: [[PaletteId; 32]; 32],
    pub tiles: [[TileIdx; 32]; 32],
    pub flips: [[Flip; 32]; 32],
}

#[derive(Serialize, Deserialize, Default)]
pub struct Area {
    #[serde(skip_serializing, skip_deserializing)]
    pub modified: bool,
    #[serde(skip_serializing, skip_deserializing)]
    pub name: String,
    #[serde(skip_serializing, skip_deserializing)]
    pub theme: String,
    pub bg_color: ColorRGB,
    // X and Y dimensions, measured in number of screens:
    pub size: (u8, u8),
    // A 'screen' is a 256x256 pixel section, roughly the size that fits on camera at once.
    // Splitting it up like this helps with formatting of the JSON, e.g. for viewing git diffs.
    pub screens: Vec<Screen>,
}

impl Area {
    pub fn get_screen_coords(&self, x: TileCoord, y: TileCoord) -> (usize, usize, usize) {
        let screen_x = (x / 32) as usize;
        let screen_y = (y / 32) as usize;
        let screen_i = screen_y * self.size.0 as usize + screen_x;
        (screen_i, (x % 32) as usize, (y % 32) as usize)
    }

    pub fn get_palette(&self, x: TileCoord, y: TileCoord) -> PaletteId {
        let (i, sx, sy) = self.get_screen_coords(x, y);
        self.screens[i].palettes[sy][sx]
    }

    pub fn get_tile(&self, x: TileCoord, y: TileCoord) -> TileIdx {
        let (i, sx, sy) = self.get_screen_coords(x, y);
        self.screens[i].tiles[sy][sx]
    }

    pub fn get_flip(&self, x: TileCoord, y: TileCoord) -> Flip {
        let (i, sx, sy) = self.get_screen_coords(x, y);
        self.screens[i].flips[sy][sx]
    }

    pub fn set_tile(&mut self, x: TileCoord, y: TileCoord, tile_idx: TileIdx) {
        if x >= self.size.0 as TileCoord * 32 || y >= self.size.1 as TileCoord * 32 {
            return;
        }
        let (i, sx, sy) = self.get_screen_coords(x, y);
        self.screens[i].tiles[sy][sx] = tile_idx;
    }

    pub fn set_palette(&mut self, x: TileCoord, y: TileCoord, palette_id: PaletteId) {
        if x >= self.size.0 as TileCoord * 32 || y >= self.size.1 as TileCoord * 32 {
            return;
        }
        let (i, sx, sy) = self.get_screen_coords(x, y);
        self.screens[i].palettes[sy][sx] = palette_id;
    }

    pub fn set_flip(&mut self, x: TileCoord, y: TileCoord, flip: Flip) {
        if x >= self.size.0 as TileCoord * 32 || y >= self.size.1 as TileCoord * 32 {
            return;
        }
        let (i, sx, sy) = self.get_screen_coords(x, y);
        self.screens[i].flips[sy][sx] = flip;
    }
}

pub enum Dialogue {
    Settings,
    ImportROMConfirm,
    ImportROMProgress,
    AddPalette { name: String, id: PaletteId },
    RenamePalette { name: String },
    DeletePalette,
    AddArea { name: String, size: (u8, u8) },
    EditArea { name: String },
    DeleteArea,
    AddTheme { name: String },
    RenameTheme { name: String },
    DeleteTheme,
}

#[derive(Default, Debug)]
pub struct TileBlock {
    pub size: (TileCoord, TileCoord),
    pub palettes: Vec<Vec<PaletteId>>,
    pub tiles: Vec<Vec<TileIdx>>,
    pub flips: Vec<Vec<Flip>>,
}

// At the moment, Iced's support for tracking widget focus is fairly incomplete,
// so we handle it manually. This is used to determine the behavior of
// keyboard inputs (e.g. arrow keys to move through pick-lists or navigate grids).
#[derive(Copy, Clone, Default, Debug)]
pub enum Focus {
    #[default]
    None,
    MainPickArea,
    MainPickTheme,
    MainArea,
    PickPalette,
    PaletteColor,
    GraphicsPixel,
    TilesetTile,
}

pub struct EditorState {
    pub global_config_path: PathBuf,
    pub global_config: GlobalConfig,

    // Project data:
    pub palettes: Vec<Palette>,
    pub area: Area,
    pub area_names: Vec<String>,
    pub theme_names: Vec<String>,

    // Settings-related data:
    pub rom_path: Option<PathBuf>,

    // General editing state:
    pub focus: Focus,
    pub brush_mode: bool,

    // Palette editing state:
    pub palette_idx: usize,
    pub color_idx: Option<ColorIdx>,
    pub selected_color: ColorRGB,
    pub identify_color: bool,

    // Tile editing state:
    pub tile_idx: Option<TileIdx>,
    pub selected_tile: Tile,
    pub identify_tile: bool,

    // Graphics editing state:
    pub pixel_coords: Option<(PixelCoord, PixelCoord)>,

    // Area editing state:
    pub selection_source: SelectionSource,
    pub start_coords: Option<(TileCoord, TileCoord)>,
    pub end_coords: Option<(TileCoord, TileCoord)>,
    pub selected_tile_block: TileBlock,
    pub selected_gfx: Vec<Vec<Tile>>,

    // Other editor state:
    pub dialogue: Option<Dialogue>,

    // Cached data:
    pub palettes_id_idx_map: HashMap<PaletteId, usize>,
}

fn get_global_config_path() -> Result<PathBuf> {
    let project_dirs = directories::ProjectDirs::from("", "", "Z3OverworldEditor")
        .context("Unable to open global config directory.")?;
    let config_dir = project_dirs.config_dir();
    let config_path = config_dir.join("config.json");
    Ok(config_path)
}

pub fn ensure_themes_non_empty(state: &mut EditorState) {
    if state.theme_names.len() == 0 {
        state.theme_names.push("Base".to_string());
    }
}

pub fn ensure_areas_non_empty(state: &mut EditorState) {
    if state.area_names.len() == 0 {
        state.area_names.push("Example".to_string());
        state.area.name = "Example".to_string();
        state.area.theme = "Base".to_string();
        state.area.size = (2, 2);
        for y in 0..2 {
            for x in 0..2 {
                state.area.screens.push(Screen {
                    position: (x, y),
                    palettes: [[0; 32]; 32],
                    tiles: [[0; 32]; 32],
                    flips: [[Flip::None; 32]; 32],
                });
            }
        }
        state.area.modified = true;
    }
}

pub fn ensure_palettes_non_empty(state: &mut EditorState) {
    if state.palettes.len() == 0 {
        let mut pal = Palette::default();
        pal.modified = true;
        pal.name = "Default".to_string();
        pal.tiles = vec![
            Tile {
                priority: false,
                collision: 0,
                h_flippable: true,
                v_flippable: true,
                pixels: [[0; 8]; 8]
            };
            16
        ];
        state.palettes.push(pal);
    }
}

pub fn get_initial_state() -> Result<EditorState> {
    let mut state = EditorState {
        global_config_path: get_global_config_path()?,
        global_config: GlobalConfig {
            modified: false,
            project_dir: None,
            pixel_size: 3.0,
        },
        rom_path: None,
        palettes: vec![],
        area: Area::default(),
        area_names: vec![],
        theme_names: vec![],
        brush_mode: false,
        focus: Focus::None,
        palette_idx: 0,
        color_idx: None,
        selected_color: [0, 0, 0],
        identify_color: false,
        tile_idx: None,
        selected_tile: Tile::default(),
        identify_tile: false,
        selection_source: SelectionSource::MainArea,
        start_coords: None,
        end_coords: None,
        selected_tile_block: TileBlock::default(),
        selected_gfx: vec![],
        pixel_coords: None,
        dialogue: None,
        palettes_id_idx_map: HashMap::new(),
    };
    match persist::load_global_config(&mut state) {
        Ok(_) => {
            persist::load_project(&mut state)?;
        }
        Err(err) => {
            info!("Unable to load global config, using default: {}", err);
        }
    }
    ensure_themes_non_empty(&mut state);
    ensure_areas_non_empty(&mut state);
    ensure_palettes_non_empty(&mut state);
    Ok(state)
}

pub fn scale_color(c: u8) -> u8 {
    ((c as u16) * 255 / 31) as u8
}
