use tauri_plugin_shell::ShellExt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;
use serde::{Serialize, Deserialize};
use rusqlite::{params, Connection};
use std::collections::HashMap;

/// Represents the categorized files within a scanned Ren'Py project.
#[derive(Serialize, Deserialize)]
pub struct ProjectFiles {
    pub rpa_files: Vec<String>,
    pub rpyc_files: Vec<String>,
    pub rpy_files: Vec<String>,
}

/// Represents a single translation entry stored in the local SQLite cache.
#[derive(Serialize, Deserialize)]
pub struct DbEntry {
    pub id: String,
    pub file_path: String,
    pub original: String,
    pub translation: String,
    pub status: String,
}

/// Stores statistical data regarding translation progression for a specific file.
#[derive(Serialize, Deserialize)]
pub struct FileStats {
    pub total: i32,
    pub translated: i32,
}

/// Represents a localized image entry for the gallery manager.
#[derive(Serialize, Deserialize)]
pub struct ImageEntry {
    pub original_path: String,
    pub rel_path: String,
    pub is_translated: bool,
    pub translated_path: Option<String>,
}

/// Security helper to prevent Path Traversal attacks.
/// Ensures the requested file operation is bounded within the project's root directory.
fn verify_project_path(project_path: &str, file_path: &str) -> Result<(), String> {
    if file_path.contains("..") {
        return Err("Path Traversal detected: '..' characters are not allowed.".to_string());
    }

    let p_path = Path::new(project_path).to_string_lossy().replace("\\", "/").to_lowercase();
    let f_path = Path::new(file_path).to_string_lossy().replace("\\", "/").to_lowercase();
    
    if !f_path.starts_with(&p_path) {
        return Err("Access denied: Target file is located outside the project root directory.".to_string());
    }
    
    Ok(())
}

/// Scans the root directory to identify the primary Ren'Py executable.
/// Ignores uninstallers and crash reporters.
fn find_game_exe(root_dir: &Path) -> Option<PathBuf> {
    if let Ok(entries) = fs::read_dir(root_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() {
                if let Some(ext) = p.extension() {
                    if ext.to_string_lossy().to_lowercase() == "exe" {
                        let file_name = p.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
                        if !file_name.contains("unins") && !file_name.contains("crash") {
                            return Some(p);
                        }
                    }
                }
            }
        }
    }
    None
}

/// Establishes a connection to the local SQLite caching database.
/// Configures WAL (Write-Ahead Logging) for optimized read/write performance.
fn get_db_conn(project_path: &str) -> Result<Connection, String> {
    let db_path = Path::new(project_path).join("renforge.db");
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;"
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS translations (
            id TEXT PRIMARY KEY,
            file_path TEXT,
            original TEXT,
            translation TEXT,
            status TEXT
        )",[],
    ).map_err(|e| e.to_string())?;
    
    Ok(conn)
}

/// Securely reads a .rpy file within the project scope.
#[tauri::command]
fn read_rpy_file(project_path: String, file_path: String) -> Result<String, String> {
    verify_project_path(&project_path, &file_path)?;
    fs::read_to_string(&file_path).map_err(|e| format!("Ошибка чтения: {}", e))
}

/// Securely overwrites a .rpy file within the project scope.
#[tauri::command]
fn write_rpy_file(project_path: String, file_path: String, content: String) -> Result<(), String> {
    verify_project_path(&project_path, &file_path)?;
    fs::write(&file_path, content).map_err(|e| format!("Ошибка сохранения: {}", e))
}

/// Reads a plain text file (used for CSV imports).
/// Restricted to `.csv` extension to prevent arbitrary file reading.
#[tauri::command]
fn read_text_file(path: String) -> Result<String, String> {
    if !path.to_lowercase().ends_with(".csv") {
        return Err("Access denied: File must have a .csv extension".to_string());
    }
    fs::read_to_string(&path).map_err(|e| format!("Ошибка чтения: {}", e))
}

/// Writes a plain text file (used for CSV exports).
/// Restricted to `.csv` extension to prevent arbitrary file execution.
#[tauri::command]
fn write_text_file(path: String, content: String) -> Result<(), String> {
    if !path.to_lowercase().ends_with(".csv") {
        return Err("Access denied: File must have a .csv extension".to_string());
    }
    fs::write(&path, content).map_err(|e| format!("Ошибка сохранения: {}", e))
}

/// Crawls the project directory and maps all specific files (.rpa, .rpyc, .rpy).
#[tauri::command]
fn scan_project(path: String) -> Result<ProjectFiles, String> {
    let mut rpa_files = Vec::new();
    let mut rpyc_files = Vec::new();
    let mut rpy_files = Vec::new();

    let game_dir = Path::new(&path).join("game");
    
    if game_dir.exists() {
        for entry in WalkDir::new(&game_dir).into_iter().filter_map(|e| e.ok()) {
            let path_str = entry.path().display().to_string();
            if path_str.ends_with(".rpa") { rpa_files.push(path_str); }
            else if path_str.ends_with(".rpyc") { rpyc_files.push(path_str); }
            else if path_str.ends_with(".rpy") { rpy_files.push(path_str); }
        }
    }
    Ok(ProjectFiles { rpa_files, rpyc_files, rpy_files })
}

/// Parses local variables mapped to string characters to display accurate speaker names.
#[tauri::command]
fn get_character_mapping(project_path: String) -> Result<HashMap<String, String>, String> {
    let mut mapping = HashMap::new();
    let game_dir = Path::new(&project_path).join("game");

    if !game_dir.exists() { return Ok(mapping); }

    for entry in WalkDir::new(&game_dir).into_iter().filter_map(|e| e.ok()) {
        let p = entry.path();
        if p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("rpy") {
            if let Ok(content) = fs::read_to_string(p) {
                for line in content.lines() {
                    let trim_line = line.trim();
                    if trim_line.starts_with("define ") && trim_line.contains("Character") {
                        let parts: Vec<&str> = trim_line.splitn(2, '=').collect();
                        if parts.len() == 2 {
                            let code = parts[0].replace("define", "").trim().to_string();
                            let val = parts[1];
                            if let Some(start) = val.find('"') {
                                if let Some(end) = val[start+1..].find('"') {
                                    let name = val[start+1..start+1+end].to_string();
                                    mapping.insert(code, name);
                                }
                            } else if let Some(start) = val.find('\'') {
                                if let Some(end) = val[start+1..].find('\'') {
                                    let name = val[start+1..start+1+end].to_string();
                                    mapping.insert(code, name);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(mapping)
}

/// Lookups a string within the SQLite cache for instant matching.
#[tauri::command]
fn search_in_db(project_path: String, query: String) -> Result<Vec<DbEntry>, String> {
    let conn = get_db_conn(&project_path)?;
    let mut stmt = conn.prepare(
        "SELECT id, file_path, original, translation, status FROM translations 
         WHERE original LIKE ?1 OR translation LIKE ?1 LIMIT 100"
    ).map_err(|e| e.to_string())?;

    let rows = stmt.query_map(params![format!("%{}%", query)], |row| {
        Ok(DbEntry { id: row.get(0)?, file_path: row.get(1)?, original: row.get(2)?, translation: row.get(3)?, status: row.get(4)? })
    }).map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    for row in rows { results.push(row.map_err(|e| e.to_string())?); }
    Ok(results)
}

/// Collects total vs translated line numbers for the UI progress bars.
#[tauri::command]
fn get_translation_stats(project_path: String) -> Result<HashMap<String, FileStats>, String> {
    let conn = get_db_conn(&project_path)?;
    let mut stmt = conn.prepare(
        "SELECT file_path, COUNT(id), SUM(CASE WHEN status = 'translated' THEN 1 ELSE 0 END) 
         FROM translations GROUP BY file_path"
    ).map_err(|e| e.to_string())?;

    let rows = stmt.query_map([], |row| {
        let path: String = row.get(0)?;
        let total: i32 = row.get(1)?;
        let translated: i32 = row.get(2).unwrap_or(0);
        Ok((path, FileStats { total, translated }))
    }).map_err(|e| e.to_string())?;

    let mut map = HashMap::new();
    for row in rows {
        if let Ok((path, stats)) = row {
            map.insert(path, stats);
        }
    }
    Ok(map)
}

/// Updates or inserts a line translation status into the SQLite cache.
#[tauri::command]
fn upsert_translation(project_path: String, entry: DbEntry) -> Result<(), String> {
    let conn = get_db_conn(&project_path)?;
    conn.execute(
        "INSERT OR REPLACE INTO translations (id, file_path, original, translation, status)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![entry.id, entry.file_path, entry.original, entry.translation, entry.status],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

/// Triggers the Ren'Py engine via subprocess to generate translation files.
/// Forces `RENPY_DEVELOPER=1` to bypass certain user restrictions.
#[tauri::command]
fn generate_translations(path: String, target_lang: String) -> Result<String, String> {
    let root_dir = Path::new(&path);
    let exe = find_game_exe(root_dir).ok_or("Не найден главный .exe файл игры.")?;
    
    let output = Command::new(&exe)
        .current_dir(root_dir)
        .env("RENPY_DEVELOPER", "1")
        .arg(&path)
        .arg("translate")
        .arg(&target_lang)
        .output()
        .map_err(|e| format!("Системная ошибка выполнения: {}", e))?;

    let tl_dir = root_dir.join("game").join("tl").join(&target_lang);
    
    if tl_dir.exists() && tl_dir.read_dir().map(|mut i| i.next().is_some()).unwrap_or(false) { 
        Ok("Структура переводов успешно сгенерирована!".to_string()) 
    } else { 
        let log_path = root_dir.join("log.txt");
        let tb_path = root_dir.join("traceback.txt");
        
        let mut err_details = String::new();
        if let Ok(log) = fs::read_to_string(log_path) {
            err_details.push_str(&format!("\n[ ИЗ ЛОГА ИГРЫ (log.txt) ]:\n{:.800}", log));
        }
        if let Ok(tb) = fs::read_to_string(tb_path) {
            err_details.push_str(&format!("\n[ ОШИБКА ДВИЖКА (traceback) ]:\n{:.800}", tb));
        }
        
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        
        Err(format!(
            "Генерация провалилась! Движок упал и не смог прочитать вскрытые скрипты.\n{}\n{}", 
            stderr, err_details
        ))
    }
}

/// Aggressively patches the game environment to force language application.
/// Removes Read-Only attributes, deletes old compiled bytecode (`.rpyc`),
/// and injects a hidden screen (`00_renforge_patch.rpy`) for live reloads.
#[tauri::command]
fn apply_renforge_patch(project_path: String, target_lang: String, force_font: bool) -> Result<(), String> {
    let root_dir = Path::new(&project_path);
    let game_dir = root_dir.join("game");
    
    for entry in walkdir::WalkDir::new(&root_dir).into_iter().filter_map(|e| e.ok()) {
        if let Ok(metadata) = entry.metadata() {
            let mut perms = metadata.permissions();
            if perms.readonly() {
                perms.set_readonly(false);
                let _ = fs::set_permissions(entry.path(), perms);
            }
        }
    }

    let mut patch_content = format!(r#"
init -999 python:
    config.developer = True
    config.console = True

screen renforge_lang_forcer():
    zorder 9999
    if _preferences.language != "{target_lang}":
        timer 0.01 action Language("{target_lang}")

init 999 python:
    if hasattr(config, "always_shown_screens"):
        if "renforge_lang_forcer" not in config.always_shown_screens:
            config.always_shown_screens.append("renforge_lang_forcer")
    elif hasattr(config, "overlay_screens"):
        if "renforge_lang_forcer" not in config.overlay_screens:
            config.overlay_screens.append("renforge_lang_forcer")
"#);

    if force_font {
        patch_content.push_str(&format!(r#"
translate {target_lang} style default:
    font "DejaVuSans.ttf"

translate {target_lang} style gui_text:
    font "DejaVuSans.ttf"

translate {target_lang} style say_dialogue:
    font "DejaVuSans.ttf"
"#));
    }

    let patch_path = game_dir.join("00_renforge_patch.rpy");
    fs::write(&patch_path, patch_content).map_err(|e| format!("Ошибка создания патча: {}", e))?;

    let cache_dir = game_dir.join("cache");
    if cache_dir.exists() { let _ = fs::remove_dir_all(cache_dir); }

    let tl_dir = game_dir.join("tl").join(&target_lang);
    if tl_dir.exists() {
        for entry in walkdir::WalkDir::new(&tl_dir).into_iter().filter_map(|e| e.ok()) {
            let p = entry.path();
            if p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("rpyc") {
                let _ = fs::remove_file(p);
            }
        }
    }

    Ok(())
}

/// Executes external Python script (unrpa) to unpack archive assets.
#[tauri::command]
async fn run_unrpa(app: tauri::AppHandle, file_path: String) -> Result<String, String> {
    let parent_dir = Path::new(&file_path).parent().unwrap().to_string_lossy().to_string();
    
    let sidecar = app.shell().sidecar("unrpa").map_err(|e| e.to_string())?;
    
    let output = sidecar.args(["--force", "RPA-3.0", "-mp", &parent_dir, &file_path])
        .output()
        .await
        .map_err(|e| e.to_string())?;
        
    if output.status.success() { 
        Ok("Распаковано".to_string()) 
    } else { 
        let err_msg = String::from_utf8_lossy(&output.stderr).to_string();
        Err(format!("Ошибка распаковки: {}", err_msg)) 
    }
}

/// Executes external Python script (unrpyc) to decompile .rpyc files into readable .rpy.
#[tauri::command]
async fn run_unrpyc(app: tauri::AppHandle, file_path: String) -> Result<String, String> {
    let sidecar = app.shell().sidecar("unrpyc").map_err(|e| e.to_string())?;
    
    let output = sidecar.arg(&file_path)
        .output()
        .await
        .map_err(|e| e.to_string())?;
        
    if output.status.success() { 
        Ok("Вскрыто".to_string()) 
    } else { 
        let err_msg = String::from_utf8_lossy(&output.stderr).to_string();
        Err(format!("Ошибка вскрытия: {}", err_msg)) 
    }
}

/// Scans the game directory for images that could be localized.
#[tauri::command]
fn get_images_list(project_path: String, target_lang: String) -> Result<Vec<ImageEntry>, String> {
    let game_dir = Path::new(&project_path).join("game");
    let tl_dir = game_dir.join("tl").join(&target_lang);
    let mut images = Vec::new();

    if !game_dir.exists() {
        return Ok(images);
    }

    for entry in WalkDir::new(&game_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        
        if path.components().any(|c| c.as_os_str() == "tl" || c.as_os_str() == "cache") {
            continue;
        }

        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|s| s.to_str()).map(|s| s.to_lowercase()) {
                if ext == "png" || ext == "jpg" || ext == "jpeg" || ext == "webp" {
                    let rel_path = path.strip_prefix(&game_dir).unwrap_or(path).to_string_lossy().replace("\\", "/");
                    let translated_path = tl_dir.join(&rel_path);
                    
                    let is_translated = translated_path.exists();
                    let trans_path_str = if is_translated {
                        Some(translated_path.to_string_lossy().to_string())
                    } else {
                        None
                    };

                    images.push(ImageEntry {
                        original_path: path.to_string_lossy().to_string(),
                        rel_path,
                        is_translated,
                        translated_path: trans_path_str,
                    });
                }
            }
        }
    }
    Ok(images)
}

/// Creates a copy of an external image and injects it into the correct translation folder.
#[tauri::command]
fn import_localized_image(project_path: String, target_lang: String, rel_path: String, source_file_path: String) -> Result<String, String> {
    let game_dir = Path::new(&project_path).join("game");
    let target_path = game_dir.join("tl").join(&target_lang).join(&rel_path);

    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Ошибка создания папок: {}", e))?;
    }

    fs::copy(&source_file_path, &target_path).map_err(|e| format!("Ошибка копирования: {}", e))?;

    Ok(target_path.to_string_lossy().to_string())
}

/// Reverts image translation by deleting the injected file.
#[tauri::command]
fn delete_localized_image(project_path: String, target_lang: String, rel_path: String) -> Result<(), String> {
    let game_dir = Path::new(&project_path).join("game");
    let target_path = game_dir.join("tl").join(&target_lang).join(&rel_path);

    if target_path.exists() {
        fs::remove_file(&target_path).map_err(|e| format!("Ошибка удаления: {}", e))?;
    }
    Ok(())
}

/// Safely opens the localized directory in the native OS File Explorer.
/// Handles the specific extended-length path (`\\?\`) bug in Windows Explorer.
#[tauri::command]
fn open_in_explorer(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let mut win_path = path.replace("/", "\\");
        
        if win_path.starts_with("\\\\?\\") {
            win_path = win_path.replace("\\\\?\\", "");
        }

        Command::new("explorer")
            .arg("/select,")
            .arg(&win_path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg("-R").arg(&path).spawn().map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        let parent = Path::new(&path).parent().unwrap_or(Path::new(&path));
        Command::new("xdg-open").arg(parent).spawn().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init()) 
        .plugin(tauri_plugin_opener::init()) 
        .invoke_handler(tauri::generate_handler![
            read_rpy_file, write_rpy_file, read_text_file, write_text_file, scan_project, run_unrpa, run_unrpyc, 
            generate_translations, search_in_db, upsert_translation, apply_renforge_patch, get_character_mapping,
            get_translation_stats, get_images_list, import_localized_image, delete_localized_image, open_in_explorer
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}