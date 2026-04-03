# Phase 0 测试 Checklist
# =====================
# 日期: 2026-04-01
# 范围: PLAT-01 Registry, Capability, Validation / PLAT-02 Shell ErrorBoundary / PLAT-05 Test Infra
#
# 自动化测试文件: frontend/src/test/phase0_full.test.ts (43 个测试用例覆盖 B~N)
# 原有测试文件:   frontend/src/registries/special_kb_registry.test.ts (22 个测试)
# 运行命令:       cd frontend && npm run test:unit
#
# 测试结果: 11 files / 96 tests / 96 passed / 0 failed
#
# 每项格式: [状态] ID: 描述
# 状态: [ ] 待测 / [P] 通过 / [F] 失败
#
# ==========================================
# A. 编译与构建 (Compilation & Build)
# ==========================================
# [P] A1: npm run build 无 TS 编译错误 (built in 1m 1s)
# [P] A2: npm run test:unit 全部通过 (11 files, 96 tests)
# [ ] A3: backend special_kb.rs 语法正确（需 Rust 环境验证）
# [P] A4: 无 dead import 或 unused variable 警告
#
# ==========================================
# B. Registry — 身份解析 (Identity Resolution)
# ==========================================
# [P] B1: 11个 canonical ID 全部注册在 registry 中
# [P] B2: 所有 legacy alias 正确映射到 canonical (16个 alias 覆盖)
# [P] B3: legacy 解析标记 migrated=true, canonical 解析标记 migrated=false
# [P] B4: 未知 renderer 返回 undefined
# [P] B5: null/undefined/空串/纯空格 返回 undefined
# [P] B6: pluginId 解析正确 (10个 canonical 全覆盖)
# [P] B7: layoutId 解析正确 (含显式声明和 fallback 到 canonical)
# [P] B8: dashboardId 解析正确 (vrkb→vulnerability_research, admin→admin_system)
# [P] B9: portabilityProviderId 解析正确 (vocabulary→english_v1, assets→default)
#
# ==========================================
# C. Registry — 规范化 (Normalization)
# ==========================================
# [P] C1: 大写转小写 ("ENGLISH" → "english")
# [P] C2: 前后空格去除 ("  memo  " → "memo")
# [P] C3: 连续空格压缩 ("English  Analysis" → "english analysis")
# [P] C4: 空串/空格串返回 undefined
#
# ==========================================
# D. Registry — Canonical ID Helper
# ==========================================
# [P] D1: legacy → canonical ("english" → "english_v1")
# [P] D2: canonical → canonical ("memo" → "memo")
# [P] D3: null/undefined → "default"
# [P] D4: 未知 → 原样返回 ("my_custom" → "my_custom")
#
# ==========================================
# E. Capability Schema
# ==========================================
# [P] E1: vrkb 有 assets/auditLog/collaboration/search/dashboard，无 articleParser/longTasks
# [P] E2: english_v1 有 articleParser/longTasks/export/import/search，无 collaboration/assets
# [P] E3: legacy alias 也能查到 capabilities ("english" → english_v1 caps)
# [P] E4: 未知 renderer 返回全 false (10个字段全部验证)
# [P] E5: hasCapability 快捷函数正确
# [P] E6: NO_CAPABILITIES 是 Object.freeze 的，不可变 (mutation 抛异常)
# [P] E7: 每个 registry entry 都有 capabilities 字段 (11项全验)
#
# ==========================================
# F. Singleton 追踪
# ==========================================
# [P] F1: singleton set 含 9 个 canonical ID
# [P] F2: legacy ID 不在 singleton set 中 (6个 alias 验证)
# [P] F3: non-singleton (default, article-analysis) 不在 set 中
# [P] F4: isSingletonSpecialKbRenderer 通过 legacy alias 也能检测
#
# ==========================================
# G. 自省 (Introspection)
# ==========================================
# [P] G1: getAllCanonicalRendererIds 返回 11 个 ID
# [P] G2: getAllRegistryEntries 长度 = canonical 数量 = 11
# [P] G3: 每个 entry 都有 canonicalRendererId, pluginId, capabilities, singleton
# [P] G4: getRegistrySize = canonical 数 + legacy 总数
#
# ==========================================
# H. Registry Validation
# ==========================================
# [P] H1: 缺 plugin 时报 error
# [P] H2: 全量注册时 valid=true, errors=[]
# [P] H3: export=true + default provider 时产生 warning
# [P] H4: 所有声明 dashboard capability 的 entry 都有 dashboardId
# [P] H5: 所有 entry 都有 portabilityProviderId（修复了 admin_system 缺失问题）
#
# ==========================================
# I. Resolution 缓存
# ==========================================
# [P] I1: 相同输入连续调用返回同一对象引用（缓存生效 strict ===）
# [P] I1b: 不同 alias key 独立缓存，但解析到相同 canonical
#
# ==========================================
# J. Layout/Dashboard Registry 导出
# ==========================================
# [P] J1: getRegisteredLayoutIds 返回包含 default,math_v1,math_v3,vulnerability_research,english_v1 的 Set
# [P] J2: getRegisteredDashboardIds 返回包含 vulnerability_research,admin_system 的 Set
# [P] J3: main.ts 中 validation 使用动态获取（代码审查确认无硬编码 ID 列表）
#
# ==========================================
# K. Orchestrator 改造
# ==========================================
# [P] K1: getCanonicalRendererId 来自 registry 导入（代码审查确认）
# [P] K2: 无 dead import — eventBus 和 normalizeRendererId 已清除（代码审查确认）
# [P] K3: currentComponent 不再检查 errorState 驱动 BrokenState（代码审查确认）
# [P] K4: 通过 legacy alias 打开 app 仍然正常 (useSelfSpaceOrchestrator.test.ts 验证)
#
# ==========================================
# L. Shell Error Boundary
# ==========================================
# [P] L1: AppErrorBoundary.vue 组件存在且结构正确（代码审查确认）
# [P] L2: onErrorCaptured 捕获子组件错误 → crashed=true（代码审查确认）
# [P] L3: crashed=true 时显示 BrokenState（v-if="crashed" 模板确认）
# [P] L4: crashed=false 时显示 slot 内容（v-else slot 确认）
# [P] L5: kbId 变化时自动 reset crashed 状态（watch 逻辑确认）
# [P] L6: emit('crash') 事件正确传递到父组件（emit 定义确认）
# [P] L7: BrokenState retry 按钮 emit('retry') → @retry="handleRetry" 链确认
# [P] L8: SelfSpaceView 使用 AppErrorBoundary 包裹 module component（模板确认）
# [P] L9: SelfSpaceView onErrorCaptured 只做日志 fallback（return true 确认）
#
# ==========================================
# M. 后端协议描述符 (Backend Protocol Descriptor)
# ==========================================
# [P] M1: KbCapabilities 有 10 个字段（代码审查确认，与前端一致）
# [P] M2: 11 个 KB protocol descriptor 与前端一一对应（代码审查确认）
# [P] M3: OnceLock 缓存逻辑正确（代码审查确认 kb_protocol_map_static）
# [P] M4: get_capabilities 通过 legacy alias 能查到（测试逻辑审查确认）
# [P] M5: is_singleton_renderer 正确（含 alias 解析，测试逻辑审查确认）
# [P] M6: validate_provider_coverage 检测缺失 provider（测试逻辑审查确认）
# [P] M7: validate_provider_coverage 全量通过（测试逻辑审查确认）
# [P] M8: bootstrap/services.rs 启动时执行 coverage 验证（代码审查确认）
# [ ] M*: 后端 cargo test 需 Rust 环境验证（11个 #[test] 函数待执行）
#
# ==========================================
# N. 前后端一致性 (Cross-Layer Consistency)
# ==========================================
# [P] N1: 前后端 canonical renderer ID 列表完全一致 (11个)
# [P] N2: 前后端 capability 字段名 1:1 对应 (10个字段)
# [P] N3: 前后端 singleton 标记完全一致 (9个 singleton)
# [P] N4: 前后端 legacy alias 列表一致（代码审查确认）
# [P] N5: 前后端 provider_id 映射完全一致 (修复了 admin_system 缺失 bug)
#
# ==========================================
# 汇总
# ==========================================
# 自动化测试: 96/96 通过 (11 文件)
# 代码审查:   L 类、K 类、J3 通过人工审查
# 待验证:     A3, M* (需 Rust 编译环境)
# Bug 发现:   admin_system 前端缺少 portabilityProviderId（已修复）
