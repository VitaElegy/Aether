<script setup lang="ts">
import { ref, reactive, onBeforeUnmount, watch, onMounted } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import axios from 'axios';
import { MessagePlugin } from 'tdesign-vue-next';
import { useEditor, EditorContent } from '@tiptap/vue-3';
import StarterKit from '@tiptap/starter-kit';
import Placeholder from '@tiptap/extension-placeholder';
import { Markdown } from 'tiptap-markdown';
import { useDebounceFn, useStorage } from '@vueuse/core';

const router = useRouter();
const route = useRoute();

// State
const isLiveMode = ref(true);
const toc = ref<{ id: string; text: string; level: number; parentId: string | null; isVisible: boolean; pos: number }[]>([]);
const activeHeadingId = ref<string | null>(null);
const draftId = ref<string | null>(route.params.id as string || null);
const isSaving = ref(false);
const autoSaveEnabled = ref(true);
const timestamps = ref<{ created: string | null; updated: string | null }>({ created: null, updated: null });

// Local Storage Cache
const localCache = useStorage('aether_editor_current', {
  title: '',
  body: '',
  category: '',
  tags: [] as string[],
  visibility: 'Public',
  status: 'Draft',
  timestamp: 0
});

const form = reactive({
  title: '',
  body: '',
  category: '',
  tags: [] as string[],
  visibility: 'Public',
  status: 'Draft'
});

const formatDate = (isoStr: string | null) => {
  if (!isoStr) return '--';
  return new Date(isoStr).toLocaleString('en-US', { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' });
};

// ... TOC updateToc ...
const updateToc = (editor: any) => {
  const headings: { id: string; text: string; level: number; parentId: string | null; pos: number }[] = [];
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

  const { from } = editor.state.selection;
  let currentActiveId = null;
  for (let i = headings.length - 1; i >= 0; i--) {
    if (headings[i].pos <= from) {
      currentActiveId = headings[i].id;
      break;
    }
  }
  activeHeadingId.value = currentActiveId;

  let activeContextH3Id: string | null = null;
  if (currentActiveId) {
    const activeItem = headings.find(h => h.id === currentActiveId);
    if (activeItem) {
      if (activeItem.level === 3) activeContextH3Id = activeItem.id;
      else if (activeItem.level > 3) activeContextH3Id = activeItem.parentId;
    }
  }

  toc.value = headings.map(h => ({
    id: h.id,
    text: h.text,
    level: h.level,
    parentId: h.parentId,
    isVisible: h.level <= 3 || h.parentId === activeContextH3Id,
    pos: h.pos
  }));
};

// Tiptap Editor Setup
const editor = useEditor({
  extensions: [
    StarterKit.configure({ heading: { levels: [1, 2, 3, 4, 5] } }),
    Markdown,
    Placeholder.configure({ placeholder: 'Start writing... (Markdown supported)' }),
  ],
  editorProps: {
    attributes: { class: 'prose prose-neutral prose-lg max-w-none focus:outline-none min-h-[500px]' },
  },
  onUpdate: ({ editor }) => {
    form.body = editor.storage.markdown.getMarkdown();
    updateToc(editor);
  },
  onSelectionUpdate: ({ editor }) => updateToc(editor),
  onCreate: ({ editor }) => updateToc(editor)
});

// Initialization
onMounted(async () => {
  if (draftId.value) {
    try {
      const res = await axios.get(`/api/content/${draftId.value}`);
      const data = res.data;

      timestamps.value = {
        created: data.created_at,
        updated: data.updated_at
      };

      if (localCache.value.body && localCache.value.timestamp > new Date(data.updated_at).getTime()) {
         MessagePlugin.info('Restored unsaved progress from local cache.', 2000);
         Object.assign(form, localCache.value);
      } else {
         form.title = data.title;
         form.body = data.body.data;
         form.tags = data.tags;
         form.category = data.category;
         form.visibility = data.visibility;
         form.status = data.status;
      }
    } catch (err) {
      console.error('Failed to fetch draft', err);
      if (localCache.value.body) {
         Object.assign(form, localCache.value);
         MessagePlugin.warning('Server offline. Restored local cache.');
      }
    }
  } else {
    // New Entry
    timestamps.value = { created: new Date().toISOString(), updated: new Date().toISOString() };
    if (localCache.value.body || localCache.value.title) {
       Object.assign(form, localCache.value);
       if (form.body.length > 10) MessagePlugin.info('Restored unsaved draft.', 2000);
    }
  }

  if (editor.value && form.body) {
    editor.value.commands.setContent(form.body);
  }
});

// Auto-Save
const saveDraft = async () => {
  if (!form.title && !form.body) return;

  isSaving.value = true;
  try {
    const payload = {
      title: form.title || 'Untitled Draft',
      body: form.body,
      tags: form.tags,
      category: form.category || null,
      visibility: form.visibility,
      status: 'Draft'
    };

    // Explicitly set header with current token
    const token = localStorage.getItem('aether_token');
    const config = {
      headers: { Authorization: `Bearer ${token}` }
    };

    let res;
    if (draftId.value) {
      res = await axios.put(`/api/content/${draftId.value}`, payload, config);
    } else {
      res = await axios.post('/api/content', payload, config);
      draftId.value = res.data.id;
      router.replace({ name: 'editor', params: { id: draftId.value } });
    }
    // Update local timestamp feedback
    timestamps.value.updated = new Date().toISOString();
  } catch (err: any) {
    if (err.response && err.response.status === 409) {
        const existingId = err.response.data.id;
        if (existingId && existingId !== draftId.value) {
            draftId.value = existingId;
            router.replace({ name: 'editor', params: { id: existingId } });
            MessagePlugin.warning('Title matches existing entry. Switched to editing mode.', 3000);
        }
    } else {
        console.error('Auto-save failed', err);
    }
  } finally {
    isSaving.value = false;
  }
};

const debouncedAutoSave = useDebounceFn(() => {
  if (autoSaveEnabled.value) saveDraft();
}, 2000);

watch(form, (newVal) => {
  localCache.value = { ...newVal, timestamp: Date.now() };
  debouncedAutoSave();
}, { deep: true });

// Toggle Mode
const toggleMode = () => {
  isLiveMode.value = !isLiveMode.value;
  if (isLiveMode.value) editor.value?.commands.setContent(form.body);
  else form.body = editor.value?.storage.markdown.getMarkdown() || form.body;
};

// Scroll
const scrollToHeading = (pos: number) => {
  if (!editor.value) return;
  editor.value.commands.setTextSelection(pos + 1);
  editor.value.commands.focus();
  const { view } = editor.value;
  const dom = view.nodeDOM(pos) as HTMLElement;
  if (dom && dom.scrollIntoView) dom.scrollIntoView({ behavior: 'smooth', block: 'center' });
};

// Publish
const isPublishing = ref(false);

const handlePublish = async () => {
  if (isPublishing.value) return;

  // Cancel any pending auto-save
  if (debouncedAutoSave && typeof debouncedAutoSave.cancel === 'function') {
      debouncedAutoSave.cancel();
  }

  // Wait for any ongoing auto-save to finish to ensure draftId is stable
  if (isSaving.value) {
      const waitForSave = async () => {
          while (isSaving.value) {
              await new Promise(resolve => setTimeout(resolve, 100));
          }
      };
      await waitForSave();
  }

  isPublishing.value = true;
  if (isLiveMode.value && editor.value) form.body = editor.value.storage.markdown.getMarkdown();

  try {
    const payload = {
      title: form.title || 'Untitled',
      body: form.body,
      tags: form.tags,
      category: form.category || null,
      visibility: form.visibility,
      status: 'Published'
    };

    // Explicitly set header with current token
    const token = localStorage.getItem('aether_token');
    const config = {
      headers: { Authorization: `Bearer ${token}` }
    };

    if (draftId.value) {
        await axios.put(`/api/content/${draftId.value}`, payload, config);
    } else {
        const res = await axios.post('/api/content', payload, config);
        // Ensure we capture the ID even though we leave, to prevent re-creation if user stays on page by error
        draftId.value = res.data.id;
    }

    localCache.value = null; // Clear cache on publish
    MessagePlugin.success('Published.');
    await router.push('/');
  } catch (err: any) {
    if (err.response && err.response.status === 409) {
        MessagePlugin.warning('Article with this title already exists. Redirecting to edit mode...', 2000);
        const existingId = err.response.data.id;
        if (existingId) {
             // Delay slightly to let toast show
             setTimeout(() => {
                draftId.value = existingId;
                router.push(`/editor/${existingId}`);
             }, 1000);
        }
    } else {
        console.error(err);
        MessagePlugin.error('Failed to publish.');
    }
  } finally {
    isPublishing.value = false;
  }
};

onBeforeUnmount(() => {
  editor.value?.destroy();
});
</script>

<template>
  <div class="h-screen w-full flex flex-col bg-paper overflow-hidden">
    <!-- Header -->
    <header class="h-16 flex-shrink-0 flex items-center justify-between px-6 border-b border-neutral-100 bg-paper/95 backdrop-blur z-20">
       <div class="flex items-center gap-4">
          <button @click="router.back()" class="text-neutral-400 hover:text-ink transition-colors">
            <i class="ri-arrow-left-line text-xl"></i>
          </button>
          <span class="text-xs font-mono uppercase tracking-widest text-neutral-400 hidden sm:block">
            Editor / {{ draftId ? 'Editing Draft' : 'New Entry' }}
          </span>
          <span v-if="isSaving" class="text-[10px] text-neutral-300 animate-pulse uppercase tracking-widest">Saving...</span>
          <span v-else-if="draftId" class="text-[10px] text-neutral-300 uppercase tracking-widest">Saved</span>
       </div>

       <div class="flex items-center gap-6">
          <button v-if="draftId" @click="router.push(`/content/${draftId}/history`)" class="flex items-center gap-2 group cursor-pointer hover:text-ink text-neutral-400 transition-colors" title="View Version History">
             <i class="ri-history-line text-lg"></i>
             <span class="text-[10px] uppercase tracking-widest group-hover:text-neutral-600 hidden sm:inline">History</span>
          </button>
         <div class="flex items-center gap-2 mr-4 group cursor-pointer" @click="autoSaveEnabled = !autoSaveEnabled">
            <div class="w-2 h-2 rounded-full transition-colors" :class="autoSaveEnabled ? 'bg-green-500' : 'bg-neutral-300'"></div>
            <span class="text-[10px] uppercase tracking-widest text-neutral-400 group-hover:text-neutral-600">Auto-Save</span>
         </div>

         <div class="flex items-center gap-3">
            <span class="text-[10px] font-bold uppercase tracking-widest text-neutral-400 transition-colors" :class="{ 'text-ink': isLiveMode }">Live</span>
            <button
              @click="toggleMode"
              class="w-10 h-5 rounded-full bg-ash relative transition-colors focus:outline-none"
              :class="{ 'bg-ink': !isLiveMode }"
            >
               <div class="absolute top-1 left-1 w-3 h-3 bg-white rounded-full transition-transform duration-200 shadow-sm" :class="{ 'translate-x-5': !isLiveMode }"></div>
            </button>
            <span class="text-[10px] font-bold uppercase tracking-widest text-neutral-400 transition-colors" :class="{ 'text-ink': !isLiveMode }">Raw</span>
         </div>
         <div class="h-4 w-px bg-neutral-200"></div>
         <button @click="handlePublish" :disabled="isPublishing" class="bg-ink text-paper px-6 py-2 text-xs font-bold uppercase tracking-widest hover:bg-neutral-800 transition-colors disabled:opacity-50 disabled:cursor-not-allowed">
           {{ isPublishing ? 'Publishing...' : 'Publish' }}
         </button>
       </div>
    </header>

    <div class="flex-1 flex overflow-hidden">
      <!-- Left Sidebar: TOC -->
      <aside class="w-64 flex-shrink-0 hidden xl:flex flex-col border-r border-neutral-100 bg-paper p-8 overflow-y-auto custom-scrollbar">
        <div class="text-[10px] font-bold uppercase tracking-widest text-neutral-400 mb-6">Table of Contents</div>
        <nav v-if="toc.length > 0" class="flex flex-col gap-1 transition-all">
           <template v-for="(item, idx) in toc" :key="idx">
             <a
              v-show="item.isVisible"
              href="#"
              @click.prevent="scrollToHeading(item.pos)"
              class="block text-xs transition-colors leading-tight py-1 border-l-2 pl-3"
              :class="[
                activeHeadingId === item.id ? 'border-ink text-ink font-bold' : 'border-transparent text-neutral-400 hover:text-neutral-600',
                item.level === 1 ? 'ml-0' : '',
                item.level === 2 ? 'ml-2' : '',
                item.level === 3 ? 'ml-4' : '',
                item.level === 4 ? 'ml-6' : '',
                item.level === 5 ? 'ml-8' : ''
              ]"
            >
              {{ item.text }}
            </a>
           </template>
        </nav>
        <div v-else class="text-xs text-neutral-300 italic font-mono">
          Headings will appear here...
        </div>
      </aside>

      <!-- Main Editor -->
      <main class="flex-1 overflow-y-auto relative custom-scrollbar">
         <div class="max-w-3xl mx-auto px-8 py-12 flex flex-col min-h-full">
            <input
              v-model="form.title"
              placeholder="Untitled Entry"
              class="text-5xl font-bold tracking-tight mb-8 placeholder:text-neutral-200 focus:outline-none w-full bg-transparent border-none p-0 flex-shrink-0"
            />

            <div class="flex-1 relative">
              <editor-content v-if="isLiveMode" :editor="editor" class="tiptap-editor h-full pb-32" />
              <textarea
                v-else
                v-model="form.body"
                placeholder="# Start typing..."
                class="w-full h-full min-h-[500px] resize-none text-base leading-relaxed text-neutral-700 placeholder:text-neutral-200 focus:outline-none bg-transparent font-mono pb-32"
              ></textarea>
            </div>
         </div>
      </main>

      <!-- Right Sidebar: Metadata -->
      <aside class="w-72 flex-shrink-0 border-l border-neutral-100 bg-ash/20 p-6 flex flex-col gap-8 overflow-y-auto hidden lg:flex">
         <!-- Timestamp Section -->
         <div class="pb-6 border-b border-neutral-200">
            <div class="text-[10px] text-neutral-400 mb-2 uppercase tracking-widest">Timestamps</div>
            <div class="flex flex-col gap-2">
               <div class="flex justify-between items-center">
                  <span class="text-xs text-neutral-500">Created</span>
                  <span class="text-xs font-mono text-ink">{{ formatDate(timestamps.created) }}</span>
               </div>
               <div class="flex justify-between items-center">
                  <span class="text-xs text-neutral-500">Updated</span>
                  <span class="text-xs font-mono text-ink">{{ formatDate(timestamps.updated) }}</span>
               </div>
            </div>
         </div>

         <div>
           <label class="block text-[10px] font-bold uppercase tracking-widest mb-2 text-neutral-400">Visibility</label>
           <select v-model="form.visibility" class="w-full bg-transparent border-b border-neutral-200 py-2 text-xs font-medium focus:outline-none focus:border-ink cursor-pointer">
             <option>Public</option>
             <option>Internal</option>
             <option>Private</option>
           </select>
         </div>

         <div>
           <label class="block text-[10px] font-bold uppercase tracking-widest mb-2 text-neutral-400">Category</label>
           <input
             v-model="form.category"
             class="w-full bg-transparent border-b border-neutral-200 py-2 text-xs font-medium focus:outline-none focus:border-ink placeholder:text-neutral-300"
             placeholder="Add category"
           />
         </div>

         <div>
           <label class="block text-[10px] font-bold uppercase tracking-widest mb-2 text-neutral-400">Tags</label>
           <input
            :value="form.tags.join(', ')"
            @input="(e: any) => form.tags = e.target.value.split(',').map((t: string) => t.trim())"
            class="w-full bg-transparent border-b border-neutral-200 py-2 text-xs font-medium focus:outline-none focus:border-ink placeholder:text-neutral-300"
            placeholder="Comma separated"
          />
          <div class="flex flex-wrap gap-2 mt-3">
             <span v-for="tag in form.tags.filter(Boolean)" :key="tag" class="text-[10px] bg-white border border-neutral-200 px-2 py-1 rounded-sm text-neutral-500 uppercase tracking-wider">
               #{{ tag }}
             </span>
          </div>
         </div>

         <div class="mt-auto pt-6 border-t border-neutral-200">
            <div class="text-[10px] text-neutral-400 mb-2 uppercase tracking-widest">Stats</div>
            <div class="grid grid-cols-2 gap-4">
               <div>
                  <div class="text-xl font-bold text-ink">{{ form.body.length }}</div>
                  <div class="text-[10px] text-neutral-400">Chars</div>
               </div>
               <div>
                  <div class="text-xl font-bold text-ink">{{ form.body.split(/\s+/).filter(Boolean).length }}</div>
                  <div class="text-[10px] text-neutral-400">Words</div>
               </div>
            </div>
         </div>
      </aside>
    </div>
  </div>
</template>

<style>
.tiptap-editor .ProseMirror { outline: none; }
.tiptap-editor .ProseMirror h1 { margin-top: 1em; margin-bottom: 0.4em; font-size: 2.25em; line-height: 1.1; letter-spacing: -0.025em; font-weight: 800; }
.tiptap-editor .ProseMirror h2 { margin-top: 1.2em; margin-bottom: 0.4em; font-size: 1.5em; letter-spacing: -0.025em; font-weight: 700; }
.tiptap-editor .ProseMirror h3 { margin-top: 1em; margin-bottom: 0.3em; font-size: 1.25em; font-weight: 600; }
.tiptap-editor .ProseMirror p { margin-bottom: 1em; line-height: 1.75; }
.tiptap-editor .ProseMirror p.is-editor-empty:first-child::before { color: #d4d4d4; content: attr(data-placeholder); float: left; height: 0; pointer-events: none; }
.custom-scrollbar::-webkit-scrollbar { width: 4px; }
.custom-scrollbar::-webkit-scrollbar-thumb { background: #e5e5e5; border-radius: 2px; }
.custom-scrollbar::-webkit-scrollbar-track { background: transparent; }
</style>
