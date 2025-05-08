use iced::{mouse, widget::{button, canvas, column, container, pick_list, row, text, text_input}, Element, Size};
use iced_aw::number_input;

use crate::{message::Message, state::{ColorIdx, EditorState}};

use super::modal_background_style;

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
            canvas::Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::ButtonPressed(button) => {
                    let message = match button {
                        mouse::Button::Left => {
                            Some(Message::ClickColor(self.color_idx))
                        }
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

        let thickness = 1.0;
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

        if self.selected {
            let border_color = if theme.extended_palette().is_dark {
                iced::Color::WHITE
            } else {
                iced::Color::BLACK
            };
            let size = Size {
                width: frame.size().width - thickness,
                height: frame.size().height - thickness - 1.0,
            };
            frame.stroke_rectangle(
                iced::Point { x: thickness / 2.0, y: thickness / 2.0 },
                size,
                canvas::Stroke {
                    width: thickness,
                    style: border_color.into(),
                    ..Default::default()
                }
            );
        }

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

pub fn palette_view(state: &EditorState) -> Element<Message> {
    let palette_names: Vec<String> = state.palettes.iter().map(|x| x.name.clone()).collect();
    let selected_palette_name = state.palettes[state.palette_idx].name.clone();

    let mut colors_row = iced::widget::Row::new();
    let pal = &state.palettes[state.palette_idx];
    let size = 25.0;
    for i in 0..16 {
        colors_row = colors_row.push(
            canvas(ColorBox {
                r: pal.colors[i].0 as f32 / 31.0,
                g: pal.colors[i].1 as f32 / 31.0,
                b: pal.colors[i].2 as f32 / 31.0,
                selected: Some(i as ColorIdx) == state.color_idx,
                brush_mode: state.brush_mode,
                color_idx: i as ColorIdx,
            })
            .width(size)
            .height(size),
        );
    }

    let rgb_width = 80;
    let mut col = column![
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
    ]
    .spacing(5);

    if state.color_idx.is_some() {
        col = col.push(
            row![
                text("Red"),
                number_input(
                    &state.selected_color.0,
                    0..=31,
                    Message::ChangeRed
                )
                .width(rgb_width),
                iced::widget::Space::with_width(10),
                text("Green"),
                number_input(
                    &state.selected_color.1,
                    0..=31,
                    Message::ChangeGreen
                )
                .width(rgb_width),
                iced::widget::Space::with_width(10),
                text("Blue"),
                number_input(
                    &state.selected_color.2,
                    0..=31,
                    Message::ChangeBlue
                )
                .width(rgb_width),
            ]
            .spacing(5)
            .align_y(iced::alignment::Vertical::Center),
        );
    }

    row![col].padding(10).into()
}

pub fn add_palette_view(name: &String) -> Element<Message> {
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

pub fn rename_palette_view(state: &EditorState, name: &String) -> Element<'static, Message> {
    let idx = state.palette_idx;
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

pub fn delete_palette_view(state: &EditorState) -> Element<Message> {
    let idx = state.palette_idx;
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