use std::{
  path::{Path, PathBuf},
  time::UNIX_EPOCH,
};

use ignore::WalkBuilder;
use rand::{seq::SliceRandom, thread_rng};
use rayon::prelude::*;
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ImageEntry {
  path: String,
  file_name: String,
  parent_folder: String,
  modified_unix: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct FolderBrowseResult {
  root: String,
  total: usize,
  items: Vec<ImageEntry>,
}

#[tauri::command]
fn browse_images(folder: String, random_mode: bool, include_subfolders: bool) -> Result<FolderBrowseResult, String> {
  let root = dunce::canonicalize(&folder)
    .map_err(|e| format!("Cannot open folder: {e}"))?;

  if !root.is_dir() {
    return Err("Selected path is not a directory".to_string());
  }

  let mut walk = WalkBuilder::new(&root);
  walk.hidden(false);
  walk.git_ignore(true);
  walk.git_global(true);
  walk.git_exclude(true);
  walk.ignore(true);
  walk.parents(true);
  walk.max_depth(if include_subfolders { None } else { Some(1) });

  let mut image_paths = Vec::<PathBuf>::new();
  for item in walk.build() {
    let entry = match item {
      Ok(v) => v,
      Err(_) => continue,
    };

    if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
      continue;
    }

    if is_image_path(entry.path()) {
      image_paths.push(entry.into_path());
    }
  }

  let mut items: Vec<ImageEntry> = image_paths
    .par_iter()
    .filter_map(|path| map_entry(path).ok())
    .collect();

  if random_mode {
    items.shuffle(&mut thread_rng());
  } else {
    items.sort_by_key(|x| x.file_name.to_lowercase());
  }

  let total = items.len();

  Ok(FolderBrowseResult {
    root: root.to_string_lossy().to_string(),
    total,
    items,
  })
}

fn map_entry(path: &Path) -> Result<ImageEntry, String> {
  let meta = std::fs::metadata(path).map_err(|e| format!("metadata error: {e}"))?;
  let modified_unix = meta
    .modified()
    .ok()
    .and_then(|v| v.duration_since(UNIX_EPOCH).ok())
    .map(|v| v.as_secs())
    .unwrap_or(0);

  let file_name = path
    .file_name()
    .map(|v| v.to_string_lossy().to_string())
    .unwrap_or_else(|| "(unknown)".to_string());

  let parent_folder = path
    .parent()
    .and_then(|p| p.file_name())
    .map(|v| v.to_string_lossy().to_string())
    .unwrap_or_else(|| "-".to_string());

  Ok(ImageEntry {
    path: path.to_string_lossy().to_string(),
    file_name,
    parent_folder,
    modified_unix,
  })
}

fn is_image_path(path: &Path) -> bool {
  let ext = path
    .extension()
    .and_then(|s| s.to_str())
    .map(|s| s.to_ascii_lowercase());

  match ext.as_deref() {
    Some("jpg")
    | Some("jpeg")
    | Some("png")
    | Some("gif")
    | Some("bmp")
    | Some("webp")
    | Some("tiff")
    | Some("tif")
    | Some("avif") => true,
    _ => false,
  }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_fs::init())
    .plugin(tauri_plugin_opener::init())
    .plugin(tauri_plugin_store::Builder::new().build())
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![browse_images])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
