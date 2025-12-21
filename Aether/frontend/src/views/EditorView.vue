<script setup lang="ts">
import { ref, reactive } from 'vue';
import { useRouter } from 'vue-router';
import axios from 'axios';
import { MessagePlugin } from 'tdesign-vue-next';
import DynamicRenderer from '../components/DynamicRenderer.vue';

const router = useRouter();

const form = reactive({
  title: '',
  body: '',
  category: '',
  visibility: 'Public',
  tags: [] as string[]
});

const handlePublish = async () => {
  try {
    await axios.post('/api/content', {
        title: form.title,
        body: form.body,
        tags: form.tags,
        category: form.category || null,
        visibility: form.visibility
    });
    MessagePlugin.success('Published.');
    router.push('/');
  } catch (err) {
    MessagePlugin.error('Failed.');
  }
};
</script>

<template>
  <div class="h-screen w-full flex flex-col bg-paper">
    <!-- Minimal Header -->
    <header class="h-16 flex items-center justify-between px-6 border-b border-neutral-100 flex-shrink-0">
       <button @click="router.back()" class="text-neutral-400 hover:text-ink transition-colors">
         <i class="ri-arrow-left-line text-xl"></i>
       </button>

       <div class="flex items-center gap-4">
         <span class="text-xs font-mono uppercase tracking-widest text-neutral-400">Drafting Mode</span>
         <button @click="handlePublish" class="bg-ink text-paper px-6 py-2 text-xs font-bold uppercase tracking-widest hover:bg-neutral-800 transition-colors">
           Publish
         </button>
       </div>
    </header>

    <div class="flex-1 flex overflow-hidden">
      <!-- Main Writing Area -->
      <main class="flex-1 flex flex-col max-w-3xl mx-auto w-full pt-12 pb-24 px-8 overflow-y-auto">
         <input
          v-model="form.title"
          placeholder="Untitled Entry"
          class="text-5xl font-bold tracking-tight mb-8 placeholder:text-neutral-200 focus:outline-none w-full bg-transparent"
        />

        <textarea
          v-model="form.body"
          placeholder="Start typing..."
          class="flex-1 w-full resize-none text-lg leading-relaxed text-neutral-700 placeholder:text-neutral-200 focus:outline-none bg-transparent font-serif"
        ></textarea>
      </main>

      <!-- Sidebar Options (Collapsible concept, currently static) -->
      <aside class="w-80 border-l border-neutral-100 bg-ash/30 p-8 hidden xl:flex flex-col gap-8">
         <div>
           <label class="block text-xs font-bold uppercase tracking-widest mb-2 text-neutral-400">Category</label>
           <input v-model="form.category" class="w-full bg-transparent border-b border-neutral-200 py-1 text-sm focus:outline-none focus:border-ink" placeholder="Add category"/>
         </div>

         <div>
           <label class="block text-xs font-bold uppercase tracking-widest mb-2 text-neutral-400">Tags</label>
           <input
            :value="form.tags.join(', ')"
            @input="(e: any) => form.tags = e.target.value.split(',').map((t: string) => t.trim())"
            class="w-full bg-transparent border-b border-neutral-200 py-1 text-sm focus:outline-none focus:border-ink" placeholder="Comma separated"
          />
         </div>

         <div>
           <label class="block text-xs font-bold uppercase tracking-widest mb-2 text-neutral-400">Visibility</label>
           <select v-model="form.visibility" class="w-full bg-transparent border-b border-neutral-200 py-1 text-sm focus:outline-none focus:border-ink">
             <option>Public</option>
             <option>Internal</option>
             <option>Private</option>
           </select>
         </div>

         <div class="mt-auto pt-8 border-t border-neutral-200">
           <div class="text-xs text-neutral-400 mb-2">PREVIEW</div>
           <div class="bg-paper p-4 border border-neutral-100 shadow-sm rounded-sm aspect-square overflow-hidden text-[10px] leading-relaxed text-neutral-500 select-none">
              <DynamicRenderer type="Markdown" :data="{ content: form.body }" />
           </div>
         </div>
      </aside>
    </div>
  </div>
</template>
