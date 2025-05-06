use crate::common::{ColorIdx, ColorValue};

#[derive(Debug, Clone)]
pub enum Message {
    ColorSelectMode,
    ColorBrushMode,
    ClickColor(ColorIdx),
    ChangeRed(ColorValue),
    ChangeGreen(ColorValue),
    ChangeBlue(ColorValue),
}
