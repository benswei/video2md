<script setup lang="ts">
import { computed, ref } from 'vue';
import { projectStore, formatSize } from '../stores/project';
import { api, type ArtifactItem } from '../services/tauri';
import { notify } from '../stores/ui';

const formatFilter = ref('all');
const sourceFilter = ref('all');
const query = ref('');
const preview = ref<{name:string; content:string; unsupported:boolean}|null>(null);
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
  if(!textTypes.includes(a.ext)){preview.value={name:a.name,content:'此格式暂不在应用内渲染。可使用“打开”交给系统默认程序查看。',unsupported:true};return;}
  try{preview.value={name:a.name,content:await api.readTextFile(a.path),unsupported:false}}catch(e){notify(`预览失败：${e}`,'error')}
}
async function reveal(a:ArtifactItem){try{await api.showInFolder(a.path)}catch(e){notify(`定位失败：${e}`,'error')}}
async function exportOne(a:ArtifactItem){if(!projectStore.project)return;const dir=await api.pickExportDir();if(!dir)return;try{const r=await api.exportArtifacts(projectStore.project.id,dir,[a.name]);notify(r.copied?`已导出 ${a.name}`:'导出失败','success')}catch(e){notify(`导出失败：${e}`,'error')}}
async function exportAll(){if(!projectStore.project)return;const dir=await api.pickExportDir();if(!dir)return;try{const r=await api.exportArtifacts(projectStore.project.id,dir,rows.value.map(a=>a.name));notify(`已导出 ${r.copied} 个文件`,'success')}catch(e){notify(`导出失败：${e}`,'error')}}
</script>

<template>
  <div class="results-view">
    <header><div><span class="eyebrow">RESULT LIBRARY</span><h1>转换结果</h1><p>按来源、格式或文件名筛选；每一行都是一个可直接使用的结果文件。</p></div><button class="export-all" :disabled="!rows.length" @click="exportAll">导出当前筛选（{{ rows.length }}）</button></header>
    <section class="filters"><input v-model="query" placeholder="搜索来源或文件名"/><select v-model="sourceFilter"><option value="all">全部来源</option><option v-for="s in sourceOptions" :key="s.id" :value="s.id">{{ s.name }}</option></select><select v-model="formatFilter"><option value="all">全部格式</option><option v-for="f in formats" :key="f" :value="f">{{ f.toUpperCase() }}</option></select><button v-if="query||sourceFilter!=='all'||formatFilter!=='all'" @click="query='';sourceFilter='all';formatFilter='all'">重置筛选</button></section>
    <section class="table-wrap"><table v-if="rows.length"><thead><tr><th>结果文件</th><th>来源</th><th>格式</th><th>大小</th><th>完成时间</th><th>操作</th></tr></thead><tbody><tr v-for="a in rows" :key="a.path"><td class="file"><b>{{ a.name }}</b></td><td class="source">{{ sourceName(a) }}</td><td><span class="type">{{ a.ext.toUpperCase() }}</span></td><td>{{ formatSize(a.size) }}</td><td>{{ a.modified || '—' }}</td><td class="actions"><button @click="openPreview(a)">预览</button><button @click="open(a)">打开</button><button @click="reveal(a)">定位</button><button @click="exportOne(a)">导出</button></td></tr></tbody></table><div v-else class="empty">{{ artifacts.length ? '没有符合当前筛选条件的结果。' : '完成转换后，结果会显示在这里。' }}</div></section>
    <div v-if="preview" class="preview-mask" @click.self="preview=null"><section class="preview-dialog"><header><b>{{preview.name}}</b><button @click="preview=null">×</button></header><pre>{{preview.content}}</pre><footer v-if="preview.unsupported">请使用列表中的“打开”按钮查看该文件。</footer></section></div>
  </div>
</template>

<style scoped>
.results-view{max-width:1180px;margin:auto;display:grid;gap:16px}.results-view header{display:flex;align-items:end;justify-content:space-between;gap:16px;padding:4px 2px}.eyebrow{font:700 10px var(--font-mono);letter-spacing:.14em;color:var(--accent)}h1{margin:4px 0;font:700 24px var(--font-display)}header p{font-size:12px;color:var(--text-3)}.export-all{padding:9px 13px;border:0;border-radius:8px;background:var(--accent);color:#fff;font-weight:700;cursor:pointer;white-space:nowrap}.export-all:disabled{opacity:.45}.filters{display:grid;grid-template-columns:minmax(180px,1fr) minmax(150px,.55fr) 130px auto;gap:8px;padding:12px;border:1px solid var(--border);border-radius:10px;background:var(--surface)}.filters input,.filters select,.filters button{padding:8px 10px;border:1px solid var(--border);border-radius:7px;background:var(--bg);color:var(--text-2);font-size:12px}.filters button{cursor:pointer}.table-wrap{border:1px solid var(--border);border-radius:11px;background:var(--surface);overflow:auto}table{width:100%;border-collapse:collapse;font-size:12px;min-width:780px}th{text-align:left;padding:10px 12px;color:var(--text-3);font-size:10px;letter-spacing:.07em;text-transform:uppercase;background:var(--bg)}td{padding:11px 12px;border-top:1px solid var(--border-soft);color:var(--text-2)}.file{color:var(--text-1);max-width:280px}.file b,.source{display:block;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.source{max-width:200px}.type{display:inline-block;padding:2px 6px;border-radius:4px;background:var(--accent-bg);color:var(--accent);font:700 10px var(--font-mono)}.actions{white-space:nowrap}.actions button{border:0;background:none;color:var(--accent);cursor:pointer;font-size:12px;margin-right:10px}.empty{padding:48px 20px;text-align:center;color:var(--text-3);font-size:12px}@media(max-width:720px){.results-view header{align-items:start;flex-direction:column}.filters{grid-template-columns:1fr 1fr}.filters input{grid-column:1/-1}}
.preview-mask{position:fixed;inset:0;z-index:500;display:grid;place-items:center;padding:24px;background:oklch(15% 0.02 55 /.46);backdrop-filter:blur(5px)}.preview-dialog{width:min(900px,100%);max-height:80vh;display:grid;grid-template-rows:auto 1fr auto;border:1px solid var(--border);border-radius:12px;background:var(--surface);overflow:hidden;box-shadow:0 24px 70px #0003}.preview-dialog header{display:flex;align-items:center;justify-content:space-between;padding:13px 16px;border-bottom:1px solid var(--border)}.preview-dialog header b{font-size:13px}.preview-dialog header button{border:0;background:none;font-size:22px;cursor:pointer;color:var(--text-3)}.preview-dialog pre{overflow:auto;padding:16px;background:var(--bg);white-space:pre-wrap;font:12px/1.75 var(--font-mono);color:var(--text-2)}.preview-dialog footer{padding:10px 16px;color:var(--text-3);font-size:12px}
</style>
