use iced::{
    keyboard::{self, key},
    widget, window, Event, Task,
};
use log::{error, info, warn};

use crate::{
    message::Message,
    persist::{self, delete_palette},
    state::{Dialogue, EditorState, TileIdx},
};

pub fn update(state: &mut EditorState, message: Message) -> Task<Message> {
    match message {
        Message::Event(event) => match event {
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(key::Named::Tab),
                modifiers,
                ..
            }) => {
                if modifiers.shift() {
                    return widget::focus_previous();
                } else {
                    return widget::focus_next();
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(key::Named::Escape),
                ..
            }) => {
                state.brush_mode = false;
                state.dialogue = None;
                state.color_idx = None;
                state.tile_idx = None;
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(key::Named::ArrowRight),
                ..
            }) => {
                if let Some(idx) = state.color_idx {
                    if idx < 15 {
                        let new_idx = idx + 1;
                        state.color_idx = Some(new_idx);
                        state.selected_color =
                            state.palettes[state.palette_idx].colors[new_idx as usize];
                    }
                } else if let Some(idx) = state.tile_idx {
                    if (idx as usize) + 1 < state.palettes[state.palette_idx].tiles.len() {
                        let new_idx = idx + 1;
                        state.tile_idx = Some(new_idx);
                    }
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(key::Named::ArrowLeft),
                ..
            }) => {
                if let Some(idx) = state.color_idx {
                    if idx > 0 {
                        let new_idx = idx - 1;
                        state.color_idx = Some(new_idx);
                        state.selected_color =
                            state.palettes[state.palette_idx].colors[new_idx as usize];
                    }
                } else if let Some(idx) = state.tile_idx {
                    if idx > 0 {
                        let new_idx = idx - 1;
                        state.tile_idx = Some(new_idx);
                    }
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(key::Named::ArrowDown),
                ..
            }) => {
                if let Some(idx) = state.tile_idx {
                    if (idx as usize) + 16 < state.palettes[state.palette_idx].tiles.len() {
                        let new_idx = idx + 16;
                        state.tile_idx = Some(new_idx);
                    }
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(key::Named::ArrowUp),
                ..
            }) => {
                if let Some(idx) = state.tile_idx {
                    if (idx as usize) >= 16 {
                        let new_idx = idx - 16;
                        state.tile_idx = Some(new_idx);
                    }
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed { modified_key, .. }) => {
                if modified_key == keyboard::Key::Character("b".into()) {
                    state.brush_mode = true;
                } else if modified_key == keyboard::Key::Character("s".into()) {
                    state.brush_mode = false;
                }
            }
            _ => {}
        },
        Message::SaveProject => {
            if let Err(e) = persist::save_project(state) {
                error!("Error saving project: {}\n{}", e, e.backtrace());
            }
        }
        Message::WindowClose(id) => {
            if let Err(e) = persist::save_project(state) {
                error!("Error saving project: {}\n{}", e, e.backtrace());
                return Task::none();
            }
            return window::close(id);
        }
        Message::ProjectOpened(path) => {
            match path {
                Some(p) => {
                    info!("Opening project at {}", p.display());
                    // Ensure that the old project has been persisted before loading the new:
                    if state.global_config.project_dir.is_some() {
                        if let Err(e) = persist::save_project(state) {
                            error!("Error saving project: {}\n{}", e, e.backtrace());
                            return Task::none();
                        }
                    }

                    // Update the global config to be set to the new project:
                    state.global_config.project_dir = Some(p);
                    state.global_config.modified = true;
                    if let Err(e) = persist::save_global_config(state) {
                        error!("Error saving global config: {}\n{}", e, e.backtrace());
                    }

                    if let Err(e) = persist::load_project(state) {
                        error!("Error loading project: {}\n{}", e, e.backtrace());
                        return Task::none();
                    }
                }
                None => {
                    if state.global_config.project_dir.is_none() {
                        info!("Project path not selected, exiting.");
                        std::process::exit(1);
                    }
                }
            }
        }
        Message::SelectPalette(name) => {
            for i in 0..state.palettes.len() {
                if name == state.palettes[i].name {
                    state.palette_idx = i;
                    state.color_idx = None;
                    break;
                }
            }
        }
        Message::AddPaletteDialogue => {
            state.dialogue = Some(Dialogue::AddPalette {
                name: "".to_string(),
            });
            return iced::widget::text_input::focus("AddPalette");
        }
        Message::SetAddPaletteName(new_name) => match &mut state.dialogue {
            Some(Dialogue::AddPalette { name }) => {
                *name = new_name;
            }
            _ => {}
        },
        Message::AddPalette => {
            match &state.dialogue {
                Some(Dialogue::AddPalette { name }) => {
                    if name.len() == 0 {
                        warn!("Empty palette name is invalid.");
                        return Task::none();
                    }
                    for p in state.palettes.iter() {
                        if &p.name == name {
                            // Don't add non-unique palette name.
                            warn!("Palette name {} already exists.", name);
                            return Task::none();
                        }
                    }
                    let mut pal = state.palettes[state.palette_idx].clone();
                    pal.name = name.clone();
                    pal.modified = true;
                    state.palettes.push(pal);
                    state.palette_idx = state.palettes.len() - 1;
                    update_palette_order(state);
                    state.dialogue = None;
                }
                _ => {}
            }
        }
        Message::RenamePaletteDialogue => {
            state.dialogue = Some(Dialogue::RenamePalette {
                name: "".to_string(),
            });
            return iced::widget::text_input::focus("RenamePalette");
        }
        Message::SetRenamePaletteName(new_name) => match &mut state.dialogue {
            Some(Dialogue::RenamePalette { name }) => {
                *name = new_name;
            }
            _ => {}
        },
        Message::RenamePalette => {
            match &state.dialogue {
                Some(Dialogue::RenamePalette { name }) => {
                    if name.len() == 0 {
                        warn!("Empty palette name is invalid.");
                        return Task::none();
                    }
                    for p in state.palettes.iter() {
                        if &p.name == name {
                            // Don't add non-unique palette name.
                            warn!("Palette name {} already exists.", name);
                            return Task::none();
                        }
                    }

                    let name = name.clone();
                    let old_name = state.palettes[state.palette_idx].name.clone();
                    state.palettes[state.palette_idx].name = name.clone();
                    state.palettes[state.palette_idx].modified = true;
                    if let Err(e) = persist::save_project(state) {
                        error!("Error saving project: {}\n{}", e, e.backtrace());
                        return Task::none();
                    }
                    if let Err(e) = delete_palette(state, &old_name) {
                        error!("Error deleting old palette: {}\n{}", e, e.backtrace());
                        return Task::none();
                    }
                    update_palette_order(state);
                    // TODO: update currently loaded screen, since palette indices may have shifted
                    state.dialogue = None;
                }
                _ => {}
            }
        }
        Message::DeletePaletteDialogue => {
            state.dialogue = Some(Dialogue::DeletePalette);
        }
        Message::DeletePalette => {
            if state.palettes.len() == 1 {
                warn!("Not allowed to delete the last palette.");
                return Task::none();
            }

            let name = state.palettes[state.palette_idx].name.clone();
            if let Err(e) = persist::delete_palette(state, &name) {
                error!("Error deleting palette file: {}\n{}", e, e.backtrace());
            }
            if state.palette_idx < state.palettes.len() {
                state.palettes.remove(state.palette_idx);
                if state.palette_idx == state.palettes.len() {
                    state.palette_idx -= 1;
                }
            }
            update_palette_order(state);
            state.dialogue = None;
        }
        Message::HideModal => {
            state.dialogue = None;
        }
        Message::ClickColor(idx) => {
            let pal_idx = state.palette_idx;
            if state.brush_mode {
                state.palettes[pal_idx].colors[idx as usize] = state.selected_color;
                state.palettes[pal_idx].modified = true;
            } else {
                state.color_idx = Some(idx);
                state.selected_color = state.palettes[pal_idx].colors[idx as usize];
            }
        }
        Message::ChangeRed(c) => {
            if let Some(color_idx) = state.color_idx {
                let pal_idx = state.palette_idx;
                state.selected_color.0 = c;
                state.palettes[pal_idx].colors[color_idx as usize].0 = c;
                state.palettes[pal_idx].modified = true;
            }
        }
        Message::ChangeGreen(c) => {
            if let Some(color_idx) = state.color_idx {
                let pal_idx = state.palette_idx;
                state.selected_color.1 = c;
                state.palettes[pal_idx].colors[color_idx as usize].1 = c;
                state.palettes[pal_idx].modified = true;
            }
        }
        Message::ChangeBlue(c) => {
            if let Some(color_idx) = state.color_idx {
                let pal_idx = state.palette_idx;
                state.selected_color.2 = c;
                state.palettes[pal_idx].colors[color_idx as usize].2 = c;
                state.palettes[pal_idx].modified = true;
            }
        }
        Message::AddTileRow => {
            state.palettes[state.palette_idx]
                .tiles
                .extend(vec![[[0; 8]; 8]; 16]);
            state.palettes[state.palette_idx].modified = true;
        }
        Message::DeleteTileRow => {
            if state.palettes[state.palette_idx].tiles.len() <= 16 {
                warn!("Not allowed to delete the last row of tiles.");
                return Task::none();
            }
            let new_size = state.palettes[state.palette_idx].tiles.len() - 16;
            state.palettes[state.palette_idx]
                .tiles
                .resize(new_size, [[0; 8]; 8]);
            if let Some(idx) = state.tile_idx {
                if idx >= new_size as TileIdx {
                    state.tile_idx = Some(new_size as TileIdx - 1);
                }
            }
            state.palettes[state.palette_idx].modified = true;
        }
        Message::ClickTile(idx) => {
            if state.brush_mode {
                state.palettes[state.palette_idx].tiles[idx as usize] = state.selected_tile;
                state.palettes[state.palette_idx].modified = true;
            } else {
                state.tile_idx = Some(idx);
                state.selected_tile = state.palettes[state.palette_idx].tiles[idx as usize];
                state.pixel_coords = None
            }
        }
        Message::ClickPixel(x, y) => {
            state.pixel_coords = Some((x, y));
            if let Some(tile_idx) = state.tile_idx {
                let pal = &mut state.palettes[state.palette_idx];
                if state.brush_mode {
                    if let Some(color_idx) = state.color_idx {
                        pal.tiles[tile_idx as usize][y as usize][x as usize] = color_idx;
                        pal.modified = true;
                    }
                } else {
                    let color_idx = pal.tiles[tile_idx as usize][y as usize][x as usize];
                    state.color_idx = Some(color_idx);
                    state.selected_color = pal.colors[color_idx as usize];
                }
            }
        }
    }
    Task::none()
}

fn update_palette_order(state: &mut EditorState) {
    let name = state.palettes[state.palette_idx].name.clone();
    state.palettes.sort_by(|x, y| x.name.cmp(&y.name));
    for i in 0..state.palettes.len() {
        if state.palettes[i].name == name {
            state.palette_idx = i;
            break;
        }
    }
}
