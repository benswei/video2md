<script setup lang="ts">
import { ref } from 'vue';
import { projectStore, openProject, createProject, renameProject } from '../stores/project';
import { notify } from '../stores/ui';

defineProps<{ open: boolean }>();
const emit = defineEmits<{ close: []; 'open-help': []; 'open-changelog': [] }>();

const creating = ref(false);
const newName = ref('');

async function onOpen(id: string) {
  try {
    await openProject(id);
    emit('close');
  } catch (error) {
    notify(`无法打开项目：${error}`, 'error');
  }
}

async function onCreate() {
  const name = newName.value.trim();
  if (!name) {
    notify('请先输入项目名称', 'info');
    return;
  }
  try {
    const p = await createProject(name);
    newName.value = '';
    creating.value = false;
    await openProject(p.id);
    emit('close');
    notify(`已创建「${p.name}」`, 'success');
  } catch (error) {
    notify(`创建失败：${error}`, 'error');
  }
}

async function onRename() {
  const project = projectStore.project;
  if (!project) return;
  const name = window.prompt('项目名称', project.name)?.trim();
  if (!name || name === project.name) return;
  try { await renameProject(project.id, name); notify(`项目已重命名为「${name}」`, 'success'); }
  catch (error) { notify(`重命名失败：${error}`, 'error'); }
}
</script>

<template>
  <!-- Overlay for mobile -->
  <div v-if="open" class="overlay" @click="$emit('close')" />

  <aside class="sidebar" :class="{ open }">
    <div class="sidebar-header">
      <span class="section-label">项目列表</span>
      <div class="header-actions"><button v-if="projectStore.project" class="add-btn" title="重命名当前项目" aria-label="重命名当前项目" @click="onRename">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 20h9"/><path d="M16.5 3.5a2.12 2.12 0 013 3L7 19l-4 1 1-4z"/></svg>
      </button><button class="add-btn" title="新建项目" aria-label="新建项目" :aria-expanded="creating" @click="creating = !creating">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
      </button></div>
    </div>

    <div v-if="creating" class="create-box">
      <input v-model="newName" placeholder="新项目名称" @keyup.enter="onCreate" />
      <button class="create-confirm" @click="onCreate">创建</button>
    </div>

    <div class="project-list">
      <button
        v-for="p in projectStore.projects"
        :key="p.id"
        class="project-item"
        :class="{ active: p.id === projectStore.project?.id }"
        @click="onOpen(p.id)"
      >
        <span class="project-icon">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z"/></svg>
        </span>
        <span class="project-title">{{ p.name }}</span>
        <span class="project-badge">{{ p.artifact_count }}</span>
      </button>
      <div v-if="projectStore.projects.length === 0" class="empty-hint">暂无项目，点上方 + 新建。</div>
    </div>

    <div class="sidebar-divider" />

    <div class="sidebar-section">
      <span class="section-label">快捷操作</span>
      <button class="nav-item" @click="$emit('open-help')">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>
        帮助文档
      </button>
      <button class="nav-item" @click="$emit('open-changelog')">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>
        更新日志
      </button>
    </div>

    <div class="sidebar-footer">
      <div class="storage-info">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"/><path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"/></svg>
        <span v-if="projectStore.deps">Python {{ projectStore.deps.python ? '✓' : '✗' }} · FFmpeg {{ projectStore.deps.ffmpeg ? '✓' : '✗' }} · B站 {{ projectStore.deps.bilibili ? '✓' : '—' }} · YouTube {{ projectStore.deps.youtube ? '✓' : '—' }}</span>
        <span v-else>依赖检测中…</span>
      </div>
    </div>
  </aside>
</template>

<style scoped>
.overlay {
  display: none;
  position: fixed; inset: 0;
  background: oklch(0% 0 0 / 0.25);
  z-index: 90;
}

.sidebar {
  display: flex; flex-direction: column;
  background: var(--surface);
  border-right: 1px solid var(--border);
  overflow-y: auto;
  height: 100%;
  min-width: 0;
}

.sidebar-header {
  display: flex; align-items: center; justify-content: space-between;
  padding: var(--s-3) var(--s-4);
}
.header-actions{display:flex;gap:5px}
.add-btn {
  width: 26px; height: 26px;
  display: flex; align-items: center; justify-content: center;
  border: 1px solid var(--border); border-radius: var(--r-sm);
  background: var(--surface); cursor: pointer;
  color: var(--text-3); transition: all 0.15s;
}
.add-btn:hover { border-color: var(--accent); color: var(--accent); background: var(--accent-bg); }
.add-btn svg { width: 14px; height: 14px; }

.create-box {
  display: flex; gap: var(--s-1);
  padding: 0 var(--s-2) var(--s-2);
}
.create-box input {
  flex: 1; min-width: 0;
  font-family: var(--font-body); font-size: 12px; color: var(--text-1);
  padding: 5px 8px; border: 1px solid var(--border); border-radius: var(--r-sm);
  background: var(--bg); outline: none;
}
.create-box input:focus { border-color: var(--accent); }
.create-confirm {
  padding: 5px 10px; border: none; border-radius: var(--r-sm);
  background: var(--accent); color: #fff; font-size: 12px; cursor: pointer;
  font-family: var(--font-body); white-space: nowrap;
}

.section-label {
  font-size: 10px; font-weight: 600;
  text-transform: uppercase; letter-spacing: 0.08em;
  color: var(--text-3);
}

.project-list {
  display: flex; flex-direction: column;
  padding: 0 var(--s-2);
  gap: 1px;
}
.empty-hint { font-size: 11px; color: var(--text-3); padding: var(--s-3) var(--s-2); }

.project-item {
  display: flex; align-items: center; gap: var(--s-2);
  padding: var(--s-2) var(--s-2);
  border: none; background: none; border-radius: var(--r-sm);
  cursor: pointer; text-align: left;
  transition: background 0.12s;
  font-family: var(--font-body);
  width: 100%;
}
.project-item:hover { background: var(--surface-2); }
.project-item.active { background: var(--accent-bg); }
.project-item.active .project-icon { color: var(--accent); }
.project-item.active .project-title { color: var(--text-1); font-weight: 500; }

.project-icon {
  flex-shrink: 0; width: 28px; height: 28px;
  display: flex; align-items: center; justify-content: center;
  border-radius: var(--r-sm);
  color: var(--text-3); background: var(--surface-2);
}
.project-icon svg { width: 15px; height: 15px; }

.project-title {
  flex: 1; min-width: 0;
  font-size: 12px; line-height: 1.4;
  color: var(--text-2);
  overflow: hidden; text-overflow: ellipsis;
  display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical;
}

.project-badge {
  flex-shrink: 0;
  font-size: 10px; font-weight: 600;
  padding: 1px 6px; border-radius: 100px;
  background: var(--surface-2); color: var(--text-3);
  font-family: var(--font-mono);
}

.sidebar-divider {
  height: 1px;
  background: var(--border-soft);
  margin: var(--s-2) var(--s-4);
}

.sidebar-section {
  display: flex; flex-direction: column;
  padding: 0 var(--s-4);
  gap: 2px;
}

.nav-item {
  display: flex; align-items: center; gap: var(--s-2);
  padding: var(--s-2) var(--s-2);
  border: none; background: none; border-radius: var(--r-sm);
  cursor: pointer;
  font-size: 12px; color: var(--text-3);
  font-family: var(--font-body);
  transition: all 0.12s;
}
.nav-item:hover { background: var(--surface-2); color: var(--text-2); }
.nav-item svg { width: 15px; height: 15px; flex-shrink: 0; }

.sidebar-footer {
  margin-top: auto;
  padding: var(--s-3) var(--s-4);
}

.storage-info {
  display: flex; align-items: center; gap: var(--s-2);
  font-size: 11px; color: var(--text-3);
  font-family: var(--font-mono);
}
.storage-info svg { width: 14px; height: 14px; flex-shrink: 0; }

@media (max-width: 900px) {
  .sidebar {
    position: fixed; left: 0; top: var(--topbar-h);
    width: var(--sidebar-w);
    height: calc(100vh - var(--topbar-h));
    transform: translateX(-100%);
    transition: transform 0.2s ease;
    z-index: 95;
    box-shadow: var(--shadow-lg, 0 8px 24px rgba(0,0,0,0.08));
  }
  .sidebar.open { transform: translateX(0); }
  .overlay { display: block; }
}
</style>
