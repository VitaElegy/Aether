<template>
  <SlideBase>
    <div class="cover-content">
      <h1 class="title">Aether</h1>
      <h2 class="subtitle">数字意识平台</h2>
      
      <div class="divider"></div>
      
      <div class="info">
        <p><strong>项目类型:</strong> 课程设计 / 技术分享</p>
        <p><strong>版本:</strong> 1.0.0</p>
      </div>

      <div class="start-hint">
        按 <span class="key">空格</span> 或 <span class="key">→</span> 开始
      </div>
    </div>
  </SlideBase>
</template>

<script setup lang="ts">
import SlideBase from '../SlideBase.vue'
</script>

<style scoped>
.cover-content {
  text-align: center;
  animation: fadeIn 1.5s ease-out;
}

.title {
  font-size: 6rem;
  font-weight: 800;
  margin: 0;
  background: linear-gradient(135deg, #fff 0%, #a5b4fc 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  letter-spacing: -2px;
  text-shadow: 0 10px 30px rgba(0,0,0,0.5);
}

.subtitle {
  font-size: 2rem;
  font-weight: 300;
  color: rgba(255, 255, 255, 0.8);
  margin-top: 1rem;
  letter-spacing: 4px;
  text-transform: uppercase;
}

.divider {
  width: 100px;
  height: 4px;
  background: rgba(255,255,255,0.2);
  margin: 3rem auto;
  border-radius: 2px;
}

.info {
  font-size: 1.2rem;
  color: rgba(255, 255, 255, 0.6);
  line-height: 1.6;
}

.start-hint {
  margin-top: 5rem;
  font-size: 0.9rem;
  color: rgba(255, 255, 255, 0.4);
  opacity: 0;
  animation: pulse 2s infinite 1s;
}

.key {
  background: rgba(255,255,255,0.1);
  padding: 0.2rem 0.6rem;
  border-radius: 4px;
  font-family: monospace;
  color: rgba(255,255,255,0.8);
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(20px); }
  to { opacity: 1; transform: translateY(0); }
}

@keyframes pulse {
  0% { opacity: 0.2; }
  50% { opacity: 0.8; }
  100% { opacity: 0.2; }
}
</style>
