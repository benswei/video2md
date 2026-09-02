<script setup lang="ts">
defineProps<{ projectName?: string }>();
defineEmits<{ 'toggle-sidebar': []; 'toggle-detail': []; 'open-settings': [] }>();
</script>

<template>
  <header class="topbar">
    <button class="menu-btn" @click="$emit('toggle-sidebar')" title="切换侧边栏">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <line x1="3" y1="6" x2="21" y2="6"/><line x1="3" y1="12" x2="21" y2="12"/><line x1="3" y1="18" x2="21" y2="18"/>
      </svg>
    </button>

    <div class="brand">
      <span class="brand-mark">V2</span>
      <span>Video2MD</span>
    </div>

    <span class="project-name" v-if="projectName">{{ projectName }}</span>

    <div class="spacer" />

    <button class="btn" title="配置中心" @click="$emit('open-settings')">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="3"/><path d="M12 1v2m0 18v2M4.22 4.22l1.42 1.42m12.72 12.72 1.42 1.42M1 12h2m18 0h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"/>
      </svg>
      设置
    </button>

    <button class="btn btn-primary" @click="$emit('toggle-detail')" title="查看当前项目详情">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <rect x="3" y="3" width="18" height="18" rx="2"/><line x1="9" y1="3" x2="9" y2="21"/>
      </svg>
      详情
    </button>
  </header>
</template>

<style scoped>
.topbar {
  display: flex;
  align-items: center;
  gap: var(--s-3);
  padding: 0 var(--s-4);
  background: var(--surface);
  border-bottom: 1px solid var(--border);
  height: 100%;
  z-index: 100;
}

.menu-btn {
  display: none;
  width: 32px; height: 32px;
  align-items: center; justify-content: center;
  border: none; background: none;
  border-radius: var(--r-sm); cursor: pointer;
  color: var(--text-2); transition: background 0.15s;
}
.menu-btn:hover { background: var(--surface-2); }
.menu-btn svg { width: 18px; height: 18px; }

.brand {
  display: flex; align-items: center; gap: var(--s-2);
  font-family: var(--font-display);
  font-weight: 700; font-size: 15px;
  letter-spacing: -0.02em;
  color: var(--text-1);
}
.brand-mark {
  width: 26px; height: 26px;
  border-radius: var(--r-sm);
  background: linear-gradient(135deg, var(--accent), oklch(58% 0.14 40));
  display: flex; align-items: center; justify-content: center;
  color: #fff; font-size: 12px; font-weight: 700;
}

.project-name {
  font-size: 12px;
  color: var(--text-3);
  background: var(--surface-2);
  padding: 2px 10px;
  border-radius: 100px;
  font-weight: 500;
  white-space: nowrap;
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.spacer { flex: 1; }

.btn {
  display: inline-flex; align-items: center; gap: 5px;
  padding: 5px 10px;
  border: 1px solid var(--border);
  border-radius: var(--r-sm);
  background: var(--surface);
  font-size: 12px; font-weight: 500;
  color: var(--text-2); cursor: pointer;
  transition: all 0.15s ease;
  font-family: var(--font-body);
  white-space: nowrap;
}
.btn:hover { border-color: var(--neutral-4); background: var(--surface-2); color: var(--text-1); }
.btn svg { width: 14px; height: 14px; }
.btn-primary { background: var(--accent); border-color: var(--accent); color: #fff; }
.btn-primary:hover { background: var(--accent-hover); border-color: var(--accent-hover); color: #fff; }

@media (max-width: 900px) {
  .menu-btn { display: flex; }
  .project-name { display: none; }
  .btn span { display: none; }
}
</style>
