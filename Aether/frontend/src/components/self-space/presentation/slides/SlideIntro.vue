<template>
  <SlideBase>
    <div class="content">
      <h2 class="section-title">为什么开发 Aether?</h2>
      <div class="cards">
        <div class="card glass">
          <div class="card-icon">🌪️</div>
          <h3>混乱现状</h3>
          <p>传统的知识管理工具碎片化严重。笔记在这里，文件在那里，缺乏统一的结构与关联。</p>
        </div>
        <div class="card glass">
          <div class="card-icon">⏱️</div>
          <h3>历史遗失</h3>
          <p>没有版本控制。你编辑，你保存，你失去了过去。“撤销”不是真正的版本控制系统。</p>
        </div>
        <div class="card glass accent">
          <div class="card-icon">⚡</div>
          <h3>解决方案</h3>
          <p>一个结构化的、内置版本控制的数字意识平台。</p>
        </div>
      </div>
    </div>
  </SlideBase>
</template>

<script setup lang="ts">
import SlideBase from '../SlideBase.vue'
</script>

<style scoped>
.content {
  width: 100%;
  max-width: 1200px;
  text-align: center;
}

.section-title {
  font-size: 3rem;
  margin-bottom: 4rem;
  font-weight: 700;
  background: linear-gradient(to right, #fff, #99f);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.cards {
  display: flex;
  justify-content: center;
  gap: 2rem;
}

.card {
  flex: 1;
  padding: 2.5rem;
  border-radius: 1rem;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  transition: all 0.3s ease;
  cursor: default;
}

.card:hover {
  transform: translateY(-10px);
  background: rgba(255, 255, 255, 0.1);
  box-shadow: 0 20px 40px rgba(0,0,0,0.5);
}

.card.accent {
  border-color: rgba(100, 100, 255, 0.5);
  background: rgba(100, 100, 255, 0.1);
}

.card-icon {
  font-size: 3rem;
  margin-bottom: 1.5rem;
}

h3 {
  font-size: 1.5rem;
  margin-bottom: 1rem;
  color: #fff;
}

p {
  color: rgba(255, 255, 255, 0.7);
  line-height: 1.6;
}
</style>
