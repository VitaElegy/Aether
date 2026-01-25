<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm" @click="$emit('close')">
    <div 
      class="bg-surface-1 w-full max-w-2xl rounded-2xl shadow-2xl border border-border overflow-hidden flex flex-col max-h-[90vh]"
      :class="colorClass"
      @click.stop
    >
      <!-- Header Actions -->
      <div class="p-4 border-b border-border/50 flex items-center justify-between shrink-0">
        <div class="flex items-center gap-2">
           <button 
             class="p-1.5 rounded-full hover:bg-surface-3 text-text-secondary transition-colors"
             :class="{ 'text-primary bg-surface-2': memo?.is_pinned || (!memo && localData.is_pinned) }"
             @click="togglePin"
             title="Pin Memo"
           >
             <div :class="(memo?.is_pinned || (!memo && localData.is_pinned)) ? 'i-ph-push-pin-fill' : 'i-ph-push-pin'" />
           </button>
           
           <div class="flex gap-1 ml-2">
             <button 
                v-for="c in colors" 
                :key="c" 
                class="w-4 h-4 rounded-full border border-black/10 hover:scale-110 transition-transform"
                :class="[
                    bgClass(c), 
                    localData.color === c ? 'ring-2 ring-primary ring-offset-1' : ''
                ]"
                @click="localData.color = c"
             />
           </div>
        </div>

        <div class="flex items-center gap-2">
          <select v-model="localData.status" class="bg-surface-2 border-none text-xs rounded-md py-1 pl-2 pr-6">
             <option value="Todo">Todo</option>
             <option value="Doing">Doing</option>
             <option value="Done">Done</option>
             <option value="Archived">Archived</option>
          </select>

           <select v-model="localData.priority" class="bg-surface-2 border-none text-xs rounded-md py-1 pl-2 pr-6">
             <option value="P0">Urgent (P0)</option>
             <option value="P1">High (P1)</option>
             <option value="P2">Normal (P2)</option>
             <option value="P3">Low (P3)</option>
          </select>

           <button @click="$emit('close')" class="p-1.5 rounded-md hover:bg-surface-3 text-text-secondary">
             <div class="i-ph-x" />
           </button>
        </div>
      </div>

      <!-- Editor Content -->
      <div class="p-6 flex-1 overflow-y-auto custom-scrollbar flex flex-col gap-4">
        <input 
          v-model="localData.title"
          type="text" 
          placeholder="Title (Optional)" 
          class="text-xl font-bold bg-transparent border-none p-0 placeholder:text-text-tertiary focus:ring-0 text-text-primary"
        />
        
        <textarea 
          v-model="localData.content"
          placeholder="Write your thoughts..." 
          class="w-full flex-1 bg-transparent border-none p-0 resize-none focus:ring-0 text-base leading-relaxed font-serif text-text-primary placeholder:text-text-tertiary"
          style="min-height: 200px;" 
        />
      </div>

      <!-- Footer: Metadata & Save -->
      <div class="p-4 border-t border-border/50 bg-surface-2/30 flex items-center justify-between shrink-0">
         <div class="flex items-center gap-2 text-xs text-text-secondary">
            <div class="flex items-center gap-1 cursor-pointer hover:text-text-primary">
               <div class="i-ph-calendar-blank" />
               <input 
                 type="datetime-local" 
                 class="bg-transparent border-none p-0 text-xs w-28 text-text-secondary focus:ring-0" 
                 v-model="dates.due"
               />
            </div>
         </div>

         <div class="flex items-center gap-2">
             <span v-if="localData.updated_at" class="text-xs text-text-tertiary mr-2">
                Last edited {{ new Date(localData.updated_at).toLocaleTimeString() }}
             </span>
             <button 
                @click="save"
                class="px-4 py-2 bg-primary text-white rounded-lg hover:bg-primary-hover shadow-md active:scale-95 transition-all font-medium flex items-center gap-2"
             >
                <span>Save</span>
             </button>
         </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import type { Memo } from '@/stores/memos';

const props = defineProps<{
  memo: Memo | null;
  isNew: boolean;
  initialDate?: Date;
}>();

const emit = defineEmits(['close', 'save']);

const colors = ['Yellow', 'Red', 'Green', 'Blue', 'Purple', 'Gray'];

const localData = ref<Partial<Memo>>({
  title: '',
  content: '',
  status: 'Todo',
  priority: 'P2',
  color: 'Yellow',
  is_pinned: false,
  tags: [],
  due_at: undefined,
});

const dates = ref({
    due: '',
});

if (props.memo) {
    localData.value = JSON.parse(JSON.stringify(props.memo));
    if (props.memo.due_at) dates.value.due = props.memo.due_at.slice(0, 16); 
} else if (props.initialDate) {
    // New memo from calendar click
    const d = new Date(props.initialDate);
    // Set default time to 9:00 AM local
    d.setHours(9, 0, 0, 0);
    // Slice ISO string to 'YYYY-MM-DDTHH:mm' for input type="datetime-local"
    // Note: input type="datetime-local" expects local time format, not ISO Z
    // Simple hack for now: 
    const offset = d.getTimezoneOffset() * 60000;
    const localISOTime = (new Date(d.getTime() - offset)).toISOString().slice(0, 16);
    dates.value.due = localISOTime;
}

function togglePin() {
    localData.value.is_pinned = !localData.value.is_pinned;
}

function bgClass(c: string) {
    switch(c) {
        case 'Red': return 'bg-red-200';
        case 'Green': return 'bg-green-200';
        case 'Blue': return 'bg-blue-200';
        case 'Purple': return 'bg-purple-200';
        case 'Yellow': return 'bg-yellow-200';
        case 'Gray': return 'bg-gray-200';
        default: return 'bg-white border-2 border-gray-100'; 
    }
}

const colorClass = computed(() => {
    switch (localData.value.color) {
        case 'Red': return 'bg-red-50 dark:bg-zinc-900 border-red-200';
        case 'Green': return 'bg-green-50 dark:bg-zinc-900 border-green-200';
        case 'Blue': return 'bg-blue-50 dark:bg-zinc-900 border-blue-200';
        case 'Purple': return 'bg-purple-50 dark:bg-zinc-900 border-purple-200';
        case 'Yellow': return 'bg-yellow-50 dark:bg-zinc-900 border-yellow-200';
        case 'Gray': return 'bg-gray-50 dark:bg-zinc-900 border-gray-200';
        default: return 'bg-surface-1';
    }
});

function save() {
    const payload: any = { ...localData.value };
    if (dates.value.due) payload.due_at = new Date(dates.value.due).toISOString();
    
    if (props.isNew) {
        payload.tags = payload.tags || [];
        payload.visibility = 'Private'; 
    }

    emit('save', payload);
}
</script>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
    width: 6px;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
    background: var(--color-border);
    border-radius: 3px;
}
</style>
