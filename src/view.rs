use iced::{keyboard, mouse, widget::{canvas, column, row, slider, text, text_input}};
use iced_aw::number_input;

use crate::{common::ColorIdx, message::Message, state::EditorState};

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
                        (canvas::event::Status::Captured, Some(Message::ColorBrushMode))
                    } else if modified_key == keyboard::Key::Character("s".into()) {
                        (canvas::event::Status::Captured, Some(Message::ColorSelectMode))
                    } else {
                        (canvas::event::Status::Ignored, None)
                    }
                },
                _ => (canvas::event::Status::Ignored, None),
            }
            canvas::Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::ButtonPressed(button) => {
                    let message = match button {
                        mouse::Button::Left => {
                            Some(Message::ClickColor(self.color_idx))
                        }
                        mouse::Button::Right => {
                            None
                        }
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
        _theme: &iced::Theme,
        bounds: iced::Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        // We prepare a new `Frame`
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        frame.fill_rectangle(
            iced::Point { x: 0.0, y: 0.0 },
            frame.size(),
            iced::Color {
                r: self.r,
                g: self.g,
                b: self.b,
                a: 1.0,
            },
        );

        if self.selected {
            frame.stroke_rectangle(
                iced::Point { x: 0.0, y: 0.0 },
                frame.size(),
                canvas::stroke::Stroke {
                    style: canvas::Style::Solid(iced::Color::from_rgb(1.0, 0.3, 1.0)),
                    width: 6.0,
                    ..Default::default()
                }
            );    
        }

        // Then, we produce the geometry
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
    let mut colors_row = iced::widget::Row::new();
    let pal = &state.palettes[state.palette_state.palette_idx];
    let size = 25.0;
    for i in 0..16 {
        colors_row = colors_row.push(
            canvas(ColorBox {
                r: pal.colors[i].red as f32 / 31.0,
                g: pal.colors[i].green as f32 / 31.0,
                b: pal.colors[i].blue as f32 / 31.0,
                selected: i as ColorIdx == state.palette_state.color_idx,
                brush_mode: state.palette_state.brush_mode,
                color_idx: i as ColorIdx,
            })
            .width(size)
            .height(size),
        );
    }

    let rgb_width = 80;
    let rgb_row = row![
        text("Red"),
        number_input(&state.palette_state.red, 0..=31, Message::ChangeRed).width(rgb_width),
        iced::widget::Space::with_width(10),
        text("Green"),
        number_input(&state.palette_state.green, 0..=31, Message::ChangeGreen).width(rgb_width),
        iced::widget::Space::with_width(10),
        text("Blue"),
        number_input(&state.palette_state.blue, 0..=31, Message::ChangeBlue).width(rgb_width),
    ].spacing(5).align_y(iced::alignment::Vertical::Center);
    column![
        colors_row,
        rgb_row
    ].spacing(5).into()
}


pub fn view(state: &EditorState) -> iced::Element<Message> {
    column![
        palette_view(&state),
        // pick_list(&state.test_options[..], Some(&state.selected_option), Message::Select),
        // text(state.selected_option.clone()).size(20),
        // button("Add option").on_press(Message::AddOption),
    ]
    .spacing(10)
    .into()
}
