# Skill: Frontend Development (Vue 3)

## 1. Critical Constraints
| Constraint               | Rule                                                                                                                  |
| ------------------------ | --------------------------------------------------------------------------------------------------------------------- |
| **Composable Supremacy** | All state-mutating requests (POST/PUT/DELETE) MUST be in Composables. Prohibited: Direct `axios` calls in Components. |
| **State Lock**           | Use global state locks (`isSaving`, `isLoading`) to prevent Race Conditions.                                          |
| **Cache Decoupling**     | `localStorage` MUST store content ONLY. NEVER restore `status`, `visibility`, `timestamps`.                           |
| **Safe Restoration**     | Cache restoration MUST NOT trigger Auto-Save. Use `isRestoring` flag.                                                 |
| **Auto-Save Rules**      | If `status` is 'Published', abort auto-save (local cache only).                                                       |
| **Status Preservation**  | Never hardcode `status: 'Draft'` in `saveDraft` - respect current `form.status`.                                      |

## 2. Coding Standards
### 2.1 State Management
- **Library**: Pinia
- **DTOs**: Strongly typed interfaces in `src/api` matching Backend exactly.

### 2.2 Atomic Operations
- Functions like "Publish" MUST await `isSaving` locks.

## 3. Tech Stack
- **Framework**: Vue 3 Composition API
- **UI**: TDesign + TailwindCSS
- **Build**: Vite
