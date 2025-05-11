use iced::{
    keyboard::{self, key},
    widget, window, Event, Point, Task,
};
use itertools::Itertools;
use log::{error, info, warn};

use crate::{
    import::Importer,
    message::{Message, SelectionSource},
    persist::{
        self, copy_area_theme, delete_area, delete_area_theme, delete_palette, load_area,
        load_area_list, load_project, rename_area, rename_area_theme, save_area,
    },
    state::{
        Area, Dialogue, EditorState, Flip, PaletteId, Screen, Tile, TileBlock, TileCoord, TileIdx,
        MAX_PIXEL_SIZE, MIN_PIXEL_SIZE,
    },
    view::{open_project, open_rom},
};

pub fn update(state: &mut EditorState, message: Message) -> Task<Message> {
    if state.global_config.project_dir.is_none() {
        let Message::ProjectOpened(_) = &message else {
            return Task::none();
        };
    }
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
                state.selected_gfx = vec![];
                state.start_coords = None;
                state.end_coords = None;
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
            Event::Keyboard(keyboard::Event::KeyPressed {
                modified_key: keyboard::Key::Character(c),
                ..
            }) => match c.as_str() {
                "b" => {
                    state.brush_mode = true;
                }
                "s" => {
                    state.brush_mode = false;
                }
                "h" => {
                    for i in 0..state.selected_tile_block.size.1 as usize {
                        state.selected_tile_block.palettes[i].reverse();
                        state.selected_tile_block.tiles[i].reverse();
                        state.selected_tile_block.flips[i].reverse();
                        state.selected_gfx[i].reverse();
                        for j in 0..state.selected_tile_block.size.0 as usize {
                            state.selected_tile_block.flips[i][j] =
                                state.selected_tile_block.flips[i][j].flip_horizontally();
                            state.selected_gfx[i][j] =
                                Flip::Horizontal.apply(state.selected_gfx[i][j]);
                        }
                    }
                }
                "v" => {
                    state.selected_tile_block.palettes.reverse();
                    state.selected_tile_block.tiles.reverse();
                    state.selected_tile_block.flips.reverse();
                    state.selected_gfx.reverse();
                    for i in 0..state.selected_tile_block.size.1 as usize {
                        for j in 0..state.selected_tile_block.size.0 as usize {
                            state.selected_tile_block.flips[i][j] =
                                state.selected_tile_block.flips[i][j].flip_vertically();
                            state.selected_gfx[i][j] =
                                Flip::Vertical.apply(state.selected_gfx[i][j]);
                        }
                    }
                }
                "-" => {
                    state.global_config.pixel_size =
                        (state.global_config.pixel_size - 1.0).max(MIN_PIXEL_SIZE);
                }
                "=" => {
                    state.global_config.pixel_size =
                        (state.global_config.pixel_size + 1.0).min(MAX_PIXEL_SIZE);
                }
                _ => {}
            },
            _ => {}
        },
        Message::SaveProject => {
            if let Err(e) = persist::save_project(state) {
                error!("Error saving project: {}\n{}", e, e.backtrace());
            }
        }
        Message::OpenProject => {
            return Task::perform(open_project(), Message::ProjectOpened);
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

                    state.dialogue = None;
                }
                None => {
                    if state.global_config.project_dir.is_none() {
                        info!("Project path not selected, exiting.");
                        std::process::exit(1);
                    }
                }
            }
        }
        Message::SettingsDialogue => {
            state.dialogue = Some(Dialogue::Settings);
        }
        Message::SetPixelSize(pixel_size) => {
            state.global_config.pixel_size = pixel_size;
        }
        Message::CloseDialogue => {
            state.dialogue = None;
        }
        Message::ImportDialogue => {
            return Task::perform(open_rom(), Message::ImportConfirm);
        }
        Message::ImportConfirm(path) => {
            state.rom_path = path;
            state.dialogue = Some(Dialogue::ImportROM);
        }
        Message::ImportROM => {
            if let Some(path) = &state.rom_path {
                if let Err(e) = Importer::import(state, &path.clone()) {
                    error!("Error importing from ROM: {}\n{}", e, e.backtrace());
                    return Task::none();
                }
                if let Err(e) = load_project(state) {
                    error!("Error loading project: {}\n{}", e, e.backtrace());
                }
                state.dialogue = None;
            }
        }
        Message::SelectPalette(name) => {
            for i in 0..state.palettes.len() {
                if name == format!("{}: {}", state.palettes[i].id, state.palettes[i].name) {
                    state.palette_idx = i;
                    state.color_idx = None;
                    break;
                }
            }
        }
        Message::AddPaletteDialogue => {
            let id = state.palettes.iter().map(|x| x.id).max().unwrap() + 1;
            state.dialogue = Some(Dialogue::AddPalette {
                name: "".to_string(),
                id,
            });
            return iced::widget::text_input::focus("AddPalette");
        }
        Message::SetAddPaletteName(new_name) => match &mut state.dialogue {
            Some(Dialogue::AddPalette { name, .. }) => {
                *name = new_name;
            }
            _ => {}
        },
        Message::SetAddPaletteID(new_id) => match &mut state.dialogue {
            Some(Dialogue::AddPalette { id, .. }) => {
                *id = new_id;
            }
            _ => {}
        },
        Message::AddPalette => {
            match &state.dialogue {
                Some(Dialogue::AddPalette { name, id }) => {
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
                        if p.id == *id {
                            // Don't add non-unique palette ID.
                            warn!("Palette ID {} already exists.", id);
                            return Task::none();
                        }
                    }
                    let mut pal = state.palettes[state.palette_idx].clone();
                    pal.name = name.clone();
                    pal.id = *id;
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
                    // TODO: update currently loaded area, since palette indices may have shifted
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
        Message::TilesetBrush(Point { x: x0, y: y0 }) => {
            let s = &state.selected_tile_block;
            for y in 0..s.size.1 {
                for x in 0..s.size.0 {
                    let y1 = y + y0;
                    let x1 = x + x0;
                    let i = (y1 * 16 + x1) as usize;
                    if i < state.palettes[state.palette_idx].tiles.len() {
                        state.palettes[state.palette_idx].tiles[i] =
                            state.selected_gfx[y as usize][x as usize];
                    }
                }
            }
            state.palettes[state.palette_idx].modified = true;
        }
        // Message::ClickTile(idx) => {
        //     if state.brush_mode {
        //         state.palettes[state.palette_idx].tiles[idx as usize] = state.selected_tile;
        //         state.palettes[state.palette_idx].modified = true;
        //     } else {
        //         state.tile_idx = Some(idx);
        //         state.selected_tile = state.palettes[state.palette_idx].tiles[idx as usize];
        //         state.pixel_coords = None
        //     }
        // }
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
        Message::SelectArea(name) => {
            if let Err(e) = load_area(state, &name, &state.area.theme.clone()) {
                error!(
                    "Error loading area {} (theme {}): {}\n{}",
                    name,
                    state.area.theme,
                    e,
                    e.backtrace()
                );
            }
        }
        Message::AddAreaDialogue => {
            state.dialogue = Some(Dialogue::AddArea {
                name: "".to_string(),
                size: (2, 2),
            });
            return iced::widget::text_input::focus("AddArea");
        }
        Message::SetAddAreaName(new_name) => match &mut state.dialogue {
            Some(Dialogue::AddArea { name, .. }) => {
                *name = new_name;
            }
            _ => {}
        },
        Message::SetAddAreaSizeX(new_x) => match &mut state.dialogue {
            Some(Dialogue::AddArea { size, .. }) => {
                size.0 = new_x;
            }
            _ => {}
        },
        Message::SetAddAreaSizeY(new_y) => match &mut state.dialogue {
            Some(Dialogue::AddArea { size, .. }) => {
                size.1 = new_y;
            }
            _ => {}
        },
        Message::AddArea => {
            match &state.dialogue {
                Some(Dialogue::AddArea { name, size }) => {
                    if name.len() == 0 {
                        warn!("Empty area name is invalid.");
                        return Task::none();
                    }
                    let name = name.clone();
                    let size = size.clone();
                    for s in &state.area_names {
                        if s == &name {
                            // Don't add a non-unique area name.
                            warn!("Area name {} already exists.", name);
                            return Task::none();
                        }
                    }
                    for theme in state.theme_names.clone() {
                        state.area = Area {
                            modified: true,
                            name: name.clone(),
                            theme,
                            size,
                            screens: (0..size.0)
                                .cartesian_product(0..size.1)
                                .map(|(x, y)| Screen {
                                    position: (x, y),
                                    palettes: [[0; 32]; 32],
                                    tiles: [[0; 32]; 32],
                                    flips: [[Flip::None; 32]; 32],
                                })
                                .collect(),
                        };
                        if let Err(e) = save_area(state) {
                            error!("Error saving new area: {}\n{}", e, e.backtrace());
                        }
                    }
                    state.dialogue = None;
                    state.area_names.push(name.clone());
                    state.area_names.sort();
                }
                _ => {}
            }
        }
        Message::RenameAreaDialogue => {
            state.dialogue = Some(Dialogue::RenameArea {
                name: state.area.name.clone(),
            });
            return iced::widget::text_input::focus("RenameArea");
        }
        Message::SetRenameAreaName(new_name) => match &mut state.dialogue {
            Some(Dialogue::RenameArea { name }) => {
                *name = new_name;
            }
            _ => {}
        },
        Message::RenameArea => {
            match &state.dialogue {
                Some(Dialogue::RenameArea { name }) => {
                    if name.len() == 0 {
                        warn!("Empty area name is invalid.");
                        return Task::none();
                    }
                    let name = name.clone();
                    for s in &state.area_names {
                        if s == &name && s != &state.area.name {
                            // Don't add a non-unique area name.
                            warn!("Area name {} already exists.", name);
                            return Task::none();
                        }
                    }
                    if let Err(e) = rename_area(state, &name) {
                        error!("Error renaming area: {}\n{}", e, e.backtrace());
                        return Task::none();
                    }
                    if let Err(e) = load_area_list(state) {
                        error!("Error reloading area listing: {}\n{}", e, e.backtrace());
                        return Task::none();
                    }
                    state.area.name = name.clone();
                    state.dialogue = None;
                }
                _ => {}
            }
        }
        Message::DeleteAreaDialogue => {
            state.dialogue = Some(Dialogue::DeleteArea);
        }
        Message::DeleteArea => {
            if state.area_names.len() == 1 {
                warn!("Not allowed to delete the last remaining area.");
                return Task::none();
            }
            if let Err(e) = delete_area(state, &state.area.name.clone()) {
                error!("Error deleting area: {}\n{}", e, e.backtrace());
                return Task::none();
            }
            if let Err(e) = load_area_list(state) {
                error!("Error reloading area listing: {}\n{}", e, e.backtrace());
                return Task::none();
            }
            if let Err(e) = load_area(
                state,
                &state.area_names[0].clone(),
                &state.area.theme.clone(),
            ) {
                error!("Error loading area: {}\n{}", e, e.backtrace());
                return Task::none();
            }
            state.dialogue = None;
        }
        Message::SelectTheme(theme) => {
            if let Err(err) = load_area(state, &state.area.name.clone(), &theme) {
                error!(
                    "Error loading theme {} (area {}): {}\n{}",
                    theme,
                    state.area.name,
                    err,
                    err.backtrace()
                );
            }
        }
        Message::AddThemeDialogue => {
            state.dialogue = Some(Dialogue::AddTheme {
                name: "".to_string(),
            });
            return iced::widget::text_input::focus("AddTheme");
        }
        Message::SetAddThemeName(new_name) => match &mut state.dialogue {
            Some(Dialogue::AddTheme { name }) => {
                *name = new_name;
            }
            _ => {}
        },
        Message::AddTheme => {
            match &state.dialogue {
                Some(Dialogue::AddTheme { name: theme }) => {
                    if theme.len() == 0 {
                        warn!("Empty theme name is invalid.");
                        return Task::none();
                    }
                    let theme = theme.to_string();
                    for t in &state.theme_names {
                        if t == &theme {
                            // Don't add a non-unique theme name.
                            warn!("Theme name {} already exists.", theme);
                            return Task::none();
                        }
                    }
                    let old_theme = state.area.theme.clone();
                    for area_name in &state.area_names.clone() {
                        if let Err(e) = copy_area_theme(state, area_name, &old_theme, &theme) {
                            error!("Error copying area: {}\n{}", e, e.backtrace());
                            return Task::none();
                        }
                    }
                    state.area.theme = theme.clone();
                    state.theme_names.push(theme.clone());
                    state.theme_names.sort();
                    state.dialogue = None;
                }
                _ => {}
            }
        }
        Message::RenameThemeDialogue => {
            state.dialogue = Some(Dialogue::RenameTheme {
                name: state.area.theme.clone(),
            });
            return iced::widget::text_input::focus("RenameTheme");
        }
        Message::SetRenameThemeName(new_name) => match &mut state.dialogue {
            Some(Dialogue::RenameTheme { name }) => {
                *name = new_name;
            }
            _ => {}
        },
        Message::RenameTheme => {
            match &state.dialogue {
                Some(Dialogue::RenameTheme { name: theme }) => {
                    if theme.len() == 0 {
                        warn!("Empty theme name is invalid.");
                        return Task::none();
                    }
                    let theme = theme.to_string();
                    for t in &state.theme_names {
                        if t == &theme {
                            // Don't add a non-unique theme name.
                            warn!("Theme name {} already exists.", theme);
                            return Task::none();
                        }
                    }
                    let old_theme = state.area.theme.clone();
                    for area_name in &state.area_names.clone() {
                        if let Err(e) = rename_area_theme(state, area_name, &old_theme, &theme) {
                            error!("Error renaming area: {}\n{}", e, e.backtrace());
                            return Task::none();
                        }
                    }
                    if let Err(e) = load_area_list(state) {
                        error!("Error reloading area listing: {}\n{}", e, e.backtrace());
                        return Task::none();
                    }
                    state.area.theme = theme;
                    state.dialogue = None;
                }
                _ => {}
            }
        }
        Message::DeleteThemeDialogue => {
            state.dialogue = Some(Dialogue::DeleteTheme);
        }
        Message::DeleteTheme => {
            if state.theme_names.len() == 1 {
                warn!("Not allowed to delete the last remaining theme.");
                return Task::none();
            }
            let theme = state.area.theme.clone();
            for area_name in &state.area_names.clone() {
                if let Err(e) = delete_area_theme(state, area_name, &theme) {
                    error!("Error deleting area: {}\n{}", e, e.backtrace());
                    return Task::none();
                }
            }
            if let Err(e) = load_area_list(state) {
                error!("Error reloading area listing: {}\n{}", e, e.backtrace());
                return Task::none();
            }
            if let Err(e) = load_area(
                state,
                &state.area.name.clone(),
                &state.theme_names[0].clone(),
            ) {
                error!("Error loading area: {}\n{}", e, e.backtrace());
                return Task::none();
            }
            state.dialogue = None;
        }
        Message::StartTileSelection(p, source) => {
            state.selection_source = source;
            state.start_coords = Some((p.x, p.y));
            state.end_coords = Some((p.x, p.y));
        }
        Message::ProgressTileSelection(p) => {
            state.end_coords = Some((p.x, p.y));
        }
        Message::EndTileSelection(p1) => {
            let p1 = (p1.x, p1.y);
            let Some(p0) = state.start_coords else {
                return Task::none();
            };

            let left = p0.0.min(p1.0);
            let right = p0.0.max(p1.0);
            let top = p0.1.min(p1.1);
            let bottom = p0.1.max(p1.1);

            match state.selection_source {
                SelectionSource::MainArea => {}
                SelectionSource::Tileset => {
                    if left == right && top == bottom {
                        let idx = p1.1 * 16 + p1.0;
                        state.tile_idx = Some(idx);
                        state.selected_tile = state.palettes[state.palette_idx].tiles[idx as usize];
                    }
                }
            }

            let mut palettes: Vec<Vec<PaletteId>> = vec![];
            let mut tiles: Vec<Vec<TileIdx>> = vec![];
            let mut flips: Vec<Vec<Flip>> = vec![];
            for y in top..=bottom {
                let mut pal_row: Vec<PaletteId> = vec![];
                let mut tile_row: Vec<TileIdx> = vec![];
                let mut flip_row: Vec<Flip> = vec![];
                for x in left..=right {
                    match state.selection_source {
                        SelectionSource::MainArea => {
                            pal_row.push(state.area.get_palette(x, y));
                            tile_row.push(state.area.get_tile(x, y));
                            flip_row.push(state.area.get_flip(x, y));
                        }
                        SelectionSource::Tileset => {
                            pal_row.push(state.palettes[state.palette_idx].id);
                            tile_row.push(y * 16 + x);
                            flip_row.push(Flip::None)
                        }
                    }
                }
                palettes.push(pal_row);
                tiles.push(tile_row);
                flips.push(flip_row);
            }
            state.selected_tile_block = TileBlock {
                size: (right - left + 1, bottom - top + 1),
                palettes,
                tiles,
                flips,
            };
            let s = &state.selected_tile_block;
            state.selected_gfx.clear();
            for y in 0..s.size.1 {
                let mut gfx_row: Vec<Tile> = vec![];
                for x in 0..s.size.0 {
                    let palette_id = s.palettes[y as usize][x as usize];
                    let palette_idx = state.palettes_id_idx_map[&palette_id];
                    let tile_idx = s.tiles[y as usize][x as usize];
                    let tile = state.palettes[palette_idx].tiles[tile_idx as usize];
                    gfx_row.push(tile);
                }
                state.selected_gfx.push(gfx_row);
            }
        }
        Message::AreaBrush(p) => {
            let s = &state.selected_tile_block;
            for y in 0..s.size.1 {
                for x in 0..s.size.0 {
                    state
                        .area
                        .set_palette(p.x + x, p.y + y, s.palettes[y as usize][x as usize]);
                    state
                        .area
                        .set_tile(p.x + x, p.y + y, s.tiles[y as usize][x as usize]);
                    state
                        .area
                        .set_flip(p.x + x, p.y + y, s.flips[y as usize][x as usize]);
                }
            }
            state.area.modified = true;
        }
    }
    Task::none()
}

pub fn update_palette_order(state: &mut EditorState) {
    let id = state.palettes[state.palette_idx].id;
    state.palettes.sort_by(|x, y| x.id.cmp(&y.id));
    state.palettes_id_idx_map.clear();
    for i in 0..state.palettes.len() {
        state.palettes_id_idx_map.insert(state.palettes[i].id, i);
        if state.palettes[i].id == id {
            state.palette_idx = i;
            break;
        }
    }
}
