# Library ID Fetch Error (400 Bad Request)

## Error Description
Navigating to the default "Library" view or triggering a refresh would result in an API error:
`AxiosError: Request failed with status code 400`
The failing request was `GET /api/knowledge-bases/library`.

## Root Cause
The frontend application uses the string identifier `'library'` to represent the "root" state where the user sees a list of all Knowledge Bases. However, the `KnowledgeModule` component logic was interpreting this `kbId` prop literally as a database ID and attempting to fetch it from the backend.

```typescript
// Old Logic
onMounted(() => {
    if (props.kbId) { // 'library' is truthy
        knowledgeApi.get(props.kbId) ... // FAIL: Backend expects UUID, gets 'library'
    }
})
```

Since `'library'` is not a valid UUID (or simply doesn't exist as a resource), the backend returned a 400 Bad Request (invalid ID format) or 404.

## Resolution
We updated the component logic in `onMounted` and the `watch`er to explicitly exclude the `'library'` keyword from being treated as a fetchable ID.

```typescript
// New Logic
if (props.kbId && props.kbId !== 'library') {
    // Only fetch if it's a real KB ID
    knowledgeApi.get(props.kbId) ...
} else {
    // Default to List View for 'library' or undefined
    fetchKBs();
}
```

This ensures the special UI state is handled purely on the frontend without generating invalid network requests.
