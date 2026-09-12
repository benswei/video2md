/**
 * HeritageScribe - Heritage Scaffolding & Academic Processing Services
 * 专为美国华裔大学生设计的语言与学术支持脚手架工具集 (v2.1 Daily Scaffolding Edition)
 */

export interface GlossaryEntry {
  hanzi: string;
  pinyin: string;
  english: string;
  context?: string;
}

export interface AnkiCard {
  front: string;
  back: string;
  tags: string;
}

/**
 * 华裔学术、历史、文化与日常高频词汇注音字典（本地零依赖极速匹配）
 * 涵盖：亚裔历史与排华法案、宗族与乡社、文化伦理、文史研讨、传统节日与日常用语
 */
export const COMMON_HERITAGE_PINYIN_MAP: Record<string, { pinyin: string; en: string; category?: string }> = {
  // 1. 亚裔历史、移民与历史概念
  '排华法案': { pinyin: 'pái huá fǎ àn', en: 'Chinese Exclusion Act of 1882', category: 'history' },
  '天使岛': { pinyin: 'tiān shǐ dǎo', en: 'Angel Island Immigration Station', category: 'history' },
  '纸儿子': { pinyin: 'zhǐ ér zi', en: 'Paper Son (immigration history)', category: 'history' },
  '金山客': { pinyin: 'jīn shān kè', en: 'Gold Mountain travellers / early Chinese immigrants', category: 'history' },
  '华工': { pinyin: 'huá gōng', en: 'Chinese laborers / railroad workers', category: 'history' },
  '修铁路': { pinyin: 'xiū tiě lù', en: 'transcontinental railroad construction', category: 'history' },
  '苦力': { pinyin: 'kǔ lì', en: 'coolie / indentured laborers (historical)', category: 'history' },
  '唐人街': { pinyin: 'táng rén jiē', en: 'Chinatown', category: 'community' },
  '同乡会': { pinyin: 'tóng xiāng huì', en: 'native-place association / fellow-townsman society', category: 'community' },
  '公所': { pinyin: 'gōng suǒ', en: 'family association hall / guild', category: 'community' },
  '会馆': { pinyin: 'huì guǎn', en: 'clan or regional guild hall', category: 'community' },
  '宗祠': { pinyin: 'zōng cí', en: 'ancestral hall / clan temple', category: 'community' },
  '宗亲': { pinyin: 'zōng qīn', en: 'clan members / relatives', category: 'community' },
  '宗亲会': { pinyin: 'zōng qīn huì', en: 'clan lineage association', category: 'community' },
  '侨胞': { pinyin: 'qiáo bāo', en: 'compatriots living abroad', category: 'community' },
  '侨乡': { pinyin: 'qiáo xiāng', en: 'hometown of overseas Chinese', category: 'community' },
  '汇款': { pinyin: 'huì kuǎn', en: 'remittance', category: 'history' },
  '银信': { pinyin: 'yín xìn', en: 'remittance letters / Qiaopi', category: 'history' },

  // 2. 身份认同、代际与文化传承
  '传承': { pinyin: 'chuán chéng', en: 'heritage / to pass on', category: 'identity' },
  '华裔': { pinyin: 'huá yì', en: 'ethnic Chinese / Chinese descent', category: 'identity' },
  '寻根': { pinyin: 'xún gēn', en: 'root-seeking / exploring ancestry', category: 'identity' },
  '寻根问祖': { pinyin: 'xún gēn wèn zǔ', en: 'tracing ancestral roots', category: 'identity' },
  '落地生根': { pinyin: 'luò dì shēng gēn', en: 'taking root in a new land', category: 'identity' },
  '落叶归根': { pinyin: 'luò yè guī gēn', en: 'falling leaves return to their roots', category: 'identity' },
  '漂洋过海': { pinyin: 'piāo yáng guò hǎi', en: 'crossing vast oceans / immigrating', category: 'identity' },
  '安身立命': { pinyin: 'ān shēn lì mìng', en: 'to settle down and make a life', category: 'identity' },
  '天下一家': { pinyin: 'tiān xià yī jiā', en: 'all under heaven are one family', category: 'identity' },
  '生计': { pinyin: 'shēng jì', en: 'livelihood / daily survival', category: 'life' },
  '代际': { pinyin: 'dài jì', en: 'intergenerational', category: 'sociology' },
  '代沟': { pinyin: 'dài gōu', en: 'generation gap', category: 'sociology' },
  '坚韧': { pinyin: 'jiān rèn', en: 'resilience / steadfastness', category: 'values' },
  '孝顺': { pinyin: 'xiào shùn', en: 'filial piety', category: 'values' },
  '中庸': { pinyin: 'zhōng yōng', en: 'the golden mean / moderation', category: 'values' },
  '勤勉': { pinyin: 'qín miǎn', en: 'diligence', category: 'values' },
  '勤俭持家': { pinyin: 'qín jiǎn chí jiā', en: 'diligent and frugal in household management', category: 'values' },
  '拼搏': { pinyin: 'pīn bó', en: 'to struggle hard / entrepreneurial drive', category: 'values' },
  '望子成龙': { pinyin: 'wàng zǐ chéng lóng', en: 'hope children achieve great success', category: 'values' },
  '光宗耀祖': { pinyin: 'guāng zōng yào zǔ', en: 'bring honor to ancestors', category: 'values' },
  '任劳任怨': { pinyin: 'rèn láo rèn yuàn', en: 'working hard without complaint', category: 'values' },
  '饮水思源': { pinyin: 'yǐn shuǐ sī yuán', en: 'remember the source when drinking water / gratitude', category: 'values' },
  '血脉相连': { pinyin: 'xuè mài xiāng lián', en: 'connected by blood / close kinship', category: 'values' },
  '家和万事兴': { pinyin: 'jiā hé wàn shì xīng', en: 'if family is harmonious, all prosper', category: 'values' },
  '羁绊': { pinyin: 'jī bàn', en: 'emotional bonds / deep ties', category: 'values' },
  '归属感': { pinyin: 'guī shǔ gǎn', en: 'sense of belonging', category: 'values' },
  '寄托': { pinyin: 'jì tuō', en: 'spiritual anchor / hopes placed upon', category: 'values' },

  // 3. 学术对谈、语言学与研讨会
  '语码转换': { pinyin: 'yǔ mǎ zhuǎn huàn', en: 'code-switching (linguistics)', category: 'linguistics' },
  '语言习得': { pinyin: 'yǔ yán xí dé', en: 'language acquisition', category: 'linguistics' },
  '母语': { pinyin: 'mǔ yǔ', en: 'mother tongue / native language', category: 'linguistics' },
  '祖语': { pinyin: 'zǔ yǔ', en: 'heritage language', category: 'linguistics' },
  '方言': { pinyin: 'fāng yán', en: 'topolect / regional dialect', category: 'linguistics' },
  '白话': { pinyin: 'bái huà', en: 'Cantonese / colloquial spoken language', category: 'linguistics' },
  '粤语': { pinyin: 'yuè yǔ', en: 'Cantonese language', category: 'linguistics' },
  '台山话': { pinyin: 'tái shān huà', en: 'Taishanese / Toisan dialect', category: 'linguistics' },
  '闽南语': { pinyin: 'mǐn nán yǔ', en: 'Southern Min / Hokkien / Taiwanese', category: 'linguistics' },
  '客家话': { pinyin: 'kè jiā huà', en: 'Hakka dialect', category: 'linguistics' },
  '文言文': { pinyin: 'wén yán wén', en: 'Classical Chinese', category: 'academics' },
  '典故': { pinyin: 'diǎn gù', en: 'historical allusion / literary reference', category: 'academics' },
  '隐喻': { pinyin: 'yǐn yù', en: 'metaphor', category: 'academics' },
  '综述': { pinyin: 'zōng shù', en: 'academic review / literature synthesis', category: 'academics' },
  '考据': { pinyin: 'kǎo jù', en: 'textual research / evidential scholarship', category: 'academics' },
  '史料': { pinyin: 'shǐ liào', en: 'historical sources / primary materials', category: 'academics' },
  '家谱': { pinyin: 'jiā pǔ', en: 'family tree / genealogy book', category: 'academics' },
  '族谱': { pinyin: 'zú pǔ', en: 'clan pedigree / lineage records', category: 'academics' },

  // 4. 节日、民俗与日常餐饮文化
  '春节': { pinyin: 'chūn jié', en: 'Spring Festival / Lunar New Year', category: 'culture' },
  '除夕': { pinyin: 'chú xī', en: 'Lunar New Year Eve', category: 'culture' },
  '年夜饭': { pinyin: 'nián yè fàn', en: 'reunion dinner on New Year Eve', category: 'culture' },
  '拜年': { pinyin: 'bài nián', en: 'pay New Year visit / extend greetings', category: 'culture' },
  '红包': { pinyin: 'hóng bāo', en: 'red envelope with lucky money', category: 'culture' },
  '压岁钱': { pinyin: 'yā suì qián', en: 'lucky money given to youth', category: 'culture' },
  '元宵节': { pinyin: 'yuán xiāo jié', en: 'Lantern Festival', category: 'culture' },
  '清明节': { pinyin: 'qīng míng jié', en: 'Tomb Sweeping Day / Qingming', category: 'culture' },
  '端午节': { pinyin: 'duān wǔ jié', en: 'Dragon Boat Festival', category: 'culture' },
  '中秋节': { pinyin: 'zhōng qiū jié', en: 'Mid-Autumn Festival', category: 'culture' },
  '重阳节': { pinyin: 'chóng yáng jié', en: 'Double Ninth Festival', category: 'culture' },
  '茶楼': { pinyin: 'chá lóu', en: 'tea house / dim sum parlor', category: 'daily' },
  '饮茶': { pinyin: 'yǐn chá', en: 'yum cha / drinking tea with dim sum', category: 'daily' },
  '早茶': { pinyin: 'zǎo chá', en: 'morning tea and dim sum', category: 'daily' },
  '点心': { pinyin: 'diǎn xīn', en: 'dim sum / snacks', category: 'daily' },
  '功夫茶': { pinyin: 'gōng fu chá', en: 'Gongfu tea brewing ritual', category: 'daily' },
};

/**
 * 将文本中的华裔核心词汇自动转换为 HTML <ruby> 拼音注音格式
 * 适合直接粘贴至 Obsidian, Anki, 支持 HTML 的 Markdown 渲染器
 */
export function annotatePinyinRuby(text: string): string {
  let result = text;
  // 按词长由长到短排序替换，避免子词提前命中破坏长词
  const sortedEntries = Object.entries(COMMON_HERITAGE_PINYIN_MAP).sort((a, b) => b[0].length - a[0].length);
  for (const [hanzi, info] of sortedEntries) {
    const regex = new RegExp(hanzi, 'g');
    result = result.replace(regex, `<ruby>${hanzi}<rt>${info.pinyin}</rt></ruby>`);
  }
  return result;
}

/**
 * 从 Markdown 文本中提取表格或列表中的生词，转换为 Anki 卡片数组
 */
export function parseGlossaryToAnkiCards(markdown: string): AnkiCard[] {
  const cards: AnkiCard[] = [];

  // 1. 尝试匹配 Markdown 表格中带有 汉字、拼音、英语 的行
  // 匹配形如: | 汉字 | 拼音 | 英语 | 语境 |
  const tableRowRegex = /\|\s*([^|\r\n]+?)\s*\|\s*([^|\r\n]+?)\s*\|\s*([^|\r\n]+?)\s*(?:\|\s*([^|\r\n]*?)\s*)?\|/g;
  let match: RegExpExecArray | null;

  while ((match = tableRowRegex.exec(markdown)) !== null) {
    const col1 = match[1].trim();
    const col2 = match[2].trim();
    const col3 = match[3].trim();
    const col4 = (match[4] || '').trim();

    // 过滤表头与分隔符
    if (
      col1.includes('汉字') ||
      col1.includes('---') ||
      col1.includes('Hanzi') ||
      col1.toLowerCase().includes('term') ||
      col1.includes('===')
    ) {
      continue;
    }

    if (col1 && col2 && col3) {
      cards.push({
        front: `${col1} [${col2}]`,
        back: `${col3}${col4 ? `<br><small style="color:gray;">Context: ${col4}</small>` : ''}`,
        tags: 'HeritageScribe::Glossary'
      });
    }
  }

  // 2. 如果未能从表格匹配到足够卡片，则自动扫描内置华裔词汇库
  if (cards.length === 0) {
    for (const [hanzi, info] of Object.entries(COMMON_HERITAGE_PINYIN_MAP)) {
      if (markdown.includes(hanzi)) {
        cards.push({
          front: `${hanzi} [${info.pinyin}]`,
          back: `${info.en}`,
          tags: `HeritageScribe::${info.category || 'Vocabulary'}`
        });
      }
    }
  }

  return cards;
}

/**
 * 将卡片列表导出为 Anki 兼容的格式 (tsv, csv, json)
 */
export function generateAnkiDeck(cards: AnkiCard[], format: 'tsv' | 'csv' | 'json' = 'tsv'): string {
  if (format === 'json') {
    return JSON.stringify(cards, null, 2);
  }

  if (format === 'csv') {
    const escapeCsv = (str: string) => `"${str.replace(/"/g, '""')}"`;
    const header = 'Front,Back,Tags\n';
    const rows = cards.map(c => `${escapeCsv(c.front)},${escapeCsv(c.back)},${escapeCsv(c.tags)}`);
    return header + rows.join('\n');
  }

  // 默认 TSV (Anki 最标准推荐格式)
  const header = '#separator:Tab\n#html:true\n#tags column:3\n';
  const rows = cards.map(c => {
    const front = c.front.replace(/\t/g, ' ');
    const back = c.back.replace(/\t/g, ' ');
    const tags = c.tags;
    return `${front}\t${back}\t${tags}`;
  });
  return header + rows.join('\n');
}

/**
 * 兼容旧版本的导出函数别名
 */
export function generateAnkiCsv(cards: AnkiCard[]): string {
  return generateAnkiDeck(cards, 'tsv');
}

/**
 * 提取双语对照段落为干净的并排纯文本，方便直接引用至论文或作业
 */
export function extractBilingualParallelText(markdown: string): string {
  const lines = markdown.split(/\r?\n/);
  const parallelBlocks: { zh: string; en: string }[] = [];
  let currentZh = '';
  let currentEn = '';

  for (const line of lines) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith('#') || trimmed.startsWith('---')) continue;

    // 匹配如 "【中文】" 或 "【原声】" 或 "中文："
    if (/^(?:【(?:中文|原声|口述)】|ZH:|中文[:：])/i.test(trimmed)) {
      if (currentZh) {
        parallelBlocks.push({ zh: currentZh, en: currentEn });
        currentZh = '';
        currentEn = '';
      }
      currentZh = trimmed.replace(/^(?:【(?:中文|原声|口述)】|ZH:|中文[:：])\s*/i, '');
    } else if (/^(?:【(?:英文|英译|翻译)】|EN:|英文[:：])/i.test(trimmed)) {
      currentEn = trimmed.replace(/^(?:【(?:英文|英译|翻译)】|EN:|英文[:：])\s*/i, '');
    } else if (currentEn) {
      currentEn += ' ' + trimmed;
    } else if (currentZh) {
      currentZh += ' ' + trimmed;
    }
  }

  if (currentZh) {
    parallelBlocks.push({ zh: currentZh, en: currentEn });
  }

  if (parallelBlocks.length === 0) {
    // 若没有结构化标签，返回清理掉 Markdown 标记的自然段落
    return markdown
      .replace(/<ruby>(.*?)<rt>.*?<\/rt><\/ruby>/g, '$1')
      .replace(/\[\^.*?\]/g, '')
      .replace(/[#*`>]/g, '');
  }

  return parallelBlocks
    .map((b, idx) => `[§${idx + 1}]\n🇨🇳 中文: ${b.zh}\n🇺🇸 英文: ${b.en || '（无对应英译）'}\n`)
    .join('\n');
}

/**
 * 提取康奈尔线索问题与考前复习精要
 */
export function extractCornellCuesAndSummary(markdown: string): string {
  const lines = markdown.split(/\r?\n/);
  const cues: string[] = [];
  const vocab: string[] = [];
  let inVocab = false;

  for (const line of lines) {
    const trimmed = line.trim();
    if (/^(?:[-*]|\d+\.)\s*(.*(?:\?|？|何为|探讨|为什么|如何).*)/.test(trimmed)) {
      cues.push(trimmed);
    }
    if (trimmed.includes('生词') || trimmed.includes('术语') || trimmed.includes('Glossary')) {
      inVocab = true;
      continue;
    }
    if (inVocab && trimmed.startsWith('|') && !trimmed.includes('---') && !trimmed.includes('汉字')) {
      vocab.push(trimmed);
    }
  }

  return [
    '# 🎯 考前复习核心线索 (Cue Questions & High-Yield Review)',
    '',
    '## 📌 核心探讨问题 (Discussion Cues)',
    cues.length ? cues.join('\n') : '- 暂未从文中提取到独立思考题，可回看完整笔记。',
    '',
    '## 📚 重点生词速记 (Glossary Flash-List)',
    vocab.length ? vocab.slice(0, 15).join('\n') : '- 词汇表可使用「导出 Anki 闪卡」查看全量。'
  ].join('\n');
}

/**
 * 格式化为适配 Obsidian Callouts 的康奈尔双语笔记 (v2.1 增强元数据)
 */
export function formatAsObsidianCornell(rawTitle: string, markdown: string): string {
  const dateStr = new Date().toISOString().split('T')[0];
  const cleanTitle = rawTitle.replace(/\.[^/.]+$/, '');
  return `---
title: "${cleanTitle}"
date: ${dateStr}
category: Heritage Studies / Oral History
tags:
  - AsianAmerican
  - HeritageSpeaker
  - OralHistory
  - BilingualLecture
  - CornellNotes
scaffold: HeritageScribe-v2.1
---

# 📚 ${cleanTitle}

> [!NOTE] 华裔双语学习提示 (Heritage Learner Scaffold · v2.1)
> 本文已由 **HeritageScribe (声华笔记 v2.1)** 完成本地端侧双语分轨对齐与拼音脚手架处理。
> - **音频隐私保障**：100% 离线端侧处理，家庭与社区口述历史原声零上传。
> - **笔记格式**：标准 Cornell Academic Note，支持一键导入 Obsidian / Anki。
> - **复习支持**：生词已打标签，支持随时导出 TSV/CSV 卡片包。

---

${markdown}

---
*Generated with [HeritageScribe](https://github.com/microsoftgame/video2md) v2.1 · Local-First Speech-to-Knowledge for Asian-American Heritage Students.*
`;
}

/**
 * 触发浏览器本地下载文本文件
 */
export function downloadFile(filename: string, content: string, mimeType = 'text/plain;charset=utf-8'): void {
  const blob = new Blob([content], { type: mimeType });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}

