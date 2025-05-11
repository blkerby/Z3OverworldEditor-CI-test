mod graphics;
mod palette;
mod screen;
mod settings;
mod tiles;

use std::path::PathBuf;

use graphics::graphics_view;
use iced::{
    widget::{button, center, column, container, mouse_area, opaque, row, stack, text, Space},
    Element, Length, Theme,
};
use iced_aw::quad;
use palette::{add_palette_view, delete_palette_view, palette_view, rename_palette_view};
use screen::{
    add_screen_view, add_theme_view, delete_screen_view, delete_theme_view, rename_screen_view,
    rename_theme_view, screen_controls, screen_grid_view,
};
use settings::settings_view;
use tiles::tile_view;

use crate::{
    import::import_rom_view, message::Message, state::{Dialogue, EditorState}
};

pub async fn open_project() -> Option<PathBuf> {
    let picked_dir = rfd::AsyncFileDialog::new()
        .set_title("Select new or existing project folder ...")
        .pick_folder()
        .await;
    picked_dir.map(|x| x.path().to_owned())
}

pub async fn open_rom() -> Option<PathBuf> {
    let picked_dir = rfd::AsyncFileDialog::new()
        .set_title("Select a ROM ...")
        .add_filter("SNES ROM", &["sfc", "smc"])
        .pick_file()
        .await;
    picked_dir.map(|x| x.path().to_owned())
}

fn modal<'a, Message>(
    base: impl Into<Element<'a, Message>>,
    content: impl Into<Element<'a, Message>>,
    on_blur: Message,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    stack![
        base.into(),
        opaque(
            mouse_area(center(opaque(content)).style(|_theme| {
                container::Style {
                    background: Some(
                        iced::Color {
                            a: 0.8,
                            ..iced::Color::BLACK
                        }
                        .into(),
                    ),
                    ..container::Style::default()
                }
            }))
            .on_press(on_blur)
        )
    ]
    .into()
}

pub fn modal_background_style(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    container::Style {
        background: Some(palette.background.base.color.into()),
        border: iced::border::rounded(4)
            .color(palette.background.weak.color)
            .width(1.0),
        // shadow: iced::Shadow::
        ..container::Style::default()
    }
}

fn vertical_separator() -> quad::Quad {
    quad::Quad {
        quad_color: iced::Color::from([0.5; 3]).into(),
        quad_border: iced::Border {
            radius: iced::border::Radius::new(1.0),
            ..Default::default()
        },
        inner_bounds: iced_aw::widget::InnerBounds::Ratio(1.0, 1.0),
        width: Length::Fixed(1.0),
        ..Default::default()
    }
}

pub fn view(state: &EditorState) -> Element<Message> {
    if state.global_config.project_dir.is_none() {
        return Space::new(Length::Fill, Length::Fill).into();
    }
    let mut main_view: Element<Message> = row![
        column![
            row![
                button(text("\u{F3E2}").font(iced_fonts::BOOTSTRAP_FONT))
                    .style(button::secondary)
                    .on_press(Message::SettingsDialogue),
                screen_controls(state),
            ]
            .spacing(10),
            screen_grid_view(state),
        ]
        .padding(10)
        .spacing(10),
        vertical_separator(),
        column![palette_view(state), graphics_view(state), tile_view(state),].width(420)
    ]
    .spacing(0)
    .width(Length::Fill)
    .height(Length::Fill)
    .into();

    if let Some(dialogue) = &state.dialogue {
        match dialogue {
            Dialogue::Settings => {
                main_view = modal(main_view, settings_view(state), Message::HideModal);
            }
            Dialogue::AddPalette { name, id } => {
                main_view = modal(main_view, add_palette_view(name, *id), Message::HideModal);
            }
            Dialogue::DeletePalette => {
                main_view = modal(main_view, delete_palette_view(state), Message::HideModal);
            }
            Dialogue::RenamePalette { name } => {
                main_view = modal(
                    main_view,
                    rename_palette_view(&state, name),
                    Message::HideModal,
                );
            }
            Dialogue::AddScreen { name, size } => {
                main_view = modal(main_view, add_screen_view(name, *size), Message::HideModal);
            }
            Dialogue::RenameScreen { name } => {
                main_view = modal(
                    main_view,
                    rename_screen_view(state, name),
                    Message::HideModal,
                );
            }
            Dialogue::DeleteScreen => {
                main_view = modal(main_view, delete_screen_view(state), Message::HideModal);
            }
            Dialogue::AddTheme { name } => {
                main_view = modal(main_view, add_theme_view(name), Message::HideModal);
            }
            Dialogue::RenameTheme { name } => {
                main_view = modal(
                    main_view,
                    rename_theme_view(state, name),
                    Message::HideModal,
                );
            }
            Dialogue::DeleteTheme => {
                main_view = modal(main_view, delete_theme_view(state), Message::HideModal);
            }
            Dialogue::ImportROM => {
                main_view = modal(main_view, import_rom_view(state), Message::HideModal);
            }
        }
    }
    main_view
}
