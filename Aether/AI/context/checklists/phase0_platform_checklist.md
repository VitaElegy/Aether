# Phase 0: 平台收口 — 测试验收 Checklist

> **状态**: ✅ 已完成 (2026-04-01)
> **用途**: 任何 AI 后续修改 Phase 0 涉及的文件时，必须重跑此 checklist 保证不回归。
> **运行**: `cd frontend && npm run test:unit`
> **测试文件**: `frontend/src/test/phase0_full.test.ts` (43项) + `frontend/src/registries/special_kb_registry.test.ts` (22项)

---

## 涉及的核心文件

| 文件 | 职责 |
|------|------|
| `frontend/src/registries/special_kb_registry.ts` | 前端 KB 身份注册表 + Capability Schema + Validation |
| `frontend/src/registries/read_layout_registry.ts` | Layout/Dashboard 注册表 + ID 导出 |
| `frontend/src/composables/useSelfSpaceOrchestrator.ts` | Shell 编排器（KB 切换、Dock 管理） |
| `frontend/src/views/SelfSpaceView.vue` | Shell 视图层（ErrorBoundary 集成） |
| `frontend/src/components/self-space/AppErrorBoundary.vue` | 模块级错误边界 |
| `frontend/src/components/self-space/BrokenState.vue` | 错误态 UI |
| `frontend/src/main.ts` | 启动注册 + DEV 校验 |
| `backend/src/domain/special_kb.rs` | 后端 KB 协议描述符 + Capability + Normalization |
| `backend/src/infrastructure/bootstrap/services.rs` | 启动 Provider 覆盖率校验 |

---

## A. 编译与构建

| ID | 描述 | 验证方式 |
|----|------|---------|
| A1 | `npm run build` 无 TS 编译错误 | 脚本 |
| A2 | `npm run test:unit` 全部通过 | 脚本 (96/96) |
| A3 | `cargo test` 后端测试通过 | 脚本 (需 Rust 环境) |
| A4 | 无 dead import 或 unused variable | 构建日志 |

## B. Registry 身份解析

| ID | 描述 | 测试用例 |
|----|------|---------|
| B1 | 11 个 canonical ID 全部注册 | `phase0_full: B1` |
| B2 | 16 个 legacy alias 正确映射 | `phase0_full: B2` |
| B3 | migrated flag: legacy=true, canonical=false | `phase0_full: B3` |
| B4 | 未知 renderer → undefined | `phase0_full: B4` |
| B5 | null/undefined/空串/空格 → undefined | `phase0_full: B5` |
| B6 | pluginId 解析 (10个 canonical 全覆盖) | `phase0_full: B6` |
| B7 | layoutId 解析 (含显式 + fallback) | `phase0_full: B7` |
| B8 | dashboardId 解析 | `phase0_full: B8` |
| B9 | portabilityProviderId 解析 | `phase0_full: B9` |

## C. 规范化

| ID | 描述 | 测试用例 |
|----|------|---------|
| C1 | 大写 → 小写 | `phase0_full: C1` |
| C2 | 前后空格去除 | `phase0_full: C2` |
| C3 | 连续空格压缩 | `phase0_full: C3` |
| C4 | 空串/空格 → undefined | `phase0_full: C4` |

## D. Canonical ID Helper

| ID | 描述 | 测试用例 |
|----|------|---------|
| D1 | legacy → canonical | `phase0_full: D1` |
| D2 | canonical 透传 | `phase0_full: D2` |
| D3 | null/undefined → "default" | `phase0_full: D3` |
| D4 | 未知 → normalized 原样 | `phase0_full: D4` |

## E. Capability Schema

| ID | 描述 | 测试用例 |
|----|------|---------|
| E1 | vrkb: assets/auditLog/collaboration/search/dashboard=true | `phase0_full: E1` |
| E2 | english_v1: articleParser/longTasks/export/import/search=true | `phase0_full: E2` |
| E3 | legacy alias 也能查到 capabilities | `phase0_full: E3` |
| E4 | 未知 renderer → 10 个字段全 false | `phase0_full: E4` |
| E5 | hasCapability 快捷函数正确 | `phase0_full: E5` |
| E6 | NO_CAPABILITIES 不可变 (Object.freeze) | `phase0_full: E6` |
| E7 | 每个 entry 都有 capabilities | `phase0_full: E7` |

## F. Singleton

| ID | 描述 | 测试用例 |
|----|------|---------|
| F1 | singleton set 含 9 个 canonical ID | `phase0_full: F1` |
| F2 | legacy ID 不在 set 中 | `phase0_full: F2` |
| F3 | non-singleton 不在 set 中 | `phase0_full: F3` |
| F4 | isSingletonSpecialKbRenderer 通过 alias 检测 | `phase0_full: F4` |

## G. 自省

| ID | 描述 | 测试用例 |
|----|------|---------|
| G1 | getAllCanonicalRendererIds 返回 11 个 | `phase0_full: G1` |
| G2 | getAllRegistryEntries 长度 = 11 | `phase0_full: G2` |
| G3 | 每个 entry 有 canonicalRendererId/pluginId/capabilities/singleton | `phase0_full: G3` |
| G4 | getRegistrySize = canonical数 + legacy总数 | `phase0_full: G4` |

## H. Validation

| ID | 描述 | 测试用例 |
|----|------|---------|
| H1 | 缺 plugin → error | `phase0_full: H1` |
| H2 | 全量注册 → valid=true | `phase0_full: H2` |
| H3 | export=true + default provider → warning | `phase0_full: H3` |
| H4 | dashboard cap 的 entry 都有 dashboardId | `phase0_full: H4` |
| H5 | 所有 entry 都有 portabilityProviderId | `phase0_full: H5` |

## I. Resolution 缓存

| ID | 描述 | 测试用例 |
|----|------|---------|
| I1 | 相同输入返回同一对象引用 (===) | `phase0_full: I1` |
| I1b | 不同 alias key 独立缓存但同 canonical | `phase0_full: I1b` |

## J. Layout/Dashboard Registry 导出

| ID | 描述 | 验证方式 |
|----|------|---------|
| J1 | getRegisteredLayoutIds 正确 | 代码审查 |
| J2 | getRegisteredDashboardIds 正确 | 代码审查 |
| J3 | main.ts 使用动态获取（无硬编码） | 代码审查 |

## K. Orchestrator

| ID | 描述 | 验证方式 |
|----|------|---------|
| K1 | getCanonicalRendererId 来自 registry 导入 | 代码审查 |
| K2 | 无 dead import | 代码审查 |
| K3 | currentComponent 不再用 errorState 驱动 BrokenState | 代码审查 |
| K4 | legacy alias 打开 app 正常 | `useSelfSpaceOrchestrator.test.ts` |

## L. Shell Error Boundary

| ID | 描述 | 验证方式 |
|----|------|---------|
| L1 | AppErrorBoundary.vue 存在且结构正确 | 代码审查 |
| L2 | onErrorCaptured → crashed=true | 代码审查 |
| L3 | crashed=true → BrokenState | 模板 v-if |
| L4 | crashed=false → slot | 模板 v-else |
| L5 | kbId 变化 → reset crashed | watch 逻辑 |
| L6 | emit('crash') 传递到父 | emit 定义 |
| L7 | retry 按钮 → boundary reset | @retry 链 |
| L8 | SelfSpaceView 用 AppErrorBoundary 包裹 | 模板确认 |
| L9 | SelfSpaceView onErrorCaptured 只做日志 | return true |

## M. 后端协议描述符

| ID | 描述 | 验证方式 |
|----|------|---------|
| M1 | KbCapabilities 10 个字段 | 代码审查 |
| M2 | 11 个 descriptor 与前端一致 | 代码审查 |
| M3 | OnceLock 缓存 | 代码审查 |
| M4 | get_capabilities 通过 alias 查询 | `#[test] capabilities_query_returns_correct_flags` |
| M5 | is_singleton_renderer 正确 | `#[test] singleton_detection_works` |
| M6 | validate_provider_coverage 检测缺失 | `#[test] provider_coverage_validation_catches_missing` |
| M7 | validate_provider_coverage 全量通过 | `#[test] provider_coverage_passes_when_all_registered` |
| M8 | bootstrap 启动时执行校验 | 代码审查 |

## N. 前后端一致性

| ID | 描述 | 测试用例 |
|----|------|---------|
| N1 | canonical ID 列表一致 (11个) | `phase0_full: N1` |
| N2 | capability 字段 10 个，名称对应 | `phase0_full: N2` |
| N3 | singleton 标记一致 (9个) | `phase0_full: N3` |
| N4 | legacy alias 列表一致 | 代码审查 |
| N5 | provider_id 映射一致 (含 admin_system 修复) | `phase0_full: N5` |

---

## 已发现并修复的 Bug

| # | 描述 | 发现方式 |
|---|------|---------|
| 1 | `admin_system` 前端缺少 `portabilityProviderId: 'default'` — 前后端 provider 映射不一致 | N5 测试自动发现 |

---

## 如何使用此 Checklist

1. **修改了上述任何文件** → 跑 `cd frontend && npm run test:unit`，确认 96/96
2. **新增 special KB** → 在前后端 registry 都加入 entry，重跑 B1/G1/N1 确认数量+1
3. **新增 capability 字段** → 前后端都加，重跑 E/N2 确认字段数+1
4. **新增 legacy alias** → 前后端都加，重跑 B2/G4 确认
5. **修改 portability provider** → 重跑 B9/N5/H3 确认
