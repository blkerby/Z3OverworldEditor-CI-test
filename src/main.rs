use iced::Theme;
use state::EditorState;

mod common;
mod message;
mod state;
mod update;
mod view;

fn theme(_state: &EditorState) -> Theme {
    match dark_light::detect().unwrap_or(dark_light::Mode::Unspecified) {
        dark_light::Mode::Light => Theme::Light,
        dark_light::Mode::Dark | dark_light::Mode::Unspecified => Theme::Dark,
    }    
}

pub fn main() -> iced::Result {
    iced::application("Z3 Overworld Editor", update::update, view::view)
        .font(iced_fonts::REQUIRED_FONT_BYTES)
        .font(iced_fonts::BOOTSTRAP_FONT_BYTES)
        .theme(theme)
        .run()
}
