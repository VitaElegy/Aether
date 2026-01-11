<template>
  <SlideBase>
    <div class="content">
      <h2 class="section-title">全 AI 驱动开发流程</h2>
      <p class="subtitle">点击各环节查看 "Human-AI" 协作细节</p>

      <div class="process-container">
        
        <!-- Step 1: Input/Context -->
        <div class="step-group">
            <div class="step-card spec clickable" @click="showDetail('Spec')">
                <div class="icon">📜</div>
                <h4>Constitution</h4>
                <p>project_spec.md</p>
            </div>
            <div class="step-card road clickable" @click="showDetail('Context')">
                <div class="icon">🗺️</div>
                <h4>Context</h4>
                <p>roadmap.md</p>
            </div>
            <div class="plus">+</div>
        </div>

        <div class="arrow">➡️</div>

        <!-- Step 2: The Agent -->
        <div class="agent-core glass clickable" @click="showDetail('Agent')">
            <div class="agent-icon">🤖</div>
            <h3>AI Agent</h3>
            <div class="agent-desc">
                <span class="tag">CoT</span>
                <span class="tag">Role-Play</span>
            </div>
            <p>Senior Architect Persona</p>
        </div>

        <div class="arrow">➡️</div>

        <!-- Step 3: Output & Loop -->
        <div class="step-group vertical">
            <div class="step-card code clickable" @click="showDetail('Output')">
                <div class="icon">💎</div>
                <h4>Delivery</h4>
                <p>High Assurance Code</p>
            </div>
            
            <div class="loop-arrow">⬇️ Feedback</div>

            <div class="step-card error clickable" @click="showDetail('Error')">
                <div class="icon">📚</div>
                <h4>Knowledge</h4>
                <p>ERROR_LOG.md</p>
            </div>
        </div>

      </div>

      <div class="stats glass">
        <div class="stat-item">
            <div class="num">100%</div>
            <div class="label">AI 生成代码</div>
        </div>
        <div class="stat-item">
            <div class="num">32+</div>
            <div class="label">已归档错误案例</div>
        </div>
        <div class="stat-item">
            <div class="num">3</div>
            <div class="label">核心规范文档</div>
        </div>
      </div>

      <!-- Detail Modal -->
      <Transition name="fade">
        <div v-if="selectedStep" class="modal-overlay" @click.self="selectedStep = null">
            <div class="modal card glass">
                <div class="modal-header">
                    <h2>{{ selectedStep.title }}</h2>
                    <button class="close-btn" @click="selectedStep = null">×</button>
                </div>
                <div class="modal-body">
                    <p class="main-desc">{{ selectedStep.description }}</p>
                     <div class="details-list">
                        <div v-for="(item, idx) in selectedStep.details" :key="idx" class="detail-row">
                            <span class="bullet">➤</span>
                            <span class="text">{{ item }}</span>
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
import SlideBase from '../SlideBase.vue'

interface WorkflowDetail {
    title: string;
    description: string;
    details: string[];
}

const selectedStep = ref<WorkflowDetail | null>(null)

const workflowData: Record<string, WorkflowDetail> = {
    'Spec': {
        title: 'Project Constitution (Spec)',
        description: 'AI 不是自由的。它必须遵循 strict constraints。`project_spec.md` 是这个项目的"宪法"。',
        details: [
            '核心指令 (Core Directive): 定义了 AI 的角色是 "Senior Systems Architect"，必须优先考虑系统稳定性。',
            '禁止行为 (Filters): 明确禁止特定的高风险模式（如 Rust panic, 使用不稳定的 SemVer 版本）。',
            'Mandatory Read: AI 在编写任何代码前，必须先调用 read_resource 读取此规范。'
        ]
    },
    'Context': {
        title: 'Context Injection (Roadmap)',
        description: '防止 AI "幻觉" 和重复劳动的关键在于提供正确的 Context。',
        details: [
            '状态同步: `roadmap.md` 提供了当前开发进度的快照，告诉 AI "我们现在在哪里"。',
            '避免冲突: AI 知道哪些功能已经完成，从而避免覆盖已有代码。',
            '上下文窗口优化: 我们不需要喂给 AI 整个代码库，只需喂给它 Roadmap 和 Spec。'
        ]
    },
    'Agent': {
        title: 'AI Agent (The Engine)',
        description: '我们不把 LLM 当作 Copilot，而是当作一个独立的 Agent。',
        details: [
            '思维链 (CoT): 通过 Prompt 强制 AI 在行动前先进行规划 (Task Boundary & Implementation Plan)。',
            '工具使用: Agent 拥有读写文件、运行终端、浏览器测试的全套工具链。',
            'Verify First: 每一段生成的代码都必须伴随一个 verification script (`debug_xxx.sh`)。'
        ]
    },
    'Output': {
        title: 'High Assurance Check',
        description: 'AI 生成的代码不仅要能运行，还要符合工程标准。',
        details: [
            '类型安全: 利用 Rust 的强类型系统作为第一道防线，AI 写的烂代码编译不过。',
            '架构一致性: 强制要求符合六边形架构 (Hexagonal Architecture)，隔离业务逻辑与基础设施。',
            '自我修正: 如果编译失败，Agent 会读取编译器报错并自动修复。'
        ]
    },
    'Error': {
        title: 'Error Log Reflection',
        description: '错误不是失败，而是进化的养料。我们将每一次 System Panic 都记录在案，作为长期记忆注入给 Agent。',
        details: [
            '🔍 <b>Ghost Articles</b>: List/Detail 视图数据不一致 (Context Miss)。',
            '🔄 <b>Auto-Save Loop</b>: 状态机陷入无限重定向 (State Lock)。',
            '🆔 <b>Schema Mismatch</b>: 前端 SemVer vs 后端 Int ID (Type Error)。',
            '👻 <b>Ghost Auto-Publish</b>: 默认值逻辑缺陷导致的意外发布 (Logic Flaw)。',
            '🛠️ <b>Compilation Panic</b>: 大规模重构后的 Trait Bounds 丢失 (Rust Safety)。'
        ]
    }
}

const showDetail = (key: string) => {
    selectedStep.value = workflowData[key] || null
}
</script>

<style scoped>
.content { width: 100%; max-width: 1200px; text-align: center; }

.section-title {
  font-size: 3rem;
  margin-bottom: 0.5rem;
  background: linear-gradient(to right, #a78bfa, #f472b6);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.subtitle {
    font-size: 1.2rem;
    color: rgba(255,255,255,0.6);
    margin-bottom: 3rem;
    letter-spacing: 2px;
}

.process-container {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 2rem;
    margin-bottom: 3rem;
}

.step-group {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    align-items: center;
}

.clickable {
    cursor: pointer;
    transition: all 0.3s;
}

.clickable:hover {
    transform: scale(1.05);
    background: rgba(255,255,255,0.15);
    box-shadow: 0 0 15px rgba(255,255,255,0.1);
}

.step-card {
    background: rgba(255,255,255,0.05);
    border: 1px solid rgba(255,255,255,0.1);
    padding: 1rem 1.5rem;
    border-radius: 0.8rem;
    width: 180px;
}

.spec { border-left: 4px solid #facc15; }
.road { border-left: 4px solid #60a5fa; }
.code { border-left: 4px solid #4ade80; }
.error { border-left: 4px solid #f87171; }

.icon { font-size: 1.5rem; margin-bottom: 0.5rem; }
h4 { margin: 0; font-size: 1rem; color: #fff; }
p { margin: 0.3rem 0 0; font-size: 0.8rem; color: #aaa; font-family: monospace; }

.plus { font-size: 2rem; color: #666; margin: -0.5rem 0; }
.arrow { font-size: 2rem; color: #666; }
.loop-arrow { color: #f87171; font-size: 0.8rem; margin: 0.5rem 0; font-weight: bold; }

.agent-core {
    width: 250px;
    height: 250px;
    border-radius: 50%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    border: 2px solid #818cf8;
    box-shadow: 0 0 30px rgba(129, 140, 248, 0.2);
    position: relative;
    animation: pulse 3s infinite;
}

.agent-icon { font-size: 4rem; margin-bottom: 1rem; }
.agent-core h3 { font-size: 1.5rem; margin: 0 0 0.5rem; color: #fff; }
.agent-core p { font-size: 0.9rem; color: #ccc; max-width: 80%; }

.agent-desc { display: flex; gap: 0.5rem; margin-bottom: 0.5rem; }
.tag { background: rgba(129, 140, 248, 0.3); padding: 2px 8px; border-radius: 10px; font-size: 0.7rem; color: #c3dafe; }

.stats {
    display: flex;
    justify-content: center;
    gap: 4rem;
    padding: 1.5rem;
    margin-top: 2rem;
    border-radius: 1rem;
    background: rgba(255,255,255,0.03);
}

.stat-item .num { font-size: 2.5rem; font-weight: bold; color: #fff; }
.stat-item .label { font-size: 0.9rem; color: #888; text-transform: uppercase; }

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
.modal-header h2 { margin: 0; color: #fff; }
.close-btn { background:none; border:none; font-size:2rem; color:#666; cursor:pointer;}
.close-btn:hover { color:#fff; }
.main-desc { font-size: 1.1rem; color: #fff; margin-bottom: 1.5rem; line-height: 1.6; }
.detail-row { display: flex; gap: 1rem; margin-bottom: 0.8rem; }
.bullet { color: #a78bfa; }
.text { color: #ccc; line-height: 1.5; }

@keyframes pulse {
    0% { box-shadow: 0 0 30px rgba(129, 140, 248, 0.2); border-color: #818cf8; }
    50% { box-shadow: 0 0 50px rgba(129, 140, 248, 0.4); border-color: #a78bfa; }
    100% { box-shadow: 0 0 30px rgba(129, 140, 248, 0.2); border-color: #818cf8; }
}

/* Transitions */
.fade-enter-active, .fade-leave-active { transition: opacity 0.3s; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
</style>

