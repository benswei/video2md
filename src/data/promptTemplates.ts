export interface PromptTemplate { id: string; title: string; content: string }

export const defaultPromptTemplates: PromptTemplate[] = [
  { id:'extract', title:'干货提炼（默认）', content:'删除寒暄、重复和无效口语，完整保留方法、步骤、案例、数据与结论。以清晰的 Markdown 层级输出可执行的知识笔记。' },
  { id:'meeting', title:'会议纪要', content:'整理为专业会议纪要：议题、关键观点、决策、待办事项、负责人、截止时间及未决问题。不要编造发言人或责任人。' },
  { id:'dialogue', title:'两人讨论', content:'按双方观点和对话脉络整理，去除寒暄与重复；保留分歧、共识、结论和后续行动。无法确认说话人时使用“参与者 A/B”。' },
  { id:'brainstorm', title:'头脑风暴', content:'提取所有创意、问题、假设和可行动建议；按主题归类，区分已确认结论与待验证想法，保留少量有价值的发散信息。' },
  { id:'course', title:'专业课程', content:'将内容整理为严谨的学习讲义：课程目标、概念定义、推导/原理、案例、关键公式或术语、重点总结和复习问题。' },
  { id:'knowledge', title:'知识干货', content:'以读者能直接应用为目标，梳理核心结论、方法框架、操作步骤、常见误区和案例证据；信息充分但表达简洁。' },
  { id:'interview', title:'访谈 / 采访', content:'保留问答逻辑和受访者核心表达，整理背景、关键观点、事实依据、金句和结论；删除重复与无意义口头语。' },
];

export function loadPromptTemplates(): PromptTemplate[] {
  try { return JSON.parse(localStorage.getItem('video2md_prompt_templates') || '') || defaultPromptTemplates; }
  catch { return defaultPromptTemplates; }
}
