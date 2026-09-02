<script setup lang="ts">
import { ref, onMounted } from 'vue';
import TopBar from './components/TopBar.vue';
import Sidebar from './components/Sidebar.vue';
import DetailPanel from './components/DetailPanel.vue';
import WorkspaceView from './components/WorkspaceView.vue';
import ResultsView from './components/ResultsView.vue';
import SettingsView from './components/SettingsView.vue';
import DocumentReorganizeView from './components/DocumentReorganizeView.vue';
import { bootstrap, projectStore, refreshArtifacts } from './stores/project';
import { onArtifactsUpdated } from './services/tauri';
import { initTaskListeners } from './stores/task';
import { uiStore, dismissNotice, notify } from './stores/ui';

const currentTab = ref('workspace');
const sidebarOpen = ref(false);
const detailOpen = ref(false);
const infoPanel = ref<'help' | 'changelog' | null>(null);

const tabs = [
  { id: 'workspace', label: '转换工作台', icon: 'zap' },
  { id: 'results', label: '结果', icon: 'file' },
  { id: 'documents', label: '文档重整', icon: 'file' },
  { id: 'settings', label: '设置', icon: 'gear' },
];

function switchTab(tabId: string) {
  currentTab.value = tabId;
}

function toggleSidebar() {
  sidebarOpen.value = !sidebarOpen.value;
}

onMounted(async () => {
  try {
    await initTaskListeners();
    await onArtifactsUpdated(() => refreshArtifacts());
    await bootstrap();
  } catch (error) {
    notify(`初始化失败：${error}`, 'error', 8000);
  }
});
</script>

<template>
  <div class="app">
    <TopBar :project-name="projectStore.project?.name ?? ''" @toggle-sidebar="toggleSidebar" @open-settings="switchTab('settings')" @toggle-detail="detailOpen = !detailOpen" />
    
    <div class="main-layout" :class="{ 'detail-open': detailOpen }">
      <!-- Sidebar -->
      <Sidebar :open="sidebarOpen" @close="sidebarOpen = false" @open-help="infoPanel = 'help'" @open-changelog="infoPanel = 'changelog'" />
      
      <!-- Content Area -->
      <main class="content-area">
        <!-- State Tabs -->
        <nav class="tab-bar">
          <button
            v-for="tab in tabs"
            :key="tab.id"
            class="tab-btn"
            :class="{ active: currentTab === tab.id }"
            @click="switchTab(tab.id)"
          >
            {{ tab.label }}
          </button>
        </nav>
        
        <!-- Tab Content -->
        <div class="tab-content">
          <WorkspaceView v-if="currentTab === 'workspace'" @switch-tab="switchTab" />
          <ResultsView v-if="currentTab === 'results'" />
          <DocumentReorganizeView v-if="currentTab === 'documents'" />
          <SettingsView v-if="currentTab === 'settings'" />
        </div>
      </main>
      
      <!-- Detail Panel -->
      <DetailPanel v-if="detailOpen" :open="detailOpen" @close="detailOpen = false" />
    </div>

    <div v-if="infoPanel" class="modal-backdrop" @click.self="infoPanel = null">
      <section class="info-modal" role="dialog" aria-modal="true" :aria-label="infoPanel === 'help' ? '使用帮助' : '更新日志'">
        <header class="info-head">
          <div>
            <span class="eyebrow">VIDEO2MD GUIDE</span>
            <h2>{{ infoPanel === 'help' ? '三步完成知识转写' : '版本 0.1 · MVP' }}</h2>
          </div>
          <button class="icon-close" aria-label="关闭" @click="infoPanel = null">×</button>
        </header>
        <div v-if="infoPanel === 'help'" class="guide-grid">
          <article><b>01</b><h3>导入素材</h3><p>在「准备」页选择本地音视频、文档，或添加公开视频链接。</p></article>
          <article><b>02</b><h3>配置并转换</h3><p>在「设置」中选择转写引擎；Bcut 免密钥，Gemini 与自定义服务需填写凭据。</p></article>
          <article><b>03</b><h3>预览与导出</h3><p>在「结果」页预览 Markdown、HTML、图片和 PDF，并导出到指定目录。</p></article>
          <p class="guide-note">运行前请确认侧栏底部 Python 与 ffmpeg 均为 ✓。密钥当前随项目保存在本机，请勿分享项目数据目录。</p>
        </div>
        <div v-else class="changelog">
          <span class="release-pill">当前版本</span>
          <h3>可用闭环与体验加固</h3>
          <ul><li>项目创建、素材导入、真实转写与产物预览</li><li>Gemini、Bcut 与 OpenAI 兼容转写引擎</li><li>Markdown 阅读器、批量导出与下载进度</li><li>统一错误反馈、详情面板与键盘焦点体验</li></ul>
          <p>本版本聚焦稳定的批量转写队列、来源级结果管理和常用输出格式。</p>
        </div>
      </section>
    </div>

    <div class="toast-stack" aria-live="polite" aria-atomic="false">
      <button v-for="notice in uiStore.notices" :key="notice.id" class="toast" :class="notice.kind" @click="dismissNotice(notice.id)">
        <span class="toast-dot" />{{ notice.message }}<span class="toast-x">×</span>
      </button>
    </div>
  </div>
</template>

<style>
/* Global CSS variables and base styles */
:root {
  --neutral-0:  oklch(99% 0.003 60);
  --neutral-1:  oklch(97% 0.005 60);
  --neutral-2:  oklch(94% 0.008 60);
  --neutral-3:  oklch(90% 0.010 60);
  --neutral-4:  oklch(82% 0.012 60);
  --neutral-5:  oklch(68% 0.014 60);
  --neutral-6:  oklch(52% 0.015 60);
  --neutral-7:  oklch(38% 0.018 60);
  --neutral-8:  oklch(28% 0.020 60);
  --neutral-9:  oklch(20% 0.015 60);
  --neutral-10: oklch(14% 0.012 60);

  --accent:       oklch(68% 0.16 55);
  --accent-hover: oklch(63% 0.18 55);
  --accent-soft:  oklch(92% 0.05 55);
  --accent-bg:    oklch(95% 0.03 55);

  --success:    oklch(60% 0.14 145);
  --success-bg: oklch(94% 0.04 145);
  --warning:    oklch(70% 0.14 70);
  --warning-bg: oklch(94% 0.04 70);
  --error:      oklch(55% 0.20 25);
  --error-bg:   oklch(94% 0.04 25);
  --info:       oklch(58% 0.12 240);
  --info-bg:    oklch(94% 0.03 240);
  --running:    oklch(62% 0.15 250);

  --bg:          var(--neutral-1);
  --surface:     var(--neutral-0);
  --surface-2:   var(--neutral-2);
  --border:      var(--neutral-3);
  --border-soft: var(--neutral-2);

  --text-1: var(--neutral-9);
  --text-2: var(--neutral-7);
  --text-3: var(--neutral-5);
  --text-inv: var(--neutral-0);

  --font-display: "Space Grotesk", "Noto Sans SC", system-ui, sans-serif;
  --font-body: "Noto Sans SC", -apple-system, "Segoe UI", system-ui, sans-serif;
  --font-mono: "JetBrains Mono", "Fira Code", ui-monospace, monospace;

  --s-1:  4px;  --s-2:  8px;  --s-3: 12px;
  --s-4: 16px;  --s-5: 20px;  --s-6: 24px;
  --s-8: 32px;  --s-10: 40px; --s-12: 48px;
  --s-16: 64px;

  --r-sm: 6px;  --r-md: 10px;  --r-lg: 14px;  --r-xl: 20px;

  --sidebar-w: 200px;
  --detail-w: 320px;
  --topbar-h: 52px;
  --tabbar-h: 44px;
}

* { margin: 0; padding: 0; box-sizing: border-box; }
button, input, select, textarea { font: inherit; }
button:focus-visible, input:focus-visible, select:focus-visible, textarea:focus-visible {
  outline: 3px solid color-mix(in oklch, var(--accent) 28%, transparent);
  outline-offset: 2px;
}

html, body {
  height: 100%;
  overflow: hidden;
  font-family: var(--font-body);
  font-size: 14px;
  line-height: 1.5;
  color: var(--text-1);
  background: var(--bg);
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

.app {
  display: grid;
  grid-template-rows: var(--topbar-h) 1fr;
  height: 100vh;
  min-width: 360px;
}

.main-layout {
  display: grid;
  grid-template-columns: var(--sidebar-w) minmax(0, 1fr);
  height: calc(100vh - var(--topbar-h));
  overflow: hidden;
}
.main-layout.detail-open { grid-template-columns: var(--sidebar-w) minmax(0, 1fr) var(--detail-w); }

.content-area {
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-width: 0;
  background: var(--bg);
}

.tab-bar {
  display: flex;
  align-items: center;
  gap: 0;
  padding: 0 var(--s-5);
  border-bottom: 1px solid var(--border);
  background: var(--surface);
  height: var(--tabbar-h);
  flex-shrink: 0;
  overflow-x: auto;
  -webkit-overflow-scrolling: touch;
  scrollbar-width: none;
}
.tab-bar::-webkit-scrollbar { display: none; }

.tab-btn {
  display: flex; align-items: center; gap: var(--s-2);
  padding: 0 var(--s-4);
  height: 100%;
  border: none; background: none;
  border-bottom: 2px solid transparent;
  margin-bottom: -1px;
  font-size: 13px; font-weight: 500;
  color: var(--text-3); cursor: pointer;
  transition: all 0.15s ease;
  font-family: var(--font-body);
  white-space: nowrap; flex-shrink: 0;
}
.tab-btn:hover { color: var(--text-2); }
.tab-btn.active {
  color: var(--accent);
  border-bottom-color: var(--accent);
}

.tab-content {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  padding: var(--s-5);
  min-width: 0;
}

/* Responsive */
@media (max-width: 1200px) {
  .main-layout, .main-layout.detail-open { grid-template-columns: var(--sidebar-w) minmax(0, 1fr); }
}

@media (max-width: 900px) {
  .main-layout, .main-layout.detail-open { grid-template-columns: minmax(0, 1fr); }
}

/* Scrollbar */
::-webkit-scrollbar { width: 6px; height: 6px; }
::-webkit-scrollbar-track { background: transparent; }
::-webkit-scrollbar-thumb { background: var(--neutral-3); border-radius: 3px; }
::-webkit-scrollbar-thumb:hover { background: var(--neutral-4); }

.modal-backdrop { position: fixed; inset: 0; z-index: 300; display: grid; place-items: center; padding: 24px; background: oklch(15% 0.02 55 / .46); backdrop-filter: blur(6px); }
.info-modal { width: min(760px, 100%); max-height: min(720px, 90vh); overflow: auto; padding: 28px; border: 1px solid color-mix(in oklch, var(--accent) 22%, var(--border)); border-radius: var(--r-xl); background: var(--surface); box-shadow: 0 24px 80px oklch(15% 0.02 55 / .24); }
.info-head { display: flex; justify-content: space-between; gap: 24px; align-items: start; margin-bottom: 24px; }
.info-head h2 { margin-top: 4px; font: 700 24px/1.2 var(--font-display); }
.eyebrow { color: var(--accent); font: 700 10px/1 var(--font-mono); letter-spacing: .14em; }
.icon-close { width: 34px; height: 34px; border: 1px solid var(--border); border-radius: 50%; background: var(--surface-2); color: var(--text-2); cursor: pointer; font-size: 22px; }
.guide-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 12px; }
.guide-grid article { padding: 18px; border: 1px solid var(--border); border-radius: var(--r-lg); background: linear-gradient(145deg, var(--surface), var(--surface-2)); }
.guide-grid b { color: var(--accent); font: 700 12px var(--font-mono); }
.guide-grid h3, .changelog h3 { margin: 14px 0 6px; font-size: 15px; }
.guide-grid p, .changelog p, .changelog li { color: var(--text-2); font-size: 13px; }
.guide-note { grid-column: 1 / -1; padding: 12px 14px; border-radius: var(--r-md); background: var(--warning-bg); color: var(--text-2); }
.release-pill { display: inline-flex; padding: 3px 9px; border-radius: 999px; background: var(--success-bg); color: var(--success); font-size: 11px; font-weight: 700; }
.changelog ul { margin: 14px 0 18px 20px; display: grid; gap: 7px; }
.toast-stack { position: fixed; right: 18px; bottom: 18px; z-index: 400; display: grid; gap: 8px; width: min(380px, calc(100vw - 36px)); }
.toast { display: flex; align-items: center; gap: 9px; width: 100%; padding: 11px 12px; border: 1px solid var(--border); border-radius: var(--r-md); background: var(--surface); color: var(--text-1); text-align: left; cursor: pointer; box-shadow: 0 10px 30px oklch(15% 0.02 55 / .16); animation: toast-in .2s ease-out; }
.toast-dot { width: 8px; height: 8px; border-radius: 50%; background: var(--info); flex: none; }.toast.success .toast-dot{background:var(--success)}.toast.error .toast-dot{background:var(--error)}.toast-x{margin-left:auto;color:var(--text-3)}
@keyframes toast-in { from { opacity: 0; transform: translateY(8px); } }
@media (max-width: 640px) { .guide-grid { grid-template-columns: 1fr; } .guide-note { grid-column: auto; } .info-modal { padding: 20px; } }
@media (prefers-reduced-motion: reduce) { *, *::before, *::after { scroll-behavior: auto !important; animation-duration: .01ms !important; transition-duration: .01ms !important; } }
</style>
