<script setup lang="ts">
import { ref, watch, computed, onMounted } from 'vue';
import { projectStore, saveConfig } from '../stores/project';
import { api, type ProjectConfig } from '../services/tauri';
import { notify } from '../stores/ui';
import { loadPromptTemplates, type PromptTemplate } from '../data/promptTemplates';

const saving = ref(false);
const testing = ref(false);
const aiTesting = ref(false);
const aiTestResult = ref('');
const testResult = ref('');
const biliLogin = ref<{key:string;url:string}|null>(null);
const biliLoginStatus = ref('');
let biliPollTimer: ReturnType<typeof window.setInterval> | null = null;
const mlxAvailable = ref(false);
onMounted(async()=>{try{const [, , mlx]=await api.getPlatformCapabilities();mlxAvailable.value=mlx}catch{/* 不阻塞设置 */}})
const templates = ref<PromptTemplate[]>(loadPromptTemplates());
const selectedTemplate = ref('');
function persistTemplates(){localStorage.setItem('video2md_prompt_templates',JSON.stringify(templates.value))}
function applyTemplate(){const t=templates.value.find(x=>x.id===selectedTemplate.value);if(t)cfg.value.prompt_template=t.content}
function addTemplate(){const title=window.prompt('模板名称');if(!title)return;const content=cfg.value.prompt_template.trim();if(!content){notify('请先填写提示词内容','info');return}const t={id:String(Date.now()),title,content};templates.value.push(t);selectedTemplate.value=t.id;persistTemplates()}
function deleteTemplate(){const i=templates.value.findIndex(x=>x.id===selectedTemplate.value);if(i<0)return;templates.value.splice(i,1);selectedTemplate.value='';persistTemplates()}

const cfg = ref<ProjectConfig>({
  engine: 'custom',
  gemini_key: '',
  gemini_model: 'gemini-2.5-flash',
  llm_provider: 'none',
  llm_api_key: '',
  llm_api_url: '',
  llm_model_name: '',
  prompt_template: '',
  custom_api_url: '',
  custom_api_key: '',
  custom_model_name: 'whisper-1',
  proxy: '',
  no_subtitle: false,
  cookies_from_browser: 'none',
  youtube_cookie_file: '',
  local_output_dir: '',
  online_output_dir: '',
  formats: 'txt,srt,md',
});

const pages = [
  { id: 'output', label: '转换默认项', icon: 'M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z' },
  { id: 'models', label: 'AI 增强', icon: 'M12 2a10 10 0 100 20 10 10 0 000-20zM2 12h20M12 2a15 15 0 010 20' },
  { id: 'net', label: '站点登录与网络', icon: 'M12 2a10 10 0 100 20 10 10 0 000-20zM2 12h20M12 2a15 15 0 010 20' },
];
const activePage = ref('output');
type ServiceTemplate = { id:string; name:string; note:string; asr?:{provider:'custom'|'gemini';url?:string;model?:string}; text?:{provider:'custom'|'gemini';url?:string;model?:string} };
const serviceTemplates: ServiceTemplate[] = [
  { id:'openai', name:'OpenAI / 兼容网关', note:'ASR 与文本优化均适用；分别填写两个模型名。', asr:{provider:'custom',model:'whisper-1'}, text:{provider:'custom'} },
  { id:'gemini', name:'Google Gemini', note:'可分别用作 Gemini ASR 与文本优化；两处模型可独立选择。', asr:{provider:'gemini',model:'gemini-2.5-flash'}, text:{provider:'gemini',model:'gemini-2.5-flash'} },
  { id:'dashscope', name:'阿里云百炼', note:'提供 ASR 与文本优化的预填模板；请确认账号能力已开通。', asr:{provider:'custom',url:'https://dashscope.aliyuncs.com/compatible-mode/v1/audio/transcriptions',model:'paraformer-v2'}, text:{provider:'custom',url:'https://dashscope.aliyuncs.com/compatible-mode/v1'} },
  { id:'volc', name:'火山引擎 / 豆包', note:'文本优化可填官方兼容地址；ASR 需要专用协议或兼容网关。', text:{provider:'custom',url:'https://ark.cn-beijing.volces.com/api/v3'} },
  { id:'zhipu', name:'智谱 AI', note:'适用于文本优化；ASR 请选择其他支持转写的服务。', text:{provider:'custom',url:'https://open.bigmodel.cn/api/paas/v4'} },
  { id:'tencent', name:'腾讯云', note:'录音识别为专用签名协议；请通过兼容网关分别接入 ASR 或文本服务。' },
  { id:'iflytek', name:'讯飞开放平台', note:'听写为专用协议；请通过兼容网关分别接入 ASR 或文本服务。' },
];
const selectedServiceTemplate = ref('openai');
const currentServiceTemplate = computed(()=>serviceTemplates.find(x=>x.id===selectedServiceTemplate.value));
function applyServiceTemplate(target:'asr'|'text'){
  const entry=currentServiceTemplate.value?.[target]; const name=currentServiceTemplate.value?.name||'该模板';
  if(!entry){notify(`「${name}」暂未提供可直接应用的${target==='asr'?'ASR':'文本优化'}模板，请使用兼容网关。`,'info');return;}
  if(target==='asr'){
    cfg.value.engine=entry.provider;
    if(entry.provider==='gemini'){if(entry.model)cfg.value.gemini_model=entry.model;}
    else {cfg.value.custom_api_url=entry.url||'';cfg.value.custom_model_name=entry.model||'';}
  }else{
    cfg.value.llm_provider=entry.provider;
    if(entry.provider==='gemini'){if(entry.model)cfg.value.llm_model_name=entry.model;}
    else {cfg.value.llm_api_url=entry.url||'';cfg.value.llm_model_name=entry.model||'';}
  }
  notify(`已将「${name}」应用到${target==='asr'?'ASR 转写':'AI 文本优化'}；可继续分别调整模型与 Key。`,'success');
}

function sync() {
  const c = projectStore.project?.config;
  if (c) cfg.value = { ...c };
  testResult.value = '';
}
sync();
watch(() => projectStore.project?.id, sync);
watch(() => cfg.value.llm_provider, (provider) => { if (provider === 'gemini' && !cfg.value.llm_model_name) cfg.value.llm_model_name = 'gemini-2.5-flash'; });

// 输出格式复选框
const fmtOptions = ['md', 'html', 'txt', 'srt', 'lrc', 'json'] as const;
const fmtSet = computed<Set<string>>(() => new Set(cfg.value.formats.split(',').map((s) => s.trim()).filter(Boolean)));
function toggleFmt(f: string, on: boolean) {
  const set = new Set(fmtSet.value);
  if (on) set.add(f);
  else set.delete(f);
  cfg.value.formats = [...set].join(',');
}

async function onSave() {
  if (!projectStore.project) return;
  saving.value = true;
  try {
    await saveConfig(cfg.value);
    testResult.value = '已保存当前项目的配置。';
    notify('配置已保存', 'success');
  } catch (error) {
    testResult.value = `保存失败：${error}`;
    notify(testResult.value, 'error');
  } finally {
    saving.value = false;
  }
}

async function onTest() {
  if (!projectStore.project) return;
  testing.value = true;
  try {
    testResult.value = await api.testConnection(projectStore.project.id, cfg.value);
    notify(testResult.value, 'success');
  } catch (error) {
    testResult.value = `连接失败：${error}`;
    notify(testResult.value, 'error', 6000);
  } finally {
    testing.value = false;
  }
}
async function onTestAi(){
  if(!projectStore.project)return;aiTesting.value=true;aiTestResult.value='';
  try{aiTestResult.value=await api.testAiConnection(projectStore.project.id,cfg.value);notify(aiTestResult.value,'success')}
  catch(error){aiTestResult.value=`AI 检测失败：${error}`;notify(aiTestResult.value,'error',7000)}
  finally{aiTesting.value=false}
}
async function chooseCookieFile(){const path=await api.pickCookieFile();if(path)cfg.value.youtube_cookie_file=path}
async function chooseOutput(kind: 'local_output_dir' | 'online_output_dir'){const path=await api.pickExportDir();if(path)cfg.value[kind]=path}
function stopBiliPoll(){if(biliPollTimer){window.clearInterval(biliPollTimer);biliPollTimer=null}}
async function pollBili(){if(!biliLogin.value||!projectStore.project)return;try{const r=await api.pollBilibiliQrLogin(projectStore.project.id,biliLogin.value.key);if(r.status==='scanned')biliLoginStatus.value='已扫码，请在 B站 App 确认登录';else if(r.status==='confirmed'){stopBiliPoll();biliLoginStatus.value='登录成功，已保存 B站采集凭据';cfg.value=await api.getConfig(projectStore.project.id);projectStore.project.config={...cfg.value};notify('B站登录成功','success')}else if(r.status==='expired'){stopBiliPoll();biliLoginStatus.value='二维码已过期，请重新获取'}else if(r.status==='error'){stopBiliPoll();biliLoginStatus.value=`登录失败：${r.message||'未知错误'}`}}catch(e){stopBiliPoll();biliLoginStatus.value=`登录失败：${e}`}}
async function startBiliLogin(){stopBiliPoll();biliLoginStatus.value='正在获取 B站二维码…';try{const r=await api.startBilibiliQrLogin();if(!r.key||!r.url)throw new Error(r.message||'未获取到二维码');biliLogin.value={key:r.key,url:r.url};biliLoginStatus.value='请使用 B站 App 扫码并确认';biliPollTimer=window.setInterval(pollBili,1800)}catch(e){biliLoginStatus.value=`无法发起登录：${e}`}}

// 已配置的 AI 服务清单
const providers = computed(() => {
  const list: { name: string; key: string; status: string; ok: boolean }[] = [];
  list.push({
    name: 'Gemini ASR',
    key: cfg.value.gemini_key ? 'gemini_key' : '',
    status: cfg.value.gemini_key ? '已配置' : '未配置',
    ok: !!cfg.value.gemini_key,
  });
  if (cfg.value.llm_provider === 'gemini') list.push({ name:'Gemini 文本优化', key:cfg.value.llm_api_key?'llm_api_key':'', status:cfg.value.llm_api_key?'已配置':'需单独填写 Key', ok:!!cfg.value.llm_api_key });
  list.push({
    name: 'B站必剪 ASR',
    key: '',
    status: '免密钥（需联网）',
    ok: true,
  });
  if (cfg.value.custom_api_url) {
    list.push({
      name: '自定义 ASR',
      key: 'custom_api_url',
      status: cfg.value.custom_api_key ? '已配置' : '已填地址',
      ok: true,
    });
  }
  if (cfg.value.llm_provider === 'custom' && cfg.value.llm_api_url) {
    list.push({
      name: '自定义 AI 文本',
      key: 'llm_api_key',
      status: cfg.value.llm_api_key ? '已配置' : '需填 Key',
      ok: !!cfg.value.llm_api_key,
    });
  }
  return list;
});
</script>

<template>
  <div class="settings-view">
    <div class="settings-head">
      <h2 class="settings-title">设置</h2>
      <p class="settings-sub">默认使用自配置 AI 接口转写；Gemini、必剪与本地引擎均可按需要切换。设置仅作用于当前项目「{{ projectStore.project?.name }}」。</p>
    </div>

    <div class="settings-body">
      <!-- 左侧导航 -->
      <nav class="cfg-nav">
        <button
          v-for="p in pages"
          :key="p.id"
          class="cfg-nav-item"
          :class="{ active: activePage === p.id }"
          @click="activePage = p.id"
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path :d="p.icon" /></svg>
          <span>{{ p.label }}</span>
        </button>
      </nav>

      <!-- 右侧内容 -->
      <div class="cfg-content">
        <!-- ① 模型与引擎 -->
        <section v-if="activePage === 'models'" class="cfg-section">
          <h3 class="cfg-section-title">AI 与转写服务</h3>
          <p class="cfg-hint">自配置 AI 接口是默认转写方式。Gemini 仍是独立 ASR 工具；必剪适合免配置的备用转写。</p>
          <div class="provider-list">
            <div v-for="pv in providers" :key="pv.name" class="provider-card">
              <div class="provider-info">
                <span class="provider-name">{{ pv.name }}</span>
                <span class="provider-status" :class="pv.ok ? 'ok' : 'warn'">{{ pv.status }}</span>
              </div>
            </div>
          </div>

          <div class="service-hub">
            <div class="service-hub-head"><div><b>服务模板库</b><p>模板只负责预填服务类型与地址；ASR 和文本优化可分别应用同一或不同模板。</p></div><select v-model="selectedServiceTemplate"><option v-for="item in serviceTemplates" :key="item.id" :value="item.id">{{item.name}}</option></select></div>
            <div class="service-hub-actions"><button type="button" @click="applyServiceTemplate('asr')">应用到 ASR 转写</button><button type="button" @click="applyServiceTemplate('text')">应用到 AI 文本优化</button><small>{{currentServiceTemplate?.note}}</small></div>
          </div>

          <h3 class="cfg-section-title" style="margin-top: 8px">ASR 语音转写引擎</h3>
          <div class="engine-cards">
            <button class="engine-card" :class="{ active: cfg.engine === 'bcut' }" @click="cfg.engine = 'bcut'">
              <span class="engine-name">必剪</span><span class="engine-desc">免费、免密钥、带时间轴</span>
            </button>
            <button class="engine-card" :class="{ active: cfg.engine === 'gemini' }" @click="cfg.engine = 'gemini'">
              <span class="engine-name">Gemini</span><span class="engine-desc">高精度，需 API Key</span>
            </button>
            <button class="engine-card" :class="{ active: cfg.engine === 'custom' }" @click="cfg.engine = 'custom'">
              <span class="engine-name">自配置 AI 接口（默认）</span><span class="engine-desc">OpenAI 兼容 ASR / 网关服务</span>
            </button>
            <button v-if="mlxAvailable" class="engine-card" :class="{ active: cfg.engine === 'mlx' }" @click="cfg.engine = 'mlx'"><span class="engine-name">mlx-whisper</span><span class="engine-desc">Apple Silicon 本地离线</span></button>
          </div>

          <div class="cfg-grid" v-if="cfg.engine === 'gemini'">
            <label class="cfg-field full">
              <span>GEMINI_API_KEY</span>
              <input type="password" v-model="cfg.gemini_key" placeholder="粘贴你的 GEMINI_API_KEY" />
            </label>
            <label class="cfg-field"><span>Gemini Flash 模型</span><select v-model="cfg.gemini_model"><option value="gemini-2.5-flash">Gemini 2.5 Flash（默认）</option><option value="gemini-2.0-flash">Gemini 2.0 Flash</option></select></label>
            <div class="inline-test"><span>仅检查 Gemini ASR 当前配置。</span><button type="button" :disabled="testing" @click="onTest">{{ testing?'检测中…':'测试 Gemini ASR' }}</button><em v-if="testResult">{{testResult}}</em></div>
            <p class="cfg-hint">仅用于 Gemini ASR。AI 文本优化如选择 Gemini，请在下方单独配置 Key 与模型。</p>
          </div>
          <div class="cfg-grid" v-if="cfg.engine === 'custom'">
            <label class="cfg-field full"><span>ASR API 地址</span><input v-model="cfg.custom_api_url" placeholder="https://your-whisper/api" /></label>
            <label class="cfg-field"><span>API Key</span><div class="input-test"><input type="password" v-model="cfg.custom_api_key" placeholder="必填" /><button type="button" :disabled="testing" @click="onTest">{{testing?'检测中…':'测试接口'}}</button></div></label>
            <label class="cfg-field"><span>模型名</span><input v-model="cfg.custom_model_name" placeholder="whisper-1" /></label>
          </div>

          <h3 class="cfg-section-title" style="margin-top: 8px">AI 文本整理</h3>
          <p class="cfg-hint">可选择与 ASR 完全不同的服务、Key 与模型；模板库的“应用到 AI 文本优化”只会修改本区字段。</p>
          <div class="cfg-grid">
            <label class="cfg-field"><span>整理服务</span>
              <select v-model="cfg.llm_provider">
                <option value="gemini">Gemini（默认与 ASR 共用 Key）</option>
                <option value="custom">自定义 OpenAI 兼容</option>
                <option value="none">不整理（仅原文）</option>
              </select>
            </label>
            <label class="cfg-field" v-if="cfg.llm_provider !== 'none'"><span>{{ cfg.llm_provider === 'gemini' ? 'Gemini 文本优化 Key' : '文本优化 API Key' }}</span><input type="password" v-model="cfg.llm_api_key" placeholder="必填；不会与 ASR Key 共用" /></label>
            <label class="cfg-field" v-if="cfg.llm_provider === 'custom'"><span>文本优化 API 地址</span><input v-model="cfg.llm_api_url" placeholder="必填，例如 https://your-api/v1" /></label>
            <label class="cfg-field" v-if="cfg.llm_provider === 'gemini'"><span>文本优化模型（独立于 ASR）</span><select v-model="cfg.llm_model_name"><option value="gemini-2.5-flash">Gemini 2.5 Flash</option><option value="gemini-2.0-flash">Gemini 2.0 Flash</option></select></label>
            <label class="cfg-field" v-else-if="cfg.llm_provider === 'custom'"><span>文本优化模型（独立于 ASR）</span><input v-model="cfg.llm_model_name" placeholder="例如 qwen-plus、glm-4-flash" /></label>
            <label class="cfg-field full" v-if="cfg.llm_provider !== 'none'"><span>整理提示词（可选）</span>
              <div class="template-tools"><select v-model="selectedTemplate" @change="applyTemplate"><option value="">选择提示词模板</option><option v-for="t in templates" :key="t.id" :value="t.id">{{ t.title }}</option></select><button type="button" @click="addTemplate">保存为模板</button><button type="button" :disabled="!selectedTemplate" @click="deleteTemplate">删除</button></div>
              <textarea v-model="cfg.prompt_template" rows="3" placeholder="留空使用默认提示词" />
            </label>
          </div>
          <div class="ai-test-bar"><span>仅检测当前 AI 文本优化配置，不影响 ASR 或软件启动。</span><button type="button" :disabled="aiTesting" @click="onTestAi">{{ aiTesting ? '检测中…' : '测试文本优化' }}</button><em v-if="aiTestResult" :class="aiTestResult.startsWith('AI 检测失败')?'error':'ok'">{{ aiTestResult }}</em></div>
        </section>

        <!-- ② 输出格式 -->
        <section v-if="activePage === 'output'" class="cfg-section">
          <h3 class="cfg-section-title">每次转换生成什么？</h3>
          <p class="cfg-hint">选择一个或多个结果。TXT 适合复制，SRT/LRC 适合字幕，Markdown/HTML 适合阅读与知识整理。</p>
          <div class="fmt-checks">
            <label v-for="f in fmtOptions" :key="f" class="fmt-check">
              <input type="checkbox" :checked="fmtSet.has(f)" @change="toggleFmt(f, ($event.target as HTMLInputElement).checked)" />
              <span>{{ { md: 'Wiki Markdown', html: '自适应 HTML', txt: '纯文本 TXT', srt: 'SRT 字幕', lrc: 'LRC 歌词', json: 'JSON 时间轴' }[f] }}</span>
            </label>
          </div>
          <div class="output-rules">
            <label class="cfg-field"><span>本地文件输出目录</span><div class="file-picker"><input v-model="cfg.local_output_dir" readonly placeholder="默认：与原始文件同目录"/><button type="button" @click="chooseOutput('local_output_dir')">选择目录</button><button v-if="cfg.local_output_dir" type="button" @click="cfg.local_output_dir=''">恢复默认</button></div></label>
            <label class="cfg-field"><span>在线链接输出目录</span><div class="file-picker"><input v-model="cfg.online_output_dir" readonly placeholder="默认：下载目录 / 项目名称"/><button type="button" @click="chooseOutput('online_output_dir')">选择目录</button><button v-if="cfg.online_output_dir" type="button" @click="cfg.online_output_dir=''">恢复默认</button></div></label>
          </div>
          <p class="cfg-hint">在线内容总会在所选目录下按项目名称创建子文件夹，避免不同批次混在一起。</p>
        </section>

        <!-- ③ 网络与代理 -->
        <section v-if="activePage === 'net'" class="cfg-section">
          <h3 class="cfg-section-title">站点登录与网络</h3>
          <div class="bili-login">
            <div><b>B站扫码登录（推荐）</b><p>使用 B站 App 扫码授权。登录态由本应用保存为采集 Cookie，不读取浏览器加密数据。</p><em v-if="biliLoginStatus">{{ biliLoginStatus }}</em></div>
            <div class="bili-qr" v-if="biliLogin"><img :src="`https://api.qrserver.com/v1/create-qr-code/?size=176x176&data=${encodeURIComponent(biliLogin.url)}`" alt="B站扫码登录二维码"/><button type="button" @click="startBiliLogin">刷新二维码</button></div>
            <button v-else type="button" class="bili-start" @click="startBiliLogin">获取扫码二维码</button>
          </div>
          <div class="cfg-grid">
            <label class="cfg-field"><span>HTTP/HTTPS 代理（可选）</span><input v-model="cfg.proxy" placeholder="http://127.0.0.1:7890" /></label>
            <label class="cfg-field checkbox"><input type="checkbox" v-model="cfg.no_subtitle" /><span>跳过字幕抓取，直接转写</span></label>
            <label class="cfg-field"><span>读取浏览器登录状态（兼容方式）</span><select v-model="cfg.cookies_from_browser"><option value="none">不使用浏览器 Cookie</option><option value="edge">Microsoft Edge</option><option value="chrome">Google Chrome</option><option value="firefox">Firefox</option></select></label>
            <label class="cfg-field full"><span>其他站点 Cookie 文件（可选）</span><div class="file-picker"><input v-model="cfg.youtube_cookie_file" readonly placeholder="导入 Netscape 格式 cookies.txt（适用于 YouTube 等）"/><button type="button" @click="chooseCookieFile">选择文件</button></div></label>
          </div>
          <p class="cfg-hint">B站请优先使用上方扫码登录。浏览器 Cookie 仅作为旧兼容路径，部分浏览器加密后无法读取；YouTube 等其他站点可按实际情况导入自己的 Netscape Cookie 文件。仅在链接无法访问时再设置代理。</p>
        </section>


        <div class="settings-actions">
          <button class="run-btn" :disabled="saving" @click="onSave">{{ saving ? '保存中…' : '保存配置' }}</button>
          <span class="test-result">保存后会应用到后续新任务；各服务的测试按钮位于对应配置旁。</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings-view { display: flex; flex-direction: column; gap: var(--s-5); max-width: 920px; animation: fadeIn 0.25s ease; }
.bili-login{display:flex;align-items:center;justify-content:space-between;gap:18px;padding:14px;border:1px solid color-mix(in oklch,var(--accent) 26%,var(--border));border-radius:10px;background:var(--accent-bg);margin-bottom:14px}.bili-login b{font-size:13px}.bili-login p,.bili-login em{display:block;margin-top:4px;font-size:11px;line-height:1.55;color:var(--text-3)}.bili-login em{color:var(--accent);font-style:normal}.bili-start,.bili-qr button{padding:8px 10px;border:0;border-radius:7px;background:var(--accent);color:#fff;font-size:12px;font-weight:700;cursor:pointer;white-space:nowrap}.bili-qr{display:grid;gap:6px;justify-items:center}.bili-qr img{width:132px;height:132px;padding:5px;border-radius:7px;background:#fff}@media(max-width:620px){.bili-login{align-items:flex-start;flex-direction:column}.bili-qr{align-self:center}}
@keyframes fadeIn { from { opacity: 0; } to { opacity: 1; } }
.settings-head { display: flex; flex-direction: column; gap: 4px; }
.settings-title { font-size: 18px; font-weight: 700; color: var(--text-1); }
.settings-sub { font-size: 12px; color: var(--text-3); line-height: 1.6; }

.settings-body { display: grid; grid-template-columns: 170px 1fr; gap: var(--s-5); align-items: start; }
.cfg-nav { display: flex; flex-direction: column; gap: 2px; position: sticky; top: 0; }
.cfg-nav-item { display: flex; align-items: center; gap: var(--s-2); padding: 8px 10px; border: none; background: none; border-radius: var(--r-sm); font-size: 12px; font-weight: 500; color: var(--text-3); cursor: pointer; font-family: var(--font-body); text-align: left; transition: all 0.12s; }
.cfg-nav-item svg { width: 16px; height: 16px; flex-shrink: 0; }
.cfg-nav-item:hover { background: var(--surface); color: var(--text-2); }
.cfg-nav-item.active { background: var(--accent-bg); color: var(--accent); font-weight: 600; }

.cfg-content { display: flex; flex-direction: column; gap: var(--s-4); }
.cfg-section { background: var(--surface); border: 1px solid var(--border); border-radius: var(--r-lg); padding: var(--s-5); display: flex; flex-direction: column; gap: var(--s-4); }
.cfg-section-title { font-size: 13px; font-weight: 600; color: var(--text-2); display: flex; align-items: center; gap: 8px; }
.cfg-hint { font-size: 11px; color: var(--text-3); line-height: 1.6; margin: -8px 0 0; }

.cfg-grid { display: grid; grid-template-columns: 1fr 1fr; gap: var(--s-3); }
.cfg-field { display: flex; flex-direction: column; gap: 4px; font-size: 11px; color: var(--text-3); }
.cfg-field.full { grid-column: 1 / -1; }
.cfg-field.disabled { opacity: 0.55; }
.cfg-field input, .cfg-field select, .cfg-field textarea { font-family: var(--font-body); font-size: 13px; color: var(--text-1); padding: 7px 10px; border: 1px solid var(--border); border-radius: var(--r-sm); background: var(--bg); outline: none; transition: border-color 0.12s; resize: vertical; }
.template-tools{display:flex;gap:6px}.template-tools select{flex:1}.template-tools button{padding:6px 9px;border:1px solid var(--border);border-radius:6px;background:var(--surface);color:var(--text-2);font-size:11px;cursor:pointer}.template-tools button:disabled{opacity:.4}
.service-hub{display:grid;gap:10px;padding:13px;border:1px solid color-mix(in oklch,var(--accent) 25%,var(--border));border-radius:10px;background:linear-gradient(135deg,var(--accent-bg),var(--surface))}.service-hub-head{display:flex;align-items:start;justify-content:space-between;gap:16px}.service-hub-head b{font-size:13px}.service-hub-head p{margin-top:3px;font-size:11px;line-height:1.55;color:var(--text-3)}.service-hub-head select{min-width:200px;padding:7px 9px;border:1px solid var(--border);border-radius:7px;background:var(--surface);color:var(--text-1)}.service-hub-actions{display:flex;align-items:center;gap:8px;flex-wrap:wrap}.service-hub-actions button{padding:7px 10px;border:1px solid var(--accent);border-radius:7px;background:var(--surface);color:var(--accent);font-size:11px;font-weight:700;cursor:pointer}.service-hub-actions small{font-size:11px;color:var(--text-3)}
.preset-row,.input-test{display:flex;gap:6px}.preset-row select,.input-test input{flex:1}.preset-row button,.input-test button,.inline-test button{padding:6px 9px;border:1px solid var(--border);border-radius:6px;background:var(--surface);color:var(--text-2);font-size:11px;cursor:pointer;white-space:nowrap}.cfg-field small{font-size:10px;line-height:1.45;color:var(--text-3)}.inline-test{grid-column:1/-1;display:flex;align-items:center;gap:8px;padding:8px 10px;border:1px solid var(--border-soft);border-radius:7px;background:var(--bg);font-size:11px;color:var(--text-3)}.inline-test em{font-style:normal;color:var(--accent)}
.file-picker{display:flex;gap:6px}.file-picker input{flex:1}.file-picker button{padding:6px 10px;border:1px solid var(--border);border-radius:6px;background:var(--surface);cursor:pointer}
.output-rules{display:grid;grid-template-columns:1fr 1fr;gap:12px}
.ai-test-bar{display:flex;align-items:center;gap:10px;padding:9px 11px;border:1px solid var(--border-soft);border-radius:8px;background:var(--bg);font-size:11px;color:var(--text-3)}.ai-test-bar button{margin-left:auto;padding:6px 9px;border:1px solid var(--border);border-radius:6px;background:var(--surface);cursor:pointer;color:var(--text-2)}.ai-test-bar em{font-style:normal}.ai-test-bar em.ok{color:var(--success)}.ai-test-bar em.error{color:var(--error)}
.cfg-field input:focus, .cfg-field select:focus, .cfg-field textarea:focus { border-color: var(--accent); }
.cfg-field.checkbox { flex-direction: row; align-items: center; gap: 8px; font-size: 12px; color: var(--text-2); }
.cfg-field.checkbox input { width: 15px; height: 15px; }

.engine-cards { display: grid; grid-template-columns: repeat(3, 1fr); gap: var(--s-3); }
.engine-card { display: flex; flex-direction: column; gap: 4px; padding: var(--s-3) var(--s-4); background: var(--surface); border: 1px solid var(--border); border-radius: var(--r-md); cursor: pointer; text-align: left; transition: all 0.15s ease; font-family: var(--font-body); }
.engine-card:hover { border-color: var(--accent); background: var(--accent-bg); }
.engine-card.active { border-color: var(--accent); background: var(--accent-bg); box-shadow: 0 0 0 1px var(--accent); }
.engine-name { font-size: 13px; font-weight: 600; color: var(--text-1); }
.engine-desc { font-size: 11px; color: var(--text-3); }

.provider-list { display: flex; flex-direction: column; gap: var(--s-2); margin-bottom: 4px; }
.provider-card { display: flex; align-items: center; justify-content: space-between; padding: var(--s-3) var(--s-4); border: 1px solid var(--border-soft); border-radius: var(--r-md); }
.provider-info { display: flex; flex-direction: column; gap: 2px; }
.provider-name { font-size: 13px; font-weight: 500; color: var(--text-1); }
.provider-status { font-size: 11px; }
.provider-status.ok { color: var(--success); }
.provider-status.warn { color: var(--warning); }

.fmt-checks { display: flex; flex-wrap: wrap; gap: var(--s-4); }
.fmt-check { display: flex; align-items: center; gap: 7px; font-size: 13px; color: var(--text-2); cursor: pointer; }
.fmt-check input { width: 15px; height: 15px; }


.settings-actions { display: flex; align-items: center; gap: var(--s-3); flex-wrap: wrap; }
.run-btn { display: inline-flex; align-items: center; justify-content: center; padding: 10px 18px; border: none; border-radius: var(--r-md); background: var(--accent); color: #fff; font-size: 13px; font-weight: 600; cursor: pointer; font-family: var(--font-body); transition: all 0.15s; }
.run-btn:hover:not(:disabled) { background: var(--accent-hover); }
.run-btn:disabled { opacity: 0.6; cursor: not-allowed; }
.btn-ghost { padding: 10px 16px; border: 1px solid var(--border); border-radius: var(--r-md); background: var(--surface); font-size: 13px; font-weight: 500; color: var(--text-2); cursor: pointer; font-family: var(--font-body); transition: all 0.15s; }
.btn-ghost:hover:not(:disabled) { border-color: var(--accent); color: var(--accent); }
.test-result { font-size: 12px; color: var(--text-2); }

@media (max-width: 760px) {
  .settings-body { grid-template-columns: 1fr; }
  .cfg-nav { flex-direction: row; flex-wrap: wrap; position: static; }
  .cfg-grid { grid-template-columns: 1fr; }
  .engine-cards { grid-template-columns: 1fr; }
  .output-rules { grid-template-columns: 1fr; }
  .service-hub-head { flex-direction: column; }.service-hub-head select{width:100%}
}
</style>
