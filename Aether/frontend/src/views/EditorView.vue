<script setup lang="ts">
import { ref, reactive, onBeforeUnmount, watch, onMounted, nextTick } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import axios from 'axios';
import { MessagePlugin } from 'tdesign-vue-next';
import { useEditor, EditorContent } from '@tiptap/vue-3';
import StarterKit from '@tiptap/starter-kit';
import Placeholder from '@tiptap/extension-placeholder';
import { Markdown } from 'tiptap-markdown';
import { useDebounceFn, useStorage } from '@vueuse/core';
import TopNavBar from '@/components/TopNavBar.vue';
import CollaboratorModal from './CollaboratorModal.vue';
import { usePreferencesStore } from '@/stores/preferences';
import { knowledgeApi, type KnowledgeBase } from '@/api/knowledge';
import { useContent } from '@/composables/useContent';
import { draftApi, type DraftData } from '@/api/draft';

const router = useRouter();
const route = useRoute();
const knowledgeBases = ref<KnowledgeBase[]>([]);
const { article, load, save, create, isSaving } = useContent();

// State
const isLiveMode = ref(true);
const toc = ref<{ id: string; text: string; level: number; parentId: string | null; isVisible: boolean; pos: number }[]>([]);
const activeHeadingId = ref<string | null>(null);
const draftId = ref<string | null>(route.params.id as string || null);
// isSaving handled by useContent now
const autoSaveEnabled = ref(true);
const timestamps = ref<{ created: string | null; updated: string | null }>({ created: null, updated: null });
const isRestoring = ref(true); // Flag to prevent auto-save during initialization
const isSyncing = ref(false); // Flag to indicate server-side sync, preventing cache timestamp updates

const prefStore = usePreferencesStore();
const showCommitModal = ref(false);
const showCollaboratorModal = ref(false);
const commitMessage = ref('');

// Local Storage Cache - Content ONLY (No status/lifecycle)
// const localCache = useStorage('aether_editor_current', { ... }); 
// REPLACED BY SERVER SIDE DRAFT API

const form = reactive({
  title: '',
  knowledge_base_id: null as string | null,
  parent_id: null as string | null,
  body: '',
  category: '',
  tags: [] as string[],
  visibility: 'Public',
  // status is managed by 'article' state now, but we keep a local reactive copy for binding if needed, 
  // OR strictly bind to 'article.status' if we want single source of truth.
  // Ideally, form binds to local state which syncs with 'article'.
  status: 'Draft' 
});

// Sync form with article when loaded
watch(() => article.value, async (newArticle) => {
    isSyncing.value = true;
    if (newArticle) {
        form.title = newArticle.title || form.title;
        // Don't overwrite body if user is editing (debounced sync?)
        // Actually, load() is only called on mount/id change. So safe to overwrite.
        if (isRestoring.value) {
             form.body = (typeof newArticle.body === 'string' ? newArticle.body : form.body);
             if (editor.value && form.body) editor.value.commands.setContent(form.body);
        }
        
        form.tags = newArticle.tags || [];
        form.category = newArticle.category || '';
        form.visibility = newArticle.visibility as string || 'Public';
        form.status = newArticle.status || 'Draft';
        form.knowledge_base_id = newArticle.knowledge_base_id;
        form.parent_id = newArticle.parent_id;
        
        // Timestamps
        timestamps.value = {
            created: new Date(newArticle.created_at).toISOString(),
            updated: new Date(newArticle.updated_at).toISOString()
        };

    }
    await nextTick();
    isSyncing.value = false;
});

const formatDate = (isoStr: string | null) => {
  if (!isoStr) return '--';
  return new Date(isoStr).toLocaleString('en-US', { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' });
};

// TOC Logic
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
    attributes: { class: 'prose prose-neutral prose-lg dark:prose-invert max-w-none focus:outline-none min-h-[500px]' },
  },
  onUpdate: ({ editor }) => {
    form.body = (editor.storage as any).markdown.getMarkdown();
    updateToc(editor);
  },
  onSelectionUpdate: ({ editor }) => updateToc(editor),
  onCreate: ({ editor }) => updateToc(editor)
});

// Initialization
// Initialization
onMounted(async () => {
  // 1. Check for Server-Side Draft
  const serverDraft = await draftApi.get();
  let draftLoaded = false;

  if (serverDraft) {
      const isNewEntry = !draftId.value;
      const conflictsWithCurrent = draftId.value && serverDraft.target_article_id && draftId.value !== serverDraft.target_article_id;
      
      let shouldLoad = false;

      if (conflictsWithCurrent) {
        // User is opening Article A, but has a draft for Article B.
        // Prompt user (Simplification: Just notify for now, or maybe don't load? 
        // For this fix, let's assume we ONLY load if it matches or is new)
        MessagePlugin.warning('You have an active draft for another article. It was not loaded.', 5000);
      } else {
        // Match! Load it.
        shouldLoad = true;
      }

      if (shouldLoad) {
          form.title = serverDraft.title || form.title;
          form.body = serverDraft.body || form.body;
          form.tags = serverDraft.tags || form.tags;
          form.category = serverDraft.category || form.category;
          form.knowledge_base_id = serverDraft.knowledge_base_id || form.knowledge_base_id;
          // form.parent_id = serverDraft.parent_id || form.parent_id; // Backend doesn't support yet
          
          if (editor.value && form.body) editor.value.commands.setContent(form.body);
          MessagePlugin.info('Restored unsaved draft from server.', 3000);
          draftLoaded = true;
      }
  }

  if (draftId.value && !draftLoaded) {
    await load(draftId.value);
    // If we loaded from live, form watcher will sync via 'watch(article)' below
  } else if (!draftId.value && !draftLoaded) {
     timestamps.value = { created: new Date().toISOString(), updated: new Date().toISOString() };
     
     const intendedKbId = route.query.knowledge_base_id as string | undefined;
     const intendedParentId = route.query.parent_id as string | undefined;
     if (intendedKbId) form.knowledge_base_id = intendedKbId;
     if (intendedParentId) form.parent_id = intendedParentId;
  }
 
  if (editor.value && form.body && !draftLoaded) {
       editor.value.commands.setContent(form.body);
  }

  // Fetch KBs ...



  // Fetch KBs
  try {
      knowledgeBases.value = await knowledgeApi.list();
  } catch (e) {
      console.error("Failed to fetch KBs", e);
  } finally {
      // Allow reactivity to settle before enabling auto-save
      setTimeout(() => {
          isRestoring.value = false;
      }, 500);
  }
});


// Auto-Save managed by Draft API (Decoupled from Live)
const saveDraft = async () => {
  if (!form.title && !form.body) return;
  
  // Always save to Draft API, never touch live content auto-magically
  try {
       await draftApi.save({
          target_article_id: draftId.value || null,
          title: form.title,
          body: form.body,
          tags: form.tags,
          category: form.category,
          knowledge_base_id: form.knowledge_base_id,
          parent_id: form.parent_id // Pass it even if backend drops it for now
      });
      timestamps.value.updated = new Date().toISOString();
  } catch (e) {
      console.error("Auto-save failed", e);
  }
};

const debouncedAutoSave = useDebounceFn(() => {
  if (autoSaveEnabled.value) saveDraft();
}, 2000);

// Decoupled Watchers
// 1. Removed Local Cache Watcher
// watch(form, (newVal) => { ... }, { deep: true });

// 2. Server Auto-Save: Only trigger if NOT restoring and enabled
watch(form, () => {
  if (!isRestoring.value && autoSaveEnabled.value) {
     debouncedAutoSave();
  }
}, { deep: true });

// Toggle Mode
const toggleMode = () => {
  isLiveMode.value = !isLiveMode.value;
  if (isLiveMode.value) editor.value?.commands.setContent(form.body);
  else form.body = (editor.value?.storage as any).markdown.getMarkdown() || form.body;
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

const onPublishClick = () => {
  commitMessage.value = prefStore.defaultCommitMessage;
  showCommitModal.value = true;
};

const executePublish = async () => {
    if (isPublishing.value) return;
    showCommitModal.value = false;

    if (debouncedAutoSave && typeof (debouncedAutoSave as any).cancel === 'function') {
        (debouncedAutoSave as any).cancel();
    }

   // Wait for pending auto-save handled by useContent via isSaving ref
    if (isSaving.value) {
         while (isSaving.value) {
            await new Promise(resolve => setTimeout(resolve, 100));
        }
    }

    isPublishing.value = true;
    if (isLiveMode.value && editor.value) form.body = (editor.value.storage as any).markdown.getMarkdown();

    try {
        // Check for ID in article state (source of truth) even if draftId ref lags
        const currentId = draftId.value || article.value?.id;

        if (!currentId) {
            // Create New as Published
             const payload = {
              title: form.title || 'Untitled',
              body: form.body,
              tags: form.tags,
              category: form.category || '',
              visibility: form.visibility as any,
              status: 'Published' as any, // TypeScript Cast
              knowledge_base_id: form.knowledge_base_id || null,
              parent_id: form.parent_id || null,
              reason: commitMessage.value,
            };
            const newId = await create(payload);
            draftId.value = newId;
        } else {
            // Update Existing to Published
            if (!draftId.value) draftId.value = currentId; // Sync local ref if needed
            await save({
              title: form.title,
              body: form.body,
              tags: form.tags,
              category: form.category,
              visibility: form.visibility as any,
              status: 'Published', // Explicitly set status to Published
              reason: commitMessage.value,
              knowledge_base_id: form.knowledge_base_id,
              parent_id: form.parent_id,
            });
        }



        // Cleanup Draft
        await draftApi.delete();
        
        MessagePlugin.success('Published.');
        await router.push('/');
    } catch (err: any) {
         if (err.response && err.response.status === 409) {
             MessagePlugin.warning('Article with this title already exists.', 3000);
         } else {
             MessagePlugin.error('Failed to publish.');
         }
    } finally {
        isPublishing.value = false;
    }
};

// Navigation
const goBack = () => {
  // Check if there is a history entry to go back to within the router's state
  if (window.history.state && window.history.state.back) {
    router.back();
  } else if (draftId.value && form.knowledge_base_id) {
    // Context-aware fallback: Go to Knowledge Base
    // Ideally we should go to the specific folder if possible, but we need a route for that.
    // SelfSpace uses internal state for folders, so simple routing might just go to root KB.
    // But let's try pushing to the space view.
    router.push('/space'); 
  } else {
    // Fallback to home if no history (e.g. direct link or fresh tab)
    router.push('/');
  }
};

onBeforeUnmount(() => {
  try {
    if (debouncedAutoSave && typeof (debouncedAutoSave as any).cancel === 'function') {
      (debouncedAutoSave as any).cancel();
    }
    if (editor.value) {
      editor.value.destroy();
    }
  } catch (e) {
    console.warn('Cleanup error during unmount (ignored to allow navigation):', e);
  }
});
</script>

<template>
  <div class="h-screen w-full flex flex-col bg-paper overflow-hidden">
    <!-- Header -->
    <TopNavBar>
       <template #left>
          <div class="flex items-center gap-6">
             <button @click="goBack" class="text-neutral-400 hover:text-ink transition-colors flex items-center gap-2" title="Go Back">
                <i class="ri-arrow-left-line text-xl"></i>
             </button>

             <div class="h-4 w-px bg-neutral-200/50"></div>

             <button v-if="draftId" @click="router.push(`/content/${draftId}/history`)" class="flex items-center gap-2 group cursor-pointer hover:text-ink text-neutral-400 transition-colors" title="View Version History">
                <i class="ri-history-line text-lg"></i>
                <span class="text-[10px] uppercase tracking-widest group-hover:text-neutral-600 hidden sm:inline">History</span>
             </button>
          </div>
       </template>

       <template #center>
          <div class="flex items-center gap-3">
             <span class="text-xs font-mono uppercase tracking-widest text-neutral-400 block">
               Editor / {{ form.status === 'Published' ? 'Editing Published' : (draftId ? 'Editing Draft' : 'New Entry') }}
             </span>
             <span v-if="isSaving" class="text-[10px] text-neutral-300 animate-pulse uppercase tracking-widest">Saving...</span>
             <span v-else-if="draftId" class="text-[10px] text-neutral-300 uppercase tracking-widest">
                {{ form.status === 'Published' ? 'Published' : 'Draft Saved' }}
             </span>
          </div>
       </template>

       <template #right>
          <div class="flex items-center gap-6">
             <div class="flex items-center gap-2 mr-2 group cursor-pointer" @click="autoSaveEnabled = !autoSaveEnabled">
                <div class="w-2 h-2 rounded-full transition-colors" :class="autoSaveEnabled ? 'bg-green-500' : 'bg-neutral-300'"></div>
                <span class="text-[10px] uppercase tracking-widest text-neutral-400 group-hover:text-neutral-600">Auto-Save</span>
             </div>

             <div class="flex items-center gap-3">
                <button v-if="draftId" @click="showCollaboratorModal = true" class="text-xs font-bold uppercase tracking-widest text-neutral-400 hover:text-ink transition-colors mr-4">
                  Collaborators
                </button>

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

             <button @click="onPublishClick" :disabled="isPublishing" class="bg-ink text-paper px-6 py-2 text-xs font-bold uppercase tracking-widest hover:bg-neutral-800 transition-colors disabled:opacity-50 disabled:cursor-not-allowed text-center min-w-[100px]">
               {{ isPublishing ? '...' : 'Publish' }}
             </button>
          </div>
       </template>
    </TopNavBar>

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
                class="w-full h-full min-h-[500px] resize-none text-base leading-relaxed text-neutral-700 dark:text-neutral-200 placeholder:text-neutral-200 focus:outline-none bg-transparent font-mono pb-32"
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
           <label class="block text-[10px] font-bold uppercase tracking-widest mb-2 text-neutral-400">Knowledge Base</label>
           <select v-model="form.knowledge_base_id" class="w-full bg-transparent border-b border-neutral-200 py-2 text-xs font-medium focus:outline-none focus:border-ink cursor-pointer">
             <option :value="null">None</option>
             <option v-for="kb in knowledgeBases" :key="kb.id" :value="kb.id">{{ kb.title }}</option>
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

    <!-- Commit Modal -->
    <Transition
      enter-active-class="transition duration-200 ease-out"
      enter-from-class="opacity-0 scale-95"
      enter-to-class="opacity-100 scale-100"
      leave-active-class="transition duration-150 ease-in"
      leave-from-class="opacity-100 scale-100"
      leave-to-class="opacity-0 scale-95"
    >
      <div v-if="showCommitModal" class="fixed inset-0 z-50 flex items-center justify-center p-4">
        <!-- Backdrop -->
        <div class="absolute inset-0 bg-white/80 dark:bg-black/80 backdrop-blur-sm" @click="showCommitModal = false"></div>

        <!-- Modal Content -->
        <div class="relative bg-paper w-full max-w-md border border-neutral-100 shadow-2xl p-8 flex flex-col gap-6 animate-in fade-in zoom-in-95 duration-200">
           <div>
              <h3 class="text-xl font-bold text-ink">Save Changes</h3>
              <p class="text-xs text-neutral-400 mt-1 uppercase tracking-widest">Describes this version in history</p>
           </div>

           <textarea
             v-model="commitMessage"
             class="w-full h-32 bg-ash/30 p-4 text-sm font-mono text-ink focus:outline-none focus:ring-1 focus:ring-ink resize-none placeholder:text-neutral-400"
             placeholder="What did you change?"
             autofocus
           ></textarea>

           <div class="flex justify-end gap-4">
              <button @click="showCommitModal = false" class="text-xs font-bold uppercase tracking-widest text-neutral-400 hover:text-ink transition-colors">
                Cancel
              </button>
              <button @click="executePublish" class="bg-ink text-paper px-6 py-2 text-xs font-bold uppercase tracking-widest hover:bg-neutral-800 transition-colors">
                Commit & Publish
              </button>
           </div>
        </div>
      </div>
    </Transition>

    <CollaboratorModal 
      v-if="draftId"
      :visible="showCollaboratorModal" 
      :article-id="draftId"
      @close="showCollaboratorModal = false"
    />
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
