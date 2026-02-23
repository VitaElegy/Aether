<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, watch } from 'vue';
import { Editor } from '@tiptap/vue-3';
import { NodeSelection, TextSelection } from '@tiptap/pm/state';
import { DOMSerializer } from '@tiptap/pm/model';
import { Icon } from 'tdesign-vue-next';

const props = defineProps<{
    editor: Editor;
}>();

const emit = defineEmits<{
    (e: 'drop-indicator', value: { 
        visible: boolean; 
        top: number; 
        left: number; 
        width: number;
        pos?: number;
        height?: number;
        html?: string;
    }): void;
}>();

const menuRef = ref<HTMLElement | null>(null);
const activeNode = ref<any>(null);
const activePos = ref<number | null>(null);
const visible = ref(false);
const top = ref(0);
const left = ref(0);
const isDragging = ref(false);

// Block highlight overlay dimensions
const blockHighlight = ref<{ top: number; left: number; width: number; height: number } | null>(null);

// Dragged block state for preview
const draggedBlockHeight = ref(0);
const draggedBlockHtml = ref('');

// Custom drag ghost
const dragGhost = ref<HTMLElement | null>(null);

// Hover-based handle: track hovered block separately from selection
const hoveredPos = ref<number | null>(null);
const hoveredNode = ref<any>(null);
const isHoverActive = ref(false);

// Throttle dragover with rAF
// Throttle dragover with rAF
let dragOverRAF: number | null = null;
let latestDragX = 0;
let latestDragY = 0;
let lastResolveTime = 0;
let lastEmitted: string | null = null;
const lastMouseX = ref(0);
const lastMouseY = ref(0);

// --- Positioning Logic (hover-based) ---
const updateHandleForBlock = (pos: number, node: any) => {
    let view;
    try {
        view = props.editor.view;
    } catch (e) {
        return;
    }
    if (!view || !view.dom) return;

    const dom = view.nodeDOM(pos) as HTMLElement;
    if (!dom || !dom.getBoundingClientRect) return;

    const editorRect = view.dom.getBoundingClientRect();
    const nodeRect = dom.getBoundingClientRect();

    activeNode.value = node;
    activePos.value = pos;
    visible.value = true;

    top.value = nodeRect.top - editorRect.top + 4;
    left.value = (nodeRect.left - editorRect.left) - 24;

    // Update block highlight overlay position
    blockHighlight.value = {
        top: nodeRect.top - editorRect.top,
        left: 0,
        width: editorRect.width,
        height: nodeRect.height,
    };
};

const updatePositionFromSelection = () => {
    // Selection updates (typing, cursor move) take precedence over passive hover
    isHoverActive.value = false;
    hoveredPos.value = null;
    hoveredNode.value = null;
    
    if (!props.editor || props.editor.isDestroyed) return;

    let view;
    try {
        view = props.editor.view;
    } catch (e) {
        return;
    }

    if (!view || !view.dom) return;

    const { selection } = view.state;
    const $anchor = selection.$anchor;
    let depth = $anchor.depth;
    
    if (depth === 0) {
        visible.value = false;
        return;
    }

    let node = $anchor.node(depth);
    let pos = $anchor.before(depth);

    if (depth > 1) {
        const headerDepth = 1;
        node = $anchor.node(headerDepth);
        pos = $anchor.before(headerDepth);
    }

    if (!node) {
        visible.value = false;
        blockHighlight.value = null;
        return;
    }

    updateHandleForBlock(pos, node);
};

// --- Watchers ---
watch(() => {
    try {
        return props.editor?.state?.selection;
    } catch {
        return null;
    }
}, () => {
    updatePositionFromSelection();
});


// --- Helper: resolve drop target position from mouse coords ---
const resolveDropTarget = (clientX: number, clientY: number): { pos: number; rect: DOMRect } | null => {
    let view;
    try {
        view = props.editor.view;
    } catch {
        return null;
    }
    if (!view) return null;

    const posInfo = view.posAtCoords({ left: clientX, top: clientY });
    if (!posInfo) return null;

    const $drop = view.state.doc.resolve(posInfo.pos);
    
    if ($drop.depth === 0) return null;

    const blockPos = $drop.before(1);
    const blockDom = view.nodeDOM(blockPos) as HTMLElement;
    if (!blockDom || !blockDom.getBoundingClientRect) return null;

    const rect = blockDom.getBoundingClientRect();
    const isLowerHalf = clientY > rect.top + rect.height / 2;

    return {
        pos: isLowerHalf ? $drop.after(1) : blockPos,
        rect: isLowerHalf
            ? new DOMRect(rect.left, rect.bottom, rect.width, 0)
            : new DOMRect(rect.left, rect.top, rect.width, 0),
    };
};

// --- Helper: resolve block at mouse position (with DOM fallback) ---
const resolveBlockFromDOM = (clientX: number, clientY: number): { pos: number; node: any } | null => {
    let view;
    try {
        view = props.editor.view;
    } catch {
        return null;
    }
    if (!view) return null;

    // Walk up from the element under the cursor to find a ProseMirror top-level block
    let el = document.elementFromPoint(clientX, clientY) as HTMLElement | null;
    if (!el) return null;

    const proseMirrorDom = view.dom;
    
    // Walk up until we find a direct child of the ProseMirror container
    while (el && el !== proseMirrorDom && el.parentElement !== proseMirrorDom) {
        el = el.parentElement;
    }

    // el should now be a direct child of proseMirrorDom (a top-level block)
    if (!el || el === proseMirrorDom || el.parentElement !== proseMirrorDom) return null;

    // Get the ProseMirror position for this DOM node
    const pos = view.posAtDOM(el, 0);
    if (pos < 0) return null;

    try {
        const $pos = view.state.doc.resolve(pos);
        if ($pos.depth === 0) return null;
        const blockPos = $pos.before(1);
        const blockNode = $pos.node(1);
        if (!blockNode) return null;
        return { pos: blockPos, node: blockNode };
    } catch {
        return null;
    }
};

const resolveBlockAtCoords = (clientX: number, clientY: number): { pos: number; node: any } | null => {
    let view;
    try {
        view = props.editor.view;
    } catch {
        return null;
    }
    if (!view) return null;

    // Try ProseMirror's coordinate resolution first
    const posInfo = view.posAtCoords({ left: clientX, top: clientY });
    if (posInfo) {
        try {
            const $pos = view.state.doc.resolve(posInfo.pos);
            if ($pos.depth > 0) {
                const blockPos = $pos.before(1);
                const blockNode = $pos.node(1);
                if (blockNode) {
                    // CRITICAL: Verify the mouse is actually over this block's DOM rect.
                    // ProseMirror resolves empty space below content to the last block,
                    // which causes the handle to "lock" to the bottom block.
                    const blockDom = view.nodeDOM(blockPos) as HTMLElement;
                    if (blockDom && blockDom.getBoundingClientRect) {
                        const rect = blockDom.getBoundingClientRect();
                        if (clientY > rect.bottom + 2 || clientY < rect.top - 2) {
                            // Mouse is outside this block's vertical bounds — ignore
                            return null;
                        }
                    }
                    return { pos: blockPos, node: blockNode };
                }
            }
        } catch {
            // Fall through to DOM fallback
        }
    }

    // Fallback: use DOM traversal
    return resolveBlockFromDOM(clientX, clientY);
};


// --- Custom Drag Ghost ---
const createDragGhost = (sourceDom: HTMLElement): HTMLElement => {
    const ghost = sourceDom.cloneNode(true) as HTMLElement;
    const rect = sourceDom.getBoundingClientRect();
    
    ghost.style.cssText = `
        position: fixed;
        top: 0;
        left: 0;
        width: ${rect.width}px;
        pointer-events: none;
        z-index: 10000;
        opacity: 0.85;
        background: white;
        border-radius: 8px;
        box-shadow: 0 8px 24px rgba(0,0,0,0.12), 0 2px 8px rgba(0,0,0,0.08);
        padding: 4px 8px;
        transform: translate3d(${rect.left}px, ${rect.top}px, 0) rotate(1.5deg) scale(1.02);
        transition: opacity 0.15s ease, transform 0.1s linear;
        overflow: hidden;
        will-change: transform;
    `;
    
    document.body.appendChild(ghost);
    return ghost;
};

const updateDragGhostPosition = (ghost: HTMLElement, clientX: number, clientY: number) => {
    // Use translate3d for GPU acceleration - significantly reduces lag on high-refresh displays
    ghost.style.transform = `translate3d(${clientX + 12}px, ${clientY - 10}px, 0) rotate(1.5deg) scale(1.02)`;
};

const removeDragGhost = () => {
    if (dragGhost.value) {
        dragGhost.value.style.opacity = '0';
        dragGhost.value.style.transform = 'rotate(0deg) scale(0.98)';
        const ghost = dragGhost.value;
        setTimeout(() => {
            ghost.remove();
        }, 150);
        dragGhost.value = null;
    }
};


// --- Drag & Drop Logic ---
const handleDragStart = (event: DragEvent) => {
    if (!props.editor || activePos.value === null) return;
    
    let view;
    try {
        view = props.editor.view;
    } catch (e) {
        return;
    }
    
    if (!view) return;

    view.focus();
    
    const pos = activePos.value;
    const node = view.state.doc.nodeAt(pos);
    if (!node) return;

    isDragging.value = true;

    // Mark the source block as "being dragged"
    const domNode = view.nodeDOM(pos) as HTMLElement;
    if (domNode) {
        domNode.classList.add('is-block-dragging');
        draggedBlockHeight.value = domNode.offsetHeight;
        draggedBlockHtml.value = domNode.outerHTML; // Use outerHTML to capture tags like <h1>, <pre>, etc.
    }

    // Select the node so ProseMirror knows what is being dragged
    const selection = NodeSelection.create(view.state.doc, pos);
    const tr = view.state.tr.setSelection(selection);
    view.dispatch(tr);
    
    // Serialize content for Drag and Drop
    const slice = selection.content();
    const serializer = DOMSerializer.fromSchema(view.state.schema);
    const fragment = serializer.serializeFragment(slice.content);
    
    const tempDiv = document.createElement('div');
    tempDiv.appendChild(fragment);
    const html = tempDiv.innerHTML;
    const text = tempDiv.textContent || '';

    if (event.dataTransfer) {
        event.dataTransfer.effectAllowed = 'move'; 
        event.dataTransfer.setData('application/x-aether-block-pos', pos.toString());
        event.dataTransfer.setData('application/x-aether-block-size', node.nodeSize.toString());
        event.dataTransfer.setData('application/x-aether-block-json', JSON.stringify(node.toJSON()));
        event.dataTransfer.setData('text/html', html);
        event.dataTransfer.setData('text/plain', text);
        
        // Use a tiny 1x1 transparent image to hide the native drag image
        const emptyImg = new Image();
        emptyImg.src = 'data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7';
        event.dataTransfer.setDragImage(emptyImg, 0, 0);
    }

    // Create our custom drag ghost
    if (domNode) {
        dragGhost.value = createDragGhost(domNode);
    }
};

const handleDragOver = (e: DragEvent) => {
    if (!isDragging.value) return;
    e.preventDefault();
    if (e.dataTransfer) {
        e.dataTransfer.dropEffect = 'move';
    }

    // Store latest coordinates for rAF
    latestDragX = e.clientX;
    latestDragY = e.clientY;

    // Use rAF to throttle BOTH ghost updates and indicator calculations
    if (dragOverRAF !== null) return;
    
    dragOverRAF = requestAnimationFrame(() => {
        dragOverRAF = null;
        
        // 1. Efficiently update ghost position (GPU transform) - Keep this at 60fps
        if (dragGhost.value) {
            updateDragGhostPosition(dragGhost.value, latestDragX, latestDragY);
        }

        // 2. Throttle drop target resolution (expensive DOM lookups) to ~30fps
        const now = Date.now();
        if (now - lastResolveTime < 32) return; // ~30fps cap for indicator updates
        lastResolveTime = now;

        const target = resolveDropTarget(latestDragX, latestDragY);
        
        let emitData;
        if (!target) {
            emitData = { visible: false, top: 0, left: 0, width: 0 };
        } else {
            let view;
            try {
                view = props.editor.view;
            } catch {
                return;
            }
            
            if (!view) return;
            const editorRect = view.dom.getBoundingClientRect();
            emitData = {
                visible: true,
                top: target.rect.y - editorRect.top,
                left: target.rect.x - editorRect.left,
                width: target.rect.width,
                pos: target.pos,
                height: draggedBlockHeight.value,
                html: draggedBlockHtml.value,
            };
        }

        // 3. Deduplicate events to prevent Vue re-renders
        const emitString = JSON.stringify(emitData);
        if (lastEmitted !== emitString) {
            emit('drop-indicator', emitData);
            lastEmitted = emitString;
        }
    });
};

const handleDragLeave = (e: DragEvent) => {
    let view;
    try {
        view = props.editor.view;
    } catch {
        return;
    }
    if (!view) return;

    const editorRect = view.dom.getBoundingClientRect();
    const { clientX, clientY } = e;
    
    if (
        clientX <= editorRect.left || clientX >= editorRect.right ||
        clientY <= editorRect.top || clientY >= editorRect.bottom
    ) {
        emit('drop-indicator', { visible: false, top: 0, left: 0, width: 0 });
    }
};

const handleDragEnd = (event: DragEvent) => {
    isDragging.value = false;
    blockHighlight.value = null;
    
    // Remove custom ghost
    removeDragGhost();
    
    // Cancel any pending rAF
    if (dragOverRAF !== null) {
        cancelAnimationFrame(dragOverRAF);
        dragOverRAF = null;
    }
    
    // Cleanup drag opacity class
    const dragging = document.querySelector('.is-block-dragging');
    if (dragging) dragging.classList.remove('is-block-dragging');

    // Hide drop indicator
    // Hide drop indicator
    if (lastEmitted !== null) { // Only emit if currently visible
        emit('drop-indicator', { visible: false, top: 0, left: 0, width: 0 });
        lastEmitted = null;
    }
};

// --- Actions ---
const handleClick = () => {
   if (activePos.value !== null && props.editor && !props.editor.isDestroyed) {
       try {
           const view = props.editor.view;
           const tr = props.editor.state.tr.setSelection(NodeSelection.create(props.editor.state.doc, activePos.value));
           view.dispatch(tr);
           view.focus();
       } catch (e) {
           // Ignore
       }
   }
};

const handleMouseMove = (e: MouseEvent) => {
    if (!props.editor || props.editor.isDestroyed) return;
    if (isDragging.value) return;

    // Filter out pseudo-moves (e.g. triggering from layout shifts without actual mouse movement)
    // We only want to enable hover mode if the user *intentionally* moves the mouse.
    if (Math.abs(e.clientX - lastMouseX.value) < 2 && Math.abs(e.clientY - lastMouseY.value) < 2) {
        return;
    }
    lastMouseX.value = e.clientX;
    lastMouseY.value = e.clientY;

    const block = resolveBlockAtCoords(e.clientX, e.clientY);
    if (!block) {
        // Mouse is in empty space — hide handle and highlight
        if (isHoverActive.value) {
            isHoverActive.value = false;
            hoveredPos.value = null;
            hoveredNode.value = null;
            visible.value = false;
            blockHighlight.value = null;
        }
        return;
    }

    // Activate hover state — this prevents selection watcher from overriding
    isHoverActive.value = true;
    hoveredPos.value = block.pos;
    hoveredNode.value = block.node;
    updateHandleForBlock(block.pos, block.node);
};

const handleMouseLeave = (e: MouseEvent) => {
    if (isDragging.value) return;
    
    // When mouse leaves editor, deactivate hover and revert to selection-based positioning
    isHoverActive.value = false;
    hoveredPos.value = null;
    hoveredNode.value = null;
    updatePositionFromSelection();
};

onMounted(() => {
    if (props.editor) {
        try {
           if (props.editor.view && props.editor.view.dom) {
               const dom = props.editor.view.dom;
               dom.addEventListener('mousemove', handleMouseMove);
               dom.addEventListener('mouseleave', handleMouseLeave);
               dom.addEventListener('dragover', handleDragOver);
               dom.addEventListener('dragleave', handleDragLeave);
           }
        } catch (e) {
            // Ignore if view is not ready
        }
    }
});

onBeforeUnmount(() => {
    if (props.editor) {
        try {
            if (props.editor.view && props.editor.view.dom) {
                const dom = props.editor.view.dom;
                dom.removeEventListener('mousemove', handleMouseMove);
                dom.removeEventListener('mouseleave', handleMouseLeave);
                dom.removeEventListener('dragover', handleDragOver);
                dom.removeEventListener('dragleave', handleDragLeave);
            }
        } catch (e) {
            // Ignore
        }
    }
    
    // Cleanup ghost if component unmounts mid-drag
    removeDragGhost();
    
    if (dragOverRAF !== null) {
        cancelAnimationFrame(dragOverRAF);
        dragOverRAF = null;
    }
});
</script>

<template>
    <!-- Block Highlight Overlay -->
    <div 
        v-if="blockHighlight && !isDragging"
        class="block-highlight"
        :style="{
            top: `${blockHighlight.top}px`,
            left: `${blockHighlight.left}px`,
            width: `${blockHighlight.width}px`,
            height: `${blockHighlight.height}px`,
        }"
    />

    <!-- Drag Handle Button -->
    <div 
        v-if="visible"
        ref="menuRef"
        class="drag-handle"
        :class="{ 'is-dragging': isDragging }"
        :style="{ top: `${top}px`, left: `${left}px` }"
        draggable="true"
        @dragstart="handleDragStart"
        @dragend="handleDragEnd"
        @click="handleClick"
    >
        <Icon name="drag-move" size="16px" />
    </div>
</template>

<style scoped>
.block-highlight {
    position: absolute;
    z-index: 1;
    pointer-events: none;
    border-radius: 4px;
    background: linear-gradient(90deg, rgba(59, 130, 246, 0.08), rgba(59, 130, 246, 0.02));
    transition: top 0.12s cubic-bezier(0.2, 0, 0, 1), height 0.1s ease, opacity 0.15s ease;
}

.drag-handle {
    position: absolute;
    z-index: 50;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    cursor: grab;
    color: #9ca3af;
    border-radius: 4px;
    transition: color 0.15s ease, background-color 0.15s ease, opacity 0.2s ease, transform 0.15s ease, top 0.12s cubic-bezier(0.2, 0, 0, 1);
    opacity: 0.4;
}

.drag-handle:hover {
    color: #4b5563;
    background-color: #f3f4f6;
    opacity: 1;
    transform: scale(1.1);
}

.drag-handle:active,
.drag-handle.is-dragging {
    cursor: grabbing;
    opacity: 0.7;
}
</style>
