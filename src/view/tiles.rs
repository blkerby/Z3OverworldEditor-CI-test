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
    helpers::{alpha_blend, scale_color},
    message::{Message, SelectionSource},
    state::{ColorIdx, EditorState, Palette, TileCoord},
};

// We use two separate canvases: one for drawing the tile raster and one for the tile selection.
// This is to work around a limitation in Iced's rendering pipeline that does not allow drawing
// objects (e.g. rectangles) on top of images within a single canvas.

struct TileGrid<'a> {
    palette: &'a Palette,
    pixel_size: f32,
    end_coords: Option<(TileCoord, TileCoord)>,
    thickness: f32,
    brush_mode: bool,
    identify_color: bool,
    color_idx: Option<ColorIdx>,
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
enum InternalStateAction {
    #[default]
    None,
    Selecting,
    Brushing,
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
struct InternalState {
    action: InternalStateAction,
    coords: Option<Point<TileCoord>>,
}

fn clamped_position_in(
    p: Point,
    bounds: iced::Rectangle,
    rows: usize,
    pixel_size: f32,
) -> Point<TileCoord> {
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
        if let Some(p) = cursor.position_over(bounds) {
            state.coords = Some(clamped_position_in(
                p,
                bounds,
                self.palette.tiles.len() / 16,
                self.pixel_size,
            ));
        }
        match event {
            canvas::Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    if let Some(p) = cursor.position_over(bounds) {
                        if self.brush_mode {
                            state.action = InternalStateAction::Brushing;
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
                            state.action = InternalStateAction::Selecting;
                            return (
                                canvas::event::Status::Captured,
                                Some(Message::StartTileSelection(
                                    clamped_position_in(
                                        p,
                                        bounds,
                                        self.palette.tiles.len() / 16,
                                        self.pixel_size,
                                    ),
                                    crate::message::SelectionSource::Tileset,
                                )),
                            );
                        }
                    };
                }
                mouse::Event::ButtonReleased(mouse::Button::Left) => {
                    let state0 = *state;
                    state.action = InternalStateAction::None;
                    if !self.brush_mode && state0.action == InternalStateAction::Selecting {
                        let coords = if let Some(p) = cursor.position() {
                            clamped_position_in(
                                p,
                                bounds,
                                self.palette.tiles.len() / 16,
                                self.pixel_size,
                            )
                        } else if let Some(c) = self.end_coords {
                            Point::new(c.0, c.1)
                        } else {
                            return (canvas::event::Status::Ignored, None);
                        };
                        return (
                            canvas::event::Status::Captured,
                            Some(Message::EndTileSelection(coords)),
                        );
                    }
                }
                mouse::Event::CursorMoved { .. } => {
                    if !self.brush_mode && state.action == InternalStateAction::Selecting {
                        if let Some(p) = cursor.position() {
                            return (
                                canvas::event::Status::Captured,
                                Some(Message::ProgressTileSelection(clamped_position_in(
                                    p,
                                    bounds,
                                    self.palette.tiles.len() / 16,
                                    self.pixel_size,
                                ))),
                            );
                        }
                    } else if self.brush_mode && state.action == InternalStateAction::Brushing {
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
        let color_bytes: Vec<[u8; 3]> = self
            .palette
            .colors
            .iter()
            .map(|&[r, g, b]| [scale_color(r), scale_color(g), scale_color(b)])
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
                let tile_idx = tile_y * num_cols + tile_x;
                if tile_idx >= tiles.len() {
                    data.extend([0, 0, 0, 0]);
                    continue;
                }
                let tile = &self.palette.tiles[tile_idx];
                let color_idx = tile.pixels[pixel_y][pixel_x];
                let mut color = color_bytes[color_idx as usize];
                if self.identify_color && self.color_idx == Some(color_idx) {
                    let alpha = 0.5;
                    let pink_highlight = [255, 105, 180];
                    color = alpha_blend(color, pink_highlight, alpha);
                }
                data.extend(&color);
                data.push(255); // alpha channel
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
        _theme: &iced::Theme,
        bounds: iced::Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        if !self.active {
            return vec![];
        }
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        let pixel_size = self.pixel_size;

        let x0 = self.left as f32 * pixel_size * 8.0 + self.thickness * 0.5;
        let x1 = (self.right + 1) as f32 * pixel_size * 8.0 + self.thickness * 0.5;
        let y0 = self.top as f32 * pixel_size * 8.0 + self.thickness * 0.5;
        let y1 = (self.bottom + 1) as f32 * pixel_size * 8.0 + self.thickness * 0.5;
        let path = canvas::Path::rectangle(
            iced::Point { x: x0, y: y0 },
            Size {
                width: x1 - x0,
                height: y1 - y0,
            },
        );
        for i in 0..2 {
            frame.stroke(
                &path,
                canvas::Stroke {
                    style: if i == 0 {
                        canvas::stroke::Style::Solid(iced::Color::WHITE)
                    } else {
                        canvas::stroke::Style::Solid(iced::Color::BLACK)
                    },
                    width: self.thickness,
                    line_dash: canvas::LineDash {
                        offset: i,
                        segments: &[0.0, 0.0, 4.0, 4.0],
                    },
                    ..Default::default()
                },
            );
        }
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
    let pixel_size = 3;

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
                    pixel_size: pixel_size as f32,
                    end_coords: state.end_coords,
                    thickness: 1.0,
                    brush_mode: state.brush_mode,
                    identify_color: state.identify_color,
                    color_idx: state.color_idx,
                })
                .width(384 + 4)
                .height((num_rows * 8 * pixel_size + 4) as f32),
                canvas(TileSelect {
                    active: state.selection_source == SelectionSource::Tileset
                        && state.start_coords.is_some()
                        && state.end_coords.is_some()
                        && !state.brush_mode,
                    left,
                    right,
                    top,
                    bottom,
                    pixel_size: pixel_size as f32,
                    thickness: 1.0,
                })
                .width(384 + 4)
                .height((num_rows * 8 * pixel_size + 4) as f32)
            ],],
            Direction::Vertical(Scrollbar::default())
        )
        .width(420)
        .height(Length::Fill),
    ]
    .spacing(5);
    row![col].padding(10).into()
}
