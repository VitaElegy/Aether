<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps<{
    modelValue: boolean; // isVisible
    form: any;
    timestamps: { created: string | null; updated: string | null };
    knowledgeBases: any[];
}>();

const emit = defineEmits<{
    (e: 'update:modelValue', value: boolean): void;
    (e: 'update:form', value: any): void;
}>();

const formatDate = (isoStr: string | null) => {
  if (!isoStr) return '--';
  return new Date(isoStr).toLocaleString('en-US', { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' });
};

const wordCount = computed(() => {
    return props.form.body ? props.form.body.split(/\s+/).filter(Boolean).length : 0;
});
</script>

<template>
    <Transition
      enter-active-class="transition ease-out duration-200"
      enter-from-class="translate-x-full opacity-0"
      enter-to-class="translate-x-0 opacity-100"
      leave-active-class="transition ease-in duration-150"
      leave-from-class="translate-x-0 opacity-100"
      leave-to-class="translate-x-full opacity-0"
    >
      <aside v-if="modelValue" class="absolute right-0 top-16 bottom-0 w-80 bg-paper border-l border-neutral-100 shadow-xl z-20 flex flex-col overflow-y-auto custom-scrollbar p-6 bg-white/95 backdrop-blur">
         <!-- Header -->
         <div class="flex justify-between items-center mb-6">
             <h3 class="text-xs font-bold uppercase tracking-widest text-neutral-400">Metadata</h3>
             <button @click="emit('update:modelValue', false)" class="text-neutral-400 hover:text-ink">
                 <i class="ri-close-line text-xl"></i>
             </button>
         </div>

         <div class="flex flex-col gap-8">
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

             <!-- Visibility -->
             <div>
               <label class="block text-[10px] font-bold uppercase tracking-widest mb-2 text-neutral-400">Visibility</label>
               <select v-model="form.visibility" class="w-full bg-transparent border-b border-neutral-200 py-2 text-xs font-medium focus:outline-none focus:border-ink cursor-pointer">
                 <option>Public</option>
                 <option>Internal</option>
                 <option>Private</option>
               </select>
             </div>

             <!-- KB -->
             <div>
               <label class="block text-[10px] font-bold uppercase tracking-widest mb-2 text-neutral-400">Knowledge Base</label>
               <select v-model="form.knowledge_base_id" class="w-full bg-transparent border-b border-neutral-200 py-2 text-xs font-medium focus:outline-none focus:border-ink cursor-pointer">
                 <option :value="null">None</option>
                 <option v-for="kb in knowledgeBases" :key="kb.id" :value="kb.id">{{ kb.title }}</option>
               </select>
             </div>

             <!-- Category -->
             <div>
               <label class="block text-[10px] font-bold uppercase tracking-widest mb-2 text-neutral-400">Category</label>
               <input
                 v-model="form.category"
                 class="w-full bg-transparent border-b border-neutral-200 py-2 text-xs font-medium focus:outline-none focus:border-ink placeholder:text-neutral-300"
                 placeholder="Add category"
               />
             </div>

             <!-- Tags -->
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

             <!-- Stats -->
             <div class="mt-auto pt-6 border-t border-neutral-200">
                <div class="text-[10px] text-neutral-400 mb-2 uppercase tracking-widest">Stats</div>
                <div class="grid grid-cols-2 gap-4">
                   <div>
                      <div class="text-xl font-bold text-ink">{{ form.body.length }}</div>
                      <div class="text-[10px] text-neutral-400">Chars</div>
                   </div>
                   <div>
                      <div class="text-xl font-bold text-ink">{{ wordCount }}</div>
                      <div class="text-[10px] text-neutral-400">Words</div>
                   </div>
                </div>
             </div>
         </div>
      </aside>
    </Transition>
</template>
