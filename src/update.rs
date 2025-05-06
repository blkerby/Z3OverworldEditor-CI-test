use crate::{message::Message, state::EditorState};

pub fn update(state: &mut EditorState, message: Message) {
    match message {
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
