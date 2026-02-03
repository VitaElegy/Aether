# Navigation Lifecycle & Loading Protocol Specification

> **Status**: ACTIVE
> **Version**: 1.0
> **Target**: Frontend Engineering & Future AI Agents

## 1. Core Philosophy
To ensure a robust ("Crash-Proof") and responsive ("Native-Like") experience, all Knowledge Base (KB) components MUST adhere to the **Self-Managed Loading Protocol**. The Global Shell provides the container, but the KB Component is responsible for its own state readiness.

## 2. The "Law" (Strict Requirements)

### 2.1. The Loading State Rule
**Rule**: Every top-level KB Dashboard component MUST have a reactive `isReady` (or `isLoading`) state.
**Rule**: The component MUST render a Skeleton or Loading UI while `!isReady`.
**Rule**: The component MUST NOT render its complex children (Graphs, Lists) until critical data is fetched.

**Example Implementation**:
```vue
<template>
  <div class="kb-container">
    <!-- 1. The Skeleton (MUST be present) -->
    <div v-if="!isReady" class="skeleton-layer">
        <MyUniqueSkeleton />
    </div>

    <!-- 2. The Content (Only when ready) -->
    <div v-else class="content-layer">
        <!-- Complex logic here -->
    </div>
  </div>
</template>
```

### 2.2. The Keep-Alive Rule
**Context**: The System uses `<KeepAlive>` to cache KBs. Components are NOT destroyed when switched away.
**Rule**: Every KB Component MUST implement the `onActivated` Vue hook.
**Rule**: Inside `onActivated`, the component MUST check if its data is stale (e.g., comparing `route.params.id` or a timestamp) and trigger a refresh if necessary.

**Example Implementation**:
```typescript
onActivated(async () => {
    if (shouldRefresh()) {
        isReady.value = false; // Show skeleton again if needed
        await fetchData();
        isReady.value = true;
    }
});
```

### 2.3. The Error Handling Rule
**Rule**: If a critical error occurs during `setup` or `fetchData`, the component MUST throw the error so it can be caught by the parent Error Boundary (in `SelfSpace.vue`).
**Rule**: Do NOT swallow critical setup errors with empty `catch` blocks. Fail fast.

## 3. Enforcement
**Mandatory**: All changes to `views/dashboard/*.vue` must be verified by running:
`npm run audit:kb`

This script statically analyzes the code to ensure `v-if` loading checks and `onActivated` hooks are present.
