# Changelog · 变更日志

All notable changes to the **HeritageScribe** (formerly Video2MD) project are documented in this file.

---

## [2.1.0] - 2026-09-12 · Daily Scaffolding Edition (学习日常深化版)

### 🚀 新增功能与深度体验优化 (New Features & UX Enhancements)
- **🔍 讲座长文内快速检索与实时高亮 (In-Document Search & Highlight)**：
  - 针对长达 1-2 小时的学术讲座与家族访谈，结果页预览窗口新增文内极速搜索与匹配计数器（`X 处匹配`）；
  - 自动对长文本中的命中词进行 `<mark class="doc-match">` 高亮渲染，支持一键清空与快速翻寻。
- **🀄 华裔文史拼音字典全面扩容至 80+ 核心高频词**：
  - 全面覆盖：排华法案（1882）、天使岛拘留所、纸儿子、金山客、同乡会、公所会馆、侨乡银信、文史研讨、宗族家谱与传统节日文化；
  - 算法优化：支持按词条长度降序贪婪匹配，彻底根除子词提前替换导致的拼音嵌套损坏问题。
- **📖 新增「东亚文史研讨 (Reading Seminar)」专属场景预设**：
  - 专为大学东亚系研讨课与一手文史史料精读打造；
  - 自动提炼历史文献源流、学者核心学派争鸣、典故隐喻与 Term Paper 论文选题构思。
- **🌐 四重视图模式快速切换 (Quad-View Navigation)**：
  - **标准正文 (Standard)**：保留原始 Markdown 排版；
  - **🀄 拼音注音 (Ruby Pinyin)**：端侧动态为 80+ 华裔文史词汇打上 `<ruby>` 拼音高亮；
  - **🌐 双语对齐 (Parallel Bilingual)**：纯净提取中英分轨对照段落，无缝粘贴至课堂作业或论文写作；
  - **🎯 考前要点 (Cues & Vocab)**：自动抓取 Cornell 思考线索与重点生词速记表。
- **⚡ 表格行级极速操作 (Row-Level Quick Actions)**：
  - 在结果文件列表中新增无需打开预览弹窗的「快速复制」与「闪卡」直提下载按钮，操作流转效率大幅提升。
- **📇 多格式 Anki 闪卡导出**：
  - 支持原生 TSV（Tab-separated，Anki 官方首选）与标准 CSV 格式双通道下载。

---

## [2.0.0] - 2026-09-11 · Heritage Edition (华裔专属升级版)

### 🌟 核心定位跃迁 (Product Repositioning)
- **品牌与定位升级**：从通用的个人音视频转文字工具，全面升级为 **HeritageScribe (声华笔记) —— 专为美国华裔大学生与家庭口述历史打造的本地端侧双语工作台**。
- **开源使命**：聚焦消除华裔二代（Heritage Speakers）在东亚课程中的汉字认读时差，保护第一代移民长辈口述历史零云端泄露。

### 🚀 新增功能 (New Features)
- **🎓 华裔双语课程 Cornell 笔记引擎**：
  - 自动提炼中英双语分轨对照、线索问题（Cue Questions）与学术生词表；
  - 导出直接适配 Obsidian Callouts (`> [!NOTE]`) 与 Notion 双链体系。
- **🀄 华裔语言学习脚手架 (Heritage Scaffolding)**：
  - 新增 HTML `<ruby>` 拼音注音渲染器，在结果预览中一键开启拼音悬浮高亮；
  - 自动识别并保留“金山客”、“同乡会”等特殊移民文化原词与社会历史注解。
- **📇 Anki 闪卡卡片包一键导出 (One-Click Anki Deck Export)**：
  - 智能提取讲座与访谈中的成语、文化隐喻、历史概念为标准 TSV/CSV 格式，支持直接批量导入 Anki。
- **🎙️ 长辈口述历史与家族大事年表 (Oral History & Timeline)**：
  - 专为方言口音与中英混杂（Code-switching）录音调优；
  - 自动提取受访长辈的地理迁移轨迹与时代事件，生成结构化大事年表。
- **🔒 本地端侧绝对隐私加固 (100% On-Device Privacy)**：
  - 全程本地运算，录音文件绝不上云；
  - 密钥存储采用 Windows DPAPI 硬件级加密，确保敏感信息物理级隔离。

### 🎨 界面与体验 (UI & Experience)
- **全新品牌设计**：顶栏升级为 HeritageScribe 品牌标识与 Heritage Edition 专属徽章；
- **场景快捷卡片**：转换工作台新增 4 套华裔场景一键预设卡片；
- **全方位指南弹窗**：新增《华裔双语学习与口述历史数字化指南》。

---

## [1.0.0] - 2026-09-02 · Initial Release (Video2MD MVP)

### 🎉 初始发布 (Initial Release)
- 基于 **Tauri v2 + Rust + Vue 3 + TypeScript** 的轻量跨平台桌面端架构；
- 支持本地音视频文件与网络公开视频导入；
- 支持 Gemini API、Bcut 在线免密钥与自定义 OpenAI 兼容接口转写；
- 支持 Markdown、TXT、SRT、LRC 格式输出与批量下载管理。
