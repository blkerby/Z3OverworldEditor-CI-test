// Module for displaying/editing a screen
use hashbrown::HashMap;
use iced::{
    alignment::Vertical,
    mouse,
    widget::{
        button, canvas, column, container, pick_list, row,
        scrollable::{Direction, Scrollbar},
        stack, text, text_input, Scrollable, Space,
    },
    Element, Length, Padding, Point, Rectangle, Size,
};
use iced_aw::number_input;
use log::info;

use crate::{
    message::{Message, SelectionSource},
    state::{scale_color, EditorState, Palette, PaletteId, Screen, TileBlock, TileCoord},
};

use super::modal_background_style;

// We use two separate canvases: one for drawing the tile raster and one for the tile selection.
// This is to work around a limitation in Iced's rendering pipeline that does not allow drawing
// primitives (e.g. rectangles) on top of images within a single canvas.

struct ScreenGrid<'a> {
    screen: &'a Screen,
    palettes: &'a [Palette],
    palettes_id_idx_map: &'a HashMap<PaletteId, usize>,
    end_coords: Option<(TileCoord, TileCoord)>,
    pixel_size: f32,
    // thickness: f32,
    brush_mode: bool,
    tile_block: &'a TileBlock,
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
    size: (u8, u8),
    pixel_size: f32,
) -> Point<TileCoord> {
    let x = f32::max(p.x - bounds.x - 1.0 - pixel_size / 2.0, 0.0) / (8.0 * pixel_size);
    let y = f32::max(p.y - bounds.y - 1.0 - pixel_size / 2.0, 0.0) / (8.0 * pixel_size);
    Point {
        x: (x as TileCoord).min(size.0 as TileCoord * 32 - 1),
        y: (y as TileCoord).min(size.1 as TileCoord * 32 - 1),
    }
}

impl<'a> canvas::Program<Message> for ScreenGrid<'a> {
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
                self.screen.size,
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
                                Some(Message::ScreenBrush(clamped_position_in(
                                    p,
                                    bounds,
                                    self.screen.size,
                                    self.pixel_size,
                                ))),
                            );
                        } else {
                            state.action = InternalStateAction::Selecting;
                            return (
                                canvas::event::Status::Captured,
                                Some(Message::StartScreenSelection(
                                    clamped_position_in(
                                        p,
                                        bounds,
                                        self.screen.size,
                                        self.pixel_size,
                                    ),
                                    crate::message::SelectionSource::MainScreen,
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
                            clamped_position_in(p, bounds, self.screen.size, self.pixel_size)
                        } else if let Some(c) = self.end_coords {
                            Point::new(c.0, c.1)
                        } else {
                            return (canvas::event::Status::Ignored, None);
                        };
                        return (
                            canvas::event::Status::Captured,
                            Some(Message::EndScreenSelection(coords)),
                        );
                    }
                }
                mouse::Event::CursorMoved { .. } => {
                    if !self.brush_mode && state.action == InternalStateAction::Selecting {
                        if let Some(p) = cursor.position() {
                            return (
                                canvas::event::Status::Captured,
                                Some(Message::ProgressScreenSelection(clamped_position_in(
                                    p,
                                    bounds,
                                    self.screen.size,
                                    self.pixel_size,
                                ))),
                            );
                        }
                    } else if self.brush_mode && state.action == InternalStateAction::Brushing {
                        if let Some(p) = cursor.position() {
                            return (
                                canvas::event::Status::Captured,
                                Some(Message::ScreenBrush(clamped_position_in(
                                    p,
                                    bounds,
                                    self.screen.size,
                                    self.pixel_size,
                                ))),
                            );
                        }
                    }
                }
                // mouse::Event::CursorLeft => {}
                _ => {}
            },
            _ => {}
        }
        (canvas::event::Status::Ignored, None)
    }

    fn draw(
        &self,
        state: &InternalState,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: iced::Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let mut color_bytes: Vec<Vec<[u8; 4]>> = vec![];

        for i in 0..self.palettes.len() {
            let cb = self.palettes[i]
                .colors
                .iter()
                .map(|&(r, g, b)| [scale_color(r), scale_color(g), scale_color(b), 255])
                .collect();
            color_bytes.push(cb);
        }

        // Add a pixel of transparent padding around the image, since Iced's
        // "nearest neighbor" filter results in the edge pixels having the wrong size.
        let num_cols = self.screen.size.1 as usize * 256 + 2;
        let num_rows = self.screen.size.0 as usize * 256 + 2;
        let mut data: Vec<u8> = vec![0; num_rows * num_cols * 4];
        let row_stride = num_cols * 4;
        let col_stride = 4;
        for sy in 0..self.screen.size.1 as usize {
            for sx in 0..self.screen.size.0 as usize {
                let subscreen = &self.screen.subscreens[sy * self.screen.size.0 as usize + sx];
                let subscreen_addr = (sy * 256 + 1) * row_stride + (sx * 256 + 1) * 4;
                for ty in 0..32 {
                    for tx in 0..32 {
                        let palette_id = subscreen.palettes[ty][tx];
                        if let Some(&palette_idx) = self.palettes_id_idx_map.get(&palette_id) {
                            let tile_idx = subscreen.tiles[ty][tx];
                            let tile = self.palettes[palette_idx].tiles[tile_idx as usize];
                            let cb = &color_bytes[palette_idx];
                            let mut tile_addr =
                                subscreen_addr + ty * 8 * row_stride + tx * 8 * col_stride;
                            for py in 0..8 {
                                let mut addr = tile_addr;
                                for px in 0..8 {
                                    let color_idx = tile[py][px];
                                    data[addr..(addr + 4)].copy_from_slice(&cb[color_idx as usize]);
                                    addr += 4;
                                }
                                tile_addr += row_stride;
                            }
                        } else {
                            // TODO: draw some indicator of the broken tile (due to invalid palette reference)
                        }
                    }
                }
            }
        }

        if self.brush_mode {
            // Overlay the block to be pasted/brushed onto the screen:
            if let Some(Point { x: base_x, y: base_y }) = state.coords {
                let base_addr =
                    (base_y * 8 + 1) as usize * row_stride + (base_x * 8 + 1) as usize * col_stride;
                let alpha = 0.75;
                let gamma = 2.2;
                for ty in 0..self.tile_block.size.1 as usize {
                    for tx in 0..self.tile_block.size.0 as usize {
                        let palette_id = self.tile_block.palettes[ty][tx];
                        if let Some(&palette_idx) = self.palettes_id_idx_map.get(&palette_id) {
                            let tile_idx = self.tile_block.tiles[ty][tx];
                            let tile = self.palettes[palette_idx].tiles[tile_idx as usize];
                            let cb = &color_bytes[palette_idx];
                            let mut tile_addr =
                                base_addr + ty * 8 * row_stride + tx * 8 * col_stride;
                            for py in 0..8 {
                                let mut addr = tile_addr;
                                for px in 0..8 {
                                    let color_idx = tile[py][px];
                                    for k in 0..3 {
                                        let old_color_val = data[addr + k] as f32;
                                        let new_color_val = cb[color_idx as usize][k] as f32;
                                        let blended_color_val = f32::powf(
                                            (1.0 - alpha) * f32::powf(old_color_val, gamma)
                                                + alpha * f32::powf(new_color_val, gamma),
                                            1.0 / gamma,
                                        );
                                        data[addr + k] = blended_color_val as u8;
                                    }
                                    addr += 4;
                                }
                                tile_addr += row_stride;
                            }
                        } else {
                            // TODO: draw some indicator of the broken tile (due to invalid palette reference)
                        }
                    }
                }
            }
        }

        let image = iced::advanced::image::Image::new(iced::advanced::image::Handle::from_rgba(
            num_cols as u32,
            num_rows as u32,
            data,
        ))
        .filter_method(iced::widget::image::FilterMethod::Nearest);

        frame.draw_image(
            Rectangle::new(
                Point::new(0.0, 0.0),
                Size {
                    width: num_cols as f32 * self.pixel_size,
                    height: num_rows as f32 * self.pixel_size,
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

struct ScreenSelect {
    top: TileCoord,
    bottom: TileCoord,
    left: TileCoord,
    right: TileCoord,
    active: bool,
    pixel_size: f32,
    brush_mode: bool,
}

impl canvas::Program<Message> for ScreenSelect {
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

        let x0 = self.left as f32 * pixel_size * 8.0 + pixel_size / 2.0;
        let x1 = (self.right + 1) as f32 * pixel_size * 8.0 + pixel_size / 2.0 + 2.0;
        let y0 = self.top as f32 * pixel_size * 8.0 + pixel_size / 2.0;
        let y1 = (self.bottom + 1) as f32 * pixel_size * 8.0 + pixel_size / 2.0 + 2.0;
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
                    width: 1.0,
                    line_dash: canvas::LineDash {
                        offset: i,
                        segments: &[0.0, 0.0, 4.0, 4.0],
                    },
                    line_join: canvas::LineJoin::Miter,
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
        if self.brush_mode && cursor.is_over(bounds) {
            mouse::Interaction::Crosshair
        } else {
            mouse::Interaction::default()
        }
    }
}

pub fn screen_grid_view(state: &EditorState) -> Element<Message> {
    let num_cols = state.screen.size.1 * 32;
    let num_rows = state.screen.size.0 * 32;
    let pixel_size = state.global_config.pixel_size;

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

    Scrollable::with_direction(
        column![stack![
            canvas(ScreenGrid {
                screen: &state.screen,
                palettes: &state.palettes,
                palettes_id_idx_map: &state.palettes_id_idx_map,
                pixel_size,
                end_coords: state.end_coords,
                // thickness: 1.0,
                brush_mode: state.brush_mode,
                tile_block: &state.selected_tile_block,
            })
            .width((num_cols as f32 * 8.0 + 2.0) * pixel_size)
            .height((num_rows as f32 * 8.0 + 2.0) * pixel_size),
            canvas(ScreenSelect {
                active: state.selection_source == SelectionSource::MainScreen
                    && state.start_coords.is_some()
                    && state.end_coords.is_some()
                    && !state.brush_mode,
                left,
                right,
                top,
                bottom,
                pixel_size,
                brush_mode: state.brush_mode,
            })
            .width((num_cols as f32 * 8.0 + 2.0) * pixel_size)
            .height((num_rows as f32 * 8.0 + 2.0) * pixel_size),
        ]]
        .padding(Padding::new(0.0).right(16.0).bottom(16.0)),
        Direction::Both {
            vertical: Scrollbar::default(),
            horizontal: Scrollbar::default(),
        },
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

pub fn screen_controls(state: &EditorState) -> Element<Message> {
    row![
        text("Screen"),
        pick_list(
            state.screen_names.clone(),
            Some(state.screen.name.clone()),
            Message::SelectScreen
        )
        .width(200),
        button(text("\u{F64D}").font(iced_fonts::BOOTSTRAP_FONT))
            .style(button::success)
            .on_press(Message::AddScreenDialogue),
        button(text("\u{F4CB}").font(iced_fonts::BOOTSTRAP_FONT))
            .on_press(Message::RenameScreenDialogue),
        text("Theme"),
        pick_list(
            state.theme_names.clone(),
            Some(state.screen.theme.clone()),
            Message::SelectTheme
        )
        .width(200),
        button(text("\u{F64D}").font(iced_fonts::BOOTSTRAP_FONT))
            .style(button::success)
            .on_press(Message::AddThemeDialogue),
        button(text("\u{F4CB}").font(iced_fonts::BOOTSTRAP_FONT))
            .on_press(Message::RenameThemeDialogue),
    ]
    .spacing(10)
    .clip(true)
    .align_y(iced::alignment::Vertical::Center)
    .into()
}

pub fn add_screen_view(name: &String, size: (u8, u8)) -> Element<Message> {
    container(
        column![
            text("Add a new screen."),
            row![
                text("Name: ").width(70),
                text_input("", name)
                    .id("AddScreen")
                    .on_input(Message::SetAddScreenName)
                    .on_submit(Message::AddScreen)
            ]
            .spacing(10)
            .align_y(Vertical::Center),
            row![
                text("Size: ").width(70),
                number_input(&size.0, 1..=8, Message::SetAddScreenSizeX)
                    .width(50)
                    .on_submit(Message::AddScreen),
                text(" by "),
                number_input(&size.1, 1..=8, Message::SetAddScreenSizeY)
                    .width(50)
                    .on_submit(Message::AddScreen),
            ]
            .spacing(10)
            .align_y(Vertical::Center),
            button(text("Add screen"))
                .style(button::success)
                .on_press(Message::AddScreen),
        ]
        .spacing(10),
    )
    .width(350)
    .padding(25)
    .style(modal_background_style)
    .into()
}

pub fn rename_screen_view(state: &EditorState, name: &String) -> Element<'static, Message> {
    let old_name = state.screen.name.clone();
    container(
        column![
            text(format!("Rename screen \"{}\"", old_name)),
            row![
                text("Name: ").width(70),
                text_input("", name)
                    .id("RenameScreen")
                    .on_input(Message::SetRenameScreenName)
                    .on_submit(Message::RenameScreen)
            ]
            .spacing(10)
            .align_y(Vertical::Center),
            row![
                button(text("Rename screen")).on_press(Message::RenameScreen),
                Space::with_width(Length::Fill),
                button(text("Delete screen"))
                    .style(button::danger)
                    .on_press(Message::DeleteScreenDialogue),
            ],
        ]
        .spacing(10),
    )
    .width(350)
    .padding(25)
    .style(modal_background_style)
    .into()
}

pub fn delete_screen_view(state: &EditorState) -> Element<Message> {
    let name = state.screen.name.clone();
    container(
        column![
            text(format!("Delete screen \"{}\"?", name)),
            text("This will delete the screen across all themes."),
            button(text("Delete screen"))
                .style(button::danger)
                .on_press(Message::DeleteScreen),
        ]
        .spacing(10),
    )
    .width(350)
    .padding(25)
    .style(modal_background_style)
    .into()
}

pub fn add_theme_view(name: &String) -> Element<Message> {
    container(
        column![
            text("Add a new theme."),
            row![
                text("Name: ").width(70),
                text_input("", name)
                    .id("AddTheme")
                    .on_input(Message::SetAddThemeName)
                    .on_submit(Message::AddTheme)
            ]
            .spacing(10)
            .align_y(Vertical::Center),
            button(text("Add theme"))
                .style(button::success)
                .on_press(Message::AddTheme),
        ]
        .spacing(10),
    )
    .width(350)
    .padding(25)
    .style(modal_background_style)
    .into()
}

pub fn rename_theme_view(state: &EditorState, name: &String) -> Element<'static, Message> {
    let old_name = state.screen.theme.clone();
    container(
        column![
            text(format!("Rename theme \"{}\"", old_name)),
            row![
                text("Name: ").width(70),
                text_input("", name)
                    .id("RenameTheme")
                    .on_input(Message::SetRenameThemeName)
                    .on_submit(Message::RenameTheme)
            ]
            .spacing(10)
            .align_y(Vertical::Center),
            row![
                button(text("Rename theme")).on_press(Message::RenameTheme),
                Space::with_width(Length::Fill),
                button(text("Delete theme"))
                    .style(button::danger)
                    .on_press(Message::DeleteThemeDialogue),
            ],
        ]
        .spacing(10),
    )
    .width(350)
    .padding(25)
    .style(modal_background_style)
    .into()
}

pub fn delete_theme_view(state: &EditorState) -> Element<Message> {
    let theme = state.screen.theme.clone();
    container(
        column![
            text(format!("Delete theme \"{}\"?", theme)),
            text("This will delete the theme across all screens."),
            button(text("Delete theme"))
                .style(button::danger)
                .on_press(Message::DeleteTheme),
        ]
        .spacing(10),
    )
    .width(350)
    .padding(25)
    .style(modal_background_style)
    .into()
}
