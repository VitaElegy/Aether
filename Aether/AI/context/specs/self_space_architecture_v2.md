# Self Space Architecture V2 (The "Silky" Standard)

> **Status**: APPROVED
> **Date**: 2026-02-02
> **Context**: Established after the "6 Questions" Deep Inquiry.

## Core Philosophy
The Frontend is a **Strict Interpreter** of the Backend state, with **Zero Tolerance** for configuration ambiguity.

---

## 1. Data Integrity (The Single Source of Truth)
*   **Principle**: Every "App" in the Dock MUST correspond to a valid record in the Database (`knowledge_bases` table).
*   **Anti-Pattern**: Hardcoded "Ghost" apps in the frontend that don't exist in the DB.
*   **Standard**: If it's not in the API list, it doesn't exist.

## 2. Identity Alignment (Strict Registry)
*   **Principle**: `renderer_id` in Database MUST match `plugin.id` in Registry exactly.
*   **Mechanism**:
    *   DB: `renderer_id: 'memo_std'`
    *   Plugin: `id: 'memo_std'`
*   **Anti-Pattern**: Fuzzy matching, aliasing (`if id == 'admin' || id == 'system'`), or implied defaults.

## 3. Error Visibility (Loud Failure)
*   **Principle**: Failures must be visible and actionable.
*   **Mechanism**:
    *   **Config Error**: If Plugin ID mismatch -> Dock Icon becomes ⚠️ (Red Alert), Tooltip shows "Config Error".
    *   **Runtime Crash**: If Component crashes -> Shell catches invalid VNode -> Renders "Broken State" Screen with Stack Trace.
*   **Anti-Pattern**: Silent fallback to a Default Book Icon (The "White Lie").

## 4. Routing Source of Truth (Silky Routing)
*   **Principle**: The URL determines the State. `activeKbId` is a derivative of the URL.
*   **Standard**: `/space/:kbId`
*   **Behavior**:
    *   **Navigation**: Use `router.replace()` for tab switching (preserves history stack cleanliness).
    *   **Smoothness**: Use `<KeepAlive>` + `<Transition>` to mock native app feel.
    *   **Deep Linking**: Users must be able to bookmark/share `/space/memo_std`.

## 5. Capability-Driven UI (The Dumb Shell)
*   **Principle**: The Shell (TopBar, TouchBar) knows nothing about the App. It renders what the App *declares*.
*   **Mechanism**:
    *   Plugins export `actionDefs` (Declarative Data).
    *   Shell iterates `activePlugin.actionDefs` to render buttons.
*   **Anti-Pattern**: Hardcoded `if (app == 'memos') renderButton('New Memo')`.

## 6. OS Service Injection (Dependency Inversion)
*   **Principle**: Plugins are sandboxed. They never import global Stores.
*   **Mechanism**:
    *   Shell provides `provide('os', osContext)`.
    *   Plugins use `inject('os')`.
    *   **OS Context API**:
        ```typescript
        interface OSContext {
            toast: (msg: string) => void;
            modal: { open: (component: any) => void };
            router: { navigate: (url: string) => void };
            network: { isOnline: boolean };
        }
        ```
*   **Goal**: Testability and strict isolation.

---

## Implementation Checklist
- [x] Strict Registry (Orchestrator)
- [x] Loud Error Icons (ModuleSwitcher)
- [x] Silky Routing (`/space/:id`)
- [x] Capability Interface (`plugin.ts`)
- [ ] OS Context Injection (Pending Implementation)
