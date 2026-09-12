<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { projectStore, runTranscribe, saveConfig, refreshArtifacts, engineLabel, clearTasks } from '../stores/project';
import { taskStore, loadTasks } from '../stores/task';
import { api, type ProjectConfig, type TaskItem } from '../services/tauri';
import { notify } from '../stores/ui';
import { loadPromptTemplates } from '../data/promptTemplates';

defineEmits<{ 'switch-tab': [tabId: string] }>();
const config = ref<ProjectConfig>({ engine:'custom',gemini_key:'',gemini_model:'gemini-2.5-flash',llm_provider:'none',llm_api_key:'',llm_api_url:'',llm_model_name:'',prompt_template:'',custom_api_url:'',custom_api_key:'',custom_model_name:'whisper-1',proxy:'',no_subtitle:false,cookies_from_browser:'none',youtube_cookie_file:'',local_output_dir:'',online_output_dir:'',formats:'txt,srt,md' });
const showLog = ref(false);
const templates = ref(loadPromptTemplates());
const selectedTemplate = ref('heritage_course');
const formats = ['txt','srt','md','lrc','html','json'];
const labels:Record<string,string>={txt:'纯文本',srt:'字幕',md:'Markdown (Obsidian)',lrc:'歌词',html:'网页',json:'时间轴'};
const stageLabels:Record<string,string>={draft:'待提交',queued:'等待中',downloading:'下载中',transcribing:'转写中',organizing:'整理中',done:'已完成',error:'失败',canceled:'已取消',interrupted:'已中断'};
const sources=computed(()=>projectStore.project?.sources??[]);
const tasks=computed(()=>taskStore.tasks);
const submitted=computed(()=>new Set(tasks.value.map(t=>t.source_id||t.id)));
const pendingIds=computed(()=>sources.value.filter(s=>!submitted.value.has(s.id)).map(s=>s.id));
const fmtSet=computed(()=>new Set(config.value.formats.split(',').filter(Boolean)));
const keyReady=computed(()=>['bcut','mlx'].includes(config.value.engine)||(config.value.engine==='gemini'?!!config.value.gemini_key:!!config.value.custom_api_url&&!!config.value.custom_api_key));

const heritagePresets = [
  {
    id: 'heritage_course',
    icon: '🎓',
    title: '华裔双语课程',
    sub: 'Cornell 笔记 + 拼音脚手架',
    templateId: 'heritage_course'
  },
  {
    id: 'oral_history',
    icon: '🎙️',
    title: '长辈口述历史',
    sub: '双语分轨 + 移民大事年表',
    templateId: 'oral_history'
  },
  {
    id: 'anki_vocab',
    icon: '📇',
    title: 'Anki 闪卡提取',
    sub: '成语/术语/生词卡片导出',
    templateId: 'anki_vocab'
  },
  {
    id: 'reading_seminar',
    icon: '📖',
    title: '东亚文史研讨',
    sub: '典故释读 + 争鸣对齐',
    templateId: 'reading_seminar'
  },
  {
    id: 'extract',
    icon: '📝',
    title: '通用干货提炼',
    sub: '快速删除废话与口语',
    templateId: 'extract'
  }
];

function selectPreset(preset: typeof heritagePresets[0]) {
  selectedTemplate.value = preset.templateId;
  applyTemplate();
  // 确保选中 md 格式
  if (!fmtSet.value.has('md')) {
    toggleFormat('md');
  }
  notify(`已切换至预设「${preset.title}」`, 'info');
}

function sync(){
  if(projectStore.project) {
    config.value={...projectStore.project.config};
    if (!config.value.prompt_template) {
      applyTemplate();
    }
  }
}
sync(); watch(()=>projectStore.project?.id,sync);
watch(()=>taskStore.running,(n,w)=>{if(w&&!n)refreshArtifacts()});

function toggleFormat(f:string){
  const s=new Set(fmtSet.value);
  s.has(f)?s.delete(f):s.add(f);
  config.value.formats=[...s].join(',');
}

function applyTemplate(){
  const template=templates.value.find((t)=>t.id===selectedTemplate.value);
  if(template) config.value.prompt_template=template.content;
}

async function submit(){
  if(!pendingIds.value.length)return;
  if(fmtSet.value.has('md'))applyTemplate();
  await saveConfig(config.value);
  notify(`${pendingIds.value.length} 个素材已加入队列`,'info');
  await runTranscribe(pendingIds.value,config.value.engine);
}

async function cancel(t:TaskItem){
  if(!projectStore.project)return;
  await api.cancelTask(projectStore.project.id,t.id);
  await loadTasks(projectStore.project.id);
}

async function retry(t:TaskItem){
  if(!projectStore.project)return;
  await api.retryTask(projectStore.project.id,t.id);
  await loadTasks(projectStore.project.id);
}

async function clear(){
  if(!taskStore.running&&window.confirm('清空任务记录？生成结果不会被删除。'))await clearTasks();
}
</script>

<template>
  <section class="target-card">
    <header>
      <div>
        <span class="kicker">02 · 华裔双语场景与输出目标</span>
        <h2>选择适配的知识加工与归档方案</h2>
      </div>
      <button class="settings-link" @click="$emit('switch-tab','settings')">转写引擎设置</button>
    </header>

    <!-- 华裔专属场景预设卡片 -->
    <div class="preset-grid">
      <div
        v-for="p in heritagePresets"
        :key="p.id"
        class="preset-card"
        :class="{ active: selectedTemplate === p.templateId }"
        @click="selectPreset(p)"
      >
        <span class="preset-icon">{{ p.icon }}</span>
        <div class="preset-info">
          <b>{{ p.title }}</b>
          <span>{{ p.sub }}</span>
        </div>
      </div>
    </div>

    <div class="target-row">
      <div class="format-list">
        <button
          v-for="f in formats"
          :key="f"
          class="format-chip"
          :class="{ active: fmtSet.has(f) }"
          @click="toggleFormat(f)"
        >
          <b>{{ f.toUpperCase() }}</b>
          <span>{{ labels[f] }}</span>
        </button>
      </div>
      <label class="ai-switch">
        <input
          type="checkbox"
          :checked="config.llm_provider!=='none'"
          @change="config.llm_provider=($event.target as HTMLInputElement).checked?'gemini':'none'"
        >
        <span>AI 深度学术整理</span>
      </label>
    </div>

    <div v-if="fmtSet.has('md')" class="template-row">
      <div class="template-desc">
        <b>Markdown 提示词模板</b>
        <span>专为华裔双语学生、东亚课程与长辈口述史调优</span>
      </div>
      <select v-model="selectedTemplate" @change="applyTemplate">
        <optgroup label="🌟 华裔学生与口述历史专属">
          <option v-for="t in templates.filter(t => t.category === 'heritage')" :key="t.id" :value="t.id">
            {{ t.title }}
          </option>
        </optgroup>
        <optgroup label="📋 通用整理模板">
          <option v-for="t in templates.filter(t => t.category !== 'heritage')" :key="t.id" :value="t.id">
            {{ t.title }}
          </option>
        </optgroup>
      </select>
    </div>

    <!-- 隐私与安全承诺提示 -->
    <div class="privacy-note">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
      </svg>
      <span>端侧隐私保护：家庭访谈与口述历史音频在本地运行转写，零第三方云端上传，零商业数据窥探。</span>
    </div>

    <div class="submit-row">
      <p><b>{{ engineLabel(config.engine) }}</b> · {{ pendingIds.length }} 个待提交素材</p>
      <button
        class="submit-btn"
        :disabled="!pendingIds.length||!fmtSet.size||!keyReady"
        @click="submit"
      >
        {{ taskStore.running?'加入队尾':'加入队列并开始转写' }}
      </button>
    </div>
    <p v-if="!keyReady" class="warning">当前引擎尚未配置凭据，请进入高级设置。</p>
  </section>

  <section class="queue-card">
    <header>
      <div><span class="kicker">03 · 转换队列</span><h2>任务进度</h2></div>
      <button v-if="tasks.length&&!taskStore.running" class="text-btn" @click="clear">清空记录</button>
    </header>
    <div v-if="tasks.length" class="task-list">
      <article v-for="task in tasks" :key="task.id" class="task" :class="task.status">
        <span class="state-dot"/>
        <div class="task-main">
          <div class="task-title">
            <b>{{ task.source_name }}</b>
            <span>{{ stageLabels[task.status]||task.status }}</span>
          </div>
          <p>{{ engineLabel(task.engine) }} · {{ task.formats||'默认输出' }}<template v-if="task.message"> · {{ task.message }}</template></p>
          <div v-if="['downloading','transcribing','organizing'].includes(task.status)" class="progress">
            <i :style="{width:(task.progress||8)+'%'}"/>
          </div>
          <p v-if="task.error" class="error-text">{{ task.error }}</p>
        </div>
        <div class="task-actions">
          <button v-if="['queued','downloading','transcribing','organizing'].includes(task.status)" @click="cancel(task)">取消</button>
          <button v-if="['error','canceled','interrupted'].includes(task.status)" @click="retry(task)">重试</button>
          <button v-if="task.status==='done'" @click="$emit('switch-tab','results')">查看结果</button>
        </div>
      </article>
    </div>
    <div v-else class="empty">素材提交后，将在这里显示等待、下载、转写和完成状态。</div>
    <button v-if="taskStore.logs.length" class="log-toggle" @click="showLog=!showLog">{{ showLog?'隐藏':'显示' }}技术日志（{{ taskStore.logs.length }}）</button>
    <pre v-if="showLog" class="logs">{{ taskStore.logs.join('\n') }}</pre>
  </section>
</template>

<style scoped>
.target-card,.queue-card{background:var(--surface);border:1px solid var(--border);border-radius:var(--r-lg);padding:20px;display:grid;gap:16px}
.target-card header,.queue-card header,.submit-row,.target-row,.task-title{display:flex;align-items:center;justify-content:space-between;gap:16px}
h2{font-size:16px;margin-top:3px}
.kicker{font:700 10px var(--font-mono);letter-spacing:.12em;color:var(--accent)}
.settings-link,.text-btn,.task-actions button,.log-toggle{border:0;background:none;color:var(--text-3);font-size:12px;cursor:pointer}

.preset-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 10px;
}
.preset-card {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  border: 1px solid var(--border);
  border-radius: 9px;
  background: var(--bg);
  cursor: pointer;
  transition: all 0.15s ease;
}
.preset-card:hover {
  border-color: var(--accent);
  background: var(--surface-2);
}
.preset-card.active {
  border-color: var(--accent);
  background: var(--accent-bg);
  box-shadow: 0 0 0 1px var(--accent);
}
.preset-icon {
  font-size: 20px;
  flex: none;
}
.preset-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}
.preset-info b {
  font-size: 13px;
  color: var(--text-1);
}
.preset-info span {
  font-size: 11px;
  color: var(--text-3);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.format-list{display:flex;flex-wrap:wrap;gap:8px}
.format-chip{display:flex;align-items:center;gap:7px;padding:8px 11px;border:1px solid var(--border);border-radius:9px;background:var(--bg);color:var(--text-3);cursor:pointer}
.format-chip.active{border-color:var(--accent);background:var(--accent-bg);color:var(--text-1)}
.format-chip span{font-size:11px}
.ai-switch{display:flex;align-items:center;gap:8px;white-space:nowrap;font-size:12px}

.template-row{display:flex;align-items:center;justify-content:space-between;gap:16px;padding:12px 13px;border:1px solid var(--border-soft);border-radius:10px;background:var(--bg)}
.template-desc{display:grid;gap:2px}
.template-desc b{font-size:12px}
.template-desc span{font-size:11px;color:var(--text-3)}
.template-row select{min-width:240px;padding:7px 9px;border:1px solid var(--border);border-radius:7px;background:var(--surface);color:var(--text-1)}

.privacy-note {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border-radius: 8px;
  background: rgba(16, 185, 129, 0.07);
  border: 1px solid rgba(16, 185, 129, 0.2);
  color: #059669;
  font-size: 11px;
}
.privacy-note svg {
  width: 15px; height: 15px; flex: none;
}

.submit-row{padding-top:14px;border-top:1px solid var(--border-soft)}
.submit-row p{font-size:12px;color:var(--text-3)}
.submit-btn{padding:10px 18px;border:0;border-radius:9px;background:var(--accent);color:#fff;font-weight:700;cursor:pointer}
.submit-btn:disabled{opacity:.45;cursor:not-allowed}
.warning,.error-text{color:var(--error);font-size:11px}
.task-list{display:grid;gap:7px}
.task{display:flex;align-items:center;gap:12px;padding:13px;border:1px solid var(--border-soft);border-radius:10px;background:var(--bg)}
.state-dot{width:9px;height:9px;border-radius:50%;background:var(--neutral-4);flex:none}
.task.downloading .state-dot,.task.transcribing .state-dot,.task.organizing .state-dot{background:var(--accent);box-shadow:0 0 0 4px var(--accent-soft)}
.task.done .state-dot{background:var(--success)}
.task.error .state-dot{background:var(--error)}
.task-main{flex:1;min-width:0}
.task-title b{overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-size:13px}
.task-title span,.task-main p{font-size:11px;color:var(--text-3)}
.progress{height:4px;margin-top:7px;background:var(--surface-2);border-radius:4px;overflow:hidden}
.progress i{display:block;height:100%;background:var(--accent);transition:width .25s}
.task-actions{display:flex;gap:8px}
.task-actions button{padding:5px 8px;border:1px solid var(--border);border-radius:6px;background:var(--surface)}
.empty{padding:28px;text-align:center;color:var(--text-3);border:1px dashed var(--border);border-radius:10px;font-size:12px}
.logs{max-height:220px;overflow:auto;padding:12px;border-radius:8px;background:var(--neutral-10);color:var(--neutral-3);font:11px/1.7 var(--font-mono);white-space:pre-wrap}
.log-toggle{justify-self:start}
.warning{margin-top:-8px}
@media(max-width:650px){
  .target-row,.submit-row,.template-row{align-items:flex-start;flex-direction:column}
  .template-row select,.submit-btn{width:100%}
  .task{align-items:flex-start}
  .task-actions{flex-direction:column}
}
</style>
