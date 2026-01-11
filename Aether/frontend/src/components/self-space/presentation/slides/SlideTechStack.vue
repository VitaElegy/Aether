<template>
  <SlideBase>
    <div class="content">
      <h2 class="section-title">主要技术栈</h2>
      <p class="hint">点击卡片查看详情</p>
      
      <div class="stack-grid">
        
        <div class="stack-section backend">
            <h3>后端 (The Power)</h3>
            <ul>
                <li class="rust clickable" @click="showDetail('Rust')">
                    Rust <span class="badge">内存安全</span>
                </li>
                <li class="clickable" @click="showDetail('Axum')">
                    Axum <span class="badge">异步 Web</span>
                </li>
                <li class="clickable" @click="showDetail('SeaORM')">
                    SeaORM <span class="badge">类型安全 SQL</span>
                </li>
                <li class="clickable" @click="showDetail('Tokio')">
                    Tokio <span class="badge">高并发运行时</span>
                </li>
            </ul>
        </div>

        <div class="stack-section frontend">
            <h3>前端 (The Beauty)</h3>
            <ul>
                <li class="vue clickable" @click="showDetail('Vue 3')">
                    Vue 3 <span class="badge">响应式</span>
                </li>
                <li class="clickable" @click="showDetail('TypeScript')">
                    TypeScript <span class="badge">强类型</span>
                </li>
                <li class="clickable" @click="showDetail('TailwindCSS')">
                    TailwindCSS <span class="badge">实用优先</span>
                </li>
                <li class="clickable" @click="showDetail('Pinia')">
                    Pinia <span class="badge">状态管理</span>
                </li>
            </ul>
        </div>

      </div>

      <!-- Detail Modal -->
      <Transition name="fade">
        <div v-if="selectedTech" class="modal-overlay" @click.self="selectedTech = null">
            <div class="modal card glass">
                <div class="modal-header">
                    <h2>{{ selectedTech.name }}</h2>
                    <button class="close-btn" @click="selectedTech = null">×</button>
                </div>
                <div class="modal-body">
                    <div class="detail-item">
                        <h4>🔍 作用</h4>
                        <p>{{ selectedTech.role }}</p>
                    </div>
                    <div class="detail-item">
                        <h4>💡 选择理由</h4>
                        <p>{{ selectedTech.reason }}</p>
                    </div>
                    <div class="detail-item">
                        <h4>🏗️ 主要应用</h4>
                        <p>{{ selectedTech.usage }}</p>
                    </div>
                </div>
            </div>
        </div>
      </Transition>

    </div>
  </SlideBase>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import SlideBase from '../SlideBase.vue'

interface TechDetail {
    name: string;
    role: string;
    reason: string;
    usage: string;
}

const selectedTech = ref<TechDetail | null>(null)

const techData: Record<string, TechDetail> = {
    'Rust': {
        name: 'Rust',
        role: '核心编程语言',
        reason: '内存安全（无GC）、零成本抽象、甚至比 C++ 更高的并发安全性。',
        usage: '整个后端核心逻辑（领域层、基础设施层）。'
    },
    'Axum': {
        name: 'Axum',
        role: 'Web 框架',
        reason: '基于 Tokio 生态，极其符合人体工程学的宏系统，类型安全，模块化。',
        usage: 'API 接口层 (Interface Layer)，处理路由和 HTTP 请求。'
    },
    'SeaORM': {
        name: 'SeaORM',
        role: 'ORM 框架',
        reason: 'Rust 生态中最好的异步 ORM 之一，提供编译期类型检查，避免 SQL 注入。',
        usage: '基础设施层 (Infrastructure Layer)，负责所有数据库交互。'
    },
    'Tokio': {
        name: 'Tokio',
        role: '异步运行时',
        reason: 'Rust 异步 IO 的事实标准，支持数百万级并发连接。',
        usage: '驱动整个后端服务的异步运行时环境。'
    },
    'Vue 3': {
        name: 'Vue 3',
        role: '前端框架',
        reason: 'Composition API 提供了极佳的代码组织能力，性能优异。',
        usage: '整个前端单页应用 (SPA)的构建。'
    },
    'TypeScript': {
        name: 'TypeScript',
        role: '开发语言',
        reason: '提供静态类型检查，极大减少前端运行时错误，提升重构信心。',
        usage: '前端所有逻辑代码。'
    },
    'TailwindCSS': {
        name: 'TailwindCSS',
        role: 'CSS 框架',
        reason: '实用优先（Utility-first），快速构建 UI，避免样式冲突，易于维护。',
        usage: '所有组件和页面的样式定义。'
    },
    'Pinia': {
        name: 'Pinia',
        role: '状态管理',
        reason: 'Vue 3 官方推荐，轻量、类型安全、去除了复杂的 Mutation 概念。',
        usage: '管理全局状态（如当前用户、文章数据、侧边栏状态）。'
    }
}

const showDetail = (key: string) => {
    selectedTech.value = techData[key] || null
}
</script>

<style scoped>
.content { text-align: center; max-width: 1000px; position: relative; }
.section-title {
  font-size: 3rem;
  margin-bottom: 1rem;
  background: linear-gradient(to right, #4ade80, #2dd4bf);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.hint {
    color: rgba(255,255,255,0.4);
    margin-bottom: 2rem;
    font-size: 0.9rem;
}

.stack-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 4rem;
}

.stack-section {
    text-align: left;
    padding: 2rem;
    border-radius: 1rem;
    background: rgba(255,255,255,0.03);
}

h3 {
    font-size: 1.5rem;
    margin-bottom: 2rem;
    border-bottom: 1px solid rgba(255,255,255,0.1);
    padding-bottom: 1rem;
}

ul {
    list-style: none;
    padding: 0;
}

li.clickable {
    font-size: 1.2rem;
    margin-bottom: 1rem;
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.5rem 1rem;
    border-radius: 0.5rem;
    transition: all 0.2s;
    cursor: pointer;
}

li.clickable:hover {
    background: rgba(255,255,255,0.1);
    transform: translateX(5px);
}

.badge {
    font-size: 0.8rem;
    padding: 0.2rem 0.6rem;
    border-radius: 1rem;
    background: rgba(255,255,255,0.1);
    color: rgba(255,255,255,0.6);
}

.rust { color: #dea584; }
.vue { color: #42b883; }

/* Modal Styles */
.modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background: rgba(0,0,0,0.7);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 100;
    backdrop-filter: blur(5px);
}

.modal {
    width: 500px;
    max-width: 90%;
    background: #1e1e1e; /* Fallback */
    background: rgba(30, 30, 30, 0.95);
    border: 1px solid rgba(255,255,255,0.2);
    border-radius: 1rem;
    padding: 2rem;
    box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.5);
    text-align: left;
}

.modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 2rem;
    border-bottom: 1px solid rgba(255,255,255,0.1);
    padding-bottom: 1rem;
}

.modal-header h2 {
    margin: 0;
    font-size: 2rem;
    background: linear-gradient(to right, #fff, #ccc);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
}

.close-btn {
    background: none;
    border: none;
    color: rgba(255,255,255,0.5);
    font-size: 2rem;
    cursor: pointer;
    line-height: 1;
}

.close-btn:hover {
    color: white;
}

.detail-item {
    margin-bottom: 1.5rem;
}

.detail-item h4 {
    color: rgba(255,255,255,0.5);
    margin-bottom: 0.5rem;
    font-size: 0.9rem;
    text-transform: uppercase;
    letter-spacing: 1px;
}

.detail-item p {
    color: white;
    font-size: 1.1rem;
    line-height: 1.6;
}

/* Transitions */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
