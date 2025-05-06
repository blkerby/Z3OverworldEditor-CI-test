mod common;
mod message;
mod state;
mod update;
mod view;

pub fn main() -> iced::Result {
    iced::application("Z3 Overworld Editor", update::update, view::view)
        .font(iced_fonts::REQUIRED_FONT_BYTES)
        .run()
}
