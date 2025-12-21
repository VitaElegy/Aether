# Absolute Zero Theme Documentation

## Design Philosophy
**"Absolute Zero"** is a purist, minimalist design system inspired by Swiss Style and High-End Editorial layouts.
- **Color Palette**: Strictly Black (Ink), White (Paper), and Gray (Ash).
- **Typography**: Inter (System UI) for structure, JetBrains Mono for data, Serif for narrative.
- **Visuals**: No shadows, no gradients, no border-radius > 2px. High contrast.

## File Structure & Usage

### 1. Configuration
- `tailwind.config.js`: Contains the specific color palette (`ink`, `paper`, `ash`) and font family extensions.
- `style.css`: Global resets and base typography rules.

### 2. Views
- `LoginView.vue`: Minimalist entry card with micro-interactions.
- `HomeView.vue`: Editorial grid layout for content feed.
- `EditorView.vue`: Distraction-free writing environment.
- `UserProfileView.vue`: Archive-style profile page.

### 3. Components
- `DynamicRenderer.vue`: Strategy pattern loader for content types.
- `renderers/`: Specialized renderers for Markdown, Code, and Video.

## Dependencies
This theme requires the following Tailwind setup:
```bash
npm install -D tailwindcss@3.4.17 postcss autoprefixer @tailwindcss/typography
```

## AI Generation Prompt
To generate similar UI designs using AI coding assistants, use the following system prompt instructions:

> **Role**: Minimalist UI/UX Expert (Specializing in Swiss Style & Editorial Design).
>
> **Design Rules ("Absolute Zero"):**
> 1.  **Strict Color Policy**:
>     -   Background: White (#ffffff) ONLY.
>     -   Foreground/Text: Black (#1a1a1a) ONLY.
>     -   Accents: Light Gray (#f3f4f6) for structure, NO colors.
> 2.  **Typography**:
>     -   Headings: Large, tight tracking (letter-spacing: -0.05em), bold sans-serif (Inter).
>     -   Body: High readability serif or sans-serif.
>     -   Data/Meta: Monospace (JetBrains Mono), uppercase, small text, wide tracking.
> 3.  **Visual Language**:
>     -   **NO** shadows, **NO** gradients, **NO** rounded corners (max 2px).
>     -   Use 1px borders for separation.
>     -   Layouts should look like a high-end architectural magazine or a printed manifest.
> 4.  **Interaction**:
>     -   Hover states: Invert colors (Black bg / White text) or simple underline.
>     -   Animations: Instant or very fast linear transitions (0.2s).
>
> **Task**: Refactor the current frontend to match this "Absolute Zero" aesthetic. Remove all decorative elements. Focus on whitespace, typography, and grid alignment.

## Restoration Guide
To restore this theme:
1. Copy `tailwind.config.js` to project root.
2. Copy `style.css` to `src/`.
3. Overwrite `App.vue` and corresponding Views in `src/views/`.
4. Ensure `components/` are placed correctly.
