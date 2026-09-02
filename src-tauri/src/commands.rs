use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use tauri::{Emitter, Manager};
use base64::Engine;

use crate::project::{
    self, ArtifactItem, Project, ProjectConfig, ProjectSummary, SourceItem, TaskItem,
};
use crate::state::AppState;

fn secret_path(app: &tauri::AppHandle, name: &str) -> Result<PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?.join("secrets");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join(format!("{name}.dpapi")))
}
fn save_secret(app: &tauri::AppHandle, name: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty() { return Ok(()); }
    #[cfg(target_os = "windows")]
    {
        let path = secret_path(app, name)?;
        let script = "Add-Type -AssemblyName System.Security;$b=[Text.Encoding]::UTF8.GetBytes($env:VIDEO2MD_SECRET);$e=[Security.Cryptography.ProtectedData]::Protect($b,$null,[Security.Cryptography.DataProtectionScope]::CurrentUser);[IO.File]::WriteAllBytes($env:VIDEO2MD_SECRET_PATH,$e)";
        let status = background_command("powershell").args(["-NoProfile","-NonInteractive","-Command",script]).env("VIDEO2MD_SECRET",value).env("VIDEO2MD_SECRET_PATH",path).status().map_err(|e| e.to_string())?;
        if !status.success() { return Err("Windows DPAPI 保存凭据失败".into()); }
    }
    #[cfg(not(target_os = "windows"))]
    { std::fs::write(secret_path(app,name)?, value.as_bytes()).map_err(|e| e.to_string())?; }
    Ok(())
}
fn load_secret(app: &tauri::AppHandle, name: &str) -> String {
    let Ok(path) = secret_path(app,name) else { return String::new(); };
    if !path.exists() { return String::new(); }
    #[cfg(target_os = "windows")]
    {
        let script = "Add-Type -AssemblyName System.Security;$e=[IO.File]::ReadAllBytes($env:VIDEO2MD_SECRET_PATH);$b=[Security.Cryptography.ProtectedData]::Unprotect($e,$null,[Security.Cryptography.DataProtectionScope]::CurrentUser);[Convert]::ToBase64String($b)";
        return background_command("powershell").args(["-NoProfile","-NonInteractive","-Command",script]).env("VIDEO2MD_SECRET_PATH",path).output().ok().filter(|o|o.status.success()).and_then(|o|base64::engine::general_purpose::STANDARD.decode(String::from_utf8_lossy(&o.stdout).trim()).ok()).and_then(|b|String::from_utf8(b).ok()).unwrap_or_default();
    }
    #[cfg(not(target_os = "windows"))]
    { std::fs::read_to_string(path).unwrap_or_default() }
}
fn hydrate_secrets(app: &tauri::AppHandle, mut config: ProjectConfig) -> ProjectConfig {
    if config.gemini_key.is_empty() { config.gemini_key = load_secret(app,"gemini_api_key"); }
    if config.llm_api_key.is_empty() { config.llm_api_key = load_secret(app,"llm_api_key"); }
    if config.custom_api_key.is_empty() { config.custom_api_key = load_secret(app,"custom_asr_api_key"); }
    config
}
fn scrub_secrets(mut config: ProjectConfig) -> ProjectConfig {
    config.gemini_key.clear(); config.llm_api_key.clear(); config.custom_api_key.clear(); config
}

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

fn background_command(program: &str) -> Command {
    let mut cmd = Command::new(program);
    #[cfg(target_os = "windows")]
    { cmd.creation_flags(0x08000000); }
    cmd
}

// ─── 辅助：定位 Python 脚本 ─────────────────────────────────

fn find_transcribe_script(app: &tauri::AppHandle) -> Option<PathBuf> {
    // 1. Tauri 资源目录（打包后 / dev 下的 src-tauri）
    if let Ok(resource_dir) = app.path().resource_dir() {
        let p = resource_dir.join("Tools/transcribe.py");
        if p.exists() {
            return Some(p);
        }
        let p2 = resource_dir.join("transcribe.py");
        if p2.exists() {
            return Some(p2);
        }
    }
    // 2. 从可执行文件目录向上递归查找 Tools/transcribe.py
    if let Ok(exe_path) = std::env::current_exe() {
        let mut current = exe_path.parent();
        while let Some(dir) = current {
            let p = dir.join("Tools/transcribe.py");
            if p.exists() {
                return Some(p);
            }
            current = dir.parent();
        }
    }
    // 3. 当前工作目录
    let cwd = PathBuf::from("Tools/transcribe.py");
    if cwd.exists() {
        return Some(cwd);
    }
    let parent = PathBuf::from("../Tools/transcribe.py");
    if parent.exists() {
        return Some(parent);
    }
    None
}

fn detect_python() -> Option<String> {
    if background_command("python").arg("--version").output().is_ok() {
        Some("python".to_string())
    } else if background_command("python3").arg("--version").output().is_ok() {
        Some("python3".to_string())
    } else {
        None
    }
}

fn default_downloads_dir() -> PathBuf {
    std::env::var_os("USERPROFILE").map(PathBuf::from).map(|p| p.join("Downloads"))
        .or_else(|| std::env::var_os("HOME").map(|p| PathBuf::from(p).join("Downloads")))
        .unwrap_or_else(|| PathBuf::from("Downloads"))
}

#[tauri::command]
pub fn get_platform_capabilities() -> (String, String, bool) {
    let os = std::env::consts::OS.to_string();
    let arch = std::env::consts::ARCH.to_string();
    let mlx = os == "macos" && (arch == "aarch64" || arch == "arm64");
    (os, arch, mlx)
}

// ─── 项目命令 ───────────────────────────────────────────────

#[tauri::command]
pub fn create_project(app: tauri::AppHandle, name: String) -> Result<Project, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = project::now_str();
    let project = Project {
        id: id.clone(),
        name: if name.trim().is_empty() {
            format!("未命名项目 {}", &id[..8])
        } else {
            name.trim().to_string()
        },
        created_at: now.clone(),
        updated_at: now,
        sources: vec![],
        config: ProjectConfig {
            engine: "custom".into(),
            gemini_key: String::new(),
            gemini_model: "gemini-2.5-flash".into(),
            llm_provider: "none".into(),
            llm_api_key: String::new(),
            llm_api_url: String::new(),
            llm_model_name: String::new(),
            prompt_template: String::new(),
            custom_api_url: String::new(),
            custom_api_key: String::new(),
            custom_model_name: "whisper-1".into(),
            proxy: String::new(),
            no_subtitle: false,
            cookies_from_browser: "none".into(),
            youtube_cookie_file: String::new(),
            local_output_dir: String::new(),
            online_output_dir: String::new(),
            formats: "txt,srt,md".into(),
        },
    };
    project::write_project(&app, &project)?;
    // 设为当前项目
    if let Ok(mut cur) = app.state::<AppState>().current_project.lock() {
        *cur = Some(id);
    }
    Ok(project)
}

#[tauri::command]
pub fn list_projects(app: tauri::AppHandle) -> Result<Vec<ProjectSummary>, String> {
    Ok(project::list_projects(&app))
}

#[tauri::command]
pub fn get_project(app: tauri::AppHandle, id: String) -> Result<Project, String> {
    project::read_project(&app, &id)
}

#[tauri::command]
pub fn set_current_project(state: tauri::State<AppState>, id: Option<String>) -> Result<(), String> {
    let mut cur = state.current_project.lock().map_err(|e| e.to_string())?;
    *cur = id;
    Ok(())
}

#[tauri::command]
pub fn get_current_project(
    app: tauri::AppHandle,
    state: tauri::State<AppState>,
) -> Result<Option<Project>, String> {
    let cur = state.current_project.lock().map_err(|e| e.to_string())?;
    match cur.clone() {
        Some(id) => {
            if !project::project_json_path(&app, &id).exists() {
                return Ok(None);
            }
            Ok(Some(project::read_project(&app, &id)?))
        }
        None => Ok(None),
    }
}

// ─── 导入 / 素材命令 ────────────────────────────────────────

fn classify_ext(ext: &str) -> &'static str {
    match ext {
        "mp4" | "mov" | "avi" | "mkv" | "flv" | "wmv" | "webm" => "video",
        "mp3" | "wav" | "m4a" | "aac" | "flac" | "ogg" | "opus" => "audio",
        "pdf" => "pdf",
        "doc" | "docx" | "txt" | "md" => "doc",
        _ => "file",
    }
}

#[tauri::command]
pub fn import_files(
    app: tauri::AppHandle,
    id: String,
    paths: Vec<String>,
) -> Result<Vec<SourceItem>, String> {
    let mut project = project::read_project(&app, &id)?;
    for p in paths {
        let is_url = p.starts_with("http://") || p.starts_with("https://");
        let path = std::path::Path::new(&p);
        if !is_url && !path.is_file() {
            continue;
        }
        let name = if is_url {
            p.clone()
        } else {
            path.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("file")
                .to_string()
        };
        let ext = if is_url {
            String::new()
        } else {
            path.extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_lowercase()
        };
        let source_type = if is_url {
            "url".to_string()
        } else {
            classify_ext(&ext).to_string()
        };
        let size = if is_url {
            0
        } else {
            path.metadata().map(|m| m.len()).unwrap_or(0)
        };
        project.sources.push(SourceItem {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            source_type,
            path: p,
            size,
            status: "ready".to_string(),
            duration: None,
        });
    }
    project.updated_at = project::now_str();
    project::write_project(&app, &project)?;
    // 更新当前项目指针
    if let Ok(mut cur) = app.state::<AppState>().current_project.lock() {
        *cur = Some(id);
    }
    Ok(project.sources.clone())
}

/// 解析一组在线视频链接。普通链接逐条加入；批量页面使用 yt-dlp 的扁平列表解析为独立视频。
#[tauri::command]
pub fn import_online_sources(
    app: tauri::AppHandle,
    id: String,
    urls: Vec<String>,
    parse_batch: bool,
) -> Result<Vec<SourceItem>, String> {
    let mut resolved: Vec<(String, String)> = Vec::new();
    for raw in urls.into_iter().map(|u| u.trim().to_string()).filter(|u| u.starts_with("http://") || u.starts_with("https://")) {
        if !parse_batch {
            resolved.push((raw.clone(), raw));
            continue;
        }
        let python = detect_python().ok_or("未找到 Python，无法解析批量页面。")?;
        let output = Command::new(python)
            .args(["-m", "yt_dlp", "--flat-playlist", "--dump-single-json", "--no-warnings", &raw])
            .output()
            .map_err(|e| format!("无法启动 yt-dlp：{e}"))?;
        if !output.status.success() {
            return Err(format!("批量页面解析失败：{}", String::from_utf8_lossy(&output.stderr).trim()));
        }
        let data: serde_json::Value = serde_json::from_slice(&output.stdout).map_err(|e| format!("解析返回内容失败：{e}"))?;
        let entries = data.get("entries").and_then(|v| v.as_array()).ok_or("该页面没有可解析的视频条目。")?;
        for entry in entries.iter().take(200) {
            let url = entry.get("webpage_url").or_else(|| entry.get("url")).and_then(|v| v.as_str()).unwrap_or("");
            let title = entry.get("title").and_then(|v| v.as_str()).unwrap_or(url);
            if url.starts_with("http") { resolved.push((url.to_string(), title.to_string())); }
        }
    }
    if resolved.is_empty() { return Err("没有发现可导入的有效链接。".into()); }
    let mut project = project::read_project(&app, &id)?;
    for (url, title) in resolved {
        if project.sources.iter().any(|s| s.path == url) { continue; }
        project.sources.push(SourceItem { id: uuid::Uuid::new_v4().to_string(), name: title, source_type: "url".into(), path: url, size: 0, status: "ready".into(), duration: None });
    }
    project.updated_at = project::now_str();
    project::write_project(&app, &project)?;
    Ok(project.sources)
}

#[tauri::command]
pub fn remove_source(
    app: tauri::AppHandle,
    id: String,
    source_id: String,
) -> Result<Vec<SourceItem>, String> {
    let mut project = project::read_project(&app, &id)?;
    project.sources.retain(|s| s.id != source_id);
    project.updated_at = project::now_str();
    project::write_project(&app, &project)?;
    Ok(project.sources.clone())
}

#[tauri::command]
pub fn clear_sources(app: tauri::AppHandle, id: String) -> Result<Vec<SourceItem>, String> {
    let tasks = project::read_tasks(&app, &id);
    if tasks.iter().any(|t| matches!(t.status.as_str(), "queued" | "downloading" | "transcribing" | "organizing")) {
        return Err("队列仍有等待或执行中的任务，不能清空素材。请先取消任务。".into());
    }
    let mut project = project::read_project(&app, &id)?;
    project.sources.clear();
    project.updated_at = project::now_str();
    project::write_project(&app, &project)?;
    Ok(project.sources)
}

#[tauri::command]
pub fn list_sources(app: tauri::AppHandle, id: String) -> Result<Vec<SourceItem>, String> {
    let project = project::read_project(&app, &id)?;
    Ok(project.sources)
}

// ─── 产物命令 ───────────────────────────────────────────────

#[tauri::command]
pub fn list_artifacts(app: tauri::AppHandle, id: String) -> Result<Vec<ArtifactItem>, String> {
    Ok(project::list_artifacts(&app, &id))
}

#[tauri::command]
pub fn get_config(app: tauri::AppHandle, id: String) -> Result<ProjectConfig, String> {
    let project = project::read_project(&app, &id)?;
    Ok(hydrate_secrets(&app, project.config))
}

#[tauri::command]
pub fn set_config(
    app: tauri::AppHandle,
    id: String,
    mut config: ProjectConfig,
) -> Result<ProjectConfig, String> {
    save_secret(&app,"gemini_api_key", &config.gemini_key)?;
    save_secret(&app,"llm_api_key", &config.llm_api_key)?;
    save_secret(&app,"custom_asr_api_key", &config.custom_api_key)?;
    let hydrated = hydrate_secrets(&app, config.clone());
    config = scrub_secrets(config);
    let mut project = project::read_project(&app, &id)?;
    project.config = config;
    project.updated_at = project::now_str();
    project::write_project(&app, &project)?;
    Ok(hydrated)
}

fn bilibili_login_script(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    find_transcribe_script(app).ok_or("未找到转写工具目录。".to_string()).map(|p| p.parent().unwrap().join("bilibili_qr_login.py"))
}

#[tauri::command]
pub fn start_bilibili_qr_login(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    let python = detect_python().ok_or("未找到 Python。".to_string())?;
    let script = bilibili_login_script(&app)?;
    let output = background_command(&python).args([script.to_string_lossy().as_ref(), "start"]).output().map_err(|e| e.to_string())?;
    serde_json::from_slice(&output.stdout).map_err(|_| String::from_utf8_lossy(&output.stderr).to_string())
}

#[tauri::command]
pub fn poll_bilibili_qr_login(app: tauri::AppHandle, id: String, key: String) -> Result<serde_json::Value, String> {
    let python = detect_python().ok_or("未找到 Python。".to_string())?;
    let script = bilibili_login_script(&app)?;
    let output = background_command(&python).args([script.to_string_lossy().as_ref(), "poll", "--key", &key]).output().map_err(|e| e.to_string())?;
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).map_err(|_| String::from_utf8_lossy(&output.stderr).to_string())?;
    if value.get("status").and_then(|v| v.as_str()) == Some("confirmed") {
        let cookies = value.get("cookies").and_then(|v| v.as_object()).ok_or("B站未返回登录 Cookie。")?;
        let dir = app.path().app_data_dir().map_err(|e| e.to_string())?.join("cookies"); std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let path = dir.join("bilibili_qr_cookies.txt");
        let mut content = "# Netscape HTTP Cookie File\n".to_string();
        for (name, value) in cookies { content.push_str(&format!(".bilibili.com\tTRUE\t/\tFALSE\t2147483647\t{}\t{}\n", name, value.as_str().unwrap_or(""))); }
        std::fs::write(&path, content).map_err(|e| e.to_string())?;
        let mut project = project::read_project(&app, &id)?;
        project.config.youtube_cookie_file = path.to_string_lossy().to_string(); project.config.cookies_from_browser = "none".into(); project.updated_at = project::now_str(); project::write_project(&app, &project)?;
    }
    Ok(value)
}

// ─── 转写命令 ───────────────────────────────────────────────

#[tauri::command]
pub fn run_transcribe(
    app: tauri::AppHandle,
    id: String,
    source_ids: Vec<String>,
    engine: Option<String>,
) -> Result<(), String> {
    let project = project::read_project(&app, &id)?;
    let config = hydrate_secrets(&app, project.config.clone());
    let engine = engine
        .filter(|e| !e.trim().is_empty())
        .unwrap_or_else(|| config.engine.clone());

    let targets: Vec<SourceItem> = if source_ids.is_empty() {
        project.sources.clone()
    } else {
        project
            .sources
            .iter()
            .filter(|s| source_ids.contains(&s.id))
            .cloned()
            .collect()
    };
    if targets.is_empty() {
        return Err("没有可转写的素材，请先导入文件。".into());
    }

    let script = find_transcribe_script(&app)
        .ok_or_else(|| "未找到 Tools/transcribe.py，请确认脚本已随程序打包。".to_string())?;
    let python = detect_python()
        .ok_or_else(|| "未找到 Python 环境，请先安装 Python 并加入 PATH。".to_string())?;

    let artifacts_dir = project::project_dir(&app, &id).join("artifacts");
    let log_dir = project::project_dir(&app, &id).join("logs");
    std::fs::create_dir_all(&artifacts_dir).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&log_dir).map_err(|e| e.to_string())?;

    // 建立本次运行的任务队列（持久化，重启可恢复）
    let now = project::now_str();
    let mut initial_tasks: Vec<TaskItem> = Vec::new();
    for source in &targets {
        initial_tasks.push(TaskItem {
            id: source.id.clone(),
            source_id: source.id.clone(),
            source_name: source.name.clone(),
            engine: engine.clone(),
            status: "queued".to_string(),
            created_at: now.clone(),
            finished_at: None,
            message: String::new(),
            progress: 0.0,
            formats: config.formats.clone(),
            llm_provider: config.llm_provider.clone(),
            artifact_ids: Vec::new(),
            artifact_paths: Vec::new(),
            error: String::new(),
            canceled: false,
            config: scrub_secrets(config.clone()),
        });
    }
    {
        let state = app.state::<AppState>();
        let mut map = state.tasks.lock().unwrap_or_else(|e| e.into_inner());
        let queue = map.entry(id.clone()).or_insert_with(|| project::read_tasks(&app, &id));
        for task in initial_tasks.clone() {
            if !queue.iter().any(|existing| existing.id == task.id) {
                queue.push(task);
            }
        }
        initial_tasks = queue.clone();
    }
    let _ = project::write_tasks(&app, &id, &initial_tasks);

    let should_start = app.state::<AppState>().active_projects.lock().map(|mut s| s.insert(id.clone())).unwrap_or(false);
    if should_start {
        let app2 = app.clone();
        std::thread::spawn(move || {
            loop {
                let next_task = app2.state::<AppState>().tasks.lock().ok().and_then(|m| m.get(&id).and_then(|items| items.iter().find(|t| t.status == "queued" && !t.canceled).cloned()));
                let Some(task) = next_task else { break; };
                let source_id = task.source_id.clone();
                let source = project::read_project(&app2, &id).ok().and_then(|p| p.sources.into_iter().find(|s| s.id == source_id));
                let Some(source) = source else {
                    let _ = update_task(&app2, &id, &source_id, "error", "原始素材已不存在");
                    continue;
                };
                run_one_source(app2.clone(), script.clone(), python.clone(), task.engine.clone(), artifacts_dir.clone(), log_dir.clone(), hydrate_secrets(&app2, task.config.clone()), id.clone(), source);
            }
            if let Ok(mut active) = app2.state::<AppState>().active_projects.lock() { active.remove(&id); }
            let pending = app2.state::<AppState>().tasks.lock().ok().and_then(|m| m.get(&id).and_then(|items| items.iter().find(|t| t.status == "queued" && !t.canceled).map(|t| t.source_id.clone())));
            if let Some(source_id) = pending { let _ = run_transcribe(app2.clone(), id.clone(), vec![source_id], None); }
        });
    }
    Ok(())
}

/// 更新某任务状态并持久化，返回更新后的任务（用于事件回流）。
fn update_task(
    app: &tauri::AppHandle,
    id: &str,
    source_id: &str,
    status: &str,
    message: &str,
) -> Option<TaskItem> {
    let state = app.state::<AppState>();
    let mut map = state
        .tasks
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let tasks = map.entry(id.to_string()).or_default();
    let mut found = None;
    if let Some(t) = tasks.iter_mut().find(|t| t.id == source_id) {
        t.status = status.to_string();
        if !message.is_empty() {
            t.message = message.to_string();
        }
        if status == "done" || status == "error" || status == "canceled" {
            t.finished_at = Some(project::now_str());
        }
        if status == "error" { t.error = message.to_string(); }
        if status == "done" { t.progress = 100.0; }
        found = Some(t.clone());
    }
    if found.is_some() {
        let _ = project::write_tasks(app, id, tasks);
    }
    found
}

/// 解析 yt-dlp 下载进度行，如 "[download]  12.3% of 45.00MiB ..."
fn parse_download_progress(line: &str) -> Option<f64> {
    let marker = "[download]";
    let idx = line.find(marker)?;
    let rest = &line[idx + marker.len()..];
    let pct_str = rest.trim_start().split('%').next()?.trim();
    let pct = pct_str.parse::<f64>().ok()?;
    if pct.is_finite() {
        Some(pct)
    } else {
        None
    }
}

fn run_one_source(
    app: tauri::AppHandle,
    script: PathBuf,
    python: String,
    engine: String,
    artifacts_dir: PathBuf,
    log_dir: PathBuf,
    config: ProjectConfig,
    id: String,
    source: SourceItem,
) {
    let label = &source.name;
    let safe_name = project::sanitize_stem(label);
    let output_dir = if source.source_type == "url" {
        let base = if config.online_output_dir.trim().is_empty() { default_downloads_dir() } else { PathBuf::from(&config.online_output_dir) };
        let project_name = project::read_project(&app, &id).map(|p| project::sanitize(&p.name)).unwrap_or_else(|_| id.clone());
        base.join(project_name)
    } else if config.local_output_dir.trim().is_empty() {
        PathBuf::from(&source.path).parent().map(|p| p.to_path_buf()).unwrap_or_else(|| artifacts_dir.clone())
    } else { PathBuf::from(&config.local_output_dir) };
    if let Err(e) = std::fs::create_dir_all(&output_dir) {
        let er = update_task(&app, &id, &source.id, "error", &format!("无法创建输出目录: {e}"));
        if let Some(t) = er { let _ = app.emit("task-updated", t); }
        return;
    }
    let output_base = output_dir.join(&safe_name).to_string_lossy().to_string();

    let mut args: Vec<String> = vec![
        script.to_string_lossy().to_string(),
        "--input".into(),
        source.path.clone(),
        "--engine".into(),
        engine.clone(),
        "--output".into(),
        output_base.clone(),
        "--formats".into(),
        config.formats.clone(),
    ];

    if config.no_subtitle {
        args.push("--no-subtitle".into());
    }
    if !config.cookies_from_browser.trim().is_empty() && config.cookies_from_browser != "none" {
        args.push("--cookies-from-browser".into());
        args.push(config.cookies_from_browser.clone());
    }

    // 文本优化与 ASR 引擎互不依赖：无论 ASR 使用必剪、Gemini 还是自定义接口，
    // 都要把文本优化自己的服务、Key 与模型完整传给 Python。
    args.push("--llm-provider".into());
    args.push(config.llm_provider.clone());
    if !config.llm_provider.trim().is_empty() && config.llm_provider != "none" {
        if !config.llm_api_key.trim().is_empty() {
            args.push("--llm-api-key".into());
            args.push(config.llm_api_key.clone());
        }
        if !config.llm_api_url.trim().is_empty() {
            args.push("--llm-api-url".into());
            args.push(config.llm_api_url.clone());
        }
        if !config.llm_model_name.trim().is_empty() {
            args.push("--llm-model-name".into());
            args.push(config.llm_model_name.clone());
        }
        if config.llm_provider == "gemini" && config.llm_model_name.trim().is_empty() {
            args.push("--llm-model-name".into());
            args.push(config.gemini_model.clone());
        }
        if !config.prompt_template.trim().is_empty() {
            args.push("--prompt-template".into());
            args.push(config.prompt_template.clone());
        }
    }

    if engine == "custom" {
        if !config.custom_api_url.trim().is_empty() {
            args.push("--api-url".into());
            args.push(config.custom_api_url.clone());
        }
        if !config.custom_api_key.trim().is_empty() {
            args.push("--api-key".into());
            args.push(config.custom_api_key.clone());
        }
        if !config.custom_model_name.trim().is_empty() {
            args.push("--model-name".into());
            args.push(config.custom_model_name.clone());
        }
    }

    if !config.proxy.trim().is_empty() {
        args.push("--proxy".into());
        args.push(config.proxy.clone());
    }

    let log_path = log_dir.join(format!("{}_{}.log", safe_name, project::now_str_compact()));

    let mut cmd = Command::new(python);
    cmd.args(&args).stdout(Stdio::piped()).stderr(Stdio::piped());

    #[cfg(target_os = "windows")]
    {
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    if !config.gemini_key.trim().is_empty() {
        cmd.env("GEMINI_API_KEY", &config.gemini_key);
    }
    cmd.env("PYTHONIOENCODING", "utf-8");
    if !config.youtube_cookie_file.trim().is_empty() { cmd.env("YOUTUBE_COOKIES_FILE", &config.youtube_cookie_file); }

    let mut log_file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&log_path)
        .ok();

    let emit_log = |app: &tauri::AppHandle, line: &str| {
        let _ = app.emit("transcribe-log", format!("[{}] {}", label, line));
    };

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            let _ = app.emit(
                "transcribe-error",
                format!("[{}] 启动转写进程失败: {}", label, e),
            );
            let er = update_task(&app, &id, &source.id, "error", &format!("启动失败: {}", e));
            if let Some(t) = er {
                let _ = app.emit("task-updated", t);
            }
            return;
        }
    };

    if let Ok(mut pids) = app.state::<AppState>().active_pids.lock() {
        pids.insert(source.id.clone(), child.id());
    }

    let running = update_task(&app, &id, &source.id, "transcribing", "正在转写");
    if let Some(t) = running {
        let _ = app.emit("task-updated", t);
    }

    if let Some(stdout) = child.stdout.take() {
        let mut reader = BufReader::new(stdout);
        let mut buf = Vec::new();
        while let Ok(n) = reader.read_until(b'\n', &mut buf) {
            if n == 0 {
                break;
            }
            let line = String::from_utf8_lossy(&buf)
                .trim_end_matches(|c| c == '\r' || c == '\n')
                .to_string();
            if let Some(ref mut f) = log_file {
                let _ = writeln!(f, "{}", line);
            }
            // URL 下载进度：解析 yt-dlp 的 "[download]  X%" 行
            if let Some(pct) = parse_download_progress(&line) {
                let _ = app.emit("download-progress", format!("{}|{}", source.id, pct));
                if let Some(mut task) = update_task(&app, &id, &source.id, "downloading", "正在下载在线媒体") {
                    task.progress = pct;
                    if let Ok(mut map) = app.state::<AppState>().tasks.lock() {
                        if let Some(items) = map.get_mut(&id) {
                            if let Some(item) = items.iter_mut().find(|x| x.id == source.id) { item.progress = pct; }
                            let _ = project::write_tasks(&app, &id, items);
                        }
                    }
                    let _ = app.emit("task-updated", task);
                }
            }
            emit_log(&app, &line);
            buf.clear();
        }
    }

    let mut err_msg = String::new();
    if let Some(stderr) = child.stderr.take() {
        let mut reader = BufReader::new(stderr);
        let mut buf = Vec::new();
        while let Ok(n) = reader.read_until(b'\n', &mut buf) {
            if n == 0 {
                break;
            }
            let line = String::from_utf8_lossy(&buf)
                .trim_end_matches(|c| c == '\r' || c == '\n')
                .to_string();
            err_msg.push_str(&line);
            err_msg.push('\n');
            if let Some(ref mut f) = log_file {
                let _ = writeln!(f, "[调试] {}", line);
            }
            let _ = app.emit("transcribe-log", format!("[{}][调试] {}", label, line));
            buf.clear();
        }
    }

    let wait_result = child.wait();
    let was_canceled = app.state::<AppState>().tasks.lock().ok().and_then(|m| m.get(&id).and_then(|v| v.iter().find(|t| t.id == source.id)).map(|t| t.canceled)).unwrap_or(false);
    if was_canceled {
        let canceled = update_task(&app, &id, &source.id, "canceled", "已取消");
        if let Some(t) = canceled { let _ = app.emit("task-updated", t); }
        if let Ok(mut pids) = app.state::<AppState>().active_pids.lock() { pids.remove(&source.id); }
        return;
    }
    match wait_result {
        Ok(status) => {
            if status.success() {
                let _ = app.emit(
                    "transcribe-success",
                    format!("[{}] 转写完成 ✅", label),
                );
                let done = update_task(&app, &id, &source.id, "done", "完成");
                if let Some(t) = done {
                    let _ = app.emit("task-updated", t);
                }
                let generated_paths: Vec<String> = config.formats.split(',').map(str::trim).filter(|f| !f.is_empty()).map(|f| format!("{}.{}", output_base, f)).filter(|p| PathBuf::from(p).exists()).collect();
                let generated: Vec<String> = generated_paths.iter().filter_map(|p| PathBuf::from(p).file_name().and_then(|n| n.to_str()).map(String::from)).collect();
                if let Ok(mut map) = app.state::<AppState>().tasks.lock() {
                    if let Some(items) = map.get_mut(&id) {
                        if let Some(task) = items.iter_mut().find(|t| t.id == source.id) { task.artifact_ids = generated; task.artifact_paths = generated_paths; }
                        let _ = project::write_tasks(&app, &id, items);
                    }
                }
                let _ = app.emit("artifacts-updated", source.id.clone());
            } else {
                let detail = if err_msg.trim().is_empty() {
                    "详细错误请查看日志".to_string()
                } else {
                    err_msg.trim().to_string()
                };
                let _ = app.emit(
                    "transcribe-error",
                    format!("[{}] 转写异常结束: {}", label, detail),
                );
                let er = update_task(&app, &id, &source.id, "error", &detail);
                if let Some(t) = er {
                    let _ = app.emit("task-updated", t);
                }
            }
        }
        Err(e) => {
            let _ = app.emit(
                "transcribe-error",
                format!("[{}] 等待进程结束出错: {}", label, e),
            );
            let er = update_task(&app, &id, &source.id, "error", &e.to_string());
            if let Some(t) = er {
                let _ = app.emit("task-updated", t);
            }
        }
    }
    if let Ok(mut pids) = app.state::<AppState>().active_pids.lock() { pids.remove(&source.id); }
}

#[tauri::command]
pub fn cancel_task(app: tauri::AppHandle, id: String, task_id: String) -> Result<TaskItem, String> {
    let pid = app.state::<AppState>().active_pids.lock().ok().and_then(|m| m.get(&task_id).copied());
    #[cfg(target_os = "windows")]
    if let Some(pid) = pid {
        let _ = Command::new("taskkill").args(["/PID", &pid.to_string(), "/T", "/F"]).creation_flags(0x08000000).output();
    }
    let state = app.state::<AppState>();
    let mut map = state.tasks.lock().map_err(|e| e.to_string())?;
    let tasks = map.entry(id.clone()).or_default();
    let task = tasks.iter_mut().find(|t| t.id == task_id).ok_or("未找到任务")?;
    task.canceled = true;
    task.status = "canceled".into();
    task.message = "已取消".into();
    task.finished_at = Some(project::now_str());
    let result = task.clone();
    project::write_tasks(&app, &id, tasks)?;
    let _ = app.emit("task-updated", result.clone());
    Ok(result)
}

#[tauri::command]
pub fn retry_task(app: tauri::AppHandle, id: String, task_id: String) -> Result<(), String> {
    {
        let state = app.state::<AppState>();
        let mut map = state.tasks.lock().map_err(|e| e.to_string())?;
        let tasks = map.entry(id.clone()).or_default();
        let task = tasks.iter_mut().find(|t| t.id == task_id).ok_or("未找到任务")?;
        task.status = "queued".into(); task.canceled = false; task.error.clear(); task.message.clear(); task.finished_at = None; task.progress = 0.0;
        project::write_tasks(&app, &id, tasks)?;
    }
    run_transcribe(app, id, vec![task_id], None)
}

// ─── 文件 / 系统命令 ────────────────────────────────────────

#[tauri::command]
pub fn read_text_file(path: String) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_file(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/c", "start", "", &path])
            .creation_flags(0x08000000)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn show_in_folder(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let path = path.replace("/", "\\");
        Command::new("explorer")
            .arg("/select,")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .args(["-R", &path])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let parent = std::path::Path::new(&path)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| ".".to_string());
        Command::new("xdg-open")
            .arg(&parent)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn get_app_dir() -> Result<String, String> {
    let exe = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));
    Ok(exe.to_string_lossy().to_string())
}

#[tauri::command]
pub fn check_dependencies() -> Result<(bool, bool, bool, bool, bool), String> {
    let has_python = detect_python().is_some();
    let has_ffmpeg = background_command("ffmpeg").arg("-version").output().is_ok();
    let has_ytdlp = background_command("yt-dlp").arg("--version").output().is_ok();
    Ok((has_python, has_ffmpeg, has_ytdlp, false, false))
}

/// 打开系统原生多文件选择对话框，返回所选文件的绝对路径列表。
#[tauri::command]
pub fn pick_import_files(app: tauri::AppHandle) -> Result<Option<Vec<String>>, String> {
    let (tx, rx) = std::sync::mpsc::channel();
    app.run_on_main_thread(move || {
        let files = rfd::FileDialog::new()
            .add_filter(
                "媒体与文档",
                &[
                    "mp4", "mov", "avi", "mkv", "mp3", "wav", "m4a", "aac", "flac", "ogg", "pdf",
                    "doc", "docx", "txt", "md",
                ],
            )
            .pick_files();
        let paths = files.map(|l| l.into_iter().map(|p| p.to_string_lossy().to_string()).collect());
        let _ = tx.send(paths);
    })
    .map_err(|e| e.to_string())?;
    rx.recv().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn pick_document_files(app: tauri::AppHandle) -> Result<Option<Vec<String>>, String> {
    let (tx, rx) = std::sync::mpsc::channel();
    app.run_on_main_thread(move || {
        let files = rfd::FileDialog::new().add_filter("文档", &["txt", "md", "markdown", "srt", "pdf"]).pick_files();
        let _ = tx.send(files.map(|list| list.into_iter().map(|p| p.to_string_lossy().to_string()).collect()));
    }).map_err(|e| e.to_string())?;
    rx.recv().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn run_document_reorganize(
    app: tauri::AppHandle, id: String, inputs: Vec<String>, prompt: String, provider: String,
    model_name: String, series: bool, output_dir: Option<String>,
) -> Result<(), String> {
    if inputs.is_empty() { return Err("请先选择 TXT、Markdown 或 PDF 文件。".into()); }
    if prompt.trim().is_empty() { return Err("请选择或填写整理提示词。".into()); }
    let mut cfg = hydrate_secrets(&app, project::read_project(&app, &id)?.config);
    cfg.llm_provider = provider.clone();
    if provider == "gemini" && cfg.gemini_key.trim().is_empty() && cfg.llm_api_key.trim().is_empty() { return Err("请先在设置中配置 Gemini API Key。".into()); }
    if provider == "custom" && (cfg.llm_api_url.trim().is_empty() || cfg.llm_api_key.trim().is_empty()) { return Err("请先配置自定义 AI 的 API 地址和 Key。".into()); }
    let script = find_transcribe_script(&app).ok_or("未找到转写工具目录。")?.parent().unwrap().join("reorganize_documents.py");
    let target = output_dir.map(PathBuf::from).unwrap_or_else(|| default_downloads_dir().join(project::sanitize(&project::read_project(&app, &id).map(|p| p.name).unwrap_or_else(|_| id.clone()))).join("文档重整"));
    let python = detect_python().ok_or("未找到 Python。")?;
    let task_id = uuid::Uuid::new_v4().to_string();
    let task_name = if series { "系列课程重整".to_string() } else { inputs.first().and_then(|p| PathBuf::from(p).file_stem().and_then(|s| s.to_str()).map(|s| format!("{} · 文档重整", s))).unwrap_or_else(|| "文档重整".to_string()) };
    let final_model = if model_name.trim().is_empty() { cfg.gemini_model.clone() } else { model_name };
    let mut document_tasks = project::read_document_tasks(&app, &id);
    document_tasks.insert(0, project::DocumentTask { id: task_id.clone(), name: task_name, inputs: inputs.clone(), status: "running".into(), template: prompt.clone(), model: final_model.clone(), series, created_at: project::now_str(), finished_at: String::new(), output_path: String::new(), error: String::new() });
    project::write_document_tasks(&app, &id, &document_tasks)?;
    let app2 = app.clone();
    std::thread::spawn(move || {
        let mut cmd = background_command(&python);
        cmd.args([script.to_string_lossy().as_ref(), "--inputs", &serde_json::to_string(&inputs).unwrap_or_default(), "--output-dir", target.to_string_lossy().as_ref(), "--provider", &provider, "--prompt", &prompt]);
        if series { cmd.arg("--series"); }
        if !final_model.trim().is_empty() { cmd.args(["--model-name", &final_model]); }
        if !cfg.llm_api_key.trim().is_empty() { cmd.args(["--api-key", &cfg.llm_api_key]); }
        if !cfg.llm_api_url.trim().is_empty() { cmd.args(["--api-url", &cfg.llm_api_url]); }
        if !cfg.gemini_key.trim().is_empty() { cmd.env("GEMINI_API_KEY", &cfg.gemini_key); }
        match cmd.output() {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                let output_path = stdout.lines().find_map(|line| line.strip_prefix("OUTPUT:")).unwrap_or("").trim().to_string();
                let mut tasks = project::read_document_tasks(&app2, &id);
                if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) { task.status = "done".into(); task.finished_at = project::now_str(); task.output_path = output_path.clone(); }
                let _ = project::write_document_tasks(&app2, &id, &tasks);
                let _ = app2.emit("document-reorganize-complete", stdout);
            }
            Ok(out) => {
                let error = String::from_utf8_lossy(&out.stderr).to_string();
                let mut tasks = project::read_document_tasks(&app2, &id);
                if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) { task.status = "error".into(); task.finished_at = project::now_str(); task.error = error.clone(); }
                let _ = project::write_document_tasks(&app2, &id, &tasks);
                let _ = app2.emit("document-reorganize-error", error);
            }
            Err(e) => {
                let error = e.to_string(); let mut tasks = project::read_document_tasks(&app2, &id);
                if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) { task.status = "error".into(); task.finished_at = project::now_str(); task.error = error.clone(); }
                let _ = project::write_document_tasks(&app2, &id, &tasks);
                let _ = app2.emit("document-reorganize-error", error);
            }
        }
    });
    Ok(())
}

#[tauri::command]
pub fn list_document_tasks(app: tauri::AppHandle, id: String) -> Result<Vec<project::DocumentTask>, String> { Ok(project::read_document_tasks(&app, &id)) }

#[tauri::command]
pub fn pick_cookie_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let (tx, rx) = std::sync::mpsc::channel();
    app.run_on_main_thread(move || {
        let path = rfd::FileDialog::new().add_filter("Netscape Cookie", &["txt"]).pick_file().map(|p| p.to_string_lossy().to_string());
        let _ = tx.send(path);
    }).map_err(|e| e.to_string())?;
    rx.recv().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn pick_import_folder(app: tauri::AppHandle) -> Result<Option<Vec<String>>, String> {
    let (tx, rx) = std::sync::mpsc::channel();
    app.run_on_main_thread(move || {
        let paths = rfd::FileDialog::new().pick_folder().map(|dir| {
            let mut files = Vec::new();
            let mut pending = vec![dir];
            while let Some(current) = pending.pop() {
                if let Ok(entries) = std::fs::read_dir(current) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() { pending.push(path); }
                        else if path.extension().and_then(|x| x.to_str()).map(|x| matches!(x.to_ascii_lowercase().as_str(), "mp4"|"mov"|"avi"|"mkv"|"flv"|"wmv"|"webm"|"mp3"|"wav"|"m4a"|"aac"|"flac"|"ogg"|"opus")).unwrap_or(false) {
                            files.push(path.to_string_lossy().to_string());
                        }
                    }
                }
            }
            files
        });
        let _ = tx.send(paths);
    }).map_err(|e| e.to_string())?;
    rx.recv().map_err(|e| e.to_string())
}

// ─── 任务队列命令 ───────────────────────────────────────────

#[tauri::command]
pub fn list_tasks(app: tauri::AppHandle, id: String) -> Result<Vec<TaskItem>, String> {
    let state = app.state::<AppState>();
    let map = state.tasks.lock().map_err(|e| e.to_string())?;
    if let Some(tasks) = map.get(&id) {
        return Ok(tasks.clone());
    }
    drop(map);
    Ok(project::read_tasks(&app, &id))
}

#[tauri::command]
pub fn clear_tasks(app: tauri::AppHandle, id: String) -> Result<(), String> {
    {
        let state = app.state::<AppState>();
        let mut map = state.tasks.lock().map_err(|e| e.to_string())?;
        map.insert(id.clone(), Vec::new());
    }
    project::write_tasks(&app, &id, &[])
}

// ─── 文件读取（预览用） ─────────────────────────────────────

/// 打开系统原生文件夹选择对话框，返回所选目录的绝对路径。
#[tauri::command]
pub fn pick_export_dir(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let (tx, rx) = std::sync::mpsc::channel();
    app.run_on_main_thread(move || {
        let dir = rfd::FileDialog::new().pick_folder();
        let p = dir.map(|p| p.to_string_lossy().to_string());
        let _ = tx.send(p);
    })
    .map_err(|e| e.to_string())?;
    rx.recv().map_err(|e| e.to_string())
}

/// 将项目产物导出（复制）到目标目录。names 为空表示导出全部。
#[tauri::command]
pub fn export_artifacts(
    app: tauri::AppHandle,
    id: String,
    target_dir: String,
    names: Option<Vec<String>>,
) -> Result<project::ExportResult, String> {
    let arts = project::list_artifacts(&app, &id);
    if arts.is_empty() {
        return Err("当前项目暂无产物可导出。".into());
    }
    let target = PathBuf::from(&target_dir);
    std::fs::create_dir_all(&target).map_err(|e| e.to_string())?;
    let mut copied: u32 = 0;
    let mut failed: Vec<String> = Vec::new();
    for a in arts {
        if let Some(names) = &names {
            if !names.contains(&a.name) {
                continue;
            }
        }
        let dest = target.join(&a.name);
        match std::fs::copy(&a.path, &dest) {
            Ok(_) => copied += 1,
            Err(e) => failed.push(format!("{}: {}", a.name, e)),
        }
    }
    if copied == 0 && !failed.is_empty() {
        return Err(format!("导出失败：{}", failed.join("; ")));
    }
    Ok(project::ExportResult {
        copied,
        failed,
        target: target_dir,
    })
}

#[tauri::command]
pub fn read_file_base64(path: String) -> Result<String, String> {
    let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
    Ok(base64::engine::general_purpose::STANDARD.encode(&bytes))
}

// ─── 配置中心辅助命令 ───────────────────────────────────────

/// 轻量连接检查：根据引擎判断配置是否齐备（Bcut 免密钥联网即可，Gemini 需 Key）。
#[tauri::command]
pub fn test_connection(app: tauri::AppHandle, id: String, config: Option<ProjectConfig>) -> Result<String, String> {
    let cfg = hydrate_secrets(&app, config.unwrap_or(project::read_project(&app, &id)?.config));
    let msg = match cfg.engine.as_str() {
        "bcut" => "必剪 Bcut：免密钥在线转写，联网即可使用。".to_string(),
        "mlx" => "mlx-whisper：将在 Apple Silicon 上本地离线转写。".to_string(),
        "custom" => {
            if cfg.custom_api_url.trim().is_empty() || cfg.custom_api_key.trim().is_empty() {
                "自配置 AI 接口：请填写 ASR API 地址和 API Key 后再试。".to_string()
            } else {
                format!("自定义引擎：将使用 {}", cfg.custom_api_url)
            }
        }
        _ => {
            if cfg.gemini_key.trim().is_empty() {
                "Gemini 引擎：需要填写 GEMINI_API_KEY 才能转写。".to_string()
            } else {
                "Gemini 引擎：已配置 API Key，可以开始转换。".to_string()
            }
        }
    };
    Ok(msg)
}

/// 显式测试 AI 整理服务；不会在启动期自动调用。
#[tauri::command]
pub fn test_ai_connection(app: tauri::AppHandle, id: String, config: Option<ProjectConfig>) -> Result<String, String> {
    let cfg = hydrate_secrets(&app, config.unwrap_or(project::read_project(&app, &id)?.config));
    if cfg.llm_provider == "none" { return Err("请先启用 AI 整理服务。".into()); }
    #[cfg(target_os = "windows")]
    {
        let (script, key, api_url, model) = if cfg.llm_provider == "gemini" {
            let key = cfg.llm_api_key;
            if key.trim().is_empty() { return Err("缺少 Gemini 文本优化 API Key。请在“AI 文本优化”中单独填写。".into()); }
            ("$r=Invoke-WebRequest -UseBasicParsing -TimeoutSec 12 -Uri ('https://generativelanguage.googleapis.com/v1beta/models/'+$env:VIDEO2MD_AI_MODEL+'?key='+$env:VIDEO2MD_AI_KEY);if($r.StatusCode -ne 200){exit 1}", key, String::new(), if cfg.llm_model_name.trim().is_empty() { cfg.gemini_model } else { cfg.llm_model_name })
        } else {
            if cfg.llm_api_url.trim().is_empty() || cfg.llm_api_key.trim().is_empty() { return Err("自定义 AI 整理需要 API 地址和 Key。".into()); }
            ("$ErrorActionPreference='Stop';try{$u=$env:VIDEO2MD_AI_URL.TrimEnd('/');if(-not $u.EndsWith('/chat/completions')){$u+='/chat/completions'};$h=@{Authorization=('Bearer '+$env:VIDEO2MD_AI_KEY)};$body=@{model=$env:VIDEO2MD_AI_MODEL;messages=@(@{role='user';content='ping'});max_tokens=1}|ConvertTo-Json -Depth 5;$r=Invoke-WebRequest -UseBasicParsing -TimeoutSec 30 -Method POST -ContentType 'application/json' -Headers $h -Body $body -Uri $u;if($r.StatusCode -lt 200 -or $r.StatusCode -ge 300){throw ('HTTP '+$r.StatusCode)}}catch{[Console]::Error.WriteLine($_.Exception.Message);exit 1}", cfg.llm_api_key, cfg.llm_api_url, if cfg.llm_model_name.trim().is_empty() { "gpt-4o-mini".into() } else { cfg.llm_model_name })
        };
        let output = background_command("powershell").args(["-NoProfile","-NonInteractive","-Command",script]).env("VIDEO2MD_AI_KEY", key).env("VIDEO2MD_AI_URL", api_url).env("VIDEO2MD_AI_MODEL", model).output().map_err(|e| e.to_string())?;
        if !output.status.success() { return Err(format!("AI 服务不可用：{}", String::from_utf8_lossy(&output.stderr).trim())); }
        return Ok("AI 整理服务连接正常。".into());
    }
    #[cfg(not(target_os = "windows"))]
    { Err("当前平台暂未提供 AI 连通性测试。".into()) }
}

#[tauri::command]
pub fn rename_project(
    app: tauri::AppHandle,
    id: String,
    name: String,
) -> Result<Project, String> {
    let mut project = project::read_project(&app, &id)?;
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("项目名称不能为空。".into());
    }
    project.name = trimmed.to_string();
    project.updated_at = project::now_str();
    project::write_project(&app, &project)?;
    Ok(project)
}
