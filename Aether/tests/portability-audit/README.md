# Portability Audit Test Suite

> 生成日期: 2026-04-08
> 审计范围: 知识库下载/导出（Portability）功能全面测试

## 文件清单

```
tests/portability-audit/
├── README.md                          # 本文件
├── PORTABILITY_CHECKLIST.md           # 87 项功能测试清单（含状态追踪）
├── test_portability_api.ts            # 后端 API 集成测试（22 个测试点）
└── test_frontend_alignment.ts         # 前后端 API 对齐测试（11 个测试点）
```

## 运行方式

### 后端 API 集成测试
需要后端服务正在运行：
```bash
export API_BASE_URL=http://localhost:3000
export AUTH_TOKEN=your-jwt-token
npx tsx tests/portability-audit/test_portability_api.ts
```

### 前后端对齐测试
```bash
npx tsx tests/portability-audit/test_frontend_alignment.ts
```

### 前端单元测试
```bash
cd frontend
npx vitest run
```

## 修复的 Bug

| # | 文件 | 修改内容 |
|---|------|---------|
| 1 | `frontend/src/registries/special_kb_registry.ts` | VRKB portabilityProviderId: `'default'` → `'vrkb'` |
| 2 | `frontend/src/registries/special_kb_registry.ts` | Assets portabilityProviderId: `'default'` → `'assets_v1'` |
| 3 | `backend/src/.../portability/prkb.rs` | Progress stage: `"Complete"` → `"Finalizing"` |
| 4 | `backend/src/.../portability/memos.rs` | Progress stage: `"Complete"` → `"Finalizing"` |
| 5 | `frontend/src/registries/special_kb_registry.test.ts` | 更新测试断言匹配新映射 |
| 6 | `frontend/src/test/phase0_full.test.ts` | 更新 B9/N5 测试断言匹配新映射 |
