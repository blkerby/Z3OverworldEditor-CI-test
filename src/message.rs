use crate::common::{ColorIdx, ColorValue};

#[derive(Debug, Clone)]
pub enum Message {
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
