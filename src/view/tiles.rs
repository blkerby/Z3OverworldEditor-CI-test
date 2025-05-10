// Module for managing the set of 8x8 tiles belonging to a palette.
use iced::{
    mouse,
    widget::{
        button, canvas, column, row,
        scrollable::{Direction, Scrollbar},
        stack, text, Scrollable,
    },
    Element, Length, Point, Rectangle, Size,
};

use crate::{
    message::{Message, SelectionSource},
    state::{EditorState, Palette, TileCoord, TileIdx},
};

// We use two separate canvases: one for drawing the tile raster and one for the tile selection.
// This is to work around a limitation in Iced's rendering pipeline that does not allow drawing
// objects (e.g. rectangles) on top of images within a single canvas.

struct TileGrid<'a> {
    palette: &'a Palette,
    pixel_size: f32,
    thickness: f32,
    brush_mode: bool,
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
enum InternalState {
    #[default]
    None,
    Selecting,
    Brushing,
}

fn clamped_position_in(p: Point, bounds: iced::Rectangle, rows: usize, pixel_size: f32) -> Point<TileCoord> {
    let x = (f32::max(p.x - bounds.x, 0.0) / (8.0 * pixel_size)) as TileCoord;
    let y = (f32::max(p.y - bounds.y, 0.0) / (8.0 * pixel_size)) as TileCoord;
    Point {
        x: x.min(15),
        y: y.min(rows as TileCoord - 1),
    }
}

impl<'a> canvas::Program<Message> for TileGrid<'a> {
    type State = InternalState;

    fn update(
        &self,
        state: &mut Self::State,
        event: canvas::Event,
        bounds: iced::Rectangle,
        cursor: mouse::Cursor,
    ) -> (canvas::event::Status, Option<Message>) {
        match event {
            canvas::Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    if let Some(p) = cursor.position_over(bounds) {
                        if self.brush_mode {
                            *state = InternalState::Brushing;
                            return (
                                canvas::event::Status::Captured,
                                Some(Message::TilesetBrush(clamped_position_in(
                                    p,
                                    bounds,
                                    self.palette.tiles.len() / 16,
                                    self.pixel_size,
                                ))),
                            );
                        } else {
                            *state = InternalState::Selecting;
                            return (
                                canvas::event::Status::Captured,
                                Some(Message::StartScreenSelection(clamped_position_in(
                                    p,
                                    bounds,
                                    self.palette.tiles.len() / 16,
                                    self.pixel_size,
                                ), crate::message::SelectionSource::Tileset)),
                            );
                        }
                    };
                }
                mouse::Event::ButtonReleased(mouse::Button::Left) => {
                    let state0 = *state;
                    *state = InternalState::None;
                    if !self.brush_mode && state0 == InternalState::Selecting {
                        if let Some(p) = cursor.position() {
                            return (
                                canvas::event::Status::Captured,
                                Some(Message::EndScreenSelection(clamped_position_in(
                                    p,
                                    bounds,
                                    self.palette.tiles.len() / 16,
                                    self.pixel_size,
                                ))),
                            );
                        }
                    }
                }
                mouse::Event::CursorMoved { .. } => {
                    if !self.brush_mode && *state == InternalState::Selecting {
                        if let Some(p) = cursor.position() {
                            return (
                                canvas::event::Status::Captured,
                                Some(Message::ProgressScreenSelection(clamped_position_in(
                                    p,
                                    bounds,
                                    self.palette.tiles.len() / 16,
                                    self.pixel_size,
                                ))),
                            );
                        }
                    } else if self.brush_mode && *state == InternalState::Brushing {
                        if let Some(p) = cursor.position() {
                            return (
                                canvas::event::Status::Captured,
                                Some(Message::TilesetBrush(clamped_position_in(
                                    p,
                                    bounds,
                                    self.palette.tiles.len() / 16,
                                    self.pixel_size,
                                ))),
                            );
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        }
        (canvas::event::Status::Ignored, None)
    }

    fn draw(
        &self,
        _state: &InternalState,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: iced::Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let pixel_size = self.pixel_size;
        let thickness = self.thickness;
        fn scale_color(c: u8) -> u8 {
            ((c as u16) * 255 / 31) as u8
        }
        let color_bytes: Vec<[u8; 4]> = self
            .palette
            .colors
            .iter()
            .map(|&(r, g, b)| [scale_color(r), scale_color(g), scale_color(b), 255])
            .collect();

        let tiles = &self.palette.tiles;
        let num_cols = 16;
        let num_rows = (tiles.len() + num_cols - 1) / num_cols;

        let mut data: Vec<u8> = vec![];
        data.reserve_exact(num_rows * num_cols * 64 * 4);
        for y in 0..num_rows * 8 {
            for x in 0..num_cols * 8 {
                let tile_x = x / 8;
                let tile_y = y / 8;
                let pixel_x = x % 8;
                let pixel_y = y % 8;
                let i = tile_y * num_cols + tile_x;
                if i >= tiles.len() {
                    data.extend([0, 0, 0, 0]);
                    continue;
                }
                let tile = &self.palette.tiles[i];
                let color_idx = tile[pixel_y][pixel_x];
                data.extend(color_bytes[color_idx as usize]);
            }
        }

        let image = iced::advanced::image::Image::new(iced::advanced::image::Handle::from_rgba(
            (num_cols * 8) as u32,
            (num_rows * 8) as u32,
            data,
        ))
        .filter_method(iced::widget::image::FilterMethod::Nearest)
        .snap(true);

        frame.draw_image(
            Rectangle::new(
                Point::new(thickness, thickness),
                Size {
                    width: num_cols as f32 * 8.0 * (pixel_size as f32),
                    height: num_rows as f32 * 8.0 * (pixel_size as f32),
                },
            ),
            image,
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

struct TileSelect {
    top: TileCoord,
    bottom: TileCoord,
    left: TileCoord,
    right: TileCoord,
    active: bool,
    pixel_size: f32,
    thickness: f32,
}

impl canvas::Program<Message> for TileSelect {
    // No internal state
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &iced::Renderer,
        theme: &iced::Theme,
        bounds: iced::Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        if !self.active {
            return vec![];
        }
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        let pixel_size = self.pixel_size;

        let x0 = self.left as f32 * pixel_size * 8.0 + 0.5;
        let x1 = (self.right + 1) as f32 * pixel_size * 8.0 - 0.5;
        let y0 = self.top as f32 * pixel_size * 8.0 + 0.5;
        let y1 = (self.bottom + 1) as f32 * pixel_size * 8.0 - 1.0;
        let border_color = if theme.extended_palette().is_dark {
            iced::Color::WHITE
        } else {
            iced::Color::BLACK
        };
        frame.stroke_rectangle(
            iced::Point { x: x0, y: y0 },
            Size {
                width: x1 - x0,
                height: y1 - y0,
            },
            canvas::Stroke {
                width: 1.0,
                style: border_color.into(),
                ..Default::default()
            },
        );
        vec![frame.into_geometry()]
    }

    // fn mouse_interaction(
    //     &self,
    //     _interaction: &Self::State,
    //     bounds: iced::Rectangle,
    //     cursor: mouse::Cursor,
    // ) -> mouse::Interaction {
    //     // if self.brush_mode && cursor.is_over(bounds) && self.exists_selection {
    //     //     mouse::Interaction::Crosshair
    //     // } else {
    //     //     mouse::Interaction::default()
    //     // }
    // }
}

pub fn tile_view(state: &EditorState) -> Element<Message> {
    let num_cols = 16;
    let num_rows = (state.palettes[state.palette_idx].tiles.len() + num_cols - 1) / num_cols;

    let mut left = 0;
    let mut right = 0;
    let mut top = 0;
    let mut bottom = 0;

    match (state.start_coords, state.end_coords) {
        (Some(p0), Some(p1)) => {
            left = p0.0.min(p1.0);
            right = p0.0.max(p1.0);
            top = p0.1.min(p1.1);
            bottom = p0.1.max(p1.1);
        }
        _ => {}
    }

    let col = column![
        row![
            text("Tiles"),
            button(text("\u{F64D}").font(iced_fonts::BOOTSTRAP_FONT))
                .style(button::success)
                .on_press(Message::AddTileRow),
            button(text("\u{F63B}").font(iced_fonts::BOOTSTRAP_FONT))
                .style(button::danger)
                .on_press(Message::DeleteTileRow),
        ]
        .spacing(10)
        .align_y(iced::alignment::Vertical::Center),
        Scrollable::with_direction(
            column![stack![
                canvas(TileGrid {
                    palette: &state.palettes[state.palette_idx],
                    pixel_size: 3.0,
                    thickness: 1.0,
                    brush_mode: state.brush_mode,
                })
                .width(384 + 2)
                .height((num_rows * 8 * 3 + 4) as f32),
                canvas(TileSelect {
                    active: state.selection_source == SelectionSource::Tileset
                        && state.start_coords.is_some()
                        && state.end_coords.is_some(),
                    left,
                    right,
                    top,
                    bottom,
                    pixel_size: 3.0,
                    thickness: 1.0,
                })
                .width(384 + 2)
                .height((num_rows * 8 * 3 + 4) as f32)
            ],],
            Direction::Vertical(Scrollbar::default())
        )
        .width(400)
        .height(Length::Fill),
    ]
    .spacing(5);
    row![col].padding(10).into()
}
