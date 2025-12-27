# Aether Frontend Design System & Component Guide

This document outlines the design philosophy and reusable components for the Aether frontend. Our design language aims for a **literary, minimalist, and elegant** aesthetic, prioritizing typography and whitespace over heavy containers or borders.

## Design Philosophy "Literary Minimalist"
*   **Typography**: Mix of Serif (titles, actions) and Mono (metadata).
*   **Actions**: Text-based links instead of button containers.
*   **Color**: Grayscale dominancy, using distinct colors only for semantic divergence (e.g., Diff colors).
*   **Layout**: centered, breathable `PageContainer`.

---

## UI Components (`src/components/ui/`)

### 1. SerifHeading
**Path**: `@/components/ui/SerifHeading.vue`

Used for top-level page titles or significant section headers. Applies a specific serif font stack with italic styling.

**Props**:
| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `size` | `'sm' \| 'md' \| 'lg'` | `'lg'` | Controls font size. `lg` is for page titles (text-3xl). |

**Usage**:
```vue
<template>
  <SerifHeading size="lg">Version 10</SerifHeading>
</template>
```

---

### 2. TextAction
**Path**: `@/components/ui/TextAction.vue`

The primary interaction element. Replaces standard "Outline/Primary" buttons in propert context. Uses serif fonts and text decorations to indicate state.

**Props**:
| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `active` | `boolean` | `false` | If true, applies active styling (underline, dark text). |
| `size` | `'sm' \| 'md' \| 'lg'` | `'md'` | Font size. |

**Events**:
*   `click`: Emitted when the button is clicked.

**Usage**:
```vue
<TextAction :active="isModeActive" @click="toggleMode">
  switch to reader
</TextAction>
```

---

### 3. MonoRef
**Path**: `@/components/ui/MonoRef.vue`

Used for displaying technical identifiers (UUIDs, hashes, Ref IDs) in a subtle, non-intrusive way.

**Usage**:
```vue
<MonoRef>ID: {{ contentId }}</MonoRef>
```

---

### 4. PageContainer
**Path**: `@/components/ui/PageContainer.vue`

Standard wrapper for all main views. Ensures consistent max-width and padding.

**Props**:
| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `maxWidth` | `'3xl' ... 'full'` | `'5xl'` | Max width class constraint. |

**Usage**:
```vue
<template>
  <PageContainer>
     <!-- Page Content -->
  </PageContainer>
</template>
```

---

## Utility Components

### DiffViewer
**Path**: `@/components/DiffViewer.vue`

Displays a GitHub-style side-by-side or unified diff.
**Optimization**: Uses inline styles for background colors to ensure robustness against CSS purging issues.
**Props**: `changes` (Array), `emptyMessage` (String).

---

## Future Extensions
When adding new pages, prefer using the above components to maintain the "Literary" feel. Avoid introducing heavy TDesign buttons (`t-button`) in the main reading/history flow unless necessary for "System" actions (like Settings or Delete).
