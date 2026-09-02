import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

// ─── 类型定义 ───────────────────────────────────────────────

export type SourceType = 'video' | 'audio' | 'pdf' | 'doc' | 'file' | 'url';
export type SourceStatus = 'ready' | 'processing' | 'done' | 'error';

export interface SourceItem {
  id: string;
  name: string;
  type: SourceType;
  path: string;
  size: number;
  status: SourceStatus;
  duration?: string | null;
}

export interface ProjectConfig {
  engine: string;
  gemini_key: string;
  gemini_model: string;
  llm_provider: string;
  llm_api_key: string;
  llm_api_url: string;
  llm_model_name: string;
  prompt_template: string;
  custom_api_url: string;
  custom_api_key: string;
  custom_model_name: string;
  proxy: string;
  no_subtitle: boolean;
  cookies_from_browser: string;
  youtube_cookie_file: string;
  local_output_dir: string;
  online_output_dir: string;
  formats: string;
}

export interface Project {
  id: string;
  name: string;
  created_at: string;
  updated_at: string;
  sources: SourceItem[];
  config: ProjectConfig;
}

export interface ProjectSummary {
  id: string;
  name: string;
  created_at: string;
  updated_at: string;
  source_count: number;
  artifact_count: number;
}

export interface ArtifactItem {
  name: string;
  path: string;
  ext: string;
  size: number;
  modified: string;
  source_id?: string;
  task_id?: string;
  kind?: string;
}

export interface TaskItem {
  id: string;
  source_id: string;
  source_name: string;
  engine: string;
  status: string;
  created_at: string;
  finished_at: string | null;
  message: string;
  progress: number;
  formats: string;
  llm_provider: string;
  artifact_ids: string[];
  artifact_paths: string[];
  error: string;
  canceled: boolean;
  config: ProjectConfig;
}
export interface DocumentTask { id:string; name:string; inputs:string[]; status:string; template:string; model:string; series:boolean; created_at:string; finished_at:string; output_path:string; error:string; }

export interface DepStatus {
  python: boolean;
  ffmpeg: boolean;
  ytdlp: boolean;
  bilibili: boolean;
  youtube: boolean;
}

// ─── invoke 封装 ────────────────────────────────────────────

export const api = {
  createProject: (name: string) => invoke<Project>('create_project', { name }),
  listProjects: () => invoke<ProjectSummary[]>('list_projects'),
  getProject: (id: string) => invoke<Project>('get_project', { id }),
  setCurrentProject: (id: string | null) =>
    invoke('set_current_project', { id: id ?? null }),
  getCurrentProject: () => invoke<Project | null>('get_current_project'),
  getPlatformCapabilities: () => invoke<[string,string,boolean]>('get_platform_capabilities'),
  importFiles: (id: string, paths: string[]) =>
    invoke<SourceItem[]>('import_files', { id, paths }),
  importOnlineSources: (id: string, urls: string[], parseBatch: boolean) =>
    invoke<SourceItem[]>('import_online_sources', { id, urls, parseBatch }),
  removeSource: (id: string, sourceId: string) =>
    invoke<SourceItem[]>('remove_source', { id, sourceId }),
  clearSources: (id: string) => invoke<SourceItem[]>('clear_sources', { id }),
  listSources: (id: string) => invoke<SourceItem[]>('list_sources', { id }),
  listArtifacts: (id: string) => invoke<ArtifactItem[]>('list_artifacts', { id }),
  getConfig: (id: string) => invoke<ProjectConfig>('get_config', { id }),
  setConfig: (id: string, config: ProjectConfig) =>
    invoke<ProjectConfig>('set_config', { id, config }),
  runTranscribe: (id: string, sourceIds: string[], engine?: string) =>
    invoke('run_transcribe', {
      id,
      sourceIds,
      engine: engine ?? null,
    }),
  readTextFile: (path: string) => invoke<string>('read_text_file', { path }),
  readFileBase64: (path: string) => invoke<string>('read_file_base64', { path }),
  openFile: (path: string) => invoke('open_file', { path }),
  showInFolder: (path: string) => invoke('show_in_folder', { path }),
  getAppDir: () => invoke<string>('get_app_dir'),
  checkDependencies: () => invoke<[boolean, boolean, boolean, boolean, boolean]>('check_dependencies'),
  pickImportFiles: () => invoke<Array<string> | null>('pick_import_files'),
  pickDocumentFiles: () => invoke<Array<string> | null>('pick_document_files'),
  pickCookieFile: () => invoke<string | null>('pick_cookie_file'),
  startBilibiliQrLogin: () => invoke<{status:string;key?:string;url?:string;message?:string}>('start_bilibili_qr_login'),
  pollBilibiliQrLogin: (id:string,key:string) => invoke<{status:string;message?:string}>('poll_bilibili_qr_login',{id,key}),
  pickImportFolder: () => invoke<Array<string> | null>('pick_import_folder'),
  listTasks: (id: string) => invoke<TaskItem[]>('list_tasks', { id }),
  clearTasks: (id: string) => invoke('clear_tasks', { id }),
  cancelTask: (id: string, taskId: string) => invoke<TaskItem>('cancel_task', { id, taskId }),
  retryTask: (id: string, taskId: string) => invoke('retry_task', { id, taskId }),
  testConnection: (id: string, config: ProjectConfig) => invoke<string>('test_connection', { id, config }),
  testAiConnection: (id: string, config: ProjectConfig) => invoke<string>('test_ai_connection', { id, config }),
  renameProject: (id: string, name: string) =>
    invoke<Project>('rename_project', { id, name }),
  pickExportDir: () => invoke<string | null>('pick_export_dir'),
  exportArtifacts: (id: string, targetDir: string, names?: string[]) =>
    invoke<ExportResult>('export_artifacts', {
      id,
      targetDir,
      names: names ?? null,
    }),
  runDocumentReorganize: (id:string, inputs:string[], prompt:string, provider:string, modelName:string, series:boolean, outputDir?:string) => invoke('run_document_reorganize',{id,inputs,prompt,provider,modelName,series,outputDir:outputDir??null}),
  listDocumentTasks: (id:string) => invoke<DocumentTask[]>('list_document_tasks',{id}),
};

export interface ExportResult {
  copied: number;
  failed: string[];
  target: string;
}

// ─── 事件监听 ───────────────────────────────────────────────

export function onTranscribeLog(cb: (line: string) => void): Promise<() => void> {
  return listen<string>('transcribe-log', (e) => cb(e.payload));
}
export function onTranscribeSuccess(cb: (msg: string) => void): Promise<() => void> {
  return listen<string>('transcribe-success', (e) => cb(e.payload));
}
export function onTranscribeError(cb: (msg: string) => void): Promise<() => void> {
  return listen<string>('transcribe-error', (e) => cb(e.payload));
}
export function onTaskUpdated(cb: (task: TaskItem) => void): Promise<() => void> {
  return listen<TaskItem>('task-updated', (e) => cb(e.payload));
}
export function onDownloadProgress(cb: (payload: { sourceId: string; percent: number }) => void): Promise<() => void> {
  return listen<string>('download-progress', (e) => {
    const [sourceId, pct] = e.payload.split('|');
    const percent = Number(pct);
    if (sourceId) cb({ sourceId, percent: Number.isFinite(percent) ? percent : 0 });
  });
}
export function onArtifactsUpdated(cb: (sourceId: string) => void): Promise<() => void> {
  return listen<string>('artifacts-updated', (e) => cb(e.payload));
}
export function onDocumentReorganizeComplete(cb:(output:string)=>void):Promise<()=>void>{return listen<string>('document-reorganize-complete',(e)=>cb(e.payload));}
export function onDocumentReorganizeError(cb:(error:string)=>void):Promise<()=>void>{return listen<string>('document-reorganize-error',(e)=>cb(e.payload));}
