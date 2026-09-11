# Changelog · 变更日志

All notable changes to the **HeritageScribe** (formerly Video2MD) project are documented in this file.

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
