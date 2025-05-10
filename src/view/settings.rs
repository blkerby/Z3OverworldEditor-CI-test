use iced::{
    alignment::Vertical,
    widget::{button, column, container, row, text, text_input},
    Element,
};
use iced_fonts::BOOTSTRAP_FONT;

use crate::{message::Message, state::EditorState};

use super::modal_background_style;

pub fn settings_view(state: &EditorState) -> Element<Message> {
    let project_dir = state.global_config.project_dir.as_ref().unwrap();
    container(
        column![
            row![
                text("Project path").width(100),
                text_input("", project_dir.to_str().unwrap()).width(300),
                button(text("\u{F3D7}").font(BOOTSTRAP_FONT))
                    .style(button::secondary)
                    .on_press(Message::OpenProject),
            ]
            .spacing(10)
            .align_y(Vertical::Center),
            row![
                button("Close").style(button::secondary).on_press(Message::CloseDialogue)
            ]
        ].spacing(10)
    )
    .width(600)
    .padding(25)
    .style(modal_background_style)
    .into()
}
