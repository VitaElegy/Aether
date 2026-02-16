<script setup lang="ts">
import { useEditor, EditorContent } from '@tiptap/vue-3';
import StarterKit from '@tiptap/starter-kit';
import Placeholder from '@tiptap/extension-placeholder';
import { Markdown } from 'tiptap-markdown';
import { onBeforeUnmount, ref } from 'vue';
import DragHandle from '../extensions/DragHandle.vue';
import SlashCommand from '../extensions/slash-command';
import SlashMenu from '../extensions/SlashMenu.vue';
import tippy from 'tippy.js';

const props = defineProps<{
  initialContent?: string;
  placeholder?: string;
  readOnly?: boolean;
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void;
  (e: 'change', isDirty: boolean): void;
  (e: 'update:toc', toc: any[]): void;
}>();

// TOC Logic (Ported)
const updateToc = (editor: any) => {
  const headings: any[] = [];
  let currentH3Id: string | null = null;

  editor.state.doc.descendants((node: any, pos: number) => {
    if (node.type.name === 'heading') {
      const id = `heading-${headings.length}`;
      const level = node.attrs.level;
      if (level <= 2) currentH3Id = null;
      else if (level === 3) currentH3Id = id;

      headings.push({
        id,
        text: node.textContent,
        level,
        parentId: level > 3 ? currentH3Id : null,
        pos
      });
    }
  });

  emit('update:toc', headings);
};

// Slash Command State
const slashVisible = ref(false);
const slashProps = ref<any>({});
const slashPosition = ref({ top: 0, left: 0 });
const slashMenuRef = ref<any>(null);

const executeCommand = (item: any) => {
    const { command } = item;
    const { editor, range } = slashProps.value;
    command({ editor, range });
    slashVisible.value = false;
};

const editor = useEditor({
  extensions: [
    StarterKit.configure({ heading: { levels: [1, 2, 3, 4, 5] } }),
    Markdown,
    Placeholder.configure({ placeholder: props.placeholder || 'Start writing... (Markdown supported)' }),
    SlashCommand.configure({
        suggestion: {
            items: ({ query }: { query: string }) => {
                return [
                    { title: 'Text', description: 'Just start writing with plain text.', iconClass: 'ri-text', command: ({ editor, range }: { editor: any, range: any }) => editor.chain().focus().deleteRange(range).setNode('paragraph').run() },
                    { title: 'Heading 1', description: 'Big section heading.', iconClass: 'ri-h-1', command: ({ editor, range }: { editor: any, range: any }) => editor.chain().focus().deleteRange(range).setNode('heading', { level: 1 }).run() },
                    { title: 'Heading 2', description: 'Medium section heading.', iconClass: 'ri-h-2', command: ({ editor, range }: { editor: any, range: any }) => editor.chain().focus().deleteRange(range).setNode('heading', { level: 2 }).run() },
                    { title: 'Heading 3', description: 'Small section heading.', iconClass: 'ri-h-3', command: ({ editor, range }: { editor: any, range: any }) => editor.chain().focus().deleteRange(range).setNode('heading', { level: 3 }).run() },
                    { title: 'Bullet List', description: 'Create a simple bulleted list.', iconClass: 'ri-list-unordered', command: ({ editor, range }: { editor: any, range: any }) => editor.chain().focus().deleteRange(range).toggleBulletList().run() },
                    { title: 'Numbered List', description: 'Create a list with numbering.', iconClass: 'ri-list-ordered', command: ({ editor, range }: { editor: any, range: any }) => editor.chain().focus().deleteRange(range).toggleOrderedList().run() },
                    { title: 'Quote', description: 'Capture a quote.', iconClass: 'ri-double-quotes-l', command: ({ editor, range }: { editor: any, range: any }) => editor.chain().focus().deleteRange(range).toggleBlockquote().run() },
                    { title: 'Code Block', description: 'Capture a code snippet.', iconClass: 'ri-code-box-line', command: ({ editor, range }: { editor: any, range: any }) => editor.chain().focus().deleteRange(range).toggleCodeBlock().run() },
                ].filter(item => item.title.toLowerCase().startsWith(query.toLowerCase())).slice(0, 10);
            },
            render: () => {
                return {
                    onStart: (props: any) => {
                        slashProps.value = props;
                        slashVisible.value = true;
                        const rect = props.clientRect();
                        if (rect) {
                           slashPosition.value = { top: rect.bottom, left: rect.left };
                        }
                    },
                    onUpdate: (props: any) => {
                         slashProps.value = props;
                         const rect = props.clientRect();
                         if (rect) {
                             slashPosition.value = { top: rect.bottom, left: rect.left };
                         }
                    },
                    onKeyDown: (props: any) => {
                        if (props.event.key === 'Escape') {
                            slashVisible.value = false;
                            return true;
                        }
                        return slashMenuRef.value?.onKeyDown(props);
                    },
                    onExit: () => {
                        slashVisible.value = false;
                    },
                }
            }
        }
    }),
  ],
  editorProps: {
attributes: { class: 'prose prose-neutral dark:prose-invert max-w-none focus:outline-none min-h-[500px] outline-none' },
  },
  editable: !props.readOnly,
  onUpdate: ({ editor }) => {
    const content = (editor.storage as any).markdown.getMarkdown();
    emit('update:modelValue', content);
    emit('change', true);
    updateToc(editor);
  },
  onCreate: ({ editor }) => {
      if (props.initialContent) {
          editor.commands.setContent(props.initialContent);
      }
      updateToc(editor);
  }
});

// Adapter Interface Implementation
const load = async (content: any) => {
    if (editor.value) {
        editor.value.commands.setContent(content);
        updateToc(editor.value);
    }
};

const getValue = () => {
    return (editor.value?.storage as any).markdown.getMarkdown() || '';
};

const exportContent = async (format: 'markdown' | 'json'): Promise<string | Blob> => {
    if (format === 'markdown') {
        return getValue();
    }
    // TODO: JSON support
    return JSON.stringify(editor.value?.getJSON());
};

const importContent = async (content: any, format: 'markdown' | 'json') => {
    if (format === 'markdown') {
        await load(content);
    }
};

const scrollToPosition = (pos: number) => {
    if (!editor.value) return;
    editor.value.commands.setTextSelection(pos + 1);
    editor.value.commands.focus();
    const { view } = editor.value;
    const dom = view.nodeDOM(pos) as HTMLElement;
    if (dom && dom.scrollIntoView) dom.scrollIntoView({ behavior: 'smooth', block: 'center' });
};

defineExpose({
    load,
    getValue,
    export: exportContent,
    import: importContent,
    scrollToPosition // Extra helper for TOC
});

// onBeforeUnmount(() => {
//    editor.value?.destroy();
// });
</script>

<template>
  <div class="h-full w-full relative group overflow-visible">
    <editor-content :editor="editor" class="tiptap-editor h-full" />
    <DragHandle v-if="editor" :editor="editor" />
    
    <div 
        v-if="slashVisible" 
        class="fixed z-50 transition-all duration-100"
        :style="{ top: `${slashPosition.top}px`, left: `${slashPosition.left}px` }"
    >
        <SlashMenu 
            ref="slashMenuRef"
            :items="slashProps.items" 
            :command="executeCommand"
            :editor="editor"
        />
    </div>
  </div>
</template>

<style>
.tiptap-editor .ProseMirror { outline: none; }
.tiptap-editor .ProseMirror h1 { margin-top: 0.4em; margin-bottom: 0.2em; font-size: 2.25em; line-height: 1.1; letter-spacing: -0.025em; font-weight: 800; }
.tiptap-editor .ProseMirror h2 { margin-top: 0.6em; margin-bottom: 0.2em; font-size: 1.5em; letter-spacing: -0.025em; font-weight: 700; }
.tiptap-editor .ProseMirror h3 { margin-top: 0.4em; margin-bottom: 0.2em; font-size: 1.25em; font-weight: 600; }
.tiptap-editor .ProseMirror p { margin-bottom: 0.4em; line-height: 1.6; }
.tiptap-editor .ProseMirror ul, .tiptap-editor .ProseMirror ol { padding-left: 1.5em; }
.tiptap-editor .ProseMirror li { margin-bottom: 0.2em; }
.tiptap-editor .ProseMirror p.is-editor-empty:first-child::before { color: #d4d4d4; content: attr(data-placeholder); float: left; height: 0; pointer-events: none; }
</style>
