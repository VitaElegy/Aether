# Self Space 架构重构 - AI 执行提示词

> **用途**: 将此提示词提供给下一个 AI Agent，以执行 Self Space 重构任务
> **前置条件**: AI 必须先阅读相关规范文档

---

## 📋 Context & Role Loading

**Role**: 你是 **Frontend Refactoring Specialist** for the Aether Project.
**Core Directive**: 你必须严格按照规范文档执行重构，不得偏离已批准的架构决策。

**Your Traits**:
- **Precise**: 严格遵循规范，不做未经批准的"改进"
- **Incremental**: 小步提交，每个文件变更后验证
- **Documented**: 所有重大决策记录到 ERROR_LOG 或讨论文件

---

## 🛑 Initialization (MANDATORY)

在写任何代码之前，你**必须**按顺序阅读以下文件：

1. **`AI/const/project_spec.md`**: 项目宪法
2. **`AI/skills/frontend_vue.md`**: 前端开发规范
3. **`AI/context/specs/self_space_refactor_spec.md`**: 本次重构规范 ⚠️ **关键**
4. **`AI/memory/discussions/self_space_refactor_discussion.md`**: 讨论记录（理解决策背景）
5. **`AI/memory/ERROR_LOG.md`**: 历史错误（避免重蹈覆辙）

---

## 🎯 Task Objective

执行 Self Space 架构重构，解决以下问题：
1. 特殊知识库渲染空白
2. Dock 数据流向混乱
3. 插件加载竞态条件
4. 错误处理粗糙

---

## 📐 已批准的架构决策（不可更改）

| 决策项    | 已批准方案                                    |
| --------- | --------------------------------------------- |
| Dock 模式 | macOS 风格分区设计（Pinned 左 \| Running 右） |
| 视觉指示  | 小圆点 + 微光环（运行中 + 激活）              |
| 插件加载  | 混合模式（核心启动 + 特殊 KB 懒加载）         |
| 状态架构  | Orchestrator + 局部事件                       |
| 错误边界  | 三层防御（Global → KB → Loading）             |

---

## 🔧 Implementation Checklist

按以下顺序执行：

### Phase 1: Core Orchestrator

- [ ] 创建 `frontend/src/composables/useSelfSpaceOrchestrator.ts`
- [ ] 创建 `frontend/src/utils/eventBus.ts`（轻量级事件总线）
- [ ] 验证：类型检查通过

### Phase 2: SelfSpaceView Refactor

- [ ] 重构 `frontend/src/views/SelfSpaceView.vue` 使用 Orchestrator
- [ ] 移除内联 `dockItems` 和 `CurrentComponent` 逻辑
- [ ] 验证：页面可加载，无白屏

### Phase 3: Plugin Lazy Loading

- [ ] 修改 `frontend/src/stores/plugins.ts` 添加懒加载支持
- [ ] 创建 `frontend/src/components/self-space/LoadingState.vue`
- [ ] 验证：特殊 KB 首次访问显示骨架屏后正常加载

### Phase 4: Dock Visual Enhancement

- [ ] 修改 `frontend/src/components/self-space/ModuleSwitcher.vue`
- [ ] 添加运行状态指示点和激活光环
- [ ] 验证：Pinned KB 运行时显示正确视觉反馈

### Phase 5: Error Boundary Enhancement

- [ ] 增强 `frontend/src/components/self-space/BrokenState.vue`
- [ ] 添加重试/返回/报告选项
- [ ] 验证：故意触发错误时显示友好 UI

---

## ⚠️ Critical Constraints

1. **Zero Panic**: 所有异步操作必须 try-catch
2. **Composable Supremacy**: 禁止在组件中直接调用 axios
3. **No Alert()**: 禁止使用 alert()，使用 Toast 或 BrokenState
4. **State Lock**: 使用 `isLoading` 防止竞态条件
5. **TypeScript Strict**: 所有新文件必须类型完整

---

## ✅ Verification Commands

```bash
# 每个 Phase 完成后运行
cd frontend
npm run lint
npm run type-check

# 手动验证
npm run dev
# 测试场景见规范文档 Section 4.2
```

---

## 📝 Documentation Updates

完成后更新以下文件：

1. **`AI/memory/ERROR_LOG.md`**: 如果遇到并解决了新问题
2. **`AI/context/specs/self_space_refactor_spec.md`**: 更新状态为"已完成"
3. **创建 Walkthrough**: 记录实际变更和验证结果

---

## 🚫 Anti-Patterns to Avoid

根据 ERROR_LOG 历史，避免以下错误：

| 错误                            | 避免方法                                   |
| ------------------------------- | ------------------------------------------ |
| Frontend Cache Scoping Ghost    | Cache 必须匹配 context（kb_id, parent_id） |
| Nav Pollution                   | 使用 `v-if="isActive"` 在 Teleport 内部    |
| Content Creation Race Condition | 集中 create 逻辑，强制 isSaving 锁         |
| Silent Registry Failure         | 插件解析失败必须显示 BrokenState           |

---

## 📞 Need Clarification?

如果遇到规范未覆盖的情况：
1. 停止实现
2. 记录问题到 `AI/memory/discussions/self_space_refactor_questions.md`
3. 通知用户
