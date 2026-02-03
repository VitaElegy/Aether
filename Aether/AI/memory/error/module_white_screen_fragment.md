# Module White Screen (Fragment Root)

## Error Description
When loading the `KnowledgeModule` (Library), the application would sometimes render a completely white screen or fail to transition correctly between routes. No specific console error was immediately obvious until inspecting the component tree, where the component failed to mount properly within the `<Transition>` wrapper.

## Root Cause
The `KnowledgeModule.vue` file had multiple root nodes in its template:

```html
<template>
    <div class="h-full ...">
        <!-- Main Content -->
    </div>

    <!-- Sibling Root Node -->
    <LayoutSelectionModal ... />
</template>
```

In Vue 3, this creates a **Fragment**. While Vue 3 supports Fragments, they cause issues when the component is being rendered inside a `<Transition>` or `<KeepAlive>` component (which is the case in `SelfSpaceView`'s router view). The transition expects a single root element to apply classes to. When strictly enforced or when props inheritance is ambiguous, the renderer fails, resulting in a white screen (component unmounted).

## Resolution
The `<LayoutSelectionModal>` was moved **inside** the main root `<div>` container.

```diff
<template>
    <div class="h-full ...">
        <!-- Main Content -->
+       <LayoutSelectionModal ... />
    </div>
-   <LayoutSelectionModal ... />
</template>
```

This ensures the component has exactly one root element, allowing Vue Router and Transitions to function correctly.
