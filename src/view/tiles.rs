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
    message::Message,
    state::{EditorState, Palette, TileIdx},
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

#[derive(Default)]
struct InternalState {
    clicking: bool,
}

impl<'a> canvas::Program<Message> for TileGrid<'a> {
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
            let y = ((p.y - 1.0) / (self.pixel_size * 8.0)) as i32;
            let x = ((p.x - 1.0) / (self.pixel_size * 8.0)) as i32;
            if x < 0 || x >= 16 {
                return (canvas::event::Status::Ignored, None);
            }
            let i = y * 16 + x;
            if i >= 0 && i < self.palette.tiles.len() as i32 {
                let message = Some(Message::ClickTile(i as TileIdx));
                return (canvas::event::Status::Captured, message);
            }
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
    tile_idx: Option<TileIdx>,
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
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let pixel_size = self.pixel_size;
        let thickness = self.thickness;
        let num_cols = 16;

        if let Some(idx) = self.tile_idx {
            let y = (idx / num_cols) as f32 * pixel_size * 8.0 + thickness / 2.0;
            let x = (idx % num_cols) as f32 * pixel_size * 8.0 + thickness / 2.0;
            let border_color = if theme.extended_palette().is_dark {
                iced::Color::WHITE
            } else {
                iced::Color::BLACK
            };
            let size = Size {
                width: 8.0 * pixel_size as f32 + thickness,
                height: 8.0 * pixel_size as f32 + thickness,
            };
            frame.stroke_rectangle(
                iced::Point { x, y },
                size,
                canvas::Stroke {
                    width: thickness,
                    style: border_color.into(),
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
                    tile_idx: state.tile_idx,
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
