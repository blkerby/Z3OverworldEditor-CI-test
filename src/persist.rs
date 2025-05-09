use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use json_pretty_compact::PrettyCompactFormatter;
use log::info;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Serializer;

use crate::{
    state::{EditorState, Palette, Subscreen},
    update::update_palette_order,
};

fn save_json<T: Serialize>(path: &Path, data: &T) -> Result<()> {
    info!("Saving {}", path.display());
    let formatter = PrettyCompactFormatter::new();
    let mut data_bytes = vec![];
    let mut ser = Serializer::with_formatter(&mut data_bytes, formatter);
    data.serialize(&mut ser).unwrap();
    fs::create_dir_all(path.parent().context("invalid parent directory")?)?;
    fs::write(path, &data_bytes)?;
    Ok(())
}

fn load_json<T: DeserializeOwned>(path: &Path) -> Result<T> {
    info!("Loading {}", path.display());
    let data_bytes = std::fs::read(path)?;
    let data: T = serde_json::from_slice(&data_bytes)?;
    Ok(data)
}

pub fn load_global_config(state: &mut EditorState) -> Result<()> {
    state.global_config = load_json(&state.global_config_path)?;
    Ok(())
}

pub fn save_global_config(state: &mut EditorState) -> Result<()> {
    if state.global_config.modified {
        save_json(&state.global_config_path, &state.global_config)?;
        state.global_config.modified = false;
    }
    Ok(())
}

fn get_project_dir(state: &EditorState) -> Result<PathBuf> {
    Ok(state
        .global_config
        .project_dir
        .as_ref()
        .context("Project directory not set.")?
        .to_owned())
}

fn get_palette_dir(state: &EditorState) -> Result<PathBuf> {
    Ok(get_project_dir(state)?.join("Palettes"))
}

fn save_palettes(state: &mut EditorState) -> Result<()> {
    let pal_dir = get_palette_dir(state)?;
    for pal in &mut state.palettes {
        if pal.modified {
            let pal_filename = format!("{}.json", pal.name);
            let pal_path = pal_dir.join(pal_filename);
            save_json(&pal_path, pal)?;
            pal.modified = false;
        }
    }
    Ok(())
}

fn load_palettes(state: &mut EditorState) -> Result<()> {
    let pal_dir = get_palette_dir(state)?;
    let pattern = format!("{}/*.json", pal_dir.display());
    state.palettes.clear();
    for entry in glob::glob(&pattern)? {
        let path = entry?;
        let name = path
            .file_stem()
            .context(format!("bad file name: {}", path.display()))?
            .to_str()
            .context("bad file stem")?;
        let mut pal: Palette = load_json(&path)?;
        pal.name = name.to_owned();
        state.palettes.push(pal);
    }
    if state.palettes.len() == 0 {
        let mut pal = Palette::default();
        pal.name = "Default".to_string();
        pal.tiles = vec![[[0; 8]; 8]; 16];
        state.palettes.push(pal);
    }
    update_palette_order(state);
    state.palette_idx = 0;
    Ok(())
}

pub fn delete_palette(state: &mut EditorState, name: &str) -> Result<()> {
    let pal_dir = get_palette_dir(state)?;
    let path = pal_dir.join(format!("{}.json", name));
    info!("Deleting {}", path.display());
    std::fs::remove_file(path)?;
    Ok(())
}

fn get_screen_dir(state: &EditorState) -> Result<PathBuf> {
    Ok(get_project_dir(state)?.join("Screens"))
}

pub fn load_screen_list(state: &mut EditorState) -> Result<()> {
    let screen_dir = get_screen_dir(state)?;

    let pattern = format!("{}/*/*.json", screen_dir.display());
    state.theme_names.clear();
    state.screen_names.clear();
    for entry in glob::glob(&pattern)? {
        let path = entry?;
        let theme_name = path.file_stem().unwrap().to_str().unwrap();
        let parent_path = path.parent().unwrap();
        let screen_name = parent_path.file_name().unwrap().to_owned();
        state.theme_names.push(theme_name.to_string());
        state.screen_names.push(screen_name.into_string().unwrap());
    }
    if state.theme_names.len() == 0 {
        state.theme_names.push("Base".to_string());
    }
    if state.screen_names.len() == 0 {
        state.screen_names.push("Example".to_string());
        state.screen.name = "Example".to_string();
        state.screen.theme = "Base".to_string();
        state.screen.size = (2, 2);
        for y in 0..2 {
            for x in 0..2 {
                state.screen.subscreens.push(Subscreen {
                    position: (x, y),
                    palettes: [[0; 32]; 32],
                    tiles: [[0; 32]; 32],
                });
            }
        }
        state.screen.modified = true;
        save_screen(state)?;
    }
    state.screen_names.sort();
    state.screen_names.dedup();
    state.theme_names.sort();
    state.theme_names.dedup();
    Ok(())
}

pub fn load_screen(state: &mut EditorState, name: &str, theme: &str) -> Result<()> {
    let screen_path = get_screen_dir(state)?
        .join(name)
        .join(format!("{}.json", theme));
    state.screen = load_json(&screen_path)?;
    state.screen.name = name.to_owned();
    state.screen.theme = theme.to_owned();
    Ok(())
}

pub fn save_screen(state: &mut EditorState) -> Result<()> {
    let screen_dir = get_screen_dir(state)?;
    if state.screen.modified {
        let screen_filename = format!("{}.json", state.screen.theme);
        let screen_path = screen_dir.join(&state.screen.name).join(screen_filename);
        save_json(&screen_path, &state.screen)?;
        state.screen.modified = false;
    }
    Ok(())
}

pub fn copy_screen_theme(
    state: &mut EditorState,
    name: &str,
    old_theme: &str,
    new_theme: &str,
) -> Result<()> {
    let screen_dir = get_screen_dir(state)?.join(name);
    let old_screen_path = screen_dir.join(format!("{}.json", old_theme));
    let new_screen_path = screen_dir.join(format!("{}.json", new_theme));
    info!(
        "Copying {} to {}",
        old_screen_path.display(),
        new_screen_path.display()
    );
    std::fs::copy(old_screen_path, new_screen_path)?;
    Ok(())
}

pub fn rename_screen(state: &mut EditorState, new_name: &str) -> Result<()> {
    let old_screen_path = get_screen_dir(state)?.join(&state.screen.name);
    let new_screen_path = get_screen_dir(state)?.join(new_name);
    info!(
        "Renaming {} to {} (directory)",
        old_screen_path.display(),
        new_screen_path.display()
    );
    std::fs::rename(old_screen_path, new_screen_path)?;
    Ok(())
}

pub fn rename_screen_theme(
    state: &EditorState,
    screen_name: &str,
    old_theme: &str,
    new_theme: &str,
) -> Result<()> {
    let screen_dir = get_screen_dir(state)?.join(screen_name);
    let old_screen_path = screen_dir.join(format!("{}.json", old_theme));
    let new_screen_path = screen_dir.join(format!("{}.json", new_theme));
    info!(
        "Renaming {} to {}",
        old_screen_path.display(),
        new_screen_path.display()
    );
    std::fs::rename(old_screen_path, new_screen_path)?;
    Ok(())
}

pub fn delete_screen(state: &mut EditorState, name: &str) -> Result<()> {
    let screen_path = get_screen_dir(state)?.join(name);
    info!("Deleting {}", screen_path.display());
    std::fs::remove_dir_all(screen_path)?;
    Ok(())
}

pub fn delete_screen_theme(state: &mut EditorState, screen_name: &str, theme: &str) -> Result<()> {
    let screen_dir = get_screen_dir(state)?.join(screen_name);
    let screen_path = screen_dir.join(format!("{}.json", theme));
    info!("Deleting {}", screen_path.display());
    std::fs::remove_file(screen_path)?;
    Ok(())
}

pub fn save_project(state: &mut EditorState) -> Result<()> {
    if state.global_config.project_dir.is_none() {
        return Ok(());
    }
    save_palettes(state)?;
    save_screen(state)?;
    Ok(())
}

pub fn load_project(state: &mut EditorState) -> Result<()> {
    load_palettes(state)?;
    load_screen_list(state)?;
    load_screen(
        state,
        &state.screen_names[0].clone(),
        &state.theme_names[0].clone(),
    )?;
    Ok(())
}
