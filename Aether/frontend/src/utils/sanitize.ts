import DOMPurify from 'dompurify';

/**
 * Sanitize HTML to prevent XSS attacks.
 * Allows safe subset of HTML tags commonly used in rendered markdown.
 */
export function sanitizeHtml(dirty: string): string {
    return DOMPurify.sanitize(dirty, {
        ALLOWED_TAGS: [
            'h1', 'h2', 'h3', 'h4', 'h5', 'h6',
            'p', 'br', 'hr',
            'ul', 'ol', 'li',
            'blockquote', 'pre', 'code',
            'a', 'strong', 'em', 'b', 'i', 'u', 's', 'del', 'ins', 'mark', 'sub', 'sup',
            'table', 'thead', 'tbody', 'tr', 'th', 'td',
            'img', 'figure', 'figcaption',
            'div', 'span', 'section', 'article',
            'details', 'summary',
            'abbr', 'cite', 'dfn', 'kbd', 'var', 'samp',
            'dl', 'dt', 'dd',
            'math', 'semantics', 'mrow', 'mi', 'mo', 'mn', 'ms', 'mtext',
            'msup', 'msub', 'mover', 'munder', 'mfrac', 'mroot', 'msqrt',
            'mtable', 'mtr', 'mtd', 'mspace', 'annotation',
        ],
        ALLOWED_ATTR: [
            'href', 'target', 'rel', 'title', 'alt', 'src', 'width', 'height',
            'class', 'id', 'style', 'data-*',
            'colspan', 'rowspan', 'scope',
            'loading', 'decoding',
            'open',
            'encoding', 'mathvariant', 'display',
        ],
        ALLOW_DATA_ATTR: true,
    });
}
