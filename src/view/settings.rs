use iced::{
    alignment::Vertical,
    widget::{button, column, container, row, slider, text, text_input},
    Element, Length,
};
use iced_aw::number_input;
use iced_fonts::BOOTSTRAP_FONT;

use crate::{
    message::Message,
    state::{EditorState, MAX_PIXEL_SIZE, MIN_PIXEL_SIZE},
};

use super::modal_background_style;

pub fn settings_view(state: &EditorState) -> Element<Message> {
    let project_dir = state.global_config.project_dir.as_ref().unwrap();
    let zoom_range = MIN_PIXEL_SIZE..=MAX_PIXEL_SIZE;
    container(
        column![
            row![
                text("Project path").width(100),
                text_input("", project_dir.to_str().unwrap()).width(Length::Fill),
                button(text("\u{F3D7}").font(BOOTSTRAP_FONT))
                    .style(button::secondary)
                    .on_press(Message::OpenProject),
            ]
            .spacing(10)
            .align_y(Vertical::Center),
            row![
                text("Zoom").width(100),
                slider(
                    zoom_range.clone(),
                    state.global_config.pixel_size,
                    Message::SetPixelSize
                )
                .width(Length::Fill),
                number_input(
                    &state.global_config.pixel_size,
                    zoom_range.clone(),
                    Message::SetPixelSize
                )
                .width(60),
            ]
            .spacing(10)
            .align_y(Vertical::Center),
            row![button("Close")
                .style(button::secondary)
                .on_press(Message::CloseDialogue)]
        ]
        .spacing(15),
    )
    .width(600)
    .padding(25)
    .style(modal_background_style)
    .into()
}
