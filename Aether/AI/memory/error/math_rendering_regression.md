# Math Rendering Regression in Semantic Blocks

## Root Cause
The `MathManuscriptLayout.vue` component was manually processing semantic blocks (like `::: theorem`) using Regex and wrapping them in HTML `<div>` tags *before* passing the content to the Markdown Renderer. 

The `MarkdownRenderer.vue` used `marked.js` in a default configuration. By standard Markdown specification (CommonMark), content inside HTML blocks is treated as "raw HTML" and is **not parsed** for Markdown. Consequently, any Math syntax (`$$...$$`) or other Markdown link referencing inside these blocks was rendered as plain text.

The previous "inlineBlockMath tokenizer" mentioned in logs appeared to be missing or ineffective in this configuration.

## Resolution
1.  **Refactor MarkdownRenderer**: upgraded `MarkdownRenderer.vue` to use `marked.use()` with custom extensions.
    -   **Math Extension**: Added support for `$$...$$` (block) and `$ ... $` (inline) using `katex`.
    -   **Semantic Block Extension**: Added a custom tokenizer for `::: type [params] ... :::` blocks. Crucially, this tokenizer explicitly triggers recursive parsing of the inner content (`this.lexer.blockTokens(innerContent)`).
2.  **Edge Case Fixes (Iterative Debugging)**:
    -   **Regex Limitation**: Initial regex (`\w+`) failed for types with hyphens (e.g., `function-card`). Updated to `[\w-]+`.
    -   **Multiline JSON**: Users often place JSON configuration on a new line. Added "Fallback Logic" to check if the body content starts with `{` when header parameters are empty, ensuring config is parsed correctly and not rendered as text.
3.  **Refactor MathManuscriptLayout**: Removed the regex pre-processing logic. The layout now passes the raw `article.body` to the renderer.

## Prevention Strategy
-   **Architectural Alignment**: Avoid ad-hoc regex replacements for structural content in Layouts. Always use the central Parser/Renderer for syntax handling.
-   **Verification**: Ensure future renderer tests include nested markdown scenarios (e.g., Math inside Blockquotes or Custom Containers).
