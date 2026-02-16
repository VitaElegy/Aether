# Runbook: Fixing a Bug

> **Prevents**: Incomplete fixes, regressions, undocumented solutions
> **Goal**: Every fix leaves the codebase better documented

---

## Phase 1: Diagnosis

- [ ] **Reproduce** the bug reliably
- [ ] **Check ERROR_LOG** for similar past issues:
  ```bash
  grep -i "keyword" AI/memory/ERROR_LOG.md
  ```
- [ ] Identify **root cause**, not just symptoms
- [ ] Document reproduction steps

**Gate**: Do NOT fix until you understand the root cause.

---

## Phase 2: Test First

- [ ] Write a test that **fails** with current code:
  - Rust: `#[test]` in relevant module
  - Vue: Component spec or E2E test
- [ ] Confirm test fails for the right reason

**Why**: This proves the fix actually works and prevents regression.

---

## Phase 3: Implementation

- [ ] Write the minimal fix for the root cause
- [ ] Check for **related code** that might have the same bug:
  ```bash
  grep -r "similar_pattern" backend/src/ frontend/src/
  ```
- [ ] Fix related occurrences if found
- [ ] Ensure no new `unwrap()` or `panic!` introduced

---

## Phase 4: Verification

- [ ] `cargo build` passes
- [ ] `cargo test` passes (including new test)
- [ ] `npm run build` passes
- [ ] `npm run lint` passes
- [ ] **Manual verification** of original bug

---

## Phase 5: Documentation

### 5.1 If Significant Bug

- [ ] Add entry to `AI/memory/ERROR_LOG.md`:
  ```markdown
  | **Bug Title** | Description | Root Cause | Resolution | N/A |
  ```

### 5.2 If Reusable Lesson

- [ ] Add entry to `AI/memory/lessons_learned.md`

### 5.3 Commit Message

- [ ] Use descriptive commit message:
  ```
  fix(domain): resolve [issue] by [change]
  
  Root cause: [explanation]
  Fixes: [ERROR_LOG entry if applicable]
  ```

---

## Anti-Patterns

| ❌ Don't                                  | ✅ Do                             |
| :--------------------------------------- | :------------------------------- |
| Fix symptoms without understanding cause | Trace to root cause first        |
| Skip writing tests                       | Write failing test before fix    |
| Fix one occurrence                       | Search for similar patterns      |
| Commit without documenting               | Update ERROR_LOG/lessons_learned |

---

## Quick Checklist

```
□ Bug reproduced
□ ERROR_LOG checked
□ Root cause identified
□ Failing test written
□ Fix implemented
□ All tests pass
□ Manual verification done
□ Documentation updated
```
