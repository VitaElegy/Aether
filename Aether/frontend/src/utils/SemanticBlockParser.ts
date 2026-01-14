export interface SemanticNode {
    id: string;
    type: string;
    title?: string;
    metrics?: Record<string, any>;
    content: string; // Inner content (markdown)
    children: SemanticNode[];
    level: number;
    parentId?: string;
}

export class SemanticBlockParser {
    private static BLOCK_REGEX = /::: \s*(\w+)\s*(\{.*?\})?\s*\n([\s\S]*?)\n:::/g; // Keeping global for reference, but loop uses line matching

    /**
     * Parses the markdown body and extracts a flat list of nodes 
     * which can be reconstructed into a tree.
     * Note: This simple regex approach usually handles top-level blocks.
     * For nested blocks, a recursive parser or stack-based lexer is needed.
     * Given the complexity, we will implement a stack-based parser.
     */
    static parse(markdown: string): SemanticNode[] {
        if (!markdown || typeof markdown !== 'string') {
            console.warn("SemanticBlockParser: Invalid markdown input", markdown);
            return [];
        }
        const lines = markdown.split('\n');
        const nodes: SemanticNode[] = [];
        const stack: { type: string, metrics: any, contentStart: number, level: number, parentId?: string }[] = [];

        let currentIdCounter = 0;

        for (let i = 0; i < lines.length; i++) {
            const line = lines[i].trim();

            // Start Block match
            const startMatch = line.match(/^:::\s*(\w+)\s*(\{.*?\})?$/);
            if (startMatch) {
                const type = startMatch[1];
                let metrics = {};
                try {
                    if (startMatch[2]) {
                        metrics = JSON.parse(startMatch[2]);
                    }
                } catch (e) {
                    console.warn('Failed to parse metrics', e);
                }

                stack.push({
                    type,
                    metrics,
                    contentStart: i + 1,
                    level: stack.length,
                    parentId: nodes.length > 0 ? nodes[nodes.length - 1].id : undefined
                });
                continue;
            }

            // End Block match: :::
            if (line === ':::') {
                if (stack.length > 0) {
                    const block = stack.pop()!;
                    const content = lines.slice(block.contentStart, i).join('\n');

                    const id = (block.metrics as any).id || `node-${currentIdCounter++}`;
                    const title = (block.metrics as any).title || `${block.type} ${currentIdCounter}`;

                    const node: SemanticNode = {
                        id,
                        type: block.type,
                        title,
                        metrics: block.metrics,
                        content,
                        children: [],
                        level: block.level,
                        parentId: undefined
                    };
                    nodes.push(node);
                }
            }
        }

        return nodes;
    }
}
