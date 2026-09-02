use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::Manager;

// ─── 数据模型 ───────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone)]
pub struct SourceItem {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub source_type: String,
    /// 原始文件的绝对路径（Phase A 采用引用方式，不复制大文件）
    pub path: String,
    pub size: u64,
    #[serde(default = "default_status")]
    pub status: String,
    pub duration: Option<String>,
}

fn default_status() -> String {
    "ready".to_string()
}

/// 转录与整理的配置项，保存到每个项目的 project.json 中。
#[derive(Serialize, Deserialize, Clone)]
pub struct ProjectConfig {
    #[serde(default = "default_engine")]
    pub engine: String,
    #[serde(default)]
    pub gemini_key: String,
    #[serde(default = "default_gemini_model")]
    pub gemini_model: String,
    #[serde(default = "default_llm_provider")]
    pub llm_provider: String,
    #[serde(default)]
    pub llm_api_key: String,
    #[serde(default)]
    pub llm_api_url: String,
    #[serde(default)]
    pub llm_model_name: String,
    #[serde(default)]
    pub prompt_template: String,
    #[serde(default)]
    pub custom_api_url: String,
    #[serde(default)]
    pub custom_api_key: String,
    #[serde(default = "default_custom_model")]
    pub custom_model_name: String,
    #[serde(default)]
    pub proxy: String,
    #[serde(default)]
    pub no_subtitle: bool,
    #[serde(default = "default_cookie_browser")]
    pub cookies_from_browser: String,
    #[serde(default)]
    pub youtube_cookie_file: String,
    #[serde(default)]
    pub local_output_dir: String,
    #[serde(default)]
    pub online_output_dir: String,
    #[serde(default = "default_formats")]
    pub formats: String,
}

fn default_engine() -> String {
    "custom".to_string()
}
fn default_gemini_model() -> String { "gemini-2.5-flash".to_string() }
fn default_llm_provider() -> String {
    "none".to_string()
}
fn default_custom_model() -> String {
    "whisper-1".to_string()
}
fn default_formats() -> String {
    "txt,srt,md".to_string()
}
fn default_cookie_browser() -> String { "none".to_string() }

impl Default for ProjectConfig {
    fn default() -> Self {
        Self { engine: default_engine(), gemini_key: String::new(), gemini_model: default_gemini_model(), llm_provider: default_llm_provider(), llm_api_key: String::new(), llm_api_url: String::new(), llm_model_name: String::new(), prompt_template: String::new(), custom_api_url: String::new(), custom_api_key: String::new(), custom_model_name: default_custom_model(), proxy: String::new(), no_subtitle: false, cookies_from_browser: default_cookie_browser(), youtube_cookie_file: String::new(), local_output_dir: String::new(), online_output_dir: String::new(), formats: default_formats() }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub sources: Vec<SourceItem>,
    #[serde(default)]
    pub config: ProjectConfig,
}

#[derive(Serialize, Clone)]
pub struct ProjectSummary {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
    pub source_count: usize,
    pub artifact_count: usize,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ArtifactItem {
    pub name: String,
    pub path: String,
    pub ext: String,
    pub size: u64,
    pub modified: String,
    #[serde(default)]
    pub source_id: String,
    #[serde(default)]
    pub task_id: String,
    #[serde(default)]
    pub kind: String,
}

/// 导出结果：复制成功的文件数、失败项描述、目标目录。
#[derive(Serialize, Clone)]
pub struct ExportResult {
    pub copied: u32,
    pub failed: Vec<String>,
    pub target: String,
}

// ─── 文件系统辅助 ───────────────────────────────────────────

pub fn projects_root(app: &tauri::AppHandle) -> PathBuf {
    app.path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("projects")
}

pub fn project_dir(app: &tauri::AppHandle, id: &str) -> PathBuf {
    projects_root(app).join(id)
}

pub fn project_json_path(app: &tauri::AppHandle, id: &str) -> PathBuf {
    project_dir(app, id).join("project.json")
}

pub fn read_project(app: &tauri::AppHandle, id: &str) -> Result<Project, String> {
    let p = project_json_path(app, id);
    let s = std::fs::read_to_string(&p).map_err(|e| format!("读取项目失败: {}", e))?;
    serde_json::from_str(&s).map_err(|e| format!("解析项目失败: {}", e))
}

pub fn write_project(app: &tauri::AppHandle, project: &Project) -> Result<(), String> {
    let dir = project_dir(app, &project.id);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(dir.join("sources")).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(dir.join("artifacts")).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(dir.join("logs")).map_err(|e| e.to_string())?;
    let json = serde_json::to_string_pretty(project).map_err(|e| e.to_string())?;
    std::fs::write(project_json_path(app, &project.id), json).map_err(|e| e.to_string())
}

pub fn list_projects(app: &tauri::AppHandle) -> Vec<ProjectSummary> {
    let root = projects_root(app);
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&root) {
        for e in entries.flatten() {
            let pj = e.path().join("project.json");
            if let Ok(s) = std::fs::read_to_string(&pj) {
                if let Ok(prj) = serde_json::from_str::<Project>(&s) {
                    let artifact_count = count_files(&e.path().join("artifacts"));
                    out.push(ProjectSummary {
                        id: prj.id,
                        name: prj.name,
                        created_at: prj.created_at,
                        updated_at: prj.updated_at,
                        source_count: prj.sources.len(),
                        artifact_count,
                    });
                }
            }
        }
    }
    out.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    out
}

pub fn list_artifacts(app: &tauri::AppHandle, id: &str) -> Vec<ArtifactItem> {
    let art_dir = project_dir(app, id).join("artifacts");
    let sources = read_project(app, id).map(|p| p.sources).unwrap_or_default();
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&art_dir) {
        for e in entries.flatten() {
            let path = e.path();
            if path.is_file() {
                if let Ok(meta) = e.metadata() {
                    let ext = path
                        .extension()
                        .and_then(|s| s.to_str())
                        .unwrap_or("")
                        .to_lowercase();
                    let modified = meta
                        .modified()
                        .ok()
                        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                        .map(|d| ts_to_string(d.as_secs()))
                        .unwrap_or_default();
                    let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();
                    let source_id = sources.iter().find(|s| file_name.starts_with(&sanitize_stem(&s.name))).map(|s| s.id.clone()).unwrap_or_default();
                    out.push(ArtifactItem {
                        name: file_name,
                        path: path.to_string_lossy().to_string(),
                        ext: ext.clone(),
                        size: meta.len(),
                        modified,
                        source_id: source_id.clone(),
                        task_id: source_id,
                        kind: ext.clone(),
                    });
                }
            }
        }
    }
    for task in read_tasks(app, id) {
        for path_str in task.artifact_paths {
            let path = PathBuf::from(&path_str);
            if !path.is_file() || out.iter().any(|a| a.path == path_str) { continue; }
            if let Ok(meta) = path.metadata() {
                let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
                let modified = meta.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| ts_to_string(d.as_secs())).unwrap_or_default();
                out.push(ArtifactItem {
                    name: path.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string(), path: path_str,
                    ext: ext.clone(), size: meta.len(), modified, source_id: task.source_id.clone(), task_id: task.id.clone(), kind: ext,
                });
            }
        }
    }
    out.sort_by(|a, b| b.modified.cmp(&a.modified));
    out
}

fn count_files(dir: &PathBuf) -> usize {
    let mut n = 0;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for e in entries.flatten() {
            if e.path().is_file() {
                n += 1;
            }
        }
    }
    n
}

fn ts_to_string(secs: u64) -> String {
    chrono::DateTime::from_timestamp(secs as i64, 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_default()
}

pub fn now_str() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn now_str_compact() -> String {
    chrono::Local::now().format("%Y%m%d_%H%M%S").to_string()
}

pub fn sanitize(name: &str) -> String {
    name.chars()
        .filter(|c| !r#"\/:*?"<>|"#.contains(*c))
        .collect()
}

pub fn sanitize_stem(name: &str) -> String {
    let p = std::path::Path::new(name);
    let stem = p
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("source")
        .to_string();
    sanitize(&stem)
}

// ─── 任务队列（持久化到 tasks.json） ───────────────────────

#[derive(Serialize, Deserialize, Clone)]
pub struct TaskItem {
    pub id: String,
    #[serde(default)]
    pub source_id: String,
    pub source_name: String,
    pub engine: String,
    /// 状态：queued / running / done / error
    pub status: String,
    pub created_at: String,
    pub finished_at: Option<String>,
    pub message: String,
    #[serde(default)]
    pub progress: f64,
    #[serde(default)]
    pub formats: String,
    #[serde(default)]
    pub llm_provider: String,
    #[serde(default)]
    pub artifact_ids: Vec<String>,
    #[serde(default)]
    pub artifact_paths: Vec<String>,
    #[serde(default)]
    pub error: String,
    #[serde(default)]
    pub canceled: bool,
    #[serde(default)]
    pub config: ProjectConfig,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DocumentTask {
    pub id: String,
    pub name: String,
    pub inputs: Vec<String>,
    pub status: String,
    pub template: String,
    pub model: String,
    pub series: bool,
    pub created_at: String,
    #[serde(default)]
    pub finished_at: String,
    #[serde(default)]
    pub output_path: String,
    #[serde(default)]
    pub error: String,
}

pub fn tasks_path(app: &tauri::AppHandle, id: &str) -> PathBuf {
    project_dir(app, id).join("tasks.json")
}

pub fn read_tasks(app: &tauri::AppHandle, id: &str) -> Vec<TaskItem> {
    let p = tasks_path(app, id);
    std::fs::read_to_string(&p)
        .ok()
        .and_then(|s| serde_json::from_str::<Vec<TaskItem>>(&s).ok())
        .unwrap_or_default()
        .into_iter()
        .map(|mut task| {
            if task.source_id.is_empty() { task.source_id = task.id.clone(); }
            if task.status == "running" || task.status == "downloading" || task.status == "transcribing" || task.status == "organizing" {
                task.status = "interrupted".to_string();
                task.message = "应用上次退出时任务仍在运行，可重试".to_string();
            }
            task
        })
        .collect()
}

pub fn write_tasks(app: &tauri::AppHandle, id: &str, tasks: &[TaskItem]) -> Result<(), String> {
    let json = serde_json::to_string_pretty(tasks).map_err(|e| e.to_string())?;
    std::fs::write(tasks_path(app, id), json).map_err(|e| e.to_string())
}

pub fn document_tasks_path(app: &tauri::AppHandle, id: &str) -> PathBuf { project_dir(app, id).join("document_tasks.json") }
pub fn read_document_tasks(app: &tauri::AppHandle, id: &str) -> Vec<DocumentTask> {
    std::fs::read_to_string(document_tasks_path(app, id)).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
}
pub fn write_document_tasks(app: &tauri::AppHandle, id: &str, tasks: &[DocumentTask]) -> Result<(), String> {
    std::fs::write(document_tasks_path(app, id), serde_json::to_string_pretty(tasks).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_task_json_remains_compatible() {
        let json = r#"{"id":"source-1","source_name":"demo.mp4","engine":"bcut","status":"queued","created_at":"2026-01-01","finished_at":null,"message":""}"#;
        let task: TaskItem = serde_json::from_str(json).expect("legacy task should deserialize");
        assert_eq!(task.id, "source-1");
        assert_eq!(task.progress, 0.0);
        assert!(task.artifact_ids.is_empty());
        assert_eq!(task.config.engine, "custom");
    }

    #[test]
    fn project_config_defaults_to_zero_setup_conversion() {
        let config = ProjectConfig::default();
        assert_eq!(config.engine, "custom");
        assert_eq!(config.llm_provider, "none");
        assert_eq!(config.formats, "txt,srt,md");
    }
}
