<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, watch } from 'vue';
import { Editor } from '@tiptap/vue-3';
import { NodeSelection, TextSelection } from '@tiptap/pm/state';
import { Icon } from 'tdesign-vue-next';

const props = defineProps<{
    editor: Editor;
}>();

const menuRef = ref<HTMLElement | null>(null);
const activeNode = ref<any>(null);
const activePos = ref<number | null>(null);
const visible = ref(false);
const top = ref(0);
const left = ref(0);

// --- Positioning Logic ---
const updatePosition = () => {
    if (!props.editor || props.editor.isDestroyed) return;

    let view;
    try {
        view = props.editor.view;
    } catch (e) {
        return;
    }

    if (!view || !view.dom) return;

    const { selection } = view.state;
    
    // Find the current block node
    // We want the top-level block (e.g. Paragraph, Heading, List Item)
    // Tiptap's selection.$anchor tells us where the caret is.
    const $anchor = selection.$anchor;
    
    // We traverse up to find the direct child of Doc
    let depth = $anchor.depth;
    let node = $anchor.node(depth);
    let pos = $anchor.before(depth);

    // Naive block finding: Go to depth 1 (child of doc)
    // If inside a list, we might want the list item or the list itself? Notion handles list items individually.
    if (depth > 1) {
        // e.g. Doc -> List -> ListItem -> Paragraph
        // We want ListItem (depth 2) or Paragraph (depth 3)? 
        // Notion puts handle on ListItem.
        // Let's try to target depth 1 for now for simplicity, then refine.
        // Actually, depth 1 is usually the block.
        const headerDepth = 1;
        node = $anchor.node(headerDepth);
        pos = $anchor.before(headerDepth);
    }

    if (!node) {
        visible.value = false;
        return;
    }

    // Get DOM element for this node
    const dom = view.nodeDOM(pos) as HTMLElement;

    if (!dom || !dom.getBoundingClientRect) {
        visible.value = false;
        return;
    }

    const editorRect = view.dom.getBoundingClientRect();
    const nodeRect = dom.getBoundingClientRect();

    // Position handle to the left of the block
    // Relative to editor container (we assume editor has relative positioning or we use fixed/absolute)
    // Let's use absolute positioning relative to the editor wrapper.
    // We need to calculate offset relative to the editor.
    
    // Check if node is empty and not focused? Notion shows handle on hover.
    // For now, let's show on current selection (active block).
    
    activeNode.value = node;
    activePos.value = pos;
    visible.value = true;

    // Calculate Top
    // We want to align with the top of the block line-height roughly.
    // nodeRect.top is viewport relative.
    // We need the offset from the editor's bounding box.
    // But DragHandle is likely inside the EditorAdapter which is relative.
    // Let's assume DragHandle is sibling to EditorContent and parent is relative.
    
    // Add a small vertical offset to align center-to-center better for larger headings
    top.value = nodeRect.top - editorRect.top + 4;
    
    // Calculate Left
    // We want it in the gutter, to the left of the text block.
    // nodeRect.left is the text start. editorRect.left is container start.
    // Offset inside container = nodeRect.left - editorRect.left.
    // We place handle like -24px from the text start.
    left.value = (nodeRect.left - editorRect.left) - 24;
};

// --- Watchers ---
// Watch transaction to update position on typing/selection
watch(() => {
    try {
        return props.editor?.state?.selection;
    } catch {
        return null;
    }
}, () => {
    updatePosition();
});


// --- Drag & Drop Logic (Basic) ---
const handleDragStart = (event: DragEvent) => {
    if (!props.editor || !activePos.value) return;
    
    let view;
    try {
        view = props.editor.view;
    } catch (e) {
        return;
    }
    
    if (!view) return;

    view.focus();
    
    // Select the node
    const tr = view.state.tr.setSelection(NodeSelection.create(view.state.doc, activePos.value));
    view.dispatch(tr);
    
    // Let Tiptap/ProseMirror handle the actual drag of the selection
    // We just need to ensure the element being dragged is considered a valid drag handle.
    // By default, dragging an element requires valid dragstart. 
    // ProseMirror handles dragging selections.
    
    // Hack: We rely on the fact that we just selected the node. 
    // The user is technically dragging the "handle", not the text.
    // To make this work like Notion, we often need a custom DragHandle extension in PM.
    // BUT, a simple way is: Set selection, then let user drag? 
    // Actually, dragging the handle *shim* is tricky.
    
    // Alternative: Use the handle to open a menu, and use a separate "Drag Mode" or rely on a specialized library.
    // For MVP: Let's make it a Menu Trigger first (Notion +/:: icon).
    // Dragging is complex to implement from scratch in one go.
    // Let's just implement "Click to Select" and "Menu".
    
    if (event.dataTransfer) {
        event.dataTransfer.setData('text/plain', 'Block Drag');
    }
    // We can simulate drag by implementing the drag slice logic manually, but that's heavy.
};

// --- Actions ---
const handleClick = () => {
   // Open Block Menu (Delete, Turn Into, etc.)
   // For now, just select the block clearly.
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

    let view;
    try {
        view = props.editor.view;
    } catch (e) {
        return;
    }

    if (!view) return;

    // Find node at coordinates
    const pos = view.posAtCoords({ left: e.clientX, top: e.clientY });
    
    if (pos) {
        // Resolve to block
        // const $pos = view.state.doc.resolve(pos.pos);
        // ... (Reusing logic would be good)
        // For MVP, stick to "Selection Based" handle to ensure stability first.
        // Hover based is jumpy without good debouncing.
    }
}

onMounted(() => {
    // Ideally we also listen to mouseover on the editor to move the handle to hovered blocks
    // instead of just selected blocks.
    // Implementation: Editor `onHover` extension or manual event listener.
    if (props.editor) {
        try {
           if (props.editor.view && props.editor.view.dom) {
               const dom = props.editor.view.dom;
               dom.addEventListener('mousemove', handleMouseMove);
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
            }
        } catch (e) {
            // Ignore
        }
    }
});

// Re-using selection based for now.
</script>

<template>
    <div 
        v-if="visible"
        ref="menuRef"
        class="absolute z-50 flex items-center justify-center w-6 h-6 cursor-grab text-gray-400 hover:text-gray-600 hover:bg-gray-100 rounded transition-colors"
        :style="{ top: `${top}px`, left: `${left}px` }"
        draggable="true"
        @dragstart="handleDragStart"
        @click="handleClick"
    >
        <Icon name="drag-move" size="16px" />
    </div>
</template>
