<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { projectStore, importFiles, importOnlineSources, removeSource, clearSources } from '../stores/project';
import { api } from '../services/tauri';
import { notify } from '../stores/ui';

const dragOver = ref(false);
const files = computed(() => projectStore.project?.sources ?? []);
const busy = ref(false);
const urlOpen = ref(false);
const urlValue = ref('');
const parseBatch = ref(false);
const removingId = ref<string | null>(null);

const typeIcons: Record<string, string> = {
  video: 'M23 7l-8 4.5V7l-8 4.5L7 7l-6 3.5V19a2 2 0 002 2h16a2 2 0 002-2V7l-6-4-8 4z',
  audio: 'M9 18V5l12-2v13',
  pdf: 'M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z',
  url: 'M10 13a5 5 0 007.54.54l3-3a5 5 0 00-7.07-7.07l-1.72 1.71',
  doc: 'M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z',
  file: 'M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z',
};

const typeColors: Record<string, string> = {
  video: 'oklch(62% 0.15 250)',
  audio: 'oklch(60% 0.14 145)',
  pdf: 'oklch(55% 0.20 25)',
  url: 'oklch(58% 0.12 240)',
  doc: 'oklch(70% 0.14 70)',
  file: 'oklch(52% 0.015 60)',
};

const statusLabels: Record<string, string> = {
  ready: '待处理', processing: '处理中', done: '已完成', error: '出错',
};

async function chooseFiles() {
  if (busy.value) return;
  try {
    const paths = await api.pickImportFiles();
    if (paths && paths.length) {
      busy.value = true;
      await importFiles(paths);
      notify(`已导入 ${paths.length} 个素材`, 'success');
    }
  } catch (error) {
    notify(`导入失败：${error}`, 'error');
  } finally {
    busy.value = false;
  }
}

async function chooseFolder() {
  if (busy.value) return;
  try {
    const paths = await api.pickImportFolder();
    if (paths?.length) {
      busy.value = true;
      await importFiles(paths);
      notify(`已从文件夹导入 ${paths.length} 个音视频`, 'success');
    } else if (paths) notify('所选文件夹中没有支持的音视频', 'info');
  } catch (error) { notify(`文件夹导入失败：${error}`, 'error'); }
  finally { busy.value = false; }
}

async function addUrl() {
  const urls = [...new Set(urlValue.value.split(/\r?\n/).map((u) => u.trim()).filter((u) => /^https?:\/\//i.test(u)))];
  if (!urls.length) {
    notify('每行请输入一个以 http:// 或 https:// 开头的链接', 'info');
    return;
  }
  busy.value = true;
  try {
    await importOnlineSources(urls, parseBatch.value);
    urlValue.value = '';
    urlOpen.value = false;
    notify(parseBatch.value ? '批量页面已解析并加入素材列表' : `已加入 ${urls.length} 条在线素材`, 'success');
  } catch (error) {
    notify(`添加链接失败：${error}`, 'error');
  } finally {
    busy.value = false;
  }
}

async function onRemove(id: string) {
  removingId.value = id;
  try {
    await removeSource(id);
    notify('素材已移除', 'success');
  } catch (error) {
    notify(`移除失败：${error}`, 'error');
  } finally {
    removingId.value = null;
  }
}
async function onClear() {
  if (!files.value.length || !window.confirm('清空素材列表？不会删除原始文件和已生成结果。')) return;
  try { await clearSources(); notify('素材列表已清空', 'success'); }
  catch (error) { notify(`无法清空：${error}`, 'error'); }
}

// 全局拖放：Tauri 窗体级拖放事件能拿到真实文件路径
let unlisten: (() => void) | null = null;
onMounted(async () => {
  const win = getCurrentWebviewWindow();
  unlisten = await win.onDragDropEvent((event) => {
    if (event.payload.type === 'drop') {
      const paths = event.payload.paths;
      if (paths && paths.length) {
        importFiles(paths);
      }
    }
  });
});
onUnmounted(() => unlisten?.());
</script>

<template>
  <div class="prepare-view">
    <!-- Import Zone -->
    <div
      class="drop-zone"
      :class="{ 'drag-over': dragOver }"
      @dragover.prevent="dragOver = true"
      @dragleave.prevent="dragOver = false"
      @drop.prevent="dragOver = false"
    >
      <div class="drop-icon">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" y1="3" x2="12" y2="15"/></svg>
      </div>
      <p class="drop-title">{{ dragOver ? '松开即可导入' : '拖放文件到此处导入' }}</p>
      <p class="drop-desc">支持视频 (MP4/MKV/MOV)、音频 (MP3/WAV/M4A)、PDF、Word、网页链接</p>
      <div class="drop-actions">
        <button class="btn btn-primary" :disabled="busy" @click="chooseFiles">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="16 16 12 12 8 16"/><line x1="12" y1="12" x2="12" y2="21"/><path d="M20.88 18.09A5 5 0 0018 9h-1.26A6 6 0 004 10.65"/><line x1="16" y1="10" x2="16" y2="3"/></svg>
          {{ busy ? '导入中…' : '选择文件' }}
        </button>
        <button class="btn" :disabled="busy" @click="chooseFolder">选择文件夹</button>
        <button class="btn" :aria-expanded="urlOpen" @click="urlOpen = !urlOpen">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="8" y1="12" x2="16" y2="12"/><line x1="12" y1="8" x2="12" y2="16"/></svg>
          添加链接
        </button>
      </div>
      <form v-if="urlOpen" class="url-form" @submit.prevent="addUrl">
        <label for="source-url">在线链接（每行一个）</label>
        <textarea id="source-url" v-model="urlValue" rows="5" placeholder="https://www.bilibili.com/video/...&#10;https://www.youtube.com/watch?v=..." autofocus />
        <label class="batch-check"><input v-model="parseBatch" type="checkbox" /><span><b>解析批量页面</b> · 个人主页、合集、频道或播放列表会展开为多个视频</span></label>
        <div class="url-actions"><small>{{ parseBatch ? '适用于 B站空间/合集、YouTube 频道/播放列表；每个页面最多导入 200 条。' : '支持 B站、YouTube 及 yt-dlp 可识别的网站。' }}</small><button class="btn btn-primary" :disabled="busy">{{ busy ? '解析中…' : '加入素材列表' }}</button></div>
      </form>
    </div>

    <!-- File List -->
    <div class="file-section">
      <div class="file-header">
        <span class="section-title">素材列表</span>
        <div class="file-tools"><span class="file-count">{{ files.length }} 个文件</span><button v-if="files.length" class="clear-list" @click="onClear">清空素材</button></div>
      </div>

      <div v-if="files.length === 0" class="empty-hint">
        还没有素材，拖入文件或点击「选择文件」开始。
      </div>

      <div class="file-grid" v-else>
        <div v-for="file in files" :key="file.id" class="file-card" :class="{ error: file.status === 'error' }">
          <div class="file-type" :style="{ background: (typeColors[file.type] || typeColors.file) + '18', color: typeColors[file.type] || typeColors.file }">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path :d="typeIcons[file.type] || typeIcons.file" />
            </svg>
          </div>
          <div class="file-info">
            <span class="file-name">{{ file.name }}</span>
            <span class="file-meta">
              <template v-if="file.size">{{ (file.size / 1048576).toFixed(1) }} MB</template>
            </span>
          </div>
          <div class="file-status">
            <span class="status-badge" :class="file.status">{{ statusLabels[file.status] || file.status }}</span>
            <button class="file-action" :disabled="removingId === file.id" title="从项目中移除" :aria-label="`移除 ${file.name}`" @click="onRemove(file.id)">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.url-form { width: min(640px, 100%); margin: var(--s-4) auto 0; padding: var(--s-3); border: 1px solid var(--border); border-radius: var(--r-md); background: var(--surface); text-align: left; }
.url-form label { display: block; margin-bottom: 6px; color: var(--text-2); font-size: 12px; font-weight: 600; }
.url-form textarea { width:100%; min-height:96px; padding:8px 10px; border:1px solid var(--border); border-radius:var(--r-sm); background:var(--bg); color:var(--text-1); resize:vertical; line-height:1.55; }
.batch-check{display:flex;align-items:flex-start;gap:8px;margin-top:10px;color:var(--text-2);font-size:12px;cursor:pointer}.batch-check input{margin-top:2px}.url-actions{display:flex;align-items:center;justify-content:space-between;gap:12px;margin-top:10px}.url-actions small{color:var(--text-3);font-size:11px;line-height:1.5}.url-actions .btn{flex:none}
.prepare-view {
  display: flex; flex-direction: column;
  gap: var(--s-5);
  max-width: 800px;
  animation: fadeIn 0.25s ease;
}
@keyframes fadeIn { from { opacity: 0; } to { opacity: 1; } }

.drop-zone {
  display: flex; flex-direction: column; align-items: center;
  padding: var(--s-8) var(--s-5);
  border: 2px dashed var(--neutral-3);
  border-radius: var(--r-lg);
  background: var(--surface);
  transition: all 0.2s ease;
  text-align: center;
}
.drop-zone.drag-over { border-color: var(--accent); background: var(--accent-bg); }
.drop-icon {
  width: 52px; height: 52px;
  display: flex; align-items: center; justify-content: center;
  border-radius: var(--r-lg);
  background: var(--accent-bg);
  color: var(--accent);
  margin-bottom: var(--s-3);
}
.drop-icon svg { width: 26px; height: 26px; }
.drop-title { font-size: 15px; font-weight: 600; color: var(--text-1); margin-bottom: var(--s-1); }
.drop-desc { font-size: 12px; color: var(--text-3); margin-bottom: var(--s-4); }
.drop-actions { display: flex; gap: var(--s-2); }

.btn {
  display: inline-flex; align-items: center; gap: 6px;
  padding: 7px 14px;
  border: 1px solid var(--border);
  border-radius: var(--r-sm);
  background: var(--surface);
  font-size: 12px; font-weight: 500;
  color: var(--text-2); cursor: pointer;
  transition: all 0.15s ease;
  font-family: var(--font-body);
}
.btn:hover { border-color: var(--accent); color: var(--accent); background: var(--accent-bg); }
.btn:disabled { opacity: 0.6; cursor: default; }
.btn svg { width: 14px; height: 14px; }
.btn-primary { background: var(--accent); border-color: var(--accent); color: #fff; }
.btn-primary:hover { background: var(--accent-hover); border-color: var(--accent-hover); color: #fff; }

.file-section { display: flex; flex-direction: column; gap: var(--s-3); }
.file-header { display: flex; align-items: center; justify-content: space-between; }
.file-tools { display:flex; align-items:center; gap:10px; }.clear-list{border:0;background:none;color:var(--error);font-size:11px;cursor:pointer}.clear-list:hover{text-decoration:underline}
.section-title { font-size: 14px; font-weight: 600; color: var(--text-1); }
.file-count { font-size: 11px; color: var(--text-3); font-family: var(--font-mono); }
.empty-hint { font-size: 12px; color: var(--text-3); padding: var(--s-4); text-align: center; background: var(--surface); border: 1px dashed var(--border); border-radius: var(--r-md); }

.file-grid { display: flex; flex-direction: column; gap: 4px; }
.file-card {
  display: flex; align-items: center;
  gap: var(--s-3);
  padding: var(--s-3) var(--s-4);
  background: var(--surface);
  border: 1px solid var(--border-soft);
  border-radius: var(--r-md);
  transition: border-color 0.12s;
}
.file-card:hover { border-color: var(--border); }
.file-card.error { border-color: var(--error-bg); background: var(--error-bg); }
.file-type {
  flex-shrink: 0;
  width: 36px; height: 36px;
  display: flex; align-items: center; justify-content: center;
  border-radius: var(--r-sm);
}
.file-type svg { width: 18px; height: 18px; }
.file-info { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 2px; }
.file-name { font-size: 13px; font-weight: 500; color: var(--text-1); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.file-meta { font-size: 11px; color: var(--text-3); font-family: var(--font-mono); }
.file-status { flex-shrink: 0; display: flex; align-items: center; gap: var(--s-2); }
.status-badge { font-size: 10px; font-weight: 600; padding: 2px 8px; border-radius: 100px; }
.status-badge.ready { background: var(--surface-2); color: var(--text-3); }
.status-badge.done { background: var(--success-bg); color: var(--success); }
.status-badge.processing { background: var(--info-bg); color: var(--info); }
.status-badge.error { background: var(--error-bg); color: var(--error); }
.file-action {
  width: 26px; height: 26px;
  display: flex; align-items: center; justify-content: center;
  border: none; background: none; border-radius: var(--r-sm);
  cursor: pointer; color: var(--text-3);
  transition: all 0.12s;
}
.file-action:hover { background: var(--error-bg); color: var(--error); }
.file-action svg { width: 13px; height: 13px; }
</style>
