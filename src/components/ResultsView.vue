<script setup lang="ts">
import { computed, ref } from 'vue';
import { projectStore, formatSize } from '../stores/project';
import { api, type ArtifactItem } from '../services/tauri';
import { notify } from '../stores/ui';
import {
  parseGlossaryToAnkiCards,
  generateAnkiDeck,
  formatAsObsidianCornell,
  annotatePinyinRuby,
  extractBilingualParallelText,
  extractCornellCuesAndSummary,
  downloadFile
} from '../services/heritageScaffolding';

const formatFilter = ref('all');
const sourceFilter = ref('all');
const query = ref('');
const preview = ref<{ name: string; content: string; path: string; unsupported: boolean } | null>(null);
const previewMode = ref<'normal' | 'ruby' | 'parallel' | 'cues'>('normal');
const inDocSearch = ref('');

const artifacts = computed(() => projectStore.artifacts);
const sources = computed(() => projectStore.project?.sources ?? []);
const formats = computed(() => [...new Set(artifacts.value.map(a => a.ext).filter(Boolean))].sort());
const sourceName = (a: ArtifactItem) => sources.value.find(s => s.id === a.source_id)?.name || (a.source_id ? '已移除来源' : '历史产物');
const sourceOptions = computed(() => {
  const ids = [...new Set(artifacts.value.map(a => a.source_id || '__legacy'))];
  return ids.map(id => ({ id, name: id === '__legacy' ? '历史产物' : sourceName(artifacts.value.find(a => (a.source_id || '__legacy') === id)!) }));
});

const rows = computed(() => artifacts.value.filter(a => {
  const matchFormat = formatFilter.value === 'all' || a.ext === formatFilter.value;
  const matchSource = sourceFilter.value === 'all' || (a.source_id || '__legacy') === sourceFilter.value;
  const q = query.value.trim().toLowerCase();
  return matchFormat && matchSource && (!q || `${a.name} ${sourceName(a)}`.toLowerCase().includes(q));
}).sort((a,b) => b.modified.localeCompare(a.modified)));

async function open(a:ArtifactItem){try{await api.openFile(a.path)}catch(e){notify(`打开失败：${e}`,'error')}}
async function openPreview(a:ArtifactItem){
  const textTypes=['txt','md','markdown','srt','lrc','json','html'];
  if(!textTypes.includes(a.ext)){preview.value={name:a.name,path:a.path,content:'此格式暂不在应用内渲染。可使用“打开”交给系统默认程序查看。',unsupported:true};return;}
  try{
    const content = await api.readTextFile(a.path);
    preview.value={ name:a.name, path: a.path, content, unsupported:false };
    previewMode.value = 'normal';
    inDocSearch.value = '';
  }catch(e){notify(`预览失败：${e}`,'error')}
}
async function reveal(a:ArtifactItem){try{await api.showInFolder(a.path)}catch(e){notify(`定位失败：${e}`,'error')}}

async function exportOne(a:ArtifactItem){
  if(!projectStore.project)return;
  const dir=await api.pickExportDir();
  if(!dir)return;
  try{
    const r=await api.exportArtifacts(projectStore.project.id,dir,[a.name]);
    notify(r.copied?`已导出 ${a.name}`:'导出失败','success');
  }catch(e){notify(`导出失败：${e}`,'error')}
}

async function exportAll(){
  if(!projectStore.project)return;
  const dir=await api.pickExportDir();
  if(!dir)return;
  try{
    const r=await api.exportArtifacts(projectStore.project.id,dir,rows.value.map(a=>a.name));
    notify(`已导出 ${r.copied} 个文件`,'success');
  }catch(e){notify(`导出失败：${e}`,'error')}
}

// 快速表格动作：无需弹窗直接复制纯文本
async function quickCopy(a: ArtifactItem) {
  try {
    const text = await api.readTextFile(a.path);
    await navigator.clipboard.writeText(text);
    notify(`已直接复制 ${a.name} 到剪贴板`, 'success');
  } catch (e) {
    notify(`复制失败: ${e}`, 'error');
  }
}

// 快速表格动作：直接提取 Anki 闪卡
async function quickAnki(a: ArtifactItem) {
  try {
    const text = await api.readTextFile(a.path);
    const cards = parseGlossaryToAnkiCards(text);
    if (cards.length === 0) {
      notify('未检测到生词表或术语，建议在转换时勾选「Anki 闪卡提取」预设。', 'info');
      return;
    }
    const tsv = generateAnkiDeck(cards, 'tsv');
    const filename = `${a.name.replace(/\.[^/.]+$/, '')}_Anki.tsv`;
    downloadFile(filename, tsv, 'text/tab-separated-values;charset=utf-8');
    notify(`已为 ${a.name} 提取并导出 ${cards.length} 张 Anki 闪卡！`, 'success');
  } catch (e) {
    notify(`导出失败: ${e}`, 'error');
  }
}

// 华裔语言脚手架快捷功能
function exportAnkiDeckFromCurrent(format: 'tsv' | 'csv' = 'tsv') {
  if (!preview.value || preview.value.unsupported) return;
  const cards = parseGlossaryToAnkiCards(preview.value.content);
  if (cards.length === 0) {
    notify('未检测到可提取的成语或生词表，建议在转写时使用「Anki 闪卡提取」模板。', 'info');
    return;
  }
  const data = generateAnkiDeck(cards, format);
  const ext = format === 'csv' ? 'csv' : 'tsv';
  const mime = format === 'csv' ? 'text/csv;charset=utf-8' : 'text/tab-separated-values;charset=utf-8';
  const filename = `${preview.value.name.replace(/\.[^/.]+$/, '')}_Anki_Deck.${ext}`;
  downloadFile(filename, data, mime);
  notify(`已生成 ${cards.length} 张 Anki 闪卡（${format.toUpperCase()} 格式）并开始下载！`, 'success');
}

function exportObsidianCornell() {
  if (!preview.value || preview.value.unsupported) return;
  const cornell = formatAsObsidianCornell(preview.value.name, preview.value.content);
  const filename = `${preview.value.name.replace(/\.[^/.]+$/, '')}_Obsidian_Cornell.md`;
  downloadFile(filename, cornell, 'text/markdown;charset=utf-8');
  notify('已导出带 Callout 的 Obsidian 康奈尔学术双语笔记！', 'success');
}

async function copyCleanParallel() {
  if (!preview.value) return;
  try {
    const cleanText = extractBilingualParallelText(preview.value.content);
    await navigator.clipboard.writeText(cleanText);
    notify('已复制干净的双语对齐文本，可直接粘贴进论文或课堂作业！', 'success');
  } catch (e) {
    notify(`复制失败: ${e}`, 'error');
  }
}

async function copyObsidianCallout() {
  if (!preview.value) return;
  try {
    const cornell = formatAsObsidianCornell(preview.value.name, preview.value.content);
    await navigator.clipboard.writeText(cornell);
    notify('已复制标准 Obsidian 康奈尔双链格式笔记！', 'success');
  } catch (e) {
    notify(`复制失败: ${e}`, 'error');
  }
}

async function copyContent() {
  if (!preview.value) return;
  try {
    let textToCopy = preview.value.content;
    if (previewMode.value === 'ruby') {
      textToCopy = annotatePinyinRuby(preview.value.content);
    } else if (previewMode.value === 'parallel') {
      textToCopy = extractBilingualParallelText(preview.value.content);
    } else if (previewMode.value === 'cues') {
      textToCopy = extractCornellCuesAndSummary(preview.value.content);
    }
    await navigator.clipboard.writeText(textToCopy);
    notify('已成功复制到剪贴板！', 'success');
  } catch (e) {
    notify(`复制失败: ${e}`, 'error');
  }
}

// 预览文本与搜索高亮计算
const searchMatchCount = computed(() => {
  if (!preview.value || !inDocSearch.value.trim()) return 0;
  const q = inDocSearch.value.trim();
  const reg = new RegExp(q.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), 'gi');
  const matches = preview.value.content.match(reg);
  return matches ? matches.length : 0;
});

const activeProcessedContent = computed(() => {
  if (!preview.value) return '';
  if (previewMode.value === 'ruby') {
    return annotatePinyinRuby(preview.value.content);
  }
  if (previewMode.value === 'parallel') {
    return extractBilingualParallelText(preview.value.content);
  }
  if (previewMode.value === 'cues') {
    return extractCornellCuesAndSummary(preview.value.content);
  }
  return preview.value.content;
});

const highlightedHtml = computed(() => {
  let text = activeProcessedContent.value;
  if (!inDocSearch.value.trim()) return null;
  const q = inDocSearch.value.trim();
  const escaped = q.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const reg = new RegExp(`(${escaped})`, 'gi');
  // 简单转义防止 html 注入，若为 ruby 模式则保留 ruby 标签
  if (previewMode.value !== 'ruby') {
    text = text.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
  }
  return text.replace(reg, '<mark class="doc-match">$1</mark>');
});
</script>

<template>
  <div class="results-view">
    <header>
      <div>
        <span class="eyebrow">HERITAGE ARCHIVE & KNOWLEDGE VAULT</span>
        <h1>知识成果与数字档案</h1>
        <p>专为华裔学人与家庭口述史定制：Obsidian 康奈尔双语笔记、双语平行实录与 Anki 闪卡包。</p>
      </div>
      <button class="export-all" :disabled="!rows.length" @click="exportAll">
        批量导出当前结果（{{ rows.length }}）
      </button>
    </header>

    <section class="filters">
      <input v-model="query" placeholder="搜索学术讲座、长辈口述或文件名" />
      <select v-model="sourceFilter">
        <option value="all">全部素材来源</option>
        <option v-for="s in sourceOptions" :key="s.id" :value="s.id">{{ s.name }}</option>
      </select>
      <select v-model="formatFilter">
        <option value="all">全部文件格式</option>
        <option v-for="f in formats" :key="f" :value="f">{{ f.toUpperCase() }}</option>
      </select>
      <button v-if="query||sourceFilter!=='all'||formatFilter!=='all'" @click="query='';sourceFilter='all';formatFilter='all'">重置筛选</button>
    </section>

    <section class="table-wrap">
      <table v-if="rows.length">
        <thead>
          <tr>
            <th>成果文件</th>
            <th>素材来源</th>
            <th>格式</th>
            <th>大小</th>
            <th>加工时间</th>
            <th>快捷操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="a in rows" :key="a.path">
            <td class="file"><b>{{ a.name }}</b></td>
            <td class="source">{{ sourceName(a) }}</td>
            <td><span class="type">{{ a.ext.toUpperCase() }}</span></td>
            <td>{{ formatSize(a.size) }}</td>
            <td>{{ a.modified || '—' }}</td>
            <td class="actions">
              <button class="primary-act" @click="openPreview(a)">预览 & 脚手架</button>
              <button v-if="['md','txt','srt'].includes(a.ext)" @click="quickCopy(a)" title="直接复制到剪贴板">快速复制</button>
              <button v-if="a.ext==='md'" @click="quickAnki(a)" title="一键提取 Anki 闪卡 TSV">闪卡</button>
              <button @click="open(a)">打开</button>
              <button @click="reveal(a)">定位</button>
              <button @click="exportOne(a)">导出</button>
            </td>
          </tr>
        </tbody>
      </table>
      <div v-else class="empty">{{ artifacts.length ? '没有符合当前筛选条件的结果。' : '完成转写或整理后，双语笔记与产物会在这里列出。' }}</div>
    </section>

    <!-- 预览与华裔学术脚手架弹窗 -->
    <div v-if="preview" class="preview-mask" @click.self="preview=null">
      <section class="preview-dialog">
        <header>
          <div class="dialog-title">
            <b>{{ preview.name }}</b>
            <span class="preview-tag">Heritage Scaffolding v2.1</span>
          </div>

          <!-- 文内快速搜索 -->
          <div v-if="!preview.unsupported" class="doc-search-box">
            <input v-model="inDocSearch" placeholder="搜索讲座/访谈关键词..." />
            <span v-if="inDocSearch" class="match-badge">{{ searchMatchCount }} 处</span>
            <button v-if="inDocSearch" class="clear-search" @click="inDocSearch=''">×</button>
          </div>

          <!-- 模式切换 -->
          <div v-if="!preview.unsupported && (preview.name.endsWith('.md') || preview.name.endsWith('.txt'))" class="mode-toggles">
            <button
              class="mode-btn"
              :class="{ active: previewMode === 'normal' }"
              @click="previewMode = 'normal'"
            >
              标准正文
            </button>
            <button
              class="mode-btn"
              :class="{ active: previewMode === 'ruby' }"
              @click="previewMode = 'ruby'"
              title="80+ 高频华裔历史文化词汇拼音注音"
            >
              🀄 拼音注音
            </button>
            <button
              class="mode-btn"
              :class="{ active: previewMode === 'parallel' }"
              @click="previewMode = 'parallel'"
              title="纯净中英双语对齐分轨"
            >
              🌐 双语对齐
            </button>
            <button
              v-if="preview.name.endsWith('.md')"
              class="mode-btn"
              :class="{ active: previewMode === 'cues' }"
              @click="previewMode = 'cues'"
              title="Cornell 核心探讨问题与生词表"
            >
              🎯 考前要点
            </button>
          </div>

          <button class="icon-close" @click="preview=null">×</button>
        </header>

        <div class="preview-body">
          <div v-if="highlightedHtml" class="content-view highlighted-view" v-html="highlightedHtml"></div>
          <div v-else-if="previewMode === 'ruby'" class="content-view ruby-rendered" v-html="activeProcessedContent"></div>
          <pre v-else class="content-view">{{ activeProcessedContent }}</pre>
        </div>

        <footer v-if="preview.unsupported">请使用列表中的“打开”按钮查看该文件。</footer>
        <footer v-else class="preview-foot">
          <div class="foot-actions">
            <button
              v-if="preview.name.endsWith('.md')"
              class="scaffold-btn"
              @click="exportAnkiDeckFromCurrent('tsv')"
              title="下载 Anki 原生 TSV 闪卡卡片包"
            >
              📇 Anki 闪卡 (TSV)
            </button>
            <button
              v-if="preview.name.endsWith('.md')"
              class="scaffold-btn"
              @click="exportAnkiDeckFromCurrent('csv')"
              title="下载 CSV 兼容格式"
            >
              📇 Anki 闪卡 (CSV)
            </button>
            <button
              v-if="preview.name.endsWith('.md')"
              class="scaffold-btn"
              @click="exportObsidianCornell"
              title="导出带 Callout 与 YAML 元数据的 Obsidian 笔记"
            >
              📚 导出 Obsidian 笔记
            </button>
            <button
              class="scaffold-btn"
              @click="copyCleanParallel"
              title="复制干净的中英双语对齐纯文本"
            >
              📑 复制双语纯文本
            </button>
            <button
              v-if="preview.name.endsWith('.md')"
              class="scaffold-btn"
              @click="copyObsidianCallout"
              title="复制带 Obsidian Callout 格式全文"
            >
              📋 复制 Obsidian 格式
            </button>
            <button class="scaffold-btn" @click="copyContent" title="复制当前视图内容">
              复制当前视图
            </button>
          </div>
          <span class="foot-tip">💡 提示：按「拼音注音」可查看 80+ 华裔文史词汇拼音；按「双语对齐」可提取中英并列段落；「闪卡」可直接导入 Anki。</span>
        </footer>
      </section>
    </div>
  </div>
</template>

<style scoped>
.results-view{max-width:1180px;margin:auto;display:grid;gap:16px}
.results-view header{display:flex;align-items:end;justify-content:space-between;gap:16px;padding:4px 2px}
.eyebrow{font:700 10px var(--font-mono);letter-spacing:.14em;color:var(--accent)}
h1{margin:4px 0;font:700 24px var(--font-display)}
header p{font-size:12px;color:var(--text-3)}
.export-all{padding:9px 13px;border:0;border-radius:8px;background:var(--accent);color:#fff;font-weight:700;cursor:pointer;white-space:nowrap}
.export-all:disabled{opacity:.45}

.filters{display:grid;grid-template-columns:minmax(180px,1fr) minmax(150px,.55fr) 130px auto;gap:8px;padding:12px;border:1px solid var(--border);border-radius:10px;background:var(--surface)}
.filters input,.filters select,.filters button{padding:8px 10px;border:1px solid var(--border);border-radius:7px;background:var(--bg);color:var(--text-2);font-size:12px}
.filters button{cursor:pointer}

.table-wrap{border:1px solid var(--border);border-radius:11px;background:var(--surface);overflow:auto}
table{width:100%;border-collapse:collapse;font-size:12px;min-width:780px}
th{text-align:left;padding:10px 12px;color:var(--text-3);font-size:10px;letter-spacing:.07em;text-transform:uppercase;background:var(--bg)}
td{padding:11px 12px;border-top:1px solid var(--border-soft);color:var(--text-2)}
.file{color:var(--text-1);max-width:280px}
.file b,.source{display:block;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.source{max-width:200px}
.type{display:inline-block;padding:2px 6px;border-radius:4px;background:var(--accent-bg);color:var(--accent);font:700 10px var(--font-mono)}
.actions{white-space:nowrap}
.actions button{border:0;background:none;color:var(--text-2);cursor:pointer;font-size:12px;margin-right:8px}
.actions button:hover { color: var(--accent); }
.actions button.primary-act { color: var(--accent); font-weight: 600; }
.empty{padding:48px 20px;text-align:center;color:var(--text-3);font-size:12px}

.preview-mask{position:fixed;inset:0;z-index:500;display:grid;place-items:center;padding:24px;background:oklch(15% 0.02 55 /.46);backdrop-filter:blur(5px)}
.preview-dialog{width:min(960px,100%);max-height:85vh;display:grid;grid-template-rows:auto 1fr auto;border:1px solid var(--border);border-radius:12px;background:var(--surface);overflow:hidden;box-shadow:0 24px 70px #0003}
.preview-dialog header{display:flex;align-items:center;justify-content:space-between;gap:12px;padding:12px 16px;border-bottom:1px solid var(--border)}
.dialog-title { display: flex; align-items: center; gap: 8px; }
.dialog-title b{font-size:13px}
.preview-tag { font-size: 10px; padding: 2px 6px; border-radius: 4px; background: rgba(13, 148, 136, 0.12); color: #0d9488; font-weight: 600; }

.doc-search-box {
  display: flex;
  align-items: center;
  gap: 6px;
  background: var(--bg);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 3px 8px;
}
.doc-search-box input {
  border: 0;
  outline: none;
  background: none;
  font-size: 11px;
  color: var(--text-1);
  width: 130px;
}
.match-badge {
  font-size: 10px;
  font-family: var(--font-mono);
  background: rgba(234, 179, 8, 0.2);
  color: #b45309;
  padding: 1px 5px;
  border-radius: 3px;
  font-weight: 600;
}
.clear-search {
  border: 0;
  background: none;
  cursor: pointer;
  color: var(--text-3);
  font-size: 14px;
  padding: 0 2px;
}

.mode-toggles {
  display: flex;
  align-items: center;
  gap: 4px;
  background: var(--bg);
  padding: 3px;
  border-radius: 8px;
  border: 1px solid var(--border);
}
.mode-btn {
  padding: 4px 8px;
  border: 0;
  border-radius: 5px;
  background: transparent;
  color: var(--text-3);
  font-size: 11px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
}
.mode-btn:hover {
  color: var(--text-1);
}
.mode-btn.active {
  background: var(--surface);
  color: var(--accent);
  box-shadow: 0 1px 3px rgba(0,0,0,0.08);
  font-weight: 600;
}

.dialog-actions { display: flex; align-items: center; gap: 8px; }
.scaffold-btn {
  padding: 5px 9px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--bg);
  color: var(--text-2);
  font-size: 11px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
}
.scaffold-btn:hover {
  border-color: var(--accent);
  color: var(--accent);
}
.scaffold-btn.active {
  background: var(--accent);
  border-color: var(--accent);
  color: #fff;
}
.icon-close {
  border: 0; background: none; font-size: 22px; cursor: pointer; color: var(--text-3); margin-left: 6px;
}

.preview-body { overflow: auto; max-height: calc(85vh - 140px); background: var(--bg); }
.content-view { padding: 18px; margin: 0; white-space: pre-wrap; font: 12px/1.8 var(--font-mono); color: var(--text-2); }
.ruby-rendered {
  font: 14px/2.3 var(--font-body);
  color: var(--text-1);
}
:deep(ruby) {
  ruby-position: over;
  padding: 0 2px;
}
:deep(rt) {
  font-size: 10px;
  color: #0d9488;
  font-family: var(--font-mono);
  font-weight: 600;
}
:deep(mark.doc-match) {
  background-color: #fef08a;
  color: #854d0e;
  padding: 0 2px;
  border-radius: 2px;
  font-weight: 700;
}

.preview-foot{
  padding: 10px 16px;
  background: var(--surface);
  border-top: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.foot-actions {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 8px;
}
.foot-tip {
  color: var(--text-3);
  font-size: 11px;
}

@media(max-width:860px){
  .results-view header{align-items:start;flex-direction:column}
  .filters{grid-template-columns:1fr 1fr}
  .filters input{grid-column:1/-1}
  .preview-dialog header { flex-wrap: wrap; gap: 8px; }
  .mode-toggles { order: 3; width: 100%; justify-content: space-around; }
}
</style>
