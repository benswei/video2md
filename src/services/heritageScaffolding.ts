/**
 * HeritageScribe - Heritage Scaffolding & Academic Processing Services
 * 专为美国华裔大学生设计的语言与学术支持脚手架工具集
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
 * 常见华裔学术/文化词汇轻量拼音注音映射表（本地零依赖极速匹配）
 */
const COMMON_HERITAGE_PINYIN_MAP: Record<string, { pinyin: string; en: string }> = {
  '传承': { pinyin: 'chuán chéng', en: 'heritage / to pass on' },
  '华裔': { pinyin: 'huá yì', en: 'ethnic Chinese / Chinese descent' },
  '同乡会': { pinyin: 'tóng xiāng huì', en: 'native-place association / fellow-townsman society' },
  '金山客': { pinyin: 'jīn shān kè', en: 'Gold Mountain travellers / early Chinese immigrants' },
  '唐人街': { pinyin: 'táng rén jiē', en: 'Chinatown' },
  '寻根': { pinyin: 'xún gēn', en: 'root-seeking / exploring ancestry' },
  '代际': { pinyin: 'dài jì', en: 'intergenerational' },
  '坚韧': { pinyin: 'jiān rèn', en: 'resilience / steadfastness' },
  '孝顺': { pinyin: 'xiào shùn', en: 'filial piety' },
  '中庸': { pinyin: 'zhōng yōng', en: 'the golden mean / moderation' },
  '勤勉': { pinyin: 'qín miǎn', en: 'diligence' },
  '拼搏': { pinyin: 'pīn bó', en: 'to struggle hard / entrepreneurial drive' },
  '生计': { pinyin: 'shēng jì', en: 'livelihood' },
  '安身立命': { pinyin: 'ān shēn lì mìng', en: 'to settle down and build a life' },
  '落叶归根': { pinyin: 'luò yè guī gēn', en: 'falling leaves return to their roots' },
  '落地生根': { pinyin: 'luò dì shēng gēn', en: 'taking root in a new land' },
  '语码转换': { pinyin: 'yǔ mǎ zhuǎn huàn', en: 'code-switching' },
  '宗亲': { pinyin: 'zōng qīn', en: 'clan members / relatives' },
  '汇款': { pinyin: 'huì kuǎn', en: 'remittance' },
  '侨胞': { pinyin: 'qiáo bāo', en: 'compatriots living abroad' },
  '茶楼': { pinyin: 'chá lóu', en: 'tea house / dim sum parlor' },
  '公所': { pinyin: 'gōng suǒ', en: 'family association hall / guild' },
};

/**
 * 将文本中的华裔核心词汇自动转换为 HTML <ruby> 拼音注音格式
 * 适合直接粘贴至 Obsidian, Anki, 支持 HTML 的 Markdown 渲染器
 */
export function annotatePinyinRuby(text: string): string {
  let result = text;
  for (const [hanzi, info] of Object.entries(COMMON_HERITAGE_PINYIN_MAP)) {
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

    // 过滤表头
    if (col1.includes('汉字') || col1.includes('---') || col1.includes('Hanzi') || col1.toLowerCase().includes('term')) {
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
          tags: 'HeritageScribe::AutoExtracted'
        });
      }
    }
  }

  return cards;
}

/**
 * 将卡片列表导出为 Anki 兼容的 CSV/TSV 文本
 */
export function generateAnkiCsv(cards: AnkiCard[]): string {
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
 * 格式化为适配 Obsidian Callouts 的康奈尔双语笔记
 */
export function formatAsObsidianCornell(rawTitle: string, markdown: string): string {
  const dateStr = new Date().toISOString().split('T')[0];
  return `---
title: "${rawTitle}"
date: ${dateStr}
category: Heritage Studies / Oral History
tags:
  - AsianAmerican
  - HeritageSpeaker
  - OralHistory
  - BilingualLecture
scaffold: HeritageScribe-v2.0
---

# 📚 ${rawTitle}

> [!NOTE] 华裔双语学习提示 (Heritage Learner Scaffold)
> 本文已由 **HeritageScribe (声华笔记 v2.0)** 完成本地端侧双语分轨对齐与拼音脚手架处理。
> - 音频隐私保障：100% 离线端侧处理，家庭与社区原声零上传。
> - 笔记格式：标准 Cornell Academic Note，支持一键导入 Obsidian / Anki。

---

${markdown}

---
*Generated with [HeritageScribe](https://github.com/microsoftgame/video2md) v2.0 · Local-First Speech-to-Knowledge for Asian-American Heritage Students.*
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
