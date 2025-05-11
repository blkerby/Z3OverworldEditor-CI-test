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
    state::{
        ensure_areas_non_empty, ensure_palettes_non_empty, ensure_themes_non_empty, EditorState,
        Palette,
    },
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
    ensure_palettes_non_empty(state);
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

fn get_area_dir(state: &EditorState) -> Result<PathBuf> {
    Ok(get_project_dir(state)?.join("Areas"))
}

pub fn load_area_list(state: &mut EditorState) -> Result<()> {
    let area_dir = get_area_dir(state)?;

    let pattern = format!("{}/*/*.json", area_dir.display());
    state.theme_names.clear();
    state.area_names.clear();
    for entry in glob::glob(&pattern)? {
        let path = entry?;
        let theme_name = path.file_stem().unwrap().to_str().unwrap();
        let parent_path = path.parent().unwrap();
        let area_name = parent_path.file_name().unwrap().to_owned();
        state.theme_names.push(theme_name.to_string());
        state.area_names.push(area_name.into_string().unwrap());
    }
    ensure_themes_non_empty(state);
    ensure_areas_non_empty(state);
    save_area(state)?;
    state.area_names.sort();
    state.area_names.dedup();
    state.theme_names.sort();
    state.theme_names.dedup();
    Ok(())
}

pub fn load_area(state: &mut EditorState, name: &str, theme: &str) -> Result<()> {
    let area_path = get_area_dir(state)?
        .join(name)
        .join(format!("{}.json", theme));
    state.area = load_json(&area_path)?;
    state.area.name = name.to_owned();
    state.area.theme = theme.to_owned();
    Ok(())
}

pub fn save_area(state: &mut EditorState) -> Result<()> {
    let area_dir = get_area_dir(state)?;
    if state.area.modified {
        let area_filename = format!("{}.json", state.area.theme);
        let area_path = area_dir.join(&state.area.name).join(area_filename);
        save_json(&area_path, &state.area)?;
        state.area.modified = false;
    }
    Ok(())
}

pub fn copy_area_theme(
    state: &mut EditorState,
    name: &str,
    old_theme: &str,
    new_theme: &str,
) -> Result<()> {
    let area_dir = get_area_dir(state)?.join(name);
    let old_area_path = area_dir.join(format!("{}.json", old_theme));
    let new_area_path = area_dir.join(format!("{}.json", new_theme));
    info!(
        "Copying {} to {}",
        old_area_path.display(),
        new_area_path.display()
    );
    std::fs::copy(old_area_path, new_area_path)?;
    Ok(())
}

pub fn rename_area(state: &mut EditorState, new_name: &str) -> Result<()> {
    let old_area_path = get_area_dir(state)?.join(&state.area.name);
    let new_area_path = get_area_dir(state)?.join(new_name);
    info!(
        "Renaming {} to {} (directory)",
        old_area_path.display(),
        new_area_path.display()
    );
    std::fs::rename(old_area_path, new_area_path)?;
    Ok(())
}

pub fn rename_area_theme(
    state: &EditorState,
    area_name: &str,
    old_theme: &str,
    new_theme: &str,
) -> Result<()> {
    let area_dir = get_area_dir(state)?.join(area_name);
    let old_area_path = area_dir.join(format!("{}.json", old_theme));
    let new_area_path = area_dir.join(format!("{}.json", new_theme));
    info!(
        "Renaming {} to {}",
        old_area_path.display(),
        new_area_path.display()
    );
    std::fs::rename(old_area_path, new_area_path)?;
    Ok(())
}

pub fn delete_area(state: &mut EditorState, name: &str) -> Result<()> {
    let area_path = get_area_dir(state)?.join(name);
    info!("Deleting {}", area_path.display());
    std::fs::remove_dir_all(area_path)?;
    Ok(())
}

pub fn delete_area_theme(state: &mut EditorState, area_name: &str, theme: &str) -> Result<()> {
    let area_dir = get_area_dir(state)?.join(area_name);
    let area_path = area_dir.join(format!("{}.json", theme));
    info!("Deleting {}", area_path.display());
    std::fs::remove_file(area_path)?;
    Ok(())
}

pub fn save_project(state: &mut EditorState) -> Result<()> {
    if state.global_config.project_dir.is_none() {
        return Ok(());
    }
    save_palettes(state)?;
    save_area(state)?;
    Ok(())
}

pub fn load_project(state: &mut EditorState) -> Result<()> {
    load_palettes(state)?;
    load_area_list(state)?;
    load_area(
        state,
        &state.area_names[0].clone(),
        &state.theme_names[0].clone(),
    )?;
    Ok(())
}
