use crate::state::ColorRGB;

pub fn scale_color(c: u8) -> u8 {
    ((c as u16) * 255 / 31) as u8
}

pub fn alpha_blend(bg: ColorRGB, fg: ColorRGB, alpha: f32) -> ColorRGB {
    let gamma = 2.2;
    let mut out: ColorRGB = [0, 0, 0];
    for i in 0..3 {
        out[i] = f32::powf(
            (1.0 - alpha) * f32::powf(bg[i] as f32, gamma) + alpha * f32::powf(fg[i] as f32, gamma),
            1.0 / gamma,
        ) as u8;
    }
    out
}
