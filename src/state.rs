use anyhow::{Context, Result};
use log::info;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::persist;

pub type ColorValue = u8; // Color value (0-31)
pub type ColorIdx = u8; // Index into 4bpp palette (0-15)
pub type TileIdx = u16; // Index into palette's tile list

pub type ColorRGB = (ColorValue, ColorValue, ColorValue);

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct Palette {
    #[serde(skip_serializing, skip_deserializing)]
    pub modified: bool,
    pub name: String,
    pub colors: [ColorRGB; 16],
    pub tiles: Vec<[[ColorIdx; 8]; 8]>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct GlobalConfig {
    #[serde(skip_serializing, skip_deserializing)]
    pub modified: bool,
    pub project_dir: Option<PathBuf>,
}

// struct Screen {
//     theme: String,
//     world: String,
//     name: String,
//     position_x: Option<u8>,
//     position_y: Option<u8>,
//     size_x: u8,
//     size_y: u8,
//     palettes: Vec<String>,  // palette names used in this screen
//     tiles: Vec<TileIdx>,
// }

// #[derive(Default)]
// struct ScreenState {
//     selected_theme_idx: usize,
//     selected_world_idx: usize,
//     selected_screen_idx: usize,
// }

pub enum Dialogue {
    AddPalette { name: String },
    RenamePalette { name: String },
    DeletePalette,
}

pub struct EditorState {
    pub global_config_path: PathBuf,
    pub global_config: GlobalConfig,

    // Project data:
    pub palettes: Vec<Palette>,
    // screens: Vec<Screen>,

    // General editing state:
    pub brush_mode: bool,
    
    // Palette editing state:
    pub palette_idx: usize,
    pub color_idx: Option<ColorIdx>,
    pub selected_color: ColorRGB,

    // Tile editing state:
    pub tile_idx: Option<TileIdx>,

    // Other editor state:
    pub dialogue: Option<Dialogue>,
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
        brush_mode: false,
        palette_idx: 0,
        color_idx: None,
        selected_color: (0, 0, 0),
        tile_idx: None,
        dialogue: None,
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
