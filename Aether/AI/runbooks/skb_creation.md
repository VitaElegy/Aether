# Runbook: Creating a Special Knowledge Base (SKB)

> **Prevents**: Renderer mismatch, search integration failures, quarantine errors
> **Error Examples**: `Renderer Not Found`, `Dashboards Crash on Load`

---

## Phase 1: Specification

- [ ] Create spec file: `AI/context/specs/{skb_name}_spec.md`
- [ ] Define unique `renderer_id` (lowercase, snake_case)
- [ ] Document data schema and UI requirements
- [ ] **STOP**: Get user approval before proceeding

**Gate**: Never proceed without approved spec.

---

## Phase 2: Backend

### 2.1 Schema (if new data types)

- [ ] Follow [database_entity.md](file:///Users/elegy/Documents/READING/LINUX/Aether/Aether/AI/runbooks/database_entity.md) runbook
- [ ] Define Block schema for JSON validation (if applicable)

### 2.2 Search Integration

- [ ] Implement `SearchableBlock` trait:
  ```rust
  impl SearchableBlock for YourBlock {
      fn to_search_text(&self, payload: &Value) -> String {
          // Extract ONLY human-readable text
          // Ignore IDs, colors, coordinates
          format!("{} {}", 
              payload["title"].as_str().unwrap_or(""),
              payload["description"].as_str().unwrap_or("")
          )
      }
  }
  ```
- [ ] Register in `SchemaRegistry`

### 2.3 Seed Data (optional)

- [ ] Create seed script: `scripts/seed_{skb}.py`
- [ ] Test seeding works correctly

---

## Phase 3: Frontend — Registry

- [ ] Create component directory: `frontend/src/renderers/{SkbName}/`
- [ ] Create main component: `{SkbName}Dashboard.vue`
- [ ] Register in `frontend/src/main.ts`:
  ```typescript
  registerPlugin({
    id: 'your_renderer_id',  // MUST match DB renderer_id exactly
    component: () => import('./renderers/YourSKB/Dashboard.vue'),
    icon: 'ri-your-icon-line',
  });
  ```
- [ ] **CRITICAL**: Add alias for legacy IDs if they exist:
  ```typescript
  registerAlias('old_id', 'your_renderer_id');
  ```

**Common Pitfall**: DB has `math_v1_std`, Registry has `math` → "Renderer Not Found"

---

## Phase 4: Frontend — Component

### 4.1 State Management

- [ ] Create Pinia store with `$reset()` method:
  ```typescript
  actions: {
    $reset() {
      this.items = [];
      this.loading = false;
      this.error = null;
    }
  }
  ```
- [ ] Implement `watch(() => props.kbId)` to handle switching:
  ```typescript
  watch(() => props.kbId, (newId, oldId) => {
    if (newId !== oldId) {
      store.$reset();
      loadData(newId);
    }
  });
  ```

### 4.2 Error Handling

- [ ] Wrap in `<Suspense>` with fallback
- [ ] Add `onErrorCaptured` hook
- [ ] Include `text_mirror` for mobile fallback

### 4.3 Props

- [ ] Accept `kb` object prop, not just `kbId`:
  ```typescript
  const props = defineProps<{
    kb: KnowledgeBase;  // Full object, not just ID
  }>();
  ```
- [ ] Add defensive `v-if="kb"` check

---

## Phase 5: Verification

- [ ] Create test KB instance via API or seed
- [ ] Verify Dock icon renders with correct icon/label
- [ ] Switch between multiple instances — confirm state resets
- [ ] Run audit:
  ```bash
  npm run audit:kb
  ```
- [ ] Update SPEC_INDEX.md with new spec status

---

## Quick Checklist

```
□ Spec created and approved
□ renderer_id matches exactly in DB and Registry
□ $reset() implemented in store
□ watch(kbId) handler implemented
□ kb object prop (not just kbId)
□ Alias registered for legacy IDs
□ audit:kb passes
```
