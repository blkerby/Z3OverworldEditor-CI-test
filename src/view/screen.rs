use iced::{widget::{button, column, pick_list, row, text, Space}, Element, Length};

use crate::{message::Message, state::EditorState};

pub fn screen_view(state: &EditorState) -> Element<Message> {
    let mut col = column![
        row![
            text("Screen"),
            pick_list(
                state.screen_names.clone(),
                Some(state.screen.name.clone()),
                Message::SelectScreen
            )
            .width(200),
            button(text("\u{F4FA}").font(iced_fonts::BOOTSTRAP_FONT))
                .style(button::success)
                .on_press(Message::AddScreenDialogue),
            button(text("\u{F4CB}").font(iced_fonts::BOOTSTRAP_FONT))
                .on_press(Message::RenameScreenDialogue),
        ]
        .spacing(10)
        .align_y(iced::alignment::Vertical::Center),
    ]
    .spacing(5).width(Length::Fill);

    col.into()
}
