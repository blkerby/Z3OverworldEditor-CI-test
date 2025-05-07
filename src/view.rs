use std::path::PathBuf;

use iced::{
    keyboard, mouse,
    widget::{
        button, canvas, center, column, container, mouse_area, opaque, pick_list, row, stack, text,
        text_input, Space,
    },
    Element, Length, Size, Theme,
};
use iced_aw::{number_input, quad};

use crate::{
    message::Message,
    state::{ColorIdx, Dialogue, EditorState},
};

pub async fn open_project() -> Option<PathBuf> {
    let picked_dir = rfd::AsyncFileDialog::new()
        .set_title("Select new or existing project folder ...")
        .pick_folder()
        .await;
    picked_dir.map(|x| x.path().to_owned())
}

#[derive(Debug)]
struct ColorBox {
    r: f32,
    g: f32,
    b: f32,
    selected: bool,
    brush_mode: bool,
    color_idx: ColorIdx,
}

impl canvas::Program<Message> for ColorBox {
    // No internal state
    type State = ();

    fn update(
        &self,
        _interaction: &mut Self::State,
        event: canvas::Event,
        bounds: iced::Rectangle,
        cursor: mouse::Cursor,
    ) -> (canvas::event::Status, Option<Message>) {
        if cursor.position_in(bounds).is_none() {
            return (canvas::event::Status::Ignored, None);
        };

        match event {
            canvas::Event::Keyboard(key_event) => match key_event {
                keyboard::Event::KeyPressed { modified_key, .. } => {
                    if modified_key == keyboard::Key::Character("b".into()) {
                        (
                            canvas::event::Status::Captured,
                            Some(Message::ColorBrushMode),
                        )
                    } else if modified_key == keyboard::Key::Character("s".into()) {
                        (
                            canvas::event::Status::Captured,
                            Some(Message::ColorSelectMode),
                        )
                    } else {
                        (canvas::event::Status::Ignored, None)
                    }
                }
                _ => (canvas::event::Status::Ignored, None),
            },
            canvas::Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::ButtonPressed(button) => {
                    let message = match button {
                        mouse::Button::Left => Some(Message::ClickColor(self.color_idx)),
                        mouse::Button::Right => None,
                        _ => None,
                    };

                    (canvas::event::Status::Captured, message)
                }
                _ => (canvas::event::Status::Ignored, None),
            },
            _ => (canvas::event::Status::Ignored, None),
        }
    }

    fn draw(
        &self,
        _state: &(),
        renderer: &iced::Renderer,
        theme: &iced::Theme,
        bounds: iced::Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        if self.selected {
            let border_color = if theme.extended_palette().is_dark {
                iced::Color::WHITE
            } else {
                iced::Color::BLACK
            };
            frame.fill_rectangle(iced::Point { x: 0.0, y: 0.0 }, frame.size(), border_color);
        }

        let thickness = 2.0;
        let size = Size {
            width: frame.size().width - 2.0 * thickness,
            height: frame.size().height - 2.0 * thickness - 1.0,
        };
        frame.fill_rectangle(
            iced::Point {
                x: thickness,
                y: thickness,
            },
            size,
            iced::Color::from_rgb(self.r, self.g, self.b),
        );

        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        _interaction: &Self::State,
        bounds: iced::Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if self.brush_mode && cursor.is_over(bounds) {
            mouse::Interaction::Crosshair
        } else {
            mouse::Interaction::default()
        }
    }
}

fn palette_view(state: &EditorState) -> iced::Element<Message> {
    let palette_names: Vec<String> = state.palettes.iter().map(|x| x.name.clone()).collect();
    let selected_palette_name = state.palettes[state.palette_state.palette_idx].name.clone();

    let mut colors_row = iced::widget::Row::new();
    let pal = &state.palettes[state.palette_state.palette_idx];
    let size = 25.0;
    for i in 0..16 {
        colors_row = colors_row.push(
            canvas(ColorBox {
                r: pal.colors[i].0 as f32 / 31.0,
                g: pal.colors[i].1 as f32 / 31.0,
                b: pal.colors[i].2 as f32 / 31.0,
                selected: Some(i as ColorIdx) == state.palette_state.color_idx,
                brush_mode: state.palette_state.brush_mode,
                color_idx: i as ColorIdx,
            })
            .width(size)
            .height(size),
        );
    }

    let rgb_width = 80;
    let col = column![
        row![
            text("Palette"),
            pick_list(
                palette_names,
                Some(selected_palette_name),
                Message::SelectPalette
            )
            .width(200),
            button(text("\u{F4FA}").font(iced_fonts::BOOTSTRAP_FONT))
                .style(button::success)
                .on_press(Message::AddPaletteDialogue),
            button(text("\u{F4CB}").font(iced_fonts::BOOTSTRAP_FONT))
                .on_press(Message::RenamePaletteDialogue),
            button(text("\u{F5DE}").font(iced_fonts::BOOTSTRAP_FONT))
                .style(button::danger)
                .on_press(Message::DeletePaletteDialogue),
        ]
        .spacing(10)
        .align_y(iced::alignment::Vertical::Center),
        colors_row,
        row![
            text("Red"),
            number_input(
                &state.palette_state.selected_color.0,
                0..=31,
                Message::ChangeRed
            )
            .width(rgb_width),
            iced::widget::Space::with_width(10),
            text("Green"),
            number_input(
                &state.palette_state.selected_color.1,
                0..=31,
                Message::ChangeGreen
            )
            .width(rgb_width),
            iced::widget::Space::with_width(10),
            text("Blue"),
            number_input(
                &state.palette_state.selected_color.2,
                0..=31,
                Message::ChangeBlue
            )
            .width(rgb_width),
        ]
        .spacing(5)
        .align_y(iced::alignment::Vertical::Center)
    ]
    .spacing(5);
    row![col].padding(10).into()
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

fn modal_background_style(theme: &Theme) -> container::Style {
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

fn add_palette_view(name: &String) -> Element<Message> {
    container(
        column![
            text("Select a name for the new palette"),
            text_input("", name)
                .id("AddPalette")
                .on_input(Message::SetAddPaletteName)
                .on_submit(Message::AddPalette)
                .padding(5),
            button(text("Add palette"))
                .style(button::success)
                .on_press(Message::AddPalette),
        ]
        .spacing(10),
    )
    .width(300)
    .padding(20)
    .style(modal_background_style)
    .into()
}

fn rename_palette_view(state: &EditorState, name: &String) -> Element<'static, Message> {
    let idx = state.palette_state.palette_idx;
    let old_name = &state.palettes[idx].name;
    container(
        column![
            text(format!("Rename palette \"{}\"", old_name)),
            text_input("", name)
                .id("RenamePalette")
                .on_input(Message::SetRenamePaletteName)
                .on_submit(Message::RenamePalette)
                .padding(5),
            button(text("Rename palette")).on_press(Message::RenamePalette),
        ]
        .spacing(10),
    )
    .width(300)
    .padding(20)
    .style(modal_background_style)
    .into()
}

fn delete_palette_view(state: &EditorState) -> Element<Message> {
    let idx = state.palette_state.palette_idx;
    let name = &state.palettes[idx].name;
    container(
        column![
            text(format!("Delete palette \"{}\"?", name)),
            text("This will also delete all 8x8 tiles associated to this palette."),
            button(text("Delete palette"))
                .style(button::danger)
                .on_press(Message::DeletePalette),
        ]
        .spacing(10),
    )
    .width(300)
    .padding(20)
    .style(modal_background_style)
    .into()
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

pub fn screen_view(_state: &EditorState) -> iced::Element<Message> {
    Space::with_width(Length::Fill).into()
}

pub fn view(state: &EditorState) -> iced::Element<Message> {
    let mut main_view: Element<Message> = row![
        screen_view(&state),
        vertical_separator(),
        palette_view(&state),
        // text(state.selected_option.clone()).size(20),
        // button("Add option").on_press(Message::AddOption),
    ]
    .spacing(0)
    .width(Length::Fill)
    .height(Length::Fill)
    .into();

    if let Some(dialogue) = &state.dialogue {
        match dialogue {
            Dialogue::AddPalette { name } => {
                main_view = modal(main_view, add_palette_view(name), Message::HideModal);
            }
            Dialogue::DeletePalette => {
                main_view = modal(main_view, delete_palette_view(&state), Message::HideModal);
            }
            Dialogue::RenamePalette { name } => {
                main_view = modal(
                    main_view,
                    rename_palette_view(&state, name),
                    Message::HideModal,
                );
            }
        }
    }
    main_view
}
