import { reactive } from 'vue';
import {
  api,
  type Project,
  type ProjectConfig,
  type ArtifactItem,
  type ProjectSummary,
  type DepStatus,
} from '../services/tauri';
import { taskStore, loadTasks } from './task';
import { notify } from './ui';

interface ProjectState {
  project: Project | null;
  projects: ProjectSummary[];
  artifacts: ArtifactItem[];
  loading: boolean;
  deps: DepStatus | null;
}

export const projectStore = reactive<ProjectState>({
  project: null,
  projects: [],
  artifacts: [],
  loading: false,
  deps: null,
});

function summarize(p: Project): ProjectSummary {
  return {
    id: p.id,
    name: p.name,
    created_at: p.created_at,
    updated_at: p.updated_at,
    source_count: p.sources.length,
    artifact_count: 0,
  };
}

export async function bootstrap() {
  try {
    projectStore.loading = true;
    const list = await api.listProjects();
    projectStore.projects = list;
    const cur = await api.getCurrentProject();
    if (cur) {
      cur.config = await api.getConfig(cur.id);
      projectStore.project = cur;
      await refreshArtifacts();
      await loadTasks(cur.id);
    } else {
      if (list.length > 0) {
        await openProject(list[0].id);
      } else {
        const p = await api.createProject('我的知识库');
        projectStore.project = p;
        projectStore.projects = [summarize(p)];
      }
    }
  } catch (e) {
    console.error('bootstrap failed', e);
    notify(`项目初始化失败：${e}`, 'error', 8000);
  } finally {
    projectStore.loading = false;
  }

  window.setTimeout(async () => {
    try {
      const [python, ffmpeg, ytdlp, bilibili, youtube] = await api.checkDependencies();
      projectStore.deps = { python, ffmpeg, ytdlp, bilibili, youtube };
    } catch { /* 不影响启动 */ }
  }, 1500);
}

export async function openProject(id: string) {
  const p = await api.getProject(id);
  projectStore.project = p;
  taskStore.tasks = [];
  void api.setCurrentProject(id);
  // 先完成项目切换，产物、配置和任务在后台补齐，避免大项目切换时界面冻结。
  void Promise.all([api.getConfig(id), api.listArtifacts(id), api.listTasks(id)]).then(([config, artifacts, tasks]) => {
    if (projectStore.project?.id !== id) return;
    projectStore.project.config = config;
    projectStore.artifacts = artifacts;
    taskStore.tasks = tasks;
  });
}

export async function refreshProject() {
  if (!projectStore.project) return;
  projectStore.project = await api.getProject(projectStore.project.id);
  await refreshArtifacts();
}

export async function refreshArtifacts() {
  if (!projectStore.project) return;
  projectStore.artifacts = await api.listArtifacts(projectStore.project.id);
  const idx = projectStore.projects.findIndex(
    (p) => p.id === projectStore.project!.id,
  );
  if (idx >= 0) projectStore.projects[idx].artifact_count = projectStore.artifacts.length;
}

export async function importFiles(paths: string[]) {
  if (!projectStore.project) return;
  const sources = await api.importFiles(projectStore.project.id, paths);
  projectStore.project.sources = sources;
}

export async function importOnlineSources(urls: string[], parseBatch: boolean) {
  if (!projectStore.project) return;
  const sources = await api.importOnlineSources(projectStore.project.id, urls, parseBatch);
  projectStore.project.sources = sources;
}

export async function removeSource(sourceId: string) {
  if (!projectStore.project) return;
  const sources = await api.removeSource(projectStore.project.id, sourceId);
  projectStore.project.sources = sources;
  await refreshArtifacts();
}

export async function clearSources() {
  if (!projectStore.project) return;
  projectStore.project.sources = await api.clearSources(projectStore.project.id);
}

export async function saveConfig(config: ProjectConfig) {
  if (!projectStore.project) return;
  const updated = await api.setConfig(projectStore.project.id, config);
  projectStore.project.config = updated;
}

export async function runTranscribe(sourceIds: string[] = [], engine?: string) {
  if (!projectStore.project) return;
  taskStore.lastError = null;
  taskStore.running = true;
  try {
    await api.runTranscribe(projectStore.project.id, sourceIds, engine);
    if (projectStore.project) await loadTasks(projectStore.project.id);
  } catch (e: any) {
    taskStore.running = false;
    taskStore.lastError = String(e);
    taskStore.logs.push('启动失败: ' + e);
    notify(`无法启动转换：${e}`, 'error', 7000);
  }
}

export async function clearTasks() {
  if (!projectStore.project) return;
  await api.clearTasks(projectStore.project.id);
  taskStore.tasks = [];
  await refreshArtifacts();
}

export async function renameProject(id: string, name: string) {
  const p = await api.renameProject(id, name);
  projectStore.project = p;
  const idx = projectStore.projects.findIndex((x) => x.id === p.id);
  if (idx >= 0) projectStore.projects[idx].name = p.name;
  return p;
}

export async function createProject(name: string) {
  const p = await api.createProject(name);
  projectStore.project = p;
  projectStore.projects = [summarize(p), ...projectStore.projects];
  return p;
}

export function formatSize(bytes: number): string {
  if (!bytes) return '';
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  let i = 0;
  let n = bytes;
  while (n >= 1024 && i < units.length - 1) {
    n /= 1024;
    i++;
  }
  return `${n.toFixed(n >= 100 || i === 0 ? 0 : 1)} ${units[i]}`;
}

export function engineLabel(engine: string): string {
  switch (engine) {
    case 'gemini':
      return 'Gemini';
    case 'bcut':
      return '必剪 Bcut (免密钥)';
    case 'custom':
      return '自定义 OpenAI';
    case 'mlx':
      return 'mlx-whisper（本地）';
    default:
      return engine || '未设置';
  }
}
