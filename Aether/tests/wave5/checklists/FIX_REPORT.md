# 🔧 P0 Bug 修复报告

> 修复时间：2026-04-03  
> 编译状态：✅ Backend cargo check 0 errors | ✅ Frontend vue-tsc 0 errors  
> 测试状态：✅ Backend 58/58 passed | ✅ Frontend 77/77 passed

---

## 修复总览

| # | 严重级别 | 模块 | 问题 | 状态 |
|---|---------|------|------|------|
| 1 | **P0** | docs.rs | 全部 12 个端点缺失认证 | ✅ 已修复 |
| 2 | **P0** | members.rs | 全部 6 个端点缺失认证 | ✅ 已修复 |
| 3 | **P0** | specs.rs | 全部 2 个端点缺失认证 | ✅ 已修复 |
| 4 | **P0** | stats.rs | 唯一端点缺失认证 | ✅ 已修复 |
| 5 | **P0** | audit.rs | 全部 4 个端点缺失认证 | ✅ 已修复 |
| 6 | **P0** | assets.rs | 3 个端点缺失认证 | ✅ 已修复 |
| 7 | **P0** | members.rs | RBAC 权限矩阵定义但从未执行 | ✅ 已修复 |
| 8 | **P0** | findings.rs | 状态机被 PUT 和 PATCH 绕过 | ✅ 已修复 |
| 9 | **P0** | assets.rs | unlink_asset 调用 delete 而非 unlink | ✅ 已修复 |
| 10 | **P0** | AuditLog.vue | 数据解包 Bug（对象赋给数组 ref） | ✅ 已修复 |
| 11 | **P0** | vrkb.ts API | 3 个函数调用不存在的后端路由 | ✅ 已修复 |
| 12 | **P0** | ImportAnalysisModal | 类型不匹配（ImportSummary vs ImportPreview） | ✅ 已修复 |
| 13 | **P1** | vrkb store | updateFindingStatus 无回滚 | ✅ 已修复 |
| 14 | **P1** | vrkb store | exportProject/importProject 调用错误 API | ✅ 已修复 |
| 15 | **P1** | vrkb store | currentUserPermissions 从不填充 | ✅ 已修复 |
| 16 | **P1** | ExportModal.vue | EventSource 未在卸载时关闭（内存泄漏） | ✅ 已修复 |
| 17 | **P1** | portability store | EventSource 未存储/关闭（内存泄漏） | ✅ 已修复 |
| 18 | **P1** | findings.rs | severity/status/confidence 无验证 | ✅ 已修复 |
| 19 | **P1** | members.rs | 可以删除唯一 owner | ✅ 已修复 |
| 20 | **P1** | docs.rs | author_id 始终为 None | ✅ 已修复 |
| 21 | **P1** | vrkb.ts API | getAuditLog 与 listAuditLogs 重复 | ✅ 已修复 |
| 22 | **P1** | ExportModal.vue | onMounted + watch 双重触发 analyze | ✅ 已修复 |

---

## 详细修改记录

### 1. 后端：6 个模块添加认证（P0）

**文件：`backend/src/interface/api/vrkb/docs.rs`**
- 导入 `AuthenticatedUser`
- 为全部 12 个端点添加 `_user: AuthenticatedUser` 或 `user: AuthenticatedUser`
- `create_doc`: `author_id: None` → `author_id: Some(user.id)`
- `create_from_template`: `author_id: None` → `author_id: Some(user.id)`  
- `generate_report` 中的 doc: `author_id: None` → `author_id: Some(user.id)`

**文件：`backend/src/interface/api/vrkb/members.rs`**
- 导入 `AuthenticatedUser`
- 为全部 6 个端点添加认证
- `add_member`: 添加 `check_permission(&state, &project_id, &user, "manage_members")`
- `remove_member`: 添加权限检查 + 唯一 owner 保护
- `update_member`: 添加权限检查

**文件：`backend/src/interface/api/vrkb/specs.rs`**
- 导入 `AuthenticatedUser`
- 为 `get_specs` 和 `save_spec` 添加认证

**文件：`backend/src/interface/api/vrkb/stats.rs`**
- 导入 `AuthenticatedUser`
- 为 `get_project_stats` 添加认证

**文件：`backend/src/interface/api/vrkb/audit.rs`**
- 导入 `AuthenticatedUser`
- 为全部 4 个端点添加认证
- `create_audit_log`: 自动使用 `user.id` 填充 `actor_id`（若请求未提供）

**文件：`backend/src/interface/api/vrkb/assets.rs`**
- 为 `list_project_assets`, `delete_asset`, `get_asset_usage` 添加认证

### 2. 后端：RBAC 权限矩阵执行（P0）

**文件：`backend/src/interface/api/vrkb/members.rs`**
- 新增 `check_permission()` 异步辅助函数
  - 查找用户在项目中的角色
  - 根据角色查询权限矩阵
  - 返回 403 Forbidden 若权限不足
- `add_member`, `remove_member`, `update_member` 现在都通过 RBAC 检查

### 3. 后端：修复状态机绕过（P0）

**文件：`backend/src/interface/api/vrkb/findings.rs`**
- 新增常量: `VALID_STATUSES`, `VALID_SEVERITIES`, `VALID_CONFIDENCES`
- `update_finding` (PUT): **阻止** 通过 `status` 字段修改状态
  - 返回 400 错误，引导使用 `PATCH /findings/:id/status`
  - 添加 severity 和 confidence 验证
- `update_finding_status` (PATCH): 改用 `repo.transition_finding_status()`
  - 此方法验证状态转换合法性（7 态状态机）
  - 非法转换返回 400 而非 500
- `create_finding`: 添加 severity/status/confidence 验证
  - 自动将 severity 转为小写

### 4. 后端：修复 unlink_asset（P0）

**文件：`backend/src/domain/ports.rs`**
- 新增 trait 方法: `unlink_asset_from_project(project_id, asset_id)`

**文件：`backend/src/infrastructure/persistence/repositories/vrkb.rs`**
- 新增实现: 从 `project_asset` 联接表中删除关联记录
  - 使用 `project_asset::Entity::delete_many()` 配合双过滤条件

**文件：`backend/src/interface/api/vrkb/assets.rs`**
- `unlink_asset`: 从 `repo.delete_asset()` → `repo.unlink_asset_from_project()`
  - 不再删除资产本身，只移除关联关系

### 5. 前端：修复 AuditLog.vue 数据解包（P0）

**文件：`frontend/src/components/self-space/modules/vrkb/views/AuditLog.vue`**
- `entries.value = await vrkbApi.getAuditLog(...)` → 使用 `vrkbApi.listAuditLogs()`
- 正确解包: `entries.value = result.items || []`
- 添加 `AuditEntry` 类型接口
- 字段映射: `entry.action` → `entry.event_type`
- 添加 loading 状态 UI

### 6. 前端：修复 3 个不存在的后端路由（P0）

**文件：`frontend/src/api/vrkb.ts`**
- 删除 `exportProject`（`POST /api/vrkb/projects/${id}/export` — 不存在）
- 删除 `importProject`（`POST /api/vrkb/projects/import` — 不存在）
- 删除 `getActivitySummary`（`GET /api/vrkb/projects/${id}/activity` — 不存在）
- 删除重复的 `getAuditLog`（与 `listAuditLogs` 重复）
- 新增 `markNotificationRead` 函数

### 7. 前端：修复 ImportAnalysisModal 类型不匹配（P0）

**文件：`frontend/src/api/portability.ts`**
- 新增类型: `ImportConflict`, `SuggestedAction`, `ImportPreview`
- `analyzeImport` 返回类型: `ImportSummary` → `ImportPreview`

**文件：`frontend/src/components/portability/ImportAnalysisModal.vue`**
- Props: `summary: ImportSummary | null` → `preview: ImportPreview | null`
- 模板: `summary.total_items` → `preview.summary.total_items`
- 冲突列表: 展示 `ImportConflict` 对象而非字符串

### 8. 前端：修复 vrkb store 多项问题（P1）

**文件：`frontend/src/stores/vrkb.ts`**
- `updateFindingStatus`: 添加乐观更新回滚机制（API 失败时恢复原状态）
- `exportProject`: 改用 `portabilityApi.startExport()`
- `importProject`: 改用 `portabilityApi.startImport()` + 改变签名接受 `projectId`
- `fetchFindings`: 添加 catch 错误处理
- `selectProject`: 加载当前用户权限到 `currentUserPermissions`
- 导入 `portabilityApi`

### 9. 前端：修复 EventSource 内存泄漏（P1）

**文件：`frontend/src/components/portability/ExportModal.vue`**
- 添加 `currentEventSource` 变量存储引用
- `startExport`: 存储 `portabilityApi.connectProgress()` 返回的 EventSource
- 添加 `onUnmounted` 钩子关闭 EventSource
- 修复 `onMounted` + `watch` 双重触发 `analyze()`

**文件：`frontend/src/stores/portability.ts`**
- 添加 `eventSources` Map 存储所有活跃的 EventSource
- `listenToProgress`: 存储 EventSource，关闭已存在的同 taskId 连接
- 完成/错误时从 Map 中移除
- 新增 `cleanup()` 函数关闭所有 EventSource
- `clearTasks`: 清理非运行中任务的 EventSource

---

## 编译与测试结果

```
Backend:  cargo check     → 52 warnings, 0 errors ✅
Backend:  cargo test      → 58 passed, 0 failed ✅
Frontend: vue-tsc --noEmit → 0 errors ✅
Frontend: vitest run       → 77 passed, 0 failed ✅
```

## 修改文件清单

### 后端（8 文件）
1. `backend/src/interface/api/vrkb/docs.rs` — 添加认证 + author_id
2. `backend/src/interface/api/vrkb/members.rs` — 添加认证 + RBAC check_permission + 唯一 owner 保护
3. `backend/src/interface/api/vrkb/specs.rs` — 添加认证
4. `backend/src/interface/api/vrkb/stats.rs` — 添加认证
5. `backend/src/interface/api/vrkb/audit.rs` — 添加认证 + actor_id 自动填充
6. `backend/src/interface/api/vrkb/assets.rs` — 添加认证 + 修复 unlink_asset
7. `backend/src/interface/api/vrkb/findings.rs` — 状态机强制 + 输入验证
8. `backend/src/domain/ports.rs` — 新增 unlink_asset_from_project trait 方法
9. `backend/src/infrastructure/persistence/repositories/vrkb.rs` — 实现 unlink_asset_from_project

### 前端（7 文件）
1. `frontend/src/components/self-space/modules/vrkb/views/AuditLog.vue` — 数据解包修复
2. `frontend/src/api/vrkb.ts` — 删除 3 个不存在路由 + 删除重复函数
3. `frontend/src/api/portability.ts` — 添加 ImportPreview 类型
4. `frontend/src/components/portability/ImportAnalysisModal.vue` — 类型修复
5. `frontend/src/components/portability/ExportModal.vue` — EventSource 清理
6. `frontend/src/stores/vrkb.ts` — 回滚 + 正确 API + 权限加载
7. `frontend/src/stores/portability.ts` — EventSource 生命周期管理
