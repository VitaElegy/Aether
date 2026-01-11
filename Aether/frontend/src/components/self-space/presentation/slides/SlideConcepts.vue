<template>
  <SlideBase>
    <div class="content">
      <h2 class="section-title">核心隐喻: 内核与文件系统</h2>
      <p class="hint">点击组件查看设计详情</p>

      <div class="layout">
        <!-- Diagram Side -->
        <div class="diagram-container">
          
          <!-- OS Layer -->
          <div class="layer kernel clickable" @click="showDetail('Kernel')">
            <div class="label">内核态 (Kernel)</div>
            <div class="nodes">
              <div class="node" @click.stop="showDetail('Node')">文件系统 (Nodes)</div>
              <div class="node" @click.stop="showDetail('ReBAC')">权限控制 (ReBAC)</div>
              <div class="node" @click.stop="showDetail('Search')">搜索引擎</div>
            </div>
          </div>

          <div class="connector">
              <div class="arrow">↓</div>
              <div class="arrow">↑</div>
          </div>

          <!-- FS Layer -->
          <div class="layer fs clickable" @click.stop="showDetail('API')">
              <div class="label">系统调用 (System Calls)</div>
          </div>

          <div class="connector">
              <div class="arrow">↓</div>
              <div class="arrow">↑</div>
          </div>

          <!-- User Layer -->
          <div class="layer user clickable" @click.stop="showDetail('UserSpace')">
            <div class="label">用户态 (User Space)</div>
             <div class="apps">
              <div class="app">编辑器</div>
              <div class="app">阅读器</div>
              <div class="app">资源管理器</div>
            </div>
          </div>

        </div>

        <!-- Detail Side Panel (Always visible if space permits, or overlay) -->
        <div class="detail-panel card glass" v-if="selectedFocus">
             <div class="panel-header">
                <h3>{{ selectedFocus.title }}</h3>
                <button class="close-btn" @click="selectedFocus = null">×</button>
             </div>
             <div class="panel-body">
                <p class="desc">{{ selectedFocus.description }}</p>
                
                <div v-if="selectedFocus.sections" class="sections">
                    <div v-for="(sec, idx) in selectedFocus.sections" :key="idx" class="section">
                        <h4>{{ sec.title }}</h4>
                        <p>{{ sec.content }}</p>
                    </div>
                </div>

                <div v-if="selectedFocus.code" class="code-block">
                    <pre>{{ selectedFocus.code }}</pre>
                </div>
             </div>
        </div>
      </div>

    </div>
  </SlideBase>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import SlideBase from '../SlideBase.vue'

interface DetailSection {
    title: string;
    content: string;
}

interface ConceptDetail {
    title: string;
    description: string;
    sections?: DetailSection[];
    code?: string;
}

const selectedFocus = ref<ConceptDetail | null>(null)

const conceptData: Record<string, ConceptDetail> = {
    'Node': {
        title: 'Everything is a Node',
        description: '借鉴 UNIX 文件系统设计，系统中所有实体（文章、备忘录、词汇）都是"节点"。',
        sections: [
            {
                title: '类表继承 (Class Table Inheritance)',
                content: '所有特定类型表 (article_details) 必须通过外键关联到通用的 nodes 表。'
            },
            {
                title: '统一元数据',
                content: 'ID、所有者、权限模式、创建时间等元数据在 Node 层统一管理。'
            }
        ],
        code: `// Rust Entity Relation
#[derive(DeriveEntityModel)]
#[sea_orm(table_name = "nodes")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub node_type: String, // Article, Folder...
    pub permission_mode: String,
}`
    },
    'ReBAC': {
        title: 'ReBAC 权限控制',
        description: '基于关系的访问控制 (Relationship-Based Access Control)，灵感来自 Google Zanzibar。',
        sections: [
            {
                title: '图遍历授权',
                content: '权限不再是简单的列表，而是社交图谱上的路径。'
            },
            {
                title: '细粒度映射',
                content: 'Read = "Viewer" 关系; Write = "Editor" 关系。'
            }
        ],
        code: `// Tuple (Relation)
(Node:123, "viewer", User:Alice)
(Folder:456, "owner", Group:Admins)`
    },
    'Kernel': {
        title: '六边形架构 (内核)',
        description: '后端作为一个纯净的"内核"，不依赖具体的外部实现。',
        sections: [
            {
                title: '领域层 (Domain)',
                content: '纯 Rust 代码，定义核心业务逻辑和端口 (Traits)。'
            },
            {
                title: '基础设施层 (Infra)',
                content: '实现端口的适配器 (Postgres, StarDict)。'
            }
        ]
    },
    'API': {
        title: '系统调用 (Interface Layer)',
        description: 'Axum 处理器充当系统调用接口，处理 HTTP 请求边界。',
        sections: [
            {
                title: 'DTO 转换',
                content: '负责 JSON 与领域模型之间的转换。'
            },
            {
                title: '安全边界',
                content: '第一道防线，处理认证 (JWT) 和基本输入验证。'
            }
        ]
    },
     'UserSpace': {
        title: '用户态 (Frontend)',
        description: 'Vue 3 构建的用户空间，通过 API 与内核交互。',
        sections: [
            {
                title: '状态管理',
                content: 'Pinia 作为用户空间的内存管理器。'
            },
            {
                title: '应用隔离',
                content: '不同的模块 (编辑器、阅读器) 类似于用户态应用程序。'
            }
        ]
    },
    'Search': {
        title: '搜索引擎 (Search Engine)',
        description: 'MeiliSearch 提供即时搜索体验，但在 Self-Space 中我们通过 FST 算法优化了词汇检索。',
        sections: [
            {
                title: 'FST (有限状态转换机)',
                content: '将词汇映射为有限状态自动机。相比 Hash Map，FST 能极大压缩空间（共享前缀/后缀），并支持超快的模糊查询和前缀匹配，非常适合数万词汇量的实时补全。'
            },
            {
                title: 'LRU 缓存 (Least Recently Used)',
                content: '在内存中缓存最近频繁查询的词条。由于语境切换呈现局部性（Locality），LRU 能将 90% 的热点查询（如当前阅读文章的高频词）直接在内存命中，避免昂贵的磁盘/数据库 IO。'
            }
        ],
        code: `// FST Optimization
let map = fst::Map::from_iter(vec![
    ("cat", 1), 
    ("cats", 2),
    ("cation", 3),
])?;

// LRU Cache Eviction
if cache.len() > capacity {
    cache.pop_lru(); // Evict coldest item
}`
    }
}

const showDetail = (key: string) => {
    selectedFocus.value = conceptData[key] || null
}
</script>

<style scoped>
.content { text-align: center; width: 100%; max-width: 1400px; }
.section-title {
  font-size: 3rem;
  margin-bottom: 0.5rem;
  background: linear-gradient(to right, #818cf8, #c084fc);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.hint {
    color: rgba(255,255,255,0.4);
    margin-bottom: 2rem;
    font-size: 0.9rem;
}

.layout {
    display: flex;
    gap: 2rem;
    align-items: flex-start;
    justify-content: center;
    position: relative;
    height: 60vh;
}

.diagram-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1rem;
  flex: 1;
}

.detail-panel {
    flex: 1;
    max-width: 500px;
    height: 100%;
    background: rgba(30, 30, 30, 0.8);
    backdrop-filter: blur(20px);
    border: 1px solid rgba(255,255,255,0.1);
    border-radius: 1rem;
    padding: 2rem;
    text-align: left;
    overflow-y: auto;
    animation: slideIn 0.3s ease-out;
}

.panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1.5rem;
    border-bottom: 1px solid rgba(255,255,255,0.1);
    padding-bottom: 1rem;
}

.panel-header h3 {
    margin: 0;
    font-size: 1.5rem;
    color: #fff;
}

.close-btn {
    background: none;
    border: none;
    color: rgba(255,255,255,0.5);
    font-size: 2rem;
    cursor: pointer;
    line-height: 1;
}

.desc {
    font-size: 1.1rem;
    color: rgba(255,255,255,0.8);
    margin-bottom: 2rem;
    line-height: 1.6;
}

.section {
    margin-bottom: 1.5rem;
}

.section h4 {
    color: #818cf8;
    margin-bottom: 0.5rem;
    font-size: 1rem;
}

.section p {
    color: rgba(255,255,255,0.6);
    font-size: 0.95rem;
}

.code-block {
    background: #111;
    padding: 1rem;
    border-radius: 0.5rem;
    margin-top: 1rem;
    border: 1px solid rgba(255,255,255,0.05);
}

.code-block pre {
    color: #a5b4fc;
    font-family: monospace;
    font-size: 0.8rem;
    overflow-x: auto;
    white-space: pre-wrap;
}


/* Interactive Diagram Styles */
.layer {
    width: 600px;
    padding: 2rem;
    border-radius: 1rem;
    border: 1px solid rgba(255,255,255,0.1);
    position: relative;
    transition: all 0.3s;
    cursor: pointer;
}

.layer:hover {
    transform: scale(1.02);
    box-shadow: 0 0 20px rgba(255,255,255,0.1);
}

.layer.kernel {
    background: rgba(220, 38, 38, 0.1);
    border-color: rgba(220, 38, 38, 0.3);
}

.layer.fs {
    background: rgba(234, 179, 8, 0.1);
    border-color: rgba(234, 179, 8, 0.3);
    padding: 1rem;
}

.layer.user {
    background: rgba(37, 99, 235, 0.1);
    border-color: rgba(37, 99, 235, 0.3);
}

.label {
    position: absolute;
    top: -10px;
    left: 20px;
    background: #000;
    padding: 0 10px;
    font-size: 0.9rem;
    color: rgba(255,255,255,0.6);
    text-transform: uppercase;
    letter-spacing: 2px;
}

.nodes, .apps {
    display: flex;
    justify-content: center;
    gap: 2rem;
}

.node, .app {
    padding: 1rem 2rem;
    background: rgba(255,255,255,0.1);
    border-radius: 0.5rem;
    font-family: monospace;
    transition: all 0.2s;
}

.node:hover, .app:hover {
    background: rgba(255,255,255,0.3);
    transform: translateY(-2px);
}

.connector {
    color: rgba(255,255,255,0.3);
    font-size: 1.5rem;
    display: flex;
    gap: 1rem;
}

@keyframes slideIn {
    from { opacity: 0; transform: translateX(20px); }
    to { opacity: 1; transform: translateX(0); }
}
</style>
