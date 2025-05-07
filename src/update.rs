use iced::{
    keyboard::{self, key},
    widget, window, Event, Task,
};
use log::{error, info};

use crate::{
    message::Message,
    persist::{self, delete_palette},
    state::{Dialogue, EditorState, Palette},
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
                state.dialogue = None;
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
                    state.palette_state.palette_idx = i;
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
                        return Task::none();
                    }
                    for p in state.palettes.iter() {
                        if &p.name == name {
                            // Don't add non-unique palette name.
                            // TODO: display some error message.
                            return Task::none();
                        }
                    }
                    let mut pal = Palette {
                        modified: false,
                        name: name.clone(),
                        colors: state.palettes[state.palette_state.palette_idx]
                            .colors
                            .clone(),
                    };
                    pal.modified = true;
                    state.palettes.push(pal);
                    state.palette_state.palette_idx = state.palettes.len() - 1;
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
                        error!("Empty palette name is invalid.");
                        return Task::none();
                    }
                    for p in state.palettes.iter() {
                        if &p.name == name {
                            // Don't add non-unique palette name.
                            error!("Palette name {} already exists.", name);
                            return Task::none();
                        }
                    }

                    let name = name.clone();
                    let old_name = state.palettes[state.palette_state.palette_idx].name.clone();
                    state.palettes[state.palette_state.palette_idx].name = name.clone();
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
                // Don't delete the last palette.
                // TODO: display some error message.
                return Task::none();
            }

            let name = state.palettes[state.palette_state.palette_idx].name.clone();
            if let Err(e) = persist::delete_palette(state, &name) {
                error!("Error deleting palette file: {}\n{}", e, e.backtrace());
            }
            if state.palette_state.palette_idx < state.palettes.len() {
                state.palettes.remove(state.palette_state.palette_idx);
                if state.palette_state.palette_idx == state.palettes.len() {
                    state.palette_state.palette_idx -= 1;
                }
            }
            update_palette_order(state);
            state.dialogue = None;
        }
        Message::HideModal => {
            state.dialogue = None;
        }
        Message::ColorSelectMode => {
            state.palette_state.brush_mode = false;
        }
        Message::ColorBrushMode => {
            state.palette_state.brush_mode = true;
        }
        Message::ClickColor(idx) => {
            let pal_idx = state.palette_state.palette_idx;
            if state.palette_state.brush_mode {
                state.palettes[pal_idx].colors[idx as usize] = state.palette_state.selected_color;
                state.palettes[pal_idx].modified = true;
            } else {
                state.palette_state.color_idx = Some(idx);
                state.palette_state.selected_color = state.palettes[pal_idx].colors[idx as usize];
            }
        }
        Message::ChangeRed(c) => {
            if let Some(color_idx) = state.palette_state.color_idx {
                let pal_idx = state.palette_state.palette_idx;
                state.palette_state.selected_color.0 = c;
                state.palettes[pal_idx].colors[color_idx as usize].0 = c;
                state.palettes[pal_idx].modified = true;
            }
        }
        Message::ChangeGreen(c) => {
            if let Some(color_idx) = state.palette_state.color_idx {
                let pal_idx = state.palette_state.palette_idx;
                state.palette_state.selected_color.1 = c;
                state.palettes[pal_idx].colors[color_idx as usize].1 = c;
                state.palettes[pal_idx].modified = true;
            }
        }
        Message::ChangeBlue(c) => {
            if let Some(color_idx) = state.palette_state.color_idx {
                let pal_idx = state.palette_state.palette_idx;
                state.palette_state.selected_color.2 = c;
                state.palettes[pal_idx].colors[color_idx as usize].2 = c;
                state.palettes[pal_idx].modified = true;
            }
        }
    }
    Task::none()
}

fn update_palette_order(state: &mut EditorState) {
    let name = state.palettes[state.palette_state.palette_idx].name.clone();
    state.palettes.sort_by(|x, y| x.name.cmp(&y.name));
    for i in 0..state.palettes.len() {
        if state.palettes[i].name == name {
            state.palette_state.palette_idx = i;
            break;
        }
    }
}
