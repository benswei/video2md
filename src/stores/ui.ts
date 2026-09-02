import { reactive } from 'vue';

export type NoticeKind = 'success' | 'error' | 'info';

export interface UiNotice {
  id: number;
  kind: NoticeKind;
  message: string;
}

export const uiStore = reactive({
  notices: [] as UiNotice[],
});

let nextId = 1;

export function notify(message: unknown, kind: NoticeKind = 'info', timeout = 3600) {
  const text = String(message || '操作未完成');
  const notice = { id: nextId++, kind, message: text };
  uiStore.notices.push(notice);
  window.setTimeout(() => dismissNotice(notice.id), timeout);
}

export function dismissNotice(id: number) {
  const index = uiStore.notices.findIndex((item) => item.id === id);
  if (index >= 0) uiStore.notices.splice(index, 1);
}
