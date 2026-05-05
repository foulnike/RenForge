use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ProjectFiles {
    pub rpa_files: Vec<String>,
    pub rpyc_files: Vec<String>,
    pub rpy_files: Vec<String>,
    pub tl_files: Vec<String>,
    pub manual_tl_files: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct DbEntry {
    pub id: String,
    pub file_path: String,
    pub original: String,
    pub translation: String,
    pub status: String,
}

#[derive(Serialize, Deserialize)]
pub struct FileStats {
    pub total: i32,
    pub translated: i32,
}

#[derive(Serialize, Deserialize)]
pub struct ImageEntry {
    pub original_path: String,
    pub rel_path: String,
    pub is_translated: bool,
    pub translated_path: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct AudioEntry {
    pub original_path: String,
    pub rel_path: String,
    pub is_translated: bool,
    pub translated_path: Option<String>,
    pub mapped_text: Option<String>,    
    pub mapped_script: Option<String>,
}