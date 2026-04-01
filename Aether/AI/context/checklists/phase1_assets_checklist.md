# Phase 1 — Assets 底座 AI Context Checklist

## 变更摘要
Wave 1 实施完成，Assets 从"上传页"升级为"资源中枢"。

## 完成的工作包

### ASSET-01: 扩展资产类型系统
- **前端** `AssetType` 扩展为 7 种：image_asset, pdf_asset, file_asset, ip_asset, domain_asset, credential_stub, snippet_asset
- **前端** `AssetStats` 新增 ip_assets, domain_assets, credential_stubs, snippets 字段
- **前端** `getAssetTypeLabel` / `inferAssetType` / `isStructuredAssetType` 支持新类型
- **前端** `AssetPreviewKind` 新增 `structured` 类型
- **后端** `AssetStats` 对齐前端字段
- **后端** `normalize_asset_type` / `asset_type_for_article` / `increment_asset_stats` 支持新类型
- **后端** 测试更新覆盖新类型

### ASSET-02: 上传流水线增强
- **新建** `AssetUploadQueue.vue` — 拖拽上传区 + 上传队列 UI
- 显示 pending/uploading/done/failed/duplicate 状态
- 上传前 SHA-256 哈希检测重复
- `MyAssets.vue` 集成拖拽区（通过 AssetUploadQueue 包裹）

### ASSET-03: 资产工作台（多视图 + 高级筛选 + 排序）
- **新建** `AssetFiltersBar.vue` — 筛选栏独立组件，含排序下拉 + 视图切换
- **新建** `AssetTable.vue` — 表格视图（名称、类型、大小、MIME、日期、hash、引用数）
- `MyAssets.vue` 重构为 Grid/Table 切换布局
- **后端** `ListAssetsQuery` 新增 `sort_by` 参数（newest | largest | name）
- **后端** 排序逻辑在分页前执行

### ASSET-04: Usage Graph 结构化
- **新建** `AssetUsagePanel.vue` — 替代侧边栏内联引用区块
- 支持点击引用项跳转到来源文章
- 保留现有后端引用追踪逻辑

## 文件清单

| 操作 | 文件 |
|------|------|
| 改 | `frontend/src/api/assets.ts` |
| 改 | `frontend/src/views/apps/MyAssets.vue` |
| 新 | `frontend/src/components/assets/AssetFiltersBar.vue` |
| 新 | `frontend/src/components/assets/AssetTable.vue` |
| 新 | `frontend/src/components/assets/AssetUploadQueue.vue` |
| 新 | `frontend/src/components/assets/AssetUsagePanel.vue` |
| 改 | `backend/src/interface/api/assets.rs` |
| 新 | `frontend/src/test/phase1_assets.test.ts` |
| 新 | `tests/phase1/CHECKLIST.md` |
| 新 | `AI/context/checklists/phase1_assets_checklist.md` |

## 架构决策
- 结构化资产类型（IP、域名、凭据、代码片段）不走 MIME 推断，直接透传 `asset_type`
- 排序由后端和前端双层实现，确保一致性
- 上传队列使用 Web Crypto API 计算 SHA-256 进行客户端重复检测
- AssetUsagePanel 作为独立组件，可在未来被其他视图复用
