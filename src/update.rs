use crate::{common::{Color, Palette}, message::Message, state::{Dialogue, EditorState}};

pub fn update(state: &mut EditorState, message: Message) {
    match message {
        Message::SelectPalette(name) => {
            for i in 0..state.palettes.len() {
                if name == state.palettes[i].name {
                    state.palette_state.palette_idx = i;
                    break;
                }
            }
        }
        Message::AddPaletteDialogue => {
            state.dialogue = Some(Dialogue::AddPalette { name: "".to_string() });
        }
        Message::SetAddPaletteName(new_name) => {
            match &mut state.dialogue {
                Some(Dialogue::AddPalette { name }) => {
                    *name = new_name;
                }
                _ => {}
            }
        }
        Message::AddPalette => {
            match &state.dialogue {
                Some(Dialogue::AddPalette { name }) => {
                    if name.len() == 0 {
                        return;
                    }
                    for p in state.palettes.iter() {
                        if &p.name == name {
                            // Don't add non-unique palette name.
                            // TODO: display some error message.
                            return;
                        }
                    }
                    state.palettes.push(Palette {
                        name: name.clone(),
                        colors: [Color::default(); 16]
                    });
                    state.palette_state.palette_idx = state.palettes.len() - 1;
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
                return;
            }
            if state.palette_state.palette_idx < state.palettes.len() {
                state.palettes.remove(state.palette_state.palette_idx);
                if state.palette_state.palette_idx == state.palettes.len() {
                    state.palette_state.palette_idx -= 1;
                }
            }
            state.dialogue = None;
        }
        Message::RenamePaletteDialogue => {
            state.dialogue = Some(Dialogue::RenamePalette { name: "".to_string() });
        }
        Message::SetRenamePaletteName(new_name) => {
            match &mut state.dialogue {
                Some(Dialogue::RenamePalette { name }) => {
                    *name = new_name;
                }
                _ => {}
            }
        }
        Message::RenamePalette => {
            match &state.dialogue {
                Some(Dialogue::RenamePalette { name }) => {
                    if name.len() == 0 {
                        return;
                    }
                    for p in state.palettes.iter() {
                        if &p.name == name {
                            // Don't add non-unique palette name.
                            // TODO: display some error message.
                            return;
                        }
                    }
                    state.palettes[state.palette_state.palette_idx].name = name.clone();
                    state.dialogue = None;
                }
                _ => {}
            }
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
                state.palettes[pal_idx].colors[idx as usize].red = state.palette_state.red;
                state.palettes[pal_idx].colors[idx as usize].green = state.palette_state.green;
                state.palettes[pal_idx].colors[idx as usize].blue = state.palette_state.blue;
            } else {
                state.palette_state.color_idx = idx;
                state.palette_state.red = state.palettes[pal_idx].colors[idx as usize].red;
                state.palette_state.green = state.palettes[pal_idx].colors[idx as usize].green;
                state.palette_state.blue = state.palettes[pal_idx].colors[idx as usize].blue;    
            }
        },
        Message::ChangeRed(c) => {
            let pal_idx = state.palette_state.palette_idx;
            let color_idx = state.palette_state.color_idx;
            state.palette_state.red = c;
            state.palettes[pal_idx].colors[color_idx as usize].red = c;
        }
        Message::ChangeGreen(c) => {
            let pal_idx = state.palette_state.palette_idx;
            let color_idx = state.palette_state.color_idx;
            state.palette_state.green = c;
            state.palettes[pal_idx].colors[color_idx as usize].green = c;
        }
        Message::ChangeBlue(c) => {
            let pal_idx = state.palette_state.palette_idx;
            let color_idx = state.palette_state.color_idx;
            state.palette_state.blue = c;
            state.palettes[pal_idx].colors[color_idx as usize].blue = c;
        }
    }
}
