# Wave 5 Test Suite — Directory Structure

> Generated: 2026-04-03

## 📁 Structure

```
tests/wave5/
├── checklists/
│   ├── README.md                          (this file)
│   └── COMPREHENSIVE_CHECKLIST.md         (214 test points, prioritized)
├── scripts/
│   ├── test_vrkb01_projects.ts            (VRKB-01: 17 tests)
│   ├── test_vrkb02_findings.ts            (VRKB-02: 19 tests)
│   ├── test_vrkb03_triage.ts              (VRKB-03: 9 tests)
│   ├── test_vrkb04_05_06_checklist_evidence_assets.ts (VRKB-04/05/06: 28 tests)
│   ├── test_vrkb07_08_09_docs_members_audit.ts        (VRKB-07/08/09: 30 tests)
│   ├── test_plat04_vrkb10_portability.ts  (PLAT-04 + VRKB-10: 11 tests)
│   ├── test_plat03_06_frontend.ts         (PLAT-03 + PLAT-06: Vitest, 25 tests)
│   └── test_cross_module_bugs.ts          (Systemic + Bugs: 11 tests)
└── reports/
    └── (generated test reports go here)
```

## 🚀 Running Tests

### Backend API Tests (requires running server)

```bash
# Set environment variables
export API_BASE_URL=http://localhost:3000
export AUTH_TOKEN=your-jwt-token

# Run individual test suites with tsx or ts-node
npx tsx tests/wave5/scripts/test_vrkb01_projects.ts
npx tsx tests/wave5/scripts/test_vrkb02_findings.ts
npx tsx tests/wave5/scripts/test_vrkb03_triage.ts
npx tsx tests/wave5/scripts/test_vrkb04_05_06_checklist_evidence_assets.ts
npx tsx tests/wave5/scripts/test_vrkb07_08_09_docs_members_audit.ts
npx tsx tests/wave5/scripts/test_plat04_vrkb10_portability.ts
npx tsx tests/wave5/scripts/test_cross_module_bugs.ts
```

### Frontend Unit Tests (Vitest)

```bash
cd frontend
npx vitest run ../tests/wave5/scripts/test_plat03_06_frontend.ts
```

## 📊 Coverage

| Script | Module | Test Count | Category |
|--------|--------|------------|----------|
| test_vrkb01_projects.ts | VRKB-01 | 17 | Backend API |
| test_vrkb02_findings.ts | VRKB-02 | 19 | Backend API |
| test_vrkb03_triage.ts | VRKB-03 | 9 | Backend API |
| test_vrkb04_05_06_*.ts | VRKB-04/05/06 | 28 | Backend API |
| test_vrkb07_08_09_*.ts | VRKB-07/08/09 | 30 | Backend API |
| test_plat04_vrkb10_*.ts | PLAT-04/VRKB-10 | 11 | Backend API |
| test_plat03_06_frontend.ts | PLAT-03/06 | 25 | Frontend Unit |
| test_cross_module_bugs.ts | Cross-module | 11 | Integration |
| **Total** | | **150** | |

The remaining 64 test points from the COMPREHENSIVE_CHECKLIST (manual UX tests,
dead code verification, performance profiling) require manual review or
specialized tooling.
