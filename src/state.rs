use crate::common::{Color, ColorIdx, ColorValue, Palette};

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

#[derive(Default)]
pub struct PaletteState {
    pub palette_idx: usize,
    pub color_idx: ColorIdx,
    pub red: ColorValue,
    pub green: ColorValue,
    pub blue: ColorValue,
    pub brush_mode: bool,
}

pub struct EditorState {
    // Tiling data:
    pub palettes: Vec<Palette>,
    // tiles: Vec<Tile>,
    // screens: Vec<Screen>,

    // Editor selections:
    // screen_state: ScreenState,
    pub palette_state: PaletteState,
}

impl Default for EditorState {
    fn default() -> Self {
        EditorState {
            palettes: vec![Palette {
                name: "Default".to_string(),
                colors: [Color {
                    red: 0,
                    green: 0,
                    blue: 0,
                }; 16],
            }],
            palette_state: PaletteState {
                palette_idx: 0,
                color_idx: 0,
                red: 0,
                green: 0,
                blue: 0,
                brush_mode: false,
            },
        }
    }
}
