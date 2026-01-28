# Aether Architecture V3: The Unified OS Protocol

**Status**: DRAFT (Created 2026-01-28)
**Driver**: "The Unification Review"

> [!IMPORTANT]
> This specification supersedes V2. The core shift is from **"Website + Dock"** to **"Single-Window OS"**.

## 1. Core Philosophy: The OS Model
Aether is not a website. It is a web-based Operating System (like Figma or VS Code).
-   **No Page Loads**: Navigating between apps never unloads the DOM.
-   **No "Detail Pages"**: You do not "visit" an app's profile. You **launch** it.
-   **URL Sovereignty**: The OS owns `/app`, the App owns `/app/:id/*`.

## 2. Navigation Topology
### 2.1 The Launchpad (Library)
-   The "Library" is no longer a destination. It is the **Launchpad**.
-   **Left Click**: Immediately launches the Knowledge Base (KB) into the Viewport.
    -   If pinned: Highlights Dock icon.
    -   If unpinned: Adds temporary "Running" icon to Dock.
-   **Right Click**: Opens Context Menu (Properties, Settings, Pin/Unpin).

### 2.2 The Dock (Taskbar)
-   **State**: Tracks both **Pinned** (Static) and **Running** (Dynamic) apps.
-   **Grouping**: Multiple instances of the same Template strictly auto-group (Stack).
-   **Persistence**: Switching away from an app **Minimizes** it (keeps state in memory). It does not Close it.

### 2.3 System Header (The "Traffic Lights")
The OS enforces a standard control strip at the top-left of the viewport (Height: ~50px).
-   **[Home]**: Minimizes current app, returns to Launchpad.
-   **[Close]**: Kills the current app instance (removes from memory/Dock if unpinned).
-   **[Title]**: The name of the current App.
-   *The rest of the header is available for Plugin injection.*

## 3. Metadata Management
Since there is no "Detail Page", metadata editing is redundant:
1.  **Context Menu**: Right-click in Launchpad -> "Properties".
2.  **In-App Settings**: Click "Settings" gear in the App Header -> "General".

## 4. Technical Implementation Changes
-   **Route Death**: Delete `/kb/:id` route (KnowledgeBaseDetail.vue).
-   **Shell Logic**: `SelfSpaceView` must manage a list of `open_apps` in addition to `pinned_apps`.
-   **Plugin Contract**: Plugins must NOT render their own "Back" buttons. They must rely on the System Header.
