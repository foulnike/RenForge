pub mod db;
pub mod error;
pub mod models;

use error::AppError;
use models::{AudioEntry, ImageEntry, ProjectFiles};

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tauri_plugin_shell::ShellExt;
use walkdir::WalkDir;

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

#[tauri::command]
fn read_rpy_file(project_path: String, file_path: String) -> Result<String, String> {
    verify_project_path(&project_path, &file_path)?;
    fs::read_to_string(&file_path).map_err(|e| format!("Ошибка чтения: {}", e))
}

#[tauri::command]
fn write_rpy_file(project_path: String, file_path: String, content: String) -> Result<(), String> {
    verify_project_path(&project_path, &file_path)?;
    fs::write(&file_path, content).map_err(|e| format!("Ошибка сохранения: {}", e))
}

#[tauri::command]
fn read_text_file(path: String) -> Result<String, String> {
    let p_lower = path.to_lowercase();
    if !p_lower.ends_with(".csv") && !p_lower.ends_with(".json") {
        return Err("Access denied: File must have a .csv or .json extension".to_string());
    }
    std::fs::read_to_string(&path).map_err(|e| format!("Ошибка чтения: {}", e))
}

#[tauri::command]
fn write_text_file(path: String, content: String) -> Result<(), String> {
    let p_lower = path.to_lowercase();
    if !p_lower.ends_with(".csv") && !p_lower.ends_with(".json") {
        return Err("Access denied: File must have a .csv or .json extension".to_string());
    }
    std::fs::write(&path, content).map_err(|e| format!("Ошибка сохранения: {}", e))
}

#[tauri::command]
fn write_fallback_file(project_path: String, target_lang: String, orig_rel_path: String, strings: Vec<String>) -> Result<(), String> {
    let game_dir = std::path::Path::new(&project_path).join("game");
    let tl_dir = game_dir.join("tl").join(&target_lang);
    let target_path = tl_dir.join(&orig_rel_path);

    if let Some(parent) = target_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let mut preserved_content = String::new();
    let mut local_olds = std::collections::HashSet::new();

    if target_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&target_path) {
            if let Some(idx) = content.find("# RENFORGE_MANUAL_GEN") {
                preserved_content = content[..idx].to_string();
            } else {
                preserved_content = content;
                if !preserved_content.ends_with('\n') {
                    preserved_content.push('\n');
                }
                preserved_content.push('\n');
            }

            for line in preserved_content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("old ") {
                    local_olds.insert(trimmed[4..].trim().to_string());
                }
            }
        }
    }

    let mut existing_olds = std::collections::HashSet::new();
    for entry in walkdir::WalkDir::new(&tl_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("rpy") {
            if path.canonicalize().unwrap_or_default() == target_path.canonicalize().unwrap_or_default() { continue; }
            
            if let Ok(content) = std::fs::read_to_string(path) {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("old ") {
                        existing_olds.insert(trimmed[4..].trim().to_string());
                    }
                }
            }
        }
    }

    let mut manual_block = String::from("# RENFORGE_MANUAL_GEN\n");
    manual_block.push_str(&format!("translate {} strings:\n\n", target_lang));

    let mut added = 0;

    for s in strings {
        let trimmed_s = s.trim();
        
        if local_olds.contains(trimmed_s) { continue; }
        local_olds.insert(trimmed_s.to_string());
        
        if existing_olds.contains(trimmed_s) {
            manual_block.push_str(&format!("    # ДУБЛИКАТ (Уже переведено в другом файле):\n    # old {}\n    # new {}\n\n", s, s));
        } else {
            manual_block.push_str(&format!("    old {}\n    new {}\n\n", s, s));
            added += 1;
        }
    }

    if added == 0 { manual_block.push_str("    pass\n"); }
    
    let final_content = format!("{}{}", preserved_content, manual_block);
    std::fs::write(&target_path, final_content).map_err(|e| format!("Ошибка сохранения: {}", e))?;
    
    Ok(())
}

#[tauri::command]
async fn scan_project(path: String, target_lang: String) -> Result<ProjectFiles, AppError> {
    tokio::task::spawn_blocking(move || {
        let mut rpa_files = Vec::new();
        let mut rpyc_files = Vec::new();
        let mut rpy_files = Vec::new();
        let mut tl_files = Vec::new();
        let mut manual_tl_files = Vec::new();

        let game_dir = Path::new(&path).join("game");
        let tl_dir_str = format!("/tl/{}/", target_lang);
        let tl_dir_str_win = format!("\\tl\\{}\\", target_lang);
        
        if game_dir.exists() {
            for entry in walkdir::WalkDir::new(&game_dir).into_iter().filter_map(|e| e.ok()) {
                let path_str = entry.path().display().to_string();
                let is_tl = path_str.contains(&tl_dir_str) || path_str.contains(&tl_dir_str_win);

                if path_str.ends_with(".rpa") { 
                    rpa_files.push(path_str); 
                }
                else if path_str.ends_with(".rpyc") && !is_tl { 
                    rpyc_files.push(path_str); 
                }
                else if path_str.ends_with(".rpy") { 
                    if is_tl {
                        tl_files.push(path_str.clone());
                        
                        use std::io::{BufRead, BufReader};
                        if let Ok(file) = std::fs::File::open(&path_str) {
                            let mut reader = BufReader::new(file);
                            let mut first_line = String::new();
                            if reader.read_line(&mut first_line).is_ok() {
                                if first_line.contains("RENFORGE_MANUAL_GEN") {
                                    manual_tl_files.push(path_str);
                                }
                            }
                        }
                    } else {
                        rpy_files.push(path_str);
                    }
                }
            }
        }
        Ok(ProjectFiles { rpa_files, rpyc_files, rpy_files, tl_files, manual_tl_files })
    })
    .await
    .map_err(|e| AppError::Custom(format!("Сбой фонового потока: {}", e)))?
}

#[tauri::command]
fn create_empty_translation(project_path: String, target_lang: String, orig_rel_path: String) -> Result<String, String> {
    let game_dir = Path::new(&project_path).join("game");
    let target_path = game_dir.join("tl").join(&target_lang).join(&orig_rel_path);

    if let Some(parent) = target_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Ошибка создания папок: {}", e))?;
    }

    let content = format!("# RENFORGE_MANUAL_GEN\ntranslate {} strings:\n    pass\n", target_lang);
    std::fs::write(&target_path, content).map_err(|e| format!("Ошибка создания файла: {}", e))?;

    Ok("Пустой fallback-файл успешно создан".to_string())
}

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

#[tauri::command]
fn generate_translations(path: String, target_lang: String) -> Result<String, String> {
    let root_dir = Path::new(&path);
    let exe = find_game_exe(root_dir).ok_or("Не найден главный .exe файл игры.")?;
    let tl_dir = root_dir.join("game").join("tl").join(&target_lang);

    let max_retries = 20;
    let mut deleted_files = Vec::new();
    let mut last_error = String::new();

    for _ in 0..max_retries {
        let output = Command::new(&exe)
            .current_dir(root_dir)
            .env("RENPY_DEVELOPER", "1")
            .arg(&path)
            .arg("translate")
            .arg(&target_lang)
            .output()
            .map_err(|e| format!("Системная ошибка выполнения: {}", e))?;

        if tl_dir.exists() && tl_dir.read_dir().map(|mut i| i.next().is_some()).unwrap_or(false) { 
            let mut success_msg = "Структура переводов успешно сгенерирована!".to_string();
            if !deleted_files.is_empty() {
                success_msg.push_str(&format!("\n(Авто-исправлено {} сломанных скриптов интерфейса)", deleted_files.len()));
            }
            return Ok(success_msg); 
        }
        
        let log_path = root_dir.join("log.txt");
        let tb_path = root_dir.join("traceback.txt");
        
        let mut err_details = String::new();
        if let Ok(log) = fs::read_to_string(&log_path) {
            err_details.push_str(&log);
        }
        if let Ok(tb) = fs::read_to_string(&tb_path) {
            err_details.push_str("\n");
            err_details.push_str(&tb);
        }
        
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        err_details.push_str("\n");
        err_details.push_str(&stderr);

        let mut files_deleted_this_round = 0;
        
        for line in err_details.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("File \"") && trimmed.contains("\", line ") {
                let start = 6;
                if let Some(end) = trimmed[start..].find("\", line ") {
                    let rel_path = &trimmed[start..start+end];
                    
                    if rel_path.ends_with(".rpy") {
                        let abs_path = root_dir.join(rel_path);
                        if abs_path.exists() {
                            if fs::remove_file(&abs_path).is_ok() {
                                deleted_files.push(rel_path.to_string());
                                files_deleted_this_round += 1;
                            }
                        }
                    }
                }
            }
        }

        if files_deleted_this_round == 0 {
            last_error = format!(
                "Генерация провалилась по неизвестной причине.\n{}\n[ ИЗ ЛОГА ИГРЫ ]:\n{:.800}", 
                stderr, err_details
            );
            break;
        }
    }

    if last_error.is_empty() {
        Err("Превышен лимит попыток автоматического исправления. В игре слишком много сломанных скриптов.".to_string())
    } else {
        Err(last_error)
    }
}

#[tauri::command]
fn apply_renforge_patch(
    project_path: String, 
    target_lang: String, 
    force_font: bool,
    custom_font_path: Option<String>,
    custom_font_name: Option<String>
) -> Result<(), String> {
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
        let font_name = custom_font_name.unwrap_or_else(|| "DejaVuSans.ttf".to_string());
        
        if let Some(src_path) = custom_font_path {
            let dest_path = game_dir.join(&font_name);
            if Path::new(&src_path).exists() {
                fs::copy(&src_path, &dest_path).map_err(|e| format!("Ошибка копирования шрифта: {}", e))?;
            }
        }

        patch_content.push_str(&format!("\ntranslate {} python:\n", target_lang));
        patch_content.push_str(&format!("    gui.text_font = \"{}\"\n", font_name));
        patch_content.push_str(&format!("    gui.name_text_font = \"{}\"\n", font_name));
        patch_content.push_str(&format!("    gui.interface_text_font = \"{}\"\n", font_name));
        patch_content.push_str(&format!("    gui.button_text_font = \"{}\"\n", font_name));
        patch_content.push_str(&format!("    gui.choice_button_text_font = \"{}\"\n", font_name));

        let styles = vec![
            "default", "gui_text", "say_dialogue", "say_thought", "say_label",
            "button_text", "choice_button_text", "slot_button_text", "prompt_text",
            "namebox_text", "interface_text", "nvl_dialogue", "nvl_thought", "nvl_label",
            "text", "quick_button_text", "radio_button_text", "check_button_text",
            "navigation_button_text", "history_text", "history_name_text",
            "confirm_prompt_text", "confirm_button_text", "pref_label_text",
            "pref_vbox", "slider_text", "slider_label_text", "slider_button_text",
            "notify_text", "ruby_text", "alt_text", "tooltip_text", "vpgrid_text"
        ];

        for style in styles {
            patch_content.push_str(&format!("\ntranslate {} style {}:\n", target_lang, style));
            patch_content.push_str(&format!("    font \"{}\"\n", font_name));
        }
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

#[tauri::command]
fn delete_localized_image(project_path: String, target_lang: String, rel_path: String) -> Result<(), String> {
    let game_dir = Path::new(&project_path).join("game");
    let target_path = game_dir.join("tl").join(&target_lang).join(&rel_path);

    if target_path.exists() {
        fs::remove_file(&target_path).map_err(|e| format!("Ошибка удаления: {}", e))?;
    }
    Ok(())
}

fn build_audio_mapping(game_dir: &Path) -> HashMap<String, (String, String)> {
    let mut mapping = HashMap::new();
    
    for entry in WalkDir::new(game_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        
        if path.components().any(|c| c.as_os_str() == "tl" || c.as_os_str() == "cache") {
            continue;
        }
        
        if path.extension().and_then(|s| s.to_str()) == Some("rpy") {
            let rel_script_path = path.strip_prefix(game_dir).unwrap_or(path).to_string_lossy().replace("\\", "/");
            
            if let Ok(content) = fs::read_to_string(path) {
                let mut last_voice_filename: Option<String> = None;
                
                for line in content.lines() {
                    let trim_line = line.trim();
                    
                    if trim_line.is_empty() || trim_line.starts_with('#') {
                        continue;
                    }
                    
                    if trim_line.starts_with("voice ") || trim_line.starts_with("play voice ") {
                        if let Some(start) = trim_line.find('"').or_else(|| trim_line.find('\'')) {
                            if let Some(end) = trim_line[start+1..].find(trim_line.chars().nth(start).unwrap()) {
                                let full_path = trim_line[start+1..start+1+end].replace("\\", "/");
                                let fname = full_path.split('/').last().unwrap_or(&full_path).to_lowercase();
                                last_voice_filename = Some(fname);
                            }
                        }
                        continue;
                    }
                    
                    if last_voice_filename.is_some() {
                        if trim_line.starts_with("label ") || trim_line.starts_with("menu:") || 
                           trim_line.starts_with("return") || trim_line.starts_with("jump ") || 
                           trim_line.starts_with("call ") {
                            last_voice_filename = None;
                            continue;
                        }
                        
                        if trim_line.starts_with('$') || trim_line.starts_with("python:") || 
                           trim_line.starts_with("default ") || trim_line.starts_with("define ") ||
                           trim_line.starts_with("show ") || trim_line.starts_with("scene ") || 
                           trim_line.starts_with("play ") || trim_line.starts_with("hide ") ||
                           trim_line.starts_with("image ") || trim_line.starts_with("transform ") ||
                           trim_line.starts_with("camera ") || trim_line.starts_with("with ") ||
                           trim_line.starts_with("window ") {
                            continue;
                        }
                        
                        if trim_line.contains('"') || trim_line.contains('\'') {
                            mapping.insert(
                                last_voice_filename.take().unwrap(), 
                                (trim_line.to_string(), rel_script_path.clone())
                            );
                        }
                    }
                }
            }
        }
    }
    mapping
}

#[tauri::command]
fn get_audio_list(project_path: String, target_lang: String) -> Result<Vec<AudioEntry>, String> {
    let game_dir = Path::new(&project_path).join("game");
    let tl_dir = game_dir.join("tl").join(&target_lang);
    let mut audio = Vec::new();

    if !game_dir.exists() { return Ok(audio); }
    
    let audio_mapping = build_audio_mapping(&game_dir);

    for entry in WalkDir::new(&game_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.components().any(|c| c.as_os_str() == "tl" || c.as_os_str() == "cache") { continue; }

        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|s| s.to_str()).map(|s| s.to_lowercase()) {
                if ext == "ogg" || ext == "mp3" || ext == "wav" {
                    let rel_path = path.strip_prefix(&game_dir).unwrap_or(path).to_string_lossy().replace("\\", "/");
                    let translated_path = tl_dir.join(&rel_path);
                    let is_translated = translated_path.exists();
                    let trans_path_str = if is_translated { Some(translated_path.to_string_lossy().to_string()) } else { None };
                    
                    let rel_path_clean = rel_path.replace("\\", "/");
                    let audio_filename = rel_path_clean.split('/').last().unwrap_or(&rel_path_clean).to_lowercase();
                    
                    let (mapped_text, mapped_script) = match audio_mapping.get(&audio_filename) {
                        Some((t, s)) => (Some(t.clone()), Some(s.clone())),
                        None => (None, None)
                    };

                    audio.push(AudioEntry {
                        original_path: path.to_string_lossy().to_string(),
                        rel_path, is_translated, translated_path: trans_path_str,
                        mapped_text, mapped_script
                    });
                }
            }
        }
    }
    Ok(audio)
}

#[tauri::command]
fn import_localized_audio(project_path: String, target_lang: String, rel_path: String, source_file_path: String) -> Result<String, String> {
    let game_dir = Path::new(&project_path).join("game");
    let target_path = game_dir.join("tl").join(&target_lang).join(&rel_path);
    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Ошибка создания папок: {}", e))?;
    }
    fs::copy(&source_file_path, &target_path).map_err(|e| format!("Ошибка копирования: {}", e))?;
    Ok(target_path.to_string_lossy().to_string())
}

#[tauri::command]
fn delete_localized_audio(project_path: String, target_lang: String, rel_path: String) -> Result<(), String> {
    let game_dir = Path::new(&project_path).join("game");
    let target_path = game_dir.join("tl").join(&target_lang).join(&rel_path);
    if target_path.exists() {
        fs::remove_file(&target_path).map_err(|e| format!("Ошибка удаления: {}", e))?;
    }
    Ok(())
}

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
            generate_translations, apply_renforge_patch, get_character_mapping, get_images_list, import_localized_image, delete_localized_image, open_in_explorer,
            get_audio_list, import_localized_audio, delete_localized_audio, create_empty_translation, write_fallback_file,
            
            // Команды из модуля db
            db::search_in_db, 
            db::upsert_translation, 
            db::upsert_translations_batch, 
            db::get_translation_stats
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}