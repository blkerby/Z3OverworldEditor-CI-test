// Module for displaying and editing 8x8 graphics pixel-by-pixel
use iced::{
    alignment::Vertical,
    mouse,
    widget::{canvas, checkbox, column, horizontal_space, pick_list, row, text, Column},
    Element, Size,
};
use iced_aw::number_input;

use crate::{
    message::Message,
    state::{ColorRGB, EditorState, Flip, PixelCoord, Tile},
};

#[derive(Debug)]
struct GraphicsBox {
    colors: [ColorRGB; 16],
    tile: Tile,
    pixel_coords: Option<(PixelCoord, PixelCoord)>,
    pixel_size: f32,
    thickness: f32,
    brush_mode: bool,
    color_selected: bool,
}

#[derive(Default)]
struct InternalState {
    clicking: bool,
}

impl canvas::Program<Message> for GraphicsBox {
    // No internal state
    type State = InternalState;

    fn update(
        &self,
        state: &mut Self::State,
        event: canvas::Event,
        bounds: iced::Rectangle,
        cursor: mouse::Cursor,
    ) -> (canvas::event::Status, Option<Message>) {
        let Some(p) = cursor.position_in(bounds) else {
            return (canvas::event::Status::Ignored, None);
        };

        let mut click: bool = false;
        match event {
            canvas::Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    state.clicking = true;
                    click = true;
                }
                mouse::Event::ButtonReleased(mouse::Button::Left) => {
                    state.clicking = false;
                }
                mouse::Event::CursorMoved { .. } => {
                    if state.clicking {
                        click = true;
                    }
                }
                mouse::Event::CursorLeft => {
                    state.clicking = false;
                }
                _ => {}
            },
            _ => {}
        }

        if click {
            let y = (p.y / self.pixel_size) as i32;
            let x = (p.x / self.pixel_size) as i32;
            if x < 0 || x >= 8 || y < 0 || y >= 8 {
                return (canvas::event::Status::Ignored, None);
            }
            if self.brush_mode {
                (
                    canvas::event::Status::Captured,
                    Some(Message::BrushPixel(x as PixelCoord, y as PixelCoord)),
                )
            } else {
                (
                    canvas::event::Status::Captured,
                    Some(Message::SelectPixel(x as PixelCoord, y as PixelCoord)),
                )
            }
        } else {
            (canvas::event::Status::Ignored, None)
        }
    }

    fn draw(
        &self,
        _state: &InternalState,
        renderer: &iced::Renderer,
        theme: &iced::Theme,
        bounds: iced::Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        for y in 0..8 {
            for x in 0..8 {
                let color_idx = self.tile.pixels[y][x];
                let color = self.colors[color_idx as usize];
                let r = color[0] as f32 / 31.0;
                let g = color[1] as f32 / 31.0;
                let b = color[2] as f32 / 31.0;
                frame.fill_rectangle(
                    iced::Point {
                        x: x as f32 * self.pixel_size + self.thickness,
                        y: y as f32 * self.pixel_size + self.thickness,
                    },
                    Size {
                        width: self.pixel_size,
                        height: self.pixel_size,
                    },
                    iced::Color::from_rgb(r, g, b),
                );
            }
        }

        if let Some((x, y)) = self.pixel_coords {
            let border_color = if theme.extended_palette().is_dark {
                iced::Color::WHITE
            } else {
                iced::Color::BLACK
            };
            frame.stroke_rectangle(
                iced::Point {
                    x: x as f32 * self.pixel_size + self.thickness / 2.0,
                    y: y as f32 * self.pixel_size + self.thickness / 2.0,
                },
                Size {
                    width: self.pixel_size + self.thickness,
                    height: self.pixel_size + self.thickness,
                },
                canvas::Stroke {
                    width: self.thickness,
                    style: border_color.into(),
                    ..Default::default()
                },
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
        if self.brush_mode && cursor.is_over(bounds) && self.color_selected {
            mouse::Interaction::Crosshair
        } else {
            mouse::Interaction::default()
        }
    }
}

pub fn graphics_view(state: &EditorState) -> Element<Message> {
    let pal = &state.palettes[state.palette_idx];
    let pal_id = pal.id;
    let mut col: Column<Message> = Column::new()
        .width(400)
        .align_x(iced::alignment::Horizontal::Center);
    if let Some(idx) = state.tile_idx {
        let tile = pal.tiles[idx as usize];
        let label_width = 105;
        col = col.push(row![
            column![
                row![
                    text("Tile number").width(label_width),
                    text(format!("${:02X} ({})", idx, idx)),
                ]
                .align_y(Vertical::Center),
                row![
                    text("Priority").width(label_width),
                    pick_list(
                        ["No", "Yes"],
                        Some(if tile.priority { "Yes" } else { "No" }),
                        move |x| Message::SetTilePriority {
                            palette_id: pal_id,
                            tile_idx: idx,
                            priority: x == "Yes"
                        }
                    )
                    .text_size(12)
                ]
                .align_y(Vertical::Center),
                row![
                    text("Collision").width(label_width),
                    number_input(&tile.collision, 0..=255, move |x| {
                        Message::SetTileCollision {
                            palette_id: pal_id,
                            tile_idx: idx,
                            collision: x,
                        }
                    })
                    .width(60),
                ]
                .align_y(Vertical::Center),
            ]
            .spacing(15)
            .padding(10),
            horizontal_space(),
            canvas(GraphicsBox {
                colors: pal.colors,
                tile,
                pixel_coords: state.pixel_coords,
                pixel_size: 24.0,
                thickness: 1.0,
                brush_mode: state.brush_mode,
                color_selected: state.color_idx.is_some(),
            })
            .width(24 * 8 + 2)
            .height(24 * 8 + 4)
        ]);
    }
    col.into()
}
