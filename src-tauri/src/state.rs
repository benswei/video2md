use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

use crate::project::TaskItem;

/// 全局应用状态：记录当前选中的项目 ID，以及各项目的任务队列。
#[derive(Default)]
pub struct AppState {
    pub current_project: Mutex<Option<String>>,
    pub tasks: Mutex<HashMap<String, Vec<TaskItem>>>,
    /// 正在执行的任务及其子进程 PID，用于 Windows 上即时取消。
    pub active_pids: Mutex<HashMap<String, u32>>,
    pub active_projects: Mutex<HashSet<String>>,
}
