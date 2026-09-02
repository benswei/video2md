<script setup lang="ts">
defineProps<{ open: boolean }>();
defineEmits<{ close: [] }>();

import { computed } from 'vue';
import { projectStore, engineLabel } from '../stores/project';
import { taskStore } from '../stores/task';

const project = computed(() => projectStore.project);
const sources = computed(() => project.value?.sources.length ?? 0);
const artifacts = computed(() => projectStore.artifacts.length);
const engine = computed(() => (project.value ? engineLabel(project.value.config.engine) : '—'));
const activeTask = computed(() => taskStore.tasks.find(t => ['downloading', 'transcribing', 'organizing'].includes(t.status)) || taskStore.tasks.find(t => t.status === 'queued'));
const failedTask = computed(() => taskStore.tasks.find(t => t.status === 'error'));
</script>

<template>
  <div v-if="open" class="overlay" @click="$emit('close')" />

  <aside class="detail-panel" :class="{ open }">
    <div class="panel-header">
      <span class="panel-title">任务详情</span>
      <button class="close-btn" @click="$emit('close')">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
      </button>
    </div>

    <div class="panel-body">
      <div v-if="!project" class="empty-state">
        <div class="empty-icon">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="2" y="2" width="20" height="20" rx="2"/><line x1="8" y1="10" x2="16" y2="10"/><line x1="8" y1="14" x2="12" y2="14"/></svg>
        </div>
        <p class="empty-title">暂无选中项目</p>
        <p class="empty-desc">在「项目」页创建或<br/>选择一个知识库</p>
      </div>

      <template v-else>
        <div class="proj-head">
          <span class="panel-section-label">当前项目</span>
          <p class="proj-name">{{ project?.name }}</p>
        </div>

        <div v-if="activeTask" class="focus-card">
          <span class="panel-section-label">正在关注</span>
          <b>{{ activeTask.source_name }}</b>
          <p>{{ activeTask.status === 'queued' ? '等待队列执行' : activeTask.message || '正在处理' }}</p>
          <div class="mini-progress" v-if="activeTask.progress"><i :style="{width: `${activeTask.progress}%`}"/></div>
        </div>
        <div v-else class="focus-card quiet"><span class="panel-section-label">队列状态</span><b>没有进行中的任务</b><p>在转换工作台添加素材并加入队列。</p></div>

        <!-- Quick stats -->
        <div class="stats-group">
          <span class="panel-section-label">当前项目统计</span>
          <div class="stat-row">
            <span class="stat-label">素材文件</span>
            <span class="stat-value">{{ sources }}</span>
          </div>
          <div class="stat-row">
            <span class="stat-label">已生成产物</span>
            <span class="stat-value">{{ artifacts }}</span>
          </div>
          <div class="stat-row">
            <span class="stat-label">当前引擎</span>
            <span class="stat-value accent">{{ engine }}</span>
          </div>
          <div class="stat-row">
            <span class="stat-label">更新时间</span>
            <span class="stat-value">{{ project?.updated_at }}</span>
          </div>
        </div>
        <div v-if="failedTask" class="error-card"><span class="panel-section-label">最近失败</span><b>{{ failedTask.source_name }}</b><p>{{ failedTask.error || failedTask.message }}</p></div>
        <details v-if="taskStore.logs.length" class="diag"><summary>诊断日志（{{ taskStore.logs.length }}）</summary><pre>{{ taskStore.logs.slice(-12).join('\n') }}</pre></details>
      </template>
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
.detail-panel {
  display: flex; flex-direction: column;
  background: var(--surface);
  border-left: 1px solid var(--border);
  height: 100%;
  overflow-y: auto;
  min-width: 0;
}
.panel-header { display: flex; align-items: center; justify-content: space-between; padding: var(--s-3) var(--s-4); border-bottom: 1px solid var(--border-soft); flex-shrink: 0; }
.panel-title { font-size: 12px; font-weight: 600; color: var(--text-2); }
.close-btn { width: 26px; height: 26px; display: flex; align-items: center; justify-content: center; border: none; background: none; border-radius: var(--r-sm); cursor: pointer; color: var(--text-3); transition: all 0.12s; }
.close-btn:hover { background: var(--surface-2); color: var(--text-1); }
.close-btn svg { width: 14px; height: 14px; }

.panel-body { flex: 1; padding: var(--s-4); display: flex; flex-direction: column; gap: var(--s-5); }
.empty-state { display: flex; flex-direction: column; align-items: center; padding: var(--s-10) var(--s-4); text-align: center; }
.empty-icon { width: 52px; height: 52px; display: flex; align-items: center; justify-content: center; border-radius: var(--r-lg); background: var(--surface-2); color: var(--text-3); margin-bottom: var(--s-3); }
.empty-icon svg { width: 24px; height: 24px; }
.empty-title { font-size: 13px; font-weight: 500; color: var(--text-2); }
.empty-desc { font-size: 11px; color: var(--text-3); margin-top: var(--s-1); line-height: 1.6; }

.proj-head { display: flex; flex-direction: column; gap: 4px; }
.proj-name { font-size: 14px; font-weight: 600; color: var(--text-1); }
.focus-card,.error-card{display:grid;gap:5px;padding:var(--s-3);border:1px solid color-mix(in oklch,var(--accent) 25%,var(--border));border-radius:var(--r-md);background:var(--accent-bg)}.focus-card b,.error-card b{font-size:12px}.focus-card p,.error-card p{font-size:11px;color:var(--text-3);line-height:1.55}.focus-card.quiet{background:var(--bg);border-color:var(--border)}.mini-progress{height:4px;overflow:hidden;border-radius:4px;background:var(--surface)}.mini-progress i{display:block;height:100%;background:var(--accent)}.error-card{border-color:color-mix(in oklch,var(--error) 30%,var(--border));background:var(--error-bg)}.error-card p{color:var(--error)}.diag{font-size:11px;color:var(--text-3)}.diag summary{cursor:pointer}.diag pre{max-height:150px;overflow:auto;margin-top:8px;padding:8px;border-radius:6px;background:var(--neutral-10);color:var(--neutral-3);font:10px/1.6 var(--font-mono);white-space:pre-wrap}
.stats-group { background: var(--bg); border-radius: var(--r-md); padding: var(--s-4); display: flex; flex-direction: column; gap: var(--s-3); }
.panel-section-label { font-size: 10px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.06em; color: var(--text-3); margin-bottom: var(--s-1); }
.stat-row { display: flex; align-items: center; justify-content: space-between; }
.stat-label { font-size: 12px; color: var(--text-3); }
.stat-value { font-size: 12px; font-weight: 600; color: var(--text-1); font-family: var(--font-mono); }
.stat-value.accent { color: var(--accent); }

@media (max-width: 1200px) {
  .detail-panel { position: fixed; right: 0; top: var(--topbar-h); width: var(--detail-w); height: calc(100vh - var(--topbar-h)); transform: translateX(100%); transition: transform 0.2s ease; z-index: 95; box-shadow: var(--shadow-lg, 0 8px 24px rgba(0,0,0,0.08)); }
  .detail-panel.open { transform: translateX(0); }
  .overlay { display: block; }
}
</style>
