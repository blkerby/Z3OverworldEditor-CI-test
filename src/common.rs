pub type ColorValue = u8; // Color value (0-31)
pub type ColorIdx = u8; // Index into 4bpp palette (0-15)
pub type PaletteIdx = u8; // Index into palette list
pub type TileIdx = u8; // Index into palette's tile list
pub type CollisionValue = u8; // Value representing tile's collision type

#[derive(Copy, Clone, Default)]
pub struct Color {
    pub red: ColorValue,
    pub green: ColorValue,
    pub blue: ColorValue,
}

pub struct Tile {
    pub palette: PaletteIdx,
    pub collision: CollisionValue,
    pub pixels: [[ColorIdx; 8]; 8],
}

pub struct Palette {
    pub name: String,
    pub colors: [Color; 16],
}
