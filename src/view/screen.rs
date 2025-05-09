use hashbrown::HashMap;
use iced::{
    alignment::Vertical, mouse, widget::{
        button, canvas, column, container, pick_list, row, scrollable::{self, Direction, Scrollbar}, text, text_input, Scrollable, Space
    }, Element, Length, Padding, Point, Rectangle, Size
};
use iced_aw::number_input;
use log::info;

use crate::{message::Message, state::{scale_color, EditorState, Palette, Screen}};

use super::modal_background_style;



// We use two separate canvases: one for drawing the tile raster and one for the tile selection.
// This is to work around a limitation in Iced's rendering pipeline that does not allow drawing
// objects (e.g. rectangles) on top of images within a single canvas.

struct ScreenGrid<'a> {
    screen: &'a Screen,
    palettes: &'a [Palette],
    palettes_name_idx_map: &'a HashMap<String, usize>,
    pixel_size: f32,
    // thickness: f32,
    // brush_mode: bool,
}

#[derive(Default)]
struct InternalState {
    clicking: bool,
}

impl<'a> canvas::Program<Message> for ScreenGrid<'a> {
    // No internal state
    type State = InternalState;

    // fn update(
    //     &self,
    //     state: &mut Self::State,
    //     event: canvas::Event,
    //     bounds: iced::Rectangle,
    //     cursor: mouse::Cursor,
    // ) -> (canvas::event::Status, Option<Message>) {
    //     let Some(p) = cursor.position_in(bounds) else {
    //         return (canvas::event::Status::Ignored, None);
    //     };

    //     let mut click: bool = false;
    //     match event {
    //         canvas::Event::Mouse(mouse_event) => match mouse_event {
    //             mouse::Event::ButtonPressed(mouse::Button::Left) => {
    //                 state.clicking = true;
    //                 click = true;
    //             }
    //             mouse::Event::ButtonReleased(mouse::Button::Left) => {
    //                 state.clicking = false;
    //             }
    //             mouse::Event::CursorMoved { .. } => {
    //                 if state.clicking {
    //                     click = true;
    //                 }
    //             }
    //             mouse::Event::CursorLeft => {
    //                 state.clicking = false;
    //             }
    //             _ => {}
    //         },
    //         _ => {}
    //     }

    //     if click {
    //         let y = ((p.y - 1.0) / (self.pixel_size * 8.0)) as i32;
    //         let x = ((p.x - 1.0) / (self.pixel_size * 8.0)) as i32;
    //         if x < 0 || x >= 16 {
    //             return (canvas::event::Status::Ignored, None);
    //         }
    //         let i = y * 16 + x;
    //         if i >= 0 && i < self.palette.tiles.len() as i32 {
    //             let message = Some(Message::ClickTile(i as TileIdx));
    //             return (canvas::event::Status::Captured, message);
    //         }
    //     }
    //     (canvas::event::Status::Ignored, None)
    // }

    fn draw(
        &self,
        _state: &InternalState,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: iced::Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let mut palette_idxs: Vec<Option<usize>> = vec![];
        let mut color_bytes: Vec<Option<Vec<[u8; 4]>>> = vec![];

        for i in 0..self.screen.palettes.len() {
            let idx = self.palettes_name_idx_map.get(&self.screen.palettes[i]).map(|x| *x);
            palette_idxs.push(idx);
            if let Some(j) = idx {
                let cb = self.palettes[j].colors.iter().map(|&(r, g, b)| 
                    [scale_color(r), scale_color(g), scale_color(b), 255]
                ).collect();
                color_bytes.push(Some(cb));
            } else {
                color_bytes.push(None);
            }
        }

        let num_cols = self.screen.size.1 as usize * 256;
        let num_rows = self.screen.size.0 as usize * 256;
        let mut data: Vec<u8> = vec![0; num_rows * num_cols * 4];
        let row_stride = num_cols * 4;
        let col_stride = 4;
        for sy in 0..self.screen.size.1 as usize {
            for sx in 0..self.screen.size.0 as usize {
                let subscreen = &self.screen.subscreens[sy * self.screen.size.0 as usize + sx];
                let subscreen_addr = sy * 256 * row_stride + sx * 256 * 4;
                for ty in 0..32 {
                    for tx in 0..32 {
                        let palette_i = subscreen.palettes[ty][tx];
                        let tile_idx = subscreen.tiles[ty][tx];
                        if let Some(idx) = palette_idxs[palette_i as usize] {
                            // info!("{} {} {} {}",sx,sy,tx,ty);
                            let tile = self.palettes[idx].tiles[tile_idx as usize];
                            let cb = color_bytes[idx].as_ref().unwrap();
                            let mut tile_addr = subscreen_addr + ty * 8 * row_stride + tx * 8 * col_stride;
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

        let image = iced::advanced::image::Image::new(iced::advanced::image::Handle::from_rgba(
            num_cols as u32,
            num_rows as u32,
            data,
        ))
        .filter_method(iced::widget::image::FilterMethod::Nearest)
        .snap(true);

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

    // fn mouse_interaction(
    //     &self,
    //     _interaction: &Self::State,
    //     bounds: iced::Rectangle,
    //     cursor: mouse::Cursor,
    // ) -> mouse::Interaction {
    //     if self.brush_mode && cursor.is_over(bounds) {
    //         mouse::Interaction::Crosshair
    //     } else {
    //         mouse::Interaction::default()
    //     }
    // }
}

// struct TileSelect {
//     tile_idx: Option<TileIdx>,
//     pixel_size: f32,
//     thickness: f32,
// }

// impl canvas::Program<Message> for TileSelect {
//     // No internal state
//     type State = ();

//     fn draw(
//         &self,
//         _state: &(),
//         renderer: &iced::Renderer,
//         theme: &iced::Theme,
//         bounds: iced::Rectangle,
//         _cursor: mouse::Cursor,
//     ) -> Vec<canvas::Geometry> {
//         let mut frame = canvas::Frame::new(renderer, bounds.size());
//         let pixel_size = self.pixel_size;
//         let thickness = self.thickness;
//         let num_cols = 16;

//         if let Some(idx) = self.tile_idx {
//             let y = (idx / num_cols) as f32 * pixel_size * 8.0 + thickness / 2.0;
//             let x = (idx % num_cols) as f32 * pixel_size * 8.0 + thickness / 2.0;
//             let border_color = if theme.extended_palette().is_dark {
//                 iced::Color::WHITE
//             } else {
//                 iced::Color::BLACK
//             };
//             let size = Size {
//                 width: 8.0 * pixel_size as f32 + thickness,
//                 height: 8.0 * pixel_size as f32 + thickness,
//             };
//             frame.stroke_rectangle(
//                 iced::Point { x, y },
//                 size,
//                 canvas::Stroke {
//                     width: thickness,
//                     style: border_color.into(),
//                     ..Default::default()
//                 },
//             );
//         }
//         vec![frame.into_geometry()]
//     }

//     // fn mouse_interaction(
//     //     &self,
//     //     _interaction: &Self::State,
//     //     bounds: iced::Rectangle,
//     //     cursor: mouse::Cursor,
//     // ) -> mouse::Interaction {
//     //     // if self.brush_mode && cursor.is_over(bounds) && self.exists_selection {
//     //     //     mouse::Interaction::Crosshair
//     //     // } else {
//     //     //     mouse::Interaction::default()
//     //     // }
//     // }
// }

pub fn screen_grid_view(state: &EditorState) -> Element<Message> {
    let num_cols = state.screen.size.1 * 32;
    let num_rows = state.screen.size.0 * 32;
    let pixel_size = 3.0;

    Scrollable::with_direction(
        column![
        // stack![
            canvas(ScreenGrid {
                screen: &state.screen,
                palettes: &state.palettes,
                palettes_name_idx_map: &state.palettes_name_idx_map,
                pixel_size,
                // thickness: 1.0,
                // brush_mode: state.brush_mode,
            })
            .width(num_cols as f32 * 8.0 * pixel_size)
            .height(num_rows as f32 * 8.0 * pixel_size),
            // canvas(TileSelect {
            //     tile_idx: state.tile_idx,
            //     pixel_size: 3.0,
            //     thickness: 1.0,
            // })
            // .width(384 + 2)
            // .height((num_rows * 8 * 3 + 4) as f32)
        // ],    
        ].padding(Padding::new(0.0).right(15.0).bottom(15.0)),
        Direction::Both { vertical: Scrollbar::default(), horizontal: Scrollbar::default() }
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}



pub fn screen_view(state: &EditorState) -> Element<Message> {
    let col = column![
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
        .align_y(iced::alignment::Vertical::Center),
        screen_grid_view(state),
    ]
    .spacing(10)
    .padding(10)
    .width(Length::Fill);

    col.into()
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
            ].spacing(10).align_y(Vertical::Center),
            row![
                text("Size: ").width(70),
                number_input(&size.0, 1..=8, Message::SetAddScreenSizeX)
                .width(50)
                .on_submit(Message::AddScreen),
                text(" by "),
                number_input(&size.1, 1..=8, Message::SetAddScreenSizeY)
                .width(50)
                .on_submit(Message::AddScreen),
            ].spacing(10).align_y(Vertical::Center),
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
            ].spacing(10).align_y(Vertical::Center),
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
