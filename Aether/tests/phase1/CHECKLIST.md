# Phase 1 — Assets 底座 测试 Checklist

## 编译验证
- [ ] `npm run build` 编译通过（0 error）
- [ ] `npm run test:unit` 全量通过
- [ ] Phase 0 测试不回归（phase0_full.test.ts 仍绿）

## ASSET-01: 扩展资产类型系统
- [ ] `AssetType` 包含 7 种类型：image_asset, pdf_asset, file_asset, ip_asset, domain_asset, credential_stub, snippet_asset
- [ ] `getAssetTypeLabel` 为所有 7 种类型返回人类可读标签
- [ ] `inferAssetType` 对结构化类型（ip/domain/credential/snippet）透传 asset_type
- [ ] `isStructuredAssetType` 正确区分结构化 vs 文件类资产
- [ ] `AssetStats` 前后端对齐，包含 ip_assets, domain_assets, credential_stubs, snippets
- [ ] 后端 `normalize_asset_type` 接受新类型
- [ ] 后端 `asset_type_for_article` 识别新类型
- [ ] 后端 `increment_asset_stats` 正确计数新类型
- [ ] `FILTER_OPTIONS` 包含所有 7 种类型 + all
- [ ] `inferPreviewKind` 对新类型返回 `structured`

## ASSET-02: 上传流水线增强
- [ ] `AssetUploadQueue.vue` 组件存在且可渲染
- [ ] 拖拽区显示 drop overlay
- [ ] 上传队列显示文件名、状态（pending/uploading/done/failed/duplicate）
- [ ] 重复 hash 检测：上传前 SHA-256 → 查询 API → duplicate 状态
- [ ] `MyAssets.vue` 集成 drag-and-drop（dragover/drop 事件）
- [ ] 上传完成后自动刷新列表

## ASSET-03: 多视图工作台
- [ ] `AssetFiltersBar.vue` 组件存在且可渲染
- [ ] 排序下拉：newest / largest / name
- [ ] 视图切换按钮：grid / table
- [ ] `AssetTable.vue` 组件存在且可渲染
- [ ] 表格列：名称、类型、大小、MIME、日期、hash 截断、引用数
- [ ] `MyAssets.vue` 支持 grid/table 切换
- [ ] 后端 `ListAssetsQuery` 支持 `sort_by` 参数
- [ ] 排序逻辑覆盖测试：newest → 时间倒序, largest → 大小倒序, name → 字母序

## ASSET-04: Usage Graph 结构化
- [ ] `AssetUsagePanel.vue` 组件存在且可渲染
- [ ] 引用面板显示引用计数 + 引用列表
- [ ] 引用项可点击跳转到来源文章
- [ ] 替代原有 MyAssets.vue 中内联的 "Used In" 区块

## 测试覆盖
- [ ] phase1_assets.test.ts 包含 ASSET-01 类型扩展测试
- [ ] phase1_assets.test.ts 包含 ASSET-01 payload extraction 测试
- [ ] phase1_assets.test.ts 包含 ASSET-02 upload params 测试
- [ ] phase1_assets.test.ts 包含 ASSET-03 排序逻辑测试
- [ ] phase1_assets.test.ts 包含 ASSET-03 filter options 测试
- [ ] phase1_assets.test.ts 包含 ASSET-04 usage graph types 测试
- [ ] phase1_assets.test.ts 包含 preview kind mapping 测试
- [ ] phase1_assets.test.ts 包含 utility helpers 测试
