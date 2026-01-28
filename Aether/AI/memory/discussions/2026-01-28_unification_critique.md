# The Schizophrenic Architecture: A Critical Review

**Status**: CRITICAL
**Reviewer**: Lead Architect (Elegy)
**Verdict**: The current system is incoherent. It tries to be a Web Page and a Desktop OS simultaneously, failing at both.

You are right. The experience is "Fragmented" (割裂). It is unacceptable. Here are the 5 questions we must answer to kill this fragmentation.

---

### Q1: The Identity Crisis (OS vs. Website)
**The Problem**: Currently, Aether behaves like a **Website** when you click a link in the Library (Page Load, `/kb/123`), but like an **OS** when you click the Dock (View Injection, No Reload).
**The Question**: **Are we building a Browser-based OS (Figma/VSCode) or a Website (Notion/GitHub)?**
*   **A. The OS (VSCode Model)**: There is ONLY one page (`/app`). Everything else is a "Panel" or "Tab" inside it. The URL is just a serialization of the state. Clicking a link in the Library *must* launch the Dock item, not open a new page.
*   **B. The Website (Notion Model)**: The Dock is just a fancy bookmarks bar. Clicking an item *must* navigate the browser to a new URL. The "Self Space" wrapper is an illusion.

### Q2: The "Library" Lie
**The Problem**: The "Library" claims to be the source of truth, yet clicking an item there opens a "Preview" (`KnowledgeBaseDetail`) that looks and behaves differently from the "Native App" (`VrkbModule`) stored in the Dock.
**The Question**: **Why does the Library exist as a separate router view?**
*   Should the Library essentially be just "App Store"? Clicking "Open" there should effectively just **pin and focus** the app in the Dock, forcing the user into the OS workflow. Why are we allowing users to "visit" an app without "launching" it?

### Q3: Navigation Zombie (The "Back" Button)
**The Problem**: You asked for a "Back" button. In a website, "Back" means `history.back()`. In an OS, "Back" means "Close Window". We mixed them up, creating a loop where "Back" sends you to a page that immediately redirects you forward.
**The Question**: **If we are an OS (Option A in Q1), should the "Back" button be banned entirely?**
*   In an OS, you don't go "Back" from Excel to the Desktop. You **Close** Excel. Should we kill the Back button and replace it with a explicit **"Close / Minimize"** action that returns to the Dashboard?

### Q4: The URL Sovereignty (Who owns the address bar?)
**The Problem**: Currently, the `SelfSpaceView` fights with the `VrkbModule` for the URL. The Shell wants clean URLs; the Module wants deep links.
**The Question**: **Can we agree that the "Black Box" (The Plugin) owns the ENTIRE URL after the domain?**
*   If I enter the Vuln KB, the URL becomes `/app/vrkb/project/123`. The Shell surrenders completely.
*   The Shell only re-appears if the user hits a "Home" button that resets the URL to `/app`.

### Q5: The Registry Hypocrisy
**The Problem**: We spent hours defining a `Manifest` (`icon`, `color`, `component`), but the `KnowledgeBaseDetail` (Library View) **ignores it completely**, using hardcoded `v-if` logic.
**The Question**: **Why do we permit "Zombie Rendering"?**
*   Should we strictly forbid *any* component from rendering a KB unless it goes through the `PluginStore`? Even a "Static Preview" in the library should just be the Plugin rendered in "Read-Only Mode". Why do we maintain two code paths for showing the same data?
