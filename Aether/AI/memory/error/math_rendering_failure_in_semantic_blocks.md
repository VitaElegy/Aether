# Error Analysis: Math Rendering Failure in Semantic Blocks

## 1. Issue Description
**Symptom**: In the Math Knowledge Base (Minisite Layout), LaTeX math blocks (`$$...$$`) located inside semantic blocks (like `::: axiom`) were rendering as raw text instead of properly formatted equations.
**Severity**: High (Broken Core Functionality)
**Component**: Frontend (`MathMinisiteLayout.vue`) / Markdown Engine (`marked.js`)

## 2. Root Cause Analysis
- **Parser Priority**: `marked.js` prioritizes standard block tokenizers. When text exists inside a semantic block without double newlines separating the math block, the content is treated as a single "Paragraph".
- **Missing Tokenizer**: The existing `inlineMath` custom tokenizer only handled `$` (inline) but did not have a rule for `$$` (display) appearing in inline contexts (i.e., inside a paragraph).
- **Result**: `$$ a+b $$` was consumed as plain text `text` tokens within a `paragraph`, bypassing the `blockMath` renderer.

## 3. Resolution
**Strategy**: Implement an "Inline-Block" tokenizer that detects `$$...$$` patterns even when they are embedded within other text blocks.

**Code Change**:
Added a new extension `inlineBlockMath` to the `marked.use` configuration in `MathMinisiteLayout.vue`:
```javascript
{
    name: 'inlineBlockMath',
    level: 'inline',
    start(src) { return src.match(/\$\$/)?.index; },
    tokenizer(src) {
        const rule = /^\$\$([\s\S]+?)\$\$/;
        const match = rule.exec(src);
        if (match) {
            return {
                 type: 'inlineBlockMath',
                 raw: match[0],
                 text: match[1].trim()
            };
        }
    },
    renderer(token) {
        return `<div class="katex-block display-mode-inline">${katex.renderToString(token.text, { displayMode: true })}</div>`;
    }
}
```

## 4. Prevention
- **Testing**: Added `scripts/verify_math_renderer.js` to specificially test this nested parsing scenario.
- **Guideline**: When using custom Markdown parsers, always ensure that "Display" elements have a corresponding "Inline" fallback if they can legally appear inside paragraphs.
