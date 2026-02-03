# DataCloneError in Storage (Vue Proxies)

## Error Description
The console frequently showed the following error during navigation or state persistence:
`DataCloneError: The object could not be cloned.`

This prevented the application from correctly saving the navigation stack (Smart Trace) to IndexedDB via `localforage`.

## Root Cause
The `navigationStack` store was using `watch` to observe the `stacks` reactive object:

```typescript
watch(stacks, (newVal) => {
    sessionService.saveState(..., { formData: newVal });
}, { deep: true });
```

The `newVal` passed by Vue's watcher is a **Proxy** object (Vue's reactivity system). The structured clone algorithm used by IndexedDB (and `postMessage`) generally throws an error when attempting to clone a Proxy, especially if it wraps complex types that are not explicitly serializable.

## Resolution
We used Vue's `toRaw()` utility to "unwrap" the Proxy and get the plain JavaScript object before passing it to the storage service.

```typescript
import { toRaw } from 'vue';

watch(stacks, (newVal) => {
    sessionService.saveState(..., { 
        formData: toRaw(newVal) // Unwrap Proxy
    });
}, { deep: true });
```

This ensures only plain JSON-serializable data is sent to IndexedDB, resolving the `DataCloneError`.
