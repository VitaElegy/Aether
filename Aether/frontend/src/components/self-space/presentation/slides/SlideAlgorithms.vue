<template>
  <SlideBase>
    <div class="content">
      <h2 class="section-title">核心算法与机制</h2>
      <p class="hint">点击卡片查看实现细节</p>

      <div class="algo-grid">
        
        <!-- 1. Versioning -->
        <div class="algo-card glass" @click="showDetail('Version')">
            <div class="icon">🌲</div>
            <h3>Git-like Versioning</h3>
            <p>DAG 有向无环图 & 语义哈希</p>
        </div>

        <!-- 2. ReBAC -->
        <div class="algo-card glass" @click="showDetail('ReBAC')">
            <div class="icon">🕸️</div>
            <h3>ReBAC 权限算法</h3>
            <p>基于图遍历的访问控制</p>
        </div>

        <!-- 3. Myers Diff -->
        <div class="algo-card glass" @click="showDetail('Diff')">
            <div class="icon">⚖️</div>
            <h3>Myers' Diff</h3>
            <p>最小编辑距离算法 (O(ND))</p>
        </div>

        <!-- 4. English Engine -->
        <div class="algo-card glass" @click="showDetail('English')">
            <div class="icon">🧠</div>
            <h3>英语知识引擎</h3>
            <p>FST 自动机 & FSRS 记忆算法</p>
        </div>

        <!-- 5. Caching -->
        <div class="algo-card glass" @click="showDetail('Cache')">
            <div class="icon">⚡</div>
            <h3>多级缓存体系</h3>
            <p>Server-side Drafts & LRU</p>
        </div>

      </div>

      <!-- Detail Modal -->
      <Transition name="fade">
        <div v-if="selectedAlgo" class="modal-overlay" @click.self="selectedAlgo = null">
            <div class="modal card glass">
                <div class="modal-header">
                    <h2>{{ selectedAlgo.title }}</h2>
                    <button class="close-btn" @click="selectedAlgo = null">×</button>
                </div>
                <div class="modal-body">
                    <p class="main-desc">{{ selectedAlgo.description }}</p>

                    <div v-if="selectedAlgo.visual" class="visual-box" :class="selectedAlgo.visualClass">
                        <div v-html="sanitizeHtml(selectedAlgo.visual)"></div>
                    </div>

                    <div class="details-list">
                        <div v-for="(item, idx) in selectedAlgo.details" :key="idx" class="detail-row">
                            <span class="bullet">🔹</span>
                            <span class="text">{{ item }}</span>
                        </div>
                    </div>

                    <div v-if="selectedAlgo.code" class="code-snippet">
                        <pre>{{ selectedAlgo.code }}</pre>
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

interface AlgoDetail {
    title: string;
    description: string;
    details: string[];
    visual?: string; // HTML string for simple visualizations
    visualClass?: string;
    code?: string;
}

const selectedAlgo = ref<AlgoDetail | null>(null)

const algoData: Record<string, AlgoDetail> = {
    'Version': {
        title: 'Git-like Versioning (DAG)',
        description: '我们不存储全量副本。每次保存生成一个不可变的 Commit 对象，指向父节点，形成有向无环图 (DAG)。',
        details: [
            '语义哈希 (Semantic Hash): 使用 MD5 对内容计算指纹，仅在内容实质变化时创建版本。',
            '不可变历史: 任何已发布的版本都不可修改，保证绝对的审计追踪能力。',
            '原子性事务: 版本创建与内容更新在同一数据库事务中完成。'
        ],
        visual: `
            <div style="display:flex; gap:10px; justify-content:center; align-items:center;">
                <div style="background:#334155; padding:5px 10px; border-radius:4px;">v1</div>
                <div>⬇️</div>
                <div style="background:#334155; padding:5px 10px; border-radius:4px;">v2</div>
                <div>⬇️</div>
                <div style="background:#334155; padding:5px 10px; border-radius:4px;">v3</div>
            </div>
        `
    },
    'ReBAC': {
        title: 'ReBAC 权限控制算法',
        description: '灵感来自 Google Zanzibar。"检查权限"等同于在社交关系图谱上寻找路径。',
        details: [
            '元组模型: (Object, "relation", User) 定义了基本的边。',
            '递归检查: check(User, Action, Node) -> 映射 Action 为所需关系 (如 "editor") -> 检查直接关系 -> 检查组关系 -> 递归检查父节点 (继承)。',
            '性能优化: 针对 Super Admin (permissions=Max) 的 O(1) 短路检查。'
        ],
        code: `// Recursive Check Logic
async fn check_relation(node, relation, user) {
  // 1. Direct Edge?
  if has_edge(node, relation, user) return true;
  // 2. Group Membership?
  if groups.any(g => has_edge(node, relation, g)) return true;
  // 3. Parent Inheritance?
  if node.parent && check_relation(node.parent, relation, user) return true;
  return false;
}`
    },
    'Diff': {
        title: "Myers' Diff Algorithm",
        description: 'Git 核心差异算法。寻找两个序列之间 "最短编辑脚本" (Shortest Edit Script)。',
        details: [
            'O(ND) 复杂度: N 是序列长度，D 是差异大小。对于常规文本差异极其高效。',
            '行级与字符级: 支持不同粒度的差异计算。',
            '前端可视化: 将 Diff 结果渲染为红 (Delete) / 绿 (Insert) 的直观视图。'
        ],
        visual: `
            <div style="text-align:left; font-family:monospace; padding:10px; background:#1e1e1e; border-radius:5px;">
                <div style="color:#ef4444; background:rgba(239,68,68,0.1)">- old_fn()</div>
                <div style="color:#22c55e; background:rgba(34,197,94,0.1)">+ new_fn()</div>
                <div style="color:#94a3b8">  common()</div>
            </div>
        `
    },
    'English': {
        title: '英语知识引擎算法',
        description: '结合多源异构数据聚合与认知科学算法。',
        details: [
            'FST (Finite State Transducer): 使用 Rust `fst` 库构建词汇索引，实现海量词汇的毫秒级前缀匹配和模糊查询，内存占用极低。',
            'FSRS (Free Spaced Repetition Scheduler): 现代化的间隔重复算法，根据用户的记忆曲线动态安排复习时间。',
            '多源聚合 (Polyglot): 后端并发查询多个在线/离线词库，通过 `join_all` 聚合结果。'
        ]
    },
    'Cache': {
        title: '多级缓存与性能优化',
        description: '为了在保证数据一致性的前提下提供极致性能。',
        details: [
            '服务端草稿 (Server-side Drafts): 分离"自动保存"与"发布"。草稿只存入 Redis/Draft表，不污染主内容库，通过 Debounce 机制减少写入压力。',
            'Moka Cache (Dict): 使用 Rust 高性能缓存库 `moka` (TinyLFU 变体) 缓存高频词典查询结果，热点词汇 0ms 响应。',
            'SeaORM 连接池: 自动管理数据库连接复用。'
        ]
    }
}

const showDetail = (key: string) => {
    selectedAlgo.value = algoData[key] || null
}
</script>

<style scoped>
.content { width: 100%; max-width: 1200px; text-align: center; }
.section-title {
  font-size: 3rem;
  margin-bottom: 0.5rem;
  background: linear-gradient(to right, #f472b6, #fb7185);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.hint {
    color: rgba(255,255,255,0.4);
    margin-bottom: 3rem;
    font-size: 0.9rem;
}

.algo-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 2rem;
    justify-content: center;
}

/* Center the last two items if they are on a new row */
.algo-grid > *:nth-last-child(1):nth-child(3n + 2) {
  grid-column: span 1; 
  /* If we have 5 items in a 3-col grid, item 5 is at 2nd pos of 2nd row. 
     To center 4 and 5, we might need a different flex layout or specific grid manipulation.
     Let's try Flexbox for easier centering of incomplete rows.
  */
}

.algo-grid {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: 2rem;
}

.algo-card {
    width: 300px;
    padding: 2rem;
    border-radius: 1rem;
    background: rgba(255,255,255,0.05);
    border: 1px solid rgba(255,255,255,0.1);
    transition: all 0.3s;
    cursor: pointer;
    display: flex;
    flex-direction: column;
    align-items: center;
}

.algo-card:hover {
    transform: translateY(-5px) scale(1.05);
    background: rgba(255,255,255,0.1);
    box-shadow: 0 10px 30px rgba(0,0,0,0.3);
    border-color: rgba(255,255,255,0.3);
}

.icon {
    font-size: 3rem;
    margin-bottom: 1.5rem;
}

h3 {
    font-size: 1.2rem;
    margin-bottom: 0.5rem;
    color: #fff;
}

p {
    color: rgba(255,255,255,0.6);
    line-height: 1.5;
    font-size: 0.9rem;
}

/* Modal */
.modal-overlay {
    position: fixed;
    top: 0; left: 0; width: 100%; height: 100%;
    background: rgba(0,0,0,0.8);
    backdrop-filter: blur(8px);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 1000;
}

.modal {
    width: 600px;
    max-width: 90%;
    max-height: 85vh;
    padding: 2.5rem;
    background: #111;
    border: 1px solid #333;
    overflow-y: auto;
    text-align: left;
}

.modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 2rem;
    padding-bottom: 1rem;
    border-bottom: 1px solid #333;
}

.modal-header h2 {
    margin: 0;
    font-size: 1.8rem;
    color: #fff;
}

.close-btn {
    background: none; border: none; font-size: 2rem; color: #666; cursor: pointer;
}
.close-btn:hover { color: #fff; }

.modal-body {
    color: #ccc;
}

.main-desc {
    font-size: 1.1rem;
    color: #fff;
    margin-bottom: 2rem;
    line-height: 1.6;
}

.detail-row {
    display: flex;
    gap: 1rem;
    margin-bottom: 1rem;
    align-items: flex-start;
}

.bullet { color: #f472b6; margin-top: 0.2rem; }
.text { font-size: 1rem; line-height: 1.5; color: #bbb; }

.visual-box {
    margin: 1.5rem 0;
    padding: 1.5rem;
    background: rgba(0,0,0,0.3);
    border-radius: 8px;
    border: 1px dashed #444;
}

.code-snippet {
    background: #0d1117;
    padding: 1rem;
    border-radius: 6px;
    margin-top: 2rem;
    border: 1px solid #30363d;
}

.code-snippet pre {
    color: #c9d1d9;
    font-family: monospace;
    font-size: 0.85rem;
    white-space: pre-wrap;
    margin: 0;
}

/* Transitions */
.fade-enter-active, .fade-leave-active { transition: opacity 0.3s; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
</style>
