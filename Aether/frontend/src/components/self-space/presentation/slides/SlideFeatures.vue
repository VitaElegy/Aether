<template>
  <SlideBase>
    <div class="content">
      <h2 class="section-title">核心功能特性</h2>
      <p class="hint">点击卡片查看功能详情</p>
      
      <div class="feature-grid">
        
        <!-- 1. Versioning -->
        <div class="feature-box clickable" @click="showDetail('Versioning')">
             <div class="icon">🔍</div>
             <h3>类 Git 版本控制</h3>
             <p>不可变历史 · 语义哈希 · 差异回滚</p>
        </div>

        <!-- 2. Knowledge System (Updated) -->
        <div class="feature-box clickable" @click="showDetail('Knowledge')">
             <div class="icon">📂</div>
             <h3>结构化知识体系</h3>
             <p>多级目录 · 知识库联动 · 标签系统</p>
        </div>

        <!-- 3. Permission -->
        <div class="feature-box clickable" @click="showDetail('ReBAC')">
             <div class="icon">🛡️</div>
             <h3>ReBAC 权限系统</h3>
             <p>细粒度控制 · 继承与共享</p>
        </div>

        <!-- 4. Search -->
        <div class="feature-box clickable" @click="showDetail('Search')">
             <div class="icon">⚡</div>
             <h3>智能搜索与发现</h3>
             <p>全文检索 · 加权排序 · 即时反馈</p>
        </div>

        <!-- 5. English Mastery (New) -->
        <div class="feature-box clickable wide-center" @click="showDetail('English')">
             <div class="icon">🇬🇧</div>
             <h3>沉浸式英语环境</h3>
             <p>多源词典 · FSRS 记忆算法 · 词根词缀</p>
        </div>

      </div>

      <!-- Detail Modal -->
      <Transition name="fade">
        <div v-if="selectedFeature" class="modal-overlay" @click.self="selectedFeature = null">
            <div class="modal card glass">
                <div class="modal-header">
                    <h2>{{ selectedFeature.title }}</h2>
                    <button class="close-btn" @click="selectedFeature = null">×</button>
                </div>
                <div class="modal-body">
                    <p class="main-desc">{{ selectedFeature.description }}</p>
                    
                    <div class="details-list">
                        <div v-for="(item, idx) in selectedFeature.details" :key="idx" class="detail-row">
                            <span class="bullet">✨</span>
                            <span class="text" v-html="highlight(item)"></span>
                        </div>
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
import { sanitizeHtml } from '@/utils/sanitize';
import SlideBase from '../SlideBase.vue'

interface FeatureDetail {
    title: string;
    description: string;
    details: string[];
}

const selectedFeature = ref<FeatureDetail | null>(null)

const featureData: Record<string, FeatureDetail> = {
    'Versioning': {
        title: '类 Git 版本控制系统',
        description: '不仅仅是撤销/重做。我们将软件工程中的"版本控制"理念引入内容创作领域。',
        details: [
            '每次保存生成原子性的 <b>Snapshot</b>，通过语义哈希去重。',
            '支持任意两个历史版本之间的 <b>Diff 比对</b> (Myers 算法)。',
            '完整的 <b>审计日志</b>：谁在什么时候修改了什么，以及为什么修改 (Change Reason)。'
        ]
    },
    'Knowledge': {
        title: '结构化知识矩阵',
        description: '打破扁平化的笔记列表，构建有深度的知识网络。',
        details: [
            '<b>多级层级结构</b>：知识库 (KB) -> 文件夹 (Folders) -> 页面 (Pages)，无限层级嵌套。',
            '<b>强关联性</b>：文章可以直接挂载到知识库，形成体系化的文档结构。',
            '<b>标签系统 (Tags)</b>：正交于目录结构的分类维度，支持多维度筛选。',
            '<b>知识联动</b>：支持双向链接和引用，将孤立的信息点连接成网。'
        ]
    },
    'ReBAC': {
        title: 'ReBAC 关系型权限',
        description: '源自 Google Zanzibar 论文的现代权限模型。',
        details: [
            '权限不是赋予给人的，而是源于人与资源之间的 <b>关系</b> (Relation)。',
            '<b>自动继承</b>：如果你拥有文件夹的权限，你自动拥有其内部所有文档的权限。',
            '<b>细粒度</b>：可以对单个文件、文件夹或整个知识库设置 Viewer/Editor/Owner 权限。'
        ]
    },
    'Search': {
        title: '智能搜索引擎',
        description: '在海量知识中毫秒级定位信息。',
        details: [
            '<b>加权相关性排序</b>：Score = Title match (10x) + Tags match (5x) + Body match (1x)。',
            '<b>混合检索</b>：结合全文检索 (MeiliSearch) 与 数据库精确查询。',
            '<b>即时反馈</b>：支持前缀匹配和模糊搜索，输入即结果。'
        ]
    },
    'English': {
        title: '沉浸式英语环境',
        description: '专为英语学习者打造的深度阅读与记忆辅助系统。',
        details: [
            '<b>本地多源词典</b>：支持部署 StarDict 格式的本地词典，支持多个词典聚合查询。',
            '<b>自定义词汇与例句</b>：不仅仅是查词，更支持对单词进行"重定义"，添加个人笔记、词根词缀和助记例句。',
            '<b>FSRS 记忆算法</b>：集成先进的间隔重复算法 (Free Spaced Repetition Scheduler)，根据你的遗忘曲线智能安排复习。'
        ]
    }
}

const showDetail = (key: string) => {
    selectedFeature.value = featureData[key] || null
}

const highlight = (text: string) => {
    return sanitizeHtml(text) // v-html will handle the <b> tags
}
</script>

<style scoped>
.content { text-align: center; max-width: 1200px; }
.section-title {
  font-size: 3rem;
  margin-bottom: 2rem;
  background: linear-gradient(to right, #f472b6, #fb7185);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.hint {
    color: rgba(255,255,255,0.4);
    margin-bottom: 2rem;
    font-size: 0.9rem;
}

.feature-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 1.5rem;
    padding: 0 4rem;
}

.feature-box {
    background: rgba(255,255,255,0.05);
    padding: 2rem;
    border-radius: 1rem;
    border: 1px solid rgba(255,255,255,0.1);
    transition: all 0.3s;
    cursor: pointer;
    text-align: left;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
}

.feature-box.wide-center {
    grid-column: 1 / -1; /* Span full width */
    align-items: center; /* Center content for the wide one */
    text-align: center;
    background: linear-gradient(135deg, rgba(255,255,255,0.05) 0%, rgba(244, 114, 182, 0.1) 100%);
    border-color: rgba(244, 114, 182, 0.3);
}

.feature-box.wide-center .icon { margin-bottom: 0.5rem; }
.feature-box.wide-center p { max-width: 600px; }

.feature-box:hover {
    transform: translateY(-5px);
    background: rgba(255,255,255,0.1);
    border-color: rgba(255,255,255,0.3);
    box-shadow: 0 10px 30px rgba(0,0,0,0.2);
}

.feature-box.wide-center:hover {
    background: linear-gradient(135deg, rgba(255,255,255,0.1) 0%, rgba(244, 114, 182, 0.2) 100%);
}

.icon {
    font-size: 2.5rem;
    margin-bottom: 1rem;
}

h3 {
    margin-bottom: 0.5rem;
    color: white;
    font-size: 1.4rem;
}

p {
    color: rgba(255,255,255,0.6);
    line-height: 1.5;
}

/* Modal */
.modal-overlay {
    position: fixed; top: 0; left: 0; width: 100%; height: 100%;
    background: rgba(0,0,0,0.8); backdrop-filter: blur(8px);
    display: flex; justify-content: center; align-items: center; z-index: 1000;
}
.modal {
    width: 600px; max-width: 90%; background: #111; border: 1px solid #333;
    padding: 2.5rem; text-align: left;
}
.modal-header {
    display: flex; justify-content: space-between; align-items: center; border-bottom: 1px solid #333;
    padding-bottom: 1rem; margin-bottom: 1.5rem;
}
.modal-header h2 { margin: 0; color: #fff; font-size: 1.8rem; }
.close-btn { background:none; border:none; font-size:2rem; color:#666; cursor:pointer;}
.close-btn:hover { color:#fff; }
.main-desc { font-size: 1.1rem; color: #fff; margin-bottom: 2rem; line-height: 1.6; }
.detail-row { display: flex; gap: 1rem; margin-bottom: 1rem; }
.bullet { color: #f472b6; font-size: 1.2rem; }
.text { color: #ccc; line-height: 1.6; font-size: 1rem; }

/* Transitions */
.fade-enter-active, .fade-leave-active { transition: opacity 0.3s; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
</style>
