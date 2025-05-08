use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use json_pretty_compact::PrettyCompactFormatter;
use log::info;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Serializer;

use crate::state::{EditorState, Palette};

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
        let pal: Palette = load_json(&path)?;
        state.palettes.push(pal);
    }
    if state.palettes.len() == 0 {
        let mut pal = Palette::default();
        pal.name = "Default".to_string();
        state.palettes.push(pal);
    }
    state.palettes.sort_by(|x, y| x.name.cmp(&y.name));
    Ok(())
}

pub fn delete_palette(state: &mut EditorState, name: &str) -> Result<()> {
    let pal_dir = get_palette_dir(state)?;
    let path = pal_dir.join(format!("{}.json", name));
    info!("Deleting {}", path.display());
    std::fs::remove_file(path)?;
    Ok(())
}

fn load_tiles(state: &mut EditorState) -> Result<()> {
    let tiles = &mut state.palettes[state.palette_idx].tiles;
    if tiles.len() == 0 {
        tiles.extend(vec![[[0; 8]; 8]; 16]);
    }
    Ok(())
}

pub fn save_project(state: &mut EditorState) -> Result<()> {
    if state.global_config.project_dir.is_none() {
        return Ok(());
    }
    save_palettes(state)?;
    Ok(())
}

pub fn load_project(state: &mut EditorState) -> Result<()> {
    load_palettes(state)?;
    load_tiles(state)?;
    Ok(())
}
