import { reactive } from 'vue';
import {
  api,
  onTranscribeLog,
  onTranscribeSuccess,
  onTranscribeError,
  onTaskUpdated,
  onDownloadProgress,
  type TaskItem,
} from '../services/tauri';
import { notify } from './ui';

interface TaskState {
  tasks: TaskItem[];
  logs: string[];
  running: boolean;
  lastError: string | null;
  lastSuccess: string | null;
  /** 各素材的 URL 下载进度（百分比），仅下载阶段存在 */
  downloadProgress: Record<string, number>;
}

export const taskStore = reactive<TaskState>({
  tasks: [],
  logs: [],
  running: false,
  lastError: null,
  lastSuccess: null,
  downloadProgress: {},
});

let initialized = false;

function recomputeRunning() {
  taskStore.running = taskStore.tasks.some(
    (t) => ['queued', 'downloading', 'transcribing', 'organizing'].includes(t.status),
  );
}

export async function initTaskListeners() {
  if (initialized) return;
  initialized = true;
  await onTranscribeLog((line) => {
    taskStore.logs.push(line);
    taskStore.running = true;
  });
  await onTranscribeSuccess((msg) => {
    taskStore.logs.push(msg);
    taskStore.lastSuccess = msg;
    recomputeRunning();
    notify(msg || '转换完成', 'success', 5000);
  });
  await onTranscribeError((msg) => {
    taskStore.logs.push(msg);
    taskStore.lastError = msg;
    recomputeRunning();
    notify(msg || '转换失败', 'error', 8000);
  });
  await onTaskUpdated((t) => {
    const i = taskStore.tasks.findIndex((x) => x.id === t.id);
    if (i >= 0) taskStore.tasks[i] = t;
    else taskStore.tasks.push(t);
    if (t.status === 'done' || t.status === 'error') {
      delete taskStore.downloadProgress[t.id];
    }
    recomputeRunning();
  });
  await onDownloadProgress(({ sourceId, percent }) => {
    taskStore.downloadProgress[sourceId] = percent;
    taskStore.running = true;
  });
}

/** 从磁盘加载某项目的任务队列（重启后可恢复）。 */
export async function loadTasks(projectId: string) {
  try {
    taskStore.tasks = await api.listTasks(projectId);
    recomputeRunning();
  } catch {
    /* 读取失败不阻塞界面 */
  }
}

export function resetTask() {
  taskStore.logs = [];
  taskStore.lastError = null;
  taskStore.lastSuccess = null;
  taskStore.running = false;
}
