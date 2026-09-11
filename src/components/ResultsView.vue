<script setup lang="ts">
import { computed, ref } from 'vue';
import { projectStore, formatSize } from '../stores/project';
import { api, type ArtifactItem } from '../services/tauri';
import { notify } from '../stores/ui';
import {
  parseGlossaryToAnkiCards,
  generateAnkiCsv,
  formatAsObsidianCornell,
  annotatePinyinRuby,
  downloadFile
} from '../services/heritageScaffolding';

const formatFilter = ref('all');
const sourceFilter = ref('all');
const query = ref('');
const preview = ref<{ name: string; content: string; path: string; unsupported: boolean } | null>(null);
const previewRubyMode = ref(false);

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
    previewRubyMode.value = false;
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

// 华裔语言脚手架快捷功能
function exportAnkiDeckFromCurrent() {
  if (!preview.value || preview.value.unsupported) return;
  const cards = parseGlossaryToAnkiCards(preview.value.content);
  if (cards.length === 0) {
    notify('未检测到可提取的成语或生词表，建议在转写时使用「Anki 闪卡提取」模板。', 'info');
    return;
  }
  const csv = generateAnkiCsv(cards);
  const filename = `${preview.value.name.replace(/\.[^/.]+$/, '')}_Anki_Deck.csv`;
  downloadFile(filename, csv, 'text/tab-separated-values;charset=utf-8');
  notify(`已生成 ${cards.length} 张 Anki 闪卡并开始下载！`, 'success');
}

function exportObsidianCornell() {
  if (!preview.value || preview.value.unsupported) return;
  const cornell = formatAsObsidianCornell(preview.value.name, preview.value.content);
  const filename = `${preview.value.name.replace(/\.[^/.]+$/, '')}_Obsidian_Cornell.md`;
  downloadFile(filename, cornell, 'text/markdown;charset=utf-8');
  notify('已导出 Obsidian 康奈尔双语笔记！', 'success');
}

async function copyContent() {
  if (!preview.value) return;
  try {
    const textToCopy = previewRubyMode.value ? annotatePinyinRuby(preview.value.content) : preview.value.content;
    await navigator.clipboard.writeText(textToCopy);
    notify('已成功复制到剪贴板！可直接粘贴入 Obsidian 或文档。', 'success');
  } catch (e) {
    notify(`复制失败: ${e}`, 'error');
  }
}
</script>

<template>
  <div class="results-view">
    <header>
      <div>
        <span class="eyebrow">HERITAGE ARCHIVE & RESULTS</span>
        <h1>知识成果与数字档案</h1>
        <p>支持导出 Markdown、Obsidian 康奈尔学术双语笔记及 Anki 生词闪卡卡片包。</p>
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
            <span class="preview-tag">Heritage Preview</span>
          </div>
          <div class="dialog-actions">
            <button
              v-if="preview.name.endsWith('.md') || preview.name.endsWith('.txt')"
              class="scaffold-btn"
              :class="{ active: previewRubyMode }"
              @click="previewRubyMode = !previewRubyMode"
              title="切换拼音注音高亮"
            >
              拼音注音
            </button>
            <button
              v-if="preview.name.endsWith('.md')"
              class="scaffold-btn"
              @click="exportAnkiDeckFromCurrent"
              title="从本篇提取生词并下载 Anki 闪卡 CSV"
            >
              📇 导出 Anki 闪卡
            </button>
            <button
              v-if="preview.name.endsWith('.md')"
              class="scaffold-btn"
              @click="exportObsidianCornell"
              title="一键导出为 Obsidian 康奈尔双语笔记"
            >
              📚 导出 Obsidian 笔记
            </button>
            <button class="scaffold-btn" @click="copyContent" title="复制全文">
              复制
            </button>
            <button class="icon-close" @click="preview=null">×</button>
          </div>
        </header>

        <div class="preview-body">
          <div v-if="previewRubyMode" class="ruby-rendered" v-html="annotatePinyinRuby(preview.content)"></div>
          <pre v-else>{{ preview.content }}</pre>
        </div>

        <footer v-if="preview.unsupported">请使用列表中的“打开”按钮查看该文件。</footer>
        <footer v-else class="preview-foot">
          <span>💡 提示：点击「导出 Obsidian 笔记」可获取带 Callout 与双链的完整学术笔记；点击「导出 Anki 闪卡」可直接导入 Anki 复习生词。</span>
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

.preview-body { overflow: auto; max-height: calc(85vh - 120px); background: var(--bg); }
.preview-dialog pre{padding:16px;margin:0;white-space:pre-wrap;font:12px/1.75 var(--font-mono);color:var(--text-2)}
.ruby-rendered {
  padding: 18px;
  font: 14px/2.2 var(--font-body);
  color: var(--text-1);
  white-space: pre-wrap;
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

.preview-foot{padding:8px 16px;color:var(--text-3);font-size:11px;background:var(--surface);border-top:1px solid var(--border)}

@media(max-width:720px){
  .results-view header{align-items:start;flex-direction:column}
  .filters{grid-template-columns:1fr 1fr}
  .filters input{grid-column:1/-1}
  .dialog-actions { flex-wrap: wrap; }
}
</style>
