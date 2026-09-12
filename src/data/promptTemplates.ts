export interface PromptTemplate {
  id: string;
  title: string;
  category?: 'heritage' | 'general';
  badge?: string;
  description?: string;
  content: string;
}

export const defaultPromptTemplates: PromptTemplate[] = [
  {
    id: 'heritage_course',
    title: '🎓 华裔双语课程与东亚讲座 (Heritage Course)',
    category: 'heritage',
    badge: '推荐',
    description: '面向华裔大学生中文/东亚研究课程：康奈尔笔记、中英术语双向对照、重点汉字拼音标注、生词提取与复习思考题。',
    content: `你是一位专为美国华裔大学生服务的双语学术助教。请将本讲座/课程录音整理为标准的【康奈尔双语学术笔记 (Cornell Notes for Heritage Learners)】：
1. 【课程元数据】：课程主题、主讲人、语言特征（如中英夹杂/普通话/方言）、核心知识领域。
2. 【核心概念与线索栏 (Cue Questions)】：提炼 3-5 个核心探讨问题与学术概念。
3. 【双语分轨笔记与拼音脚手架 (Bilingual Notes & Scaffolding)】：
   - 梳理讲座的核心论点、推导与历史/文化背景；
   - 遇到关键中文专业词汇、成语或生僻字，在首次出现时使用拼音注音，格式如：传承 (chuán chéng)；
   - 对重要学术观点提供中英双语精准对照。
4. 【学术生词与术语表 (Vocabulary & Glossary)】：以表格形式整理：| 汉字 (Hanzi) | 拼音 (Pinyin) | English Definition | 讲座语境 (Context) |
5. 【课后复习思考题 (Discussion & Review Questions)】：提供 3 道结合亚裔美国人/东亚跨文化视角的深度思考题。`
  },
  {
    id: 'oral_history',
    title: '🎙️ 华裔长辈与社区口述历史 (Oral History Archive)',
    category: 'heritage',
    badge: '传承',
    description: '专为记录第一代移民父母、长辈及唐人街社区口述史设计：双语分轨、文化词汇保留、家族/社区大事时间轴。',
    content: `你是一位专业的亚裔美国人口述历史学家（Asian American Oral Historian）。这份录音是华裔家庭长辈或社区老人的真实访谈：
1. 【口述历史档案摘要】：受访人代际、出生/移民年代、原籍地（如台山/福州/广东/台湾等）、居住社区、核心叙事主题。
2. 【保留方言与文化特色】：绝不抹杀说话人带有地域特色或中英混杂（Code-switching）的原汁原味表达；对“金山客”、“同乡会”、“餐馆打工”、“汇款单”等具有特殊历史意义的词汇予以保留并加注英文背景解释。
3. 【双语对照口述实录 (Bilingual Parallel Oral History)】：
   - 按时间或话题分段，每一段呈现【中文原声口述】与【精炼地道英译 (English Translation)】对照，方便不会熟练读写汉字的后代子孙阅读。
4. 【家族/社区大事年表 (Milestone Timeline)】：根据访谈中提及的时间，整理出时间线表格（年份、发生事件、地理迁移、历史背景）。
5. 【给下一代的精神寄语 (Legacy & Advice to Descendants)】：提炼长辈对后辈关于坚韧（Resilience）、家庭（Family）、文化认同（Identity）的叮嘱与金句。`
  },
  {
    id: 'code_switching',
    title: '🌐 中英语码转换学术对谈 (Code-Switching Colloquium)',
    category: 'heritage',
    badge: '学术',
    description: '专治研讨会中英混杂发言：去除无效口语，将双语交织发言整理为地道、严密的学术综述。',
    content: `整理中英混杂的学术研讨/研讨会录音：
1. 识别并对齐中英文语码转换（Code-switching），消除口语犹豫（如“那个”、“you know”、“basically”）；
2. 将发散的讨论提炼为逻辑严谨的学术议题综述（Academic Review）；
3. 区分理论模型、实证证据、不同学者的争论焦点与未来研究方向；
4. 输出中英双语的 Executive Summary 及 Action Items。`
  },
  {
    id: 'reading_seminar',
    title: '📖 东亚文史精读与双语研讨 (East Asian Seminar & Text Study)',
    category: 'heritage',
    badge: '研讨',
    description: '专为大学东亚系研讨课与古籍文史精读设计：文言/白话对照、史料背景引申、核心典故注解与思辨批判。',
    content: `你是一位专修东亚文史与亚裔美国人研究（East Asian Studies）的双语学术助教。请将本篇研讨录音/文献讲解整理为深度研讨笔记：
1. 【研讨议题与史料源流】：研讨的核心议题、引用的历史文献或第一手史料（Primary Sources）、主要学派争鸣。
2. 【关键典故与文史概念释读】：
   - 提取研讨中涉及的中文成语、文言句段或历史专有名词；
   - 给出拼音、英文对照、典故出处（Historical Allusion）与在当下讨论中的隐喻义。
3. 【中英双语对齐研讨记录 (Parallel Seminar Debates)】：
   - 记录各方学者的理论论点与反驳，采用中英双语分轨呈现；
   - 对重要原文引述保留中文原貌并附精准学术英译。
4. 【生词与概念闪卡表格】：以 Markdown 表格列出核心生词与学术概念，表头必须为：| 汉字 (Hanzi) | 拼音 (Pinyin) | English Definition | 讲座语境 (Context) |
5. 【延伸研讨与论文选题建议】：结合当今亚裔跨国文化流动，提出 2-3 个可写成 Term Paper 的思辨问题。`
  },
  {
    id: 'anki_vocab',
    title: '📇 Anki 双语生词与闪卡提取 (Anki Flashcards Deck)',
    category: 'heritage',
    badge: '闪卡',
    description: '从音视频中提取高级中文成语、文化隐喻与学术词汇，输出可直接导入 Anki 的闪卡格式。',
    content: `从本音频中提取所有适合华裔学生学习的高级中文词汇、成语、历史文化专有名词：
1. 输出标准 Markdown 表格，表头为：| 汉字 (Hanzi) | 拼音 (Pinyin) | English Definition | 讲座语境 (Context) |
2. 并在最后提供一个代码块（Codeblock），以 CSV/TSV 格式输出可以直接导入 Anki 的闪卡数据：
   格式为：Front (中文+拼音)\tBack (英文释义+例句)\tTags (主题标签)`
  },
  {
    id: 'extract',
    title: '📝 通用干货提炼 (Executive Notes)',
    category: 'general',
    description: '删除寒暄与无效口语，完整保留方法、步骤、案例、数据与结论。以清晰的 Markdown 层级输出可执行知识笔记。',
    content: '删除寒暄、重复和无效口语，完整保留方法、步骤、案例、数据与结论。以清晰的 Markdown 层级输出可执行的知识笔记。'
  },
  {
    id: 'meeting',
    title: '📋 专业会议纪要 (Meeting Minutes)',
    category: 'general',
    description: '议题、关键观点、决策、待办事项、负责人与截止时间。',
    content: '整理为专业会议纪要：议题、关键观点、决策、待办事项、负责人、截止时间及未决问题。不要编造发言人或责任人。'
  },
  {
    id: 'interview',
    title: '🎤 深度人物访谈 (In-Depth Interview)',
    category: 'general',
    description: '保留问答逻辑和受访者核心表达，整理背景、关键观点、事实依据、金句和结论。',
    content: '保留问答逻辑和受访者核心表达，整理背景、关键观点、事实依据、金句和结论；删除重复与无意义口头语。'
  }
];

export function loadPromptTemplates(): PromptTemplate[] {
  try {
    const raw = localStorage.getItem('heritagescribe_prompt_templates') || localStorage.getItem('video2md_prompt_templates');
    if (!raw) return defaultPromptTemplates;
    const parsed = JSON.parse(raw);
    return Array.isArray(parsed) && parsed.length > 0 ? parsed : defaultPromptTemplates;
  } catch {
    return defaultPromptTemplates;
  }
}
