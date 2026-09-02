<script setup lang="ts">
import PrepareView from './PrepareView.vue';
import ConvertView from './ConvertView.vue';
defineEmits<{ 'switch-tab': [tabId: string] }>();
</script>

<template>
  <div class="workspace-view">
    <header class="workspace-head">
      <div><span class="eyebrow">CONVERSION DESK</span><h1>添加素材，选择结果，开始转换</h1></div>
      <p>本地文件、文件夹和在线链接只是不同的添加方式，都会进入下方同一个转换队列。</p>
    </header>
    <PrepareView />
    <ConvertView @switch-tab="$emit('switch-tab', $event)" />
  </div>
</template>

<style scoped>
.workspace-view{display:flex;flex-direction:column;gap:20px;max-width:980px;margin:0 auto;animation:fadeIn .2s ease}.workspace-head{display:flex;align-items:end;justify-content:space-between;gap:32px;padding:4px 2px 2px}.workspace-head h1{font:700 24px/1.25 var(--font-display);letter-spacing:-.025em;margin-top:5px}.workspace-head p{max-width:430px;color:var(--text-3);font-size:12px;line-height:1.7}.eyebrow{font:700 10px var(--font-mono);letter-spacing:.14em;color:var(--accent)}@keyframes fadeIn{from{opacity:0;transform:translateY(4px)}}@media(max-width:700px){.workspace-head{align-items:start;flex-direction:column;gap:8px}}
</style>
