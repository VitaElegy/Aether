<template>
  <Transition name="editorscale" appear>
    <div 
        class="fixed inset-0 z-[100] flex items-center justify-center p-4 sm:p-6" 
    >
        <!-- Backdrop -->
        <div 
            class="absolute inset-0 bg-neutral-900/40 backdrop-blur-md transition-opacity" 
            @click="$emit('close')"
        ></div>

        <!-- Editor Card -->
        <div 
            class="relative w-full max-w-2xl rounded-3xl shadow-2xl flex flex-col max-h-[85vh] overflow-hidden transform transition-all border border-black/5 dark:border-white/10 bg-white dark:bg-zinc-900"
            :class="themeClasses"
            @click.stop
        >
            <!-- Top Color Gradient Wash -->
            <div 
                class="absolute top-0 inset-x-0 h-48 pointer-events-none transition-colors duration-500 ease-in-out opacity-60 dark:opacity-40"
                :class="topGradientClasses"
            ></div>

            <!-- Top Bar (Minimal) -->
            <div class="px-6 pt-5 pb-2 flex items-center justify-between shrink-0 z-10 relative">
                <!-- Color Dots -->
                <div class="flex items-center gap-1.5">
                    <button 
                        v-for="c in colors" 
                        :key="c" 
                        @click="localData.color = c"
                        class="w-5 h-5 rounded-full border border-black/5 dark:border-white/5 transition-transform hover:scale-110 focus:outline-none"
                        :class="[
                            colorBg(c),
                            localData.color === c ? 'ring-2 ring-offset-2 ring-black/20 dark:ring-white/20 scale-110' : 'opacity-70 hover:opacity-100'
                        ]"
                        :title="c"
                    />
                </div>

                <!-- Actions -->
                <div class="flex items-center gap-2">
                    <button 
                        @click="localData.is_pinned = !localData.is_pinned"
                        class="p-2 rounded-full hover:bg-black/5 dark:hover:bg-white/10 transition-colors text-zinc-400 hover:text-zinc-800 dark:hover:text-zinc-200"
                        :class="{ '!text-amber-500': localData.is_pinned }"
                        title="Pin"
                    >
                        <i :class="localData.is_pinned ? 'ri-pushpin-fill' : 'ri-pushpin-line'" class="text-xl leading-none"></i>
                    </button>
                    
                    <button 
                        @click="$emit('close')"
                        class="px-3 py-1.5 rounded-lg hover:bg-black/5 dark:hover:bg-white/10 transition-colors text-zinc-500 hover:text-zinc-800 dark:text-zinc-400 dark:hover:text-zinc-200 text-xs font-bold uppercase tracking-widest"
                        title="Close"
                    >
                        Close
                    </button>
                </div>
            </div>

            <!-- Content Area -->
            <div class="flex-1 overflow-y-auto px-8 pb-4 custom-scrollbar">
                <!-- Title Input -->
                <input 
                    v-model="localData.title"
                    type="text" 
                    placeholder="Untitled" 
                    class="w-full bg-transparent border-none p-0 text-3xl font-bold tracking-tight text-zinc-900 dark:text-zinc-100 placeholder:text-zinc-300 dark:placeholder:text-zinc-700 focus:ring-0 mb-4 font-sans"
                />

                <!-- Body Textarea -->
                <textarea 
                    v-model="localData.content"
                    placeholder="Write your thoughts..." 
                    class="w-full min-h-[300px] bg-transparent border-none p-0 resize-none text-lg leading-relaxed text-zinc-700 dark:text-zinc-300 placeholder:text-zinc-300 dark:placeholder:text-zinc-700 focus:ring-0 font-serif"
                ></textarea>
            </div>

            <!-- Tags Section -->
            <div class="px-8 pb-4 shrink-0">
                <div class="flex flex-wrap gap-2 items-center relative">
                    <div 
                        v-for="tag in localData.tags" 
                        :key="tag" 
                        class="px-2.5 py-1 rounded-full text-xs font-medium flex items-center gap-1 group transition-colors cursor-pointer"
                        :class="getTagColor(tag)"
                        @click="removeTag(tag)"
                    >
                        <i class="ri-hashtag opacity-50"></i>
                        <span>{{ tag }}</span>
                        <i class="ri-close-line opacity-0 group-hover:opacity-100 transition-opacity"></i>
                    </div>

                    <div class="relative flex-1 min-w-[120px]">
                        <input 
                            v-model="tagInput" 
                            @keydown.enter.prevent="addTagFromInput" 
                            @keydown.backspace="handleBackspace"
                            @focus="showSuggestions = true"
                            @blur="handleBlur"
                            type="text"
                            placeholder="#Add tag..." 
                            class="w-full bg-transparent border-none p-0 text-sm text-zinc-600 dark:text-zinc-400 placeholder:text-zinc-300 dark:placeholder:text-zinc-600 focus:ring-0 font-medium"
                        />
                        
                        <!-- Autocomplete Dropdown -->
                        <div 
                            v-if="showSuggestions && suggestedTags.length > 0" 
                            class="absolute bottom-full left-0 mb-2 w-48 bg-white dark:bg-zinc-800 shadow-xl rounded-xl overflow-hidden border border-black/5 dark:border-white/5 z-20 flex flex-col py-1"
                        >
                            <button
                                v-for="tag in suggestedTags"
                                :key="tag.name"
                                @click="addTag(tag.name)"
                                class="px-3 py-2 text-left text-xs font-medium text-zinc-600 dark:text-zinc-300 hover:bg-black/5 dark:hover:bg-white/5 flex items-center justify-between group"
                            >
                                <div class="flex items-center gap-2">
                                     <i class="ri-hashtag opacity-40"></i>
                                     <span>{{ tag.name }}</span>
                                </div>
                                <span class="text-[10px] opacity-40">{{ tag.count }}</span>
                            </button>
                        </div>
                    </div>
                </div>
            </div>

            <!-- Bottom Bar (Context & Save) -->
            <div class="px-6 py-4 border-t border-black/5 dark:border-white/5 bg-gray-50/50 dark:bg-zinc-800/30 backdrop-blur-sm flex items-center justify-between shrink-0 z-20 h-16">
                <div class="flex items-center gap-4 h-full">
                    <!-- Status Pill -->
                    <div class="relative group h-9 flex items-center">
                         <select 
                            v-model="localData.status" 
                            class="appearance-none bg-transparent pl-7 pr-3 h-full text-xs font-bold uppercase tracking-widest text-zinc-500 hover:text-zinc-900 dark:text-zinc-400 dark:hover:text-zinc-200 cursor-pointer rounded-lg hover:bg-black/5 dark:hover:bg-white/5 transition-colors focus:ring-0 border-none flex items-center"
                         >
                            <option value="Todo">Todo</option>
                            <option value="Doing">Doing</option>
                            <option value="Done">Done</option>
                            <option value="Archived">Archived</option>
                         </select>
                         <i class="ri-checkbox-circle-line absolute left-2 text-lg text-zinc-400 pointer-events-none"></i>
                    </div>

                    <!-- Priority -->
                     <div class="relative group h-9 flex items-center">
                         <select 
                            v-model="localData.priority" 
                            class="appearance-none bg-transparent pl-7 pr-3 h-full text-xs font-bold uppercase tracking-widest text-zinc-500 hover:text-zinc-900 dark:text-zinc-400 dark:hover:text-zinc-200 cursor-pointer rounded-lg hover:bg-black/5 dark:hover:bg-white/5 transition-colors focus:ring-0 border-none flex items-center"
                         >
                            <option value="P0">Urgent</option>
                            <option value="P1">High</option>
                            <option value="P2">Normal</option>
                            <option value="P3">Low</option>
                         </select>
                         <i class="ri-flag-line absolute left-2 text-lg text-zinc-400 pointer-events-none"></i>
                    </div>

                    <!-- Date Picker (Hidden native input hack) -->
                    <div class="relative flex items-center group cursor-pointer hover:bg-black/5 dark:hover:bg-white/5 rounded-lg px-2 h-9 transition-colors">
                        <i class="ri-calendar-line text-lg text-zinc-400 group-hover:text-zinc-600 mr-2"></i>
                        <input 
                            type="datetime-local" 
                            v-model="dates.due"
                            class="absolute inset-0 opacity-0 cursor-pointer"
                        />
                        <span 
                            class="text-xs font-bold uppercase tracking-widest text-zinc-500 hover:text-zinc-900 pt-0.5"
                        >
                            {{ dates.due ? formatDisplayDate(dates.due) : 'Set Date' }}
                        </span>
                    </div>
                </div>

                <div class="flex items-center gap-3 h-full">
                    <span v-if="!isNew" class="text-[10px] text-zinc-400 uppercase tracking-widest hidden sm:block">
                        {{ formatTime(localData.updated_at || new Date().toISOString()) }}
                    </span>
                    
                    <button 
                        @click="save"
                        class="px-4 py-2 text-zinc-900 dark:text-zinc-100 rounded-lg hover:bg-black/5 dark:hover:bg-white/10 active:scale-95 transition-all text-sm font-bold tracking-wide flex items-center gap-2 group"
                    >
                        <span>Save Memo</span>
                        <i class="ri-arrow-right-line text-lg transition-transform group-hover:translate-x-1"></i>
                    </button>
                </div>
            </div>
        </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { useMemosStore, type Memo } from '@/stores/memos';
import { format } from 'date-fns';
import { getTagColor } from '@/utils/colors';

const store = useMemosStore();
// ... (rest of imports)


const props = defineProps<{
  memo: Memo | null;
  isNew: boolean;
  initialDate?: Date;
  initialStatus?: string;
}>();

const emit = defineEmits(['close', 'save']);

const colors = ['Yellow', 'Red', 'Green', 'Blue', 'Purple', 'Gray'];

// Local state for the form
const localData = ref<Partial<Memo>>({
  title: '',
  content: '',
  status: props.initialStatus || 'Todo',
  priority: 'P2',
  color: 'Yellow',
  is_pinned: false,
  tags: [],
  due_at: undefined,
});

// Separate ref for date input string
const dates = ref({
    due: '',
});

// Initialize form
// Watch for props change if the component is kept alive or reused? 
// Current MemosModule conditionally renders it `v-if`, so it remounts. 
// Immediate execution is fine.
if (props.memo) {
    localData.value = JSON.parse(JSON.stringify(props.memo));
    if (props.memo.due_at) dates.value.due = props.memo.due_at.slice(0, 16); 
} else {
    // Defaults for new memo
    if (props.initialStatus) localData.value.status = props.initialStatus;
    if (props.initialDate) {
         const d = new Date(props.initialDate);
         d.setHours(9, 0, 0, 0);
         const offset = d.getTimezoneOffset() * 60000;
         dates.value.due = (new Date(d.getTime() - offset)).toISOString().slice(0, 16);
    }
}

// Styling Helpers
// Styling Helpers
// Styling Helpers
function colorBg(c: string) {
    switch(c) {
        // Use gradients for the dots to imply the theme
        case 'Yellow': return 'bg-yellow-400 dark:bg-yellow-500';
        case 'Red': return 'bg-red-400 dark:bg-red-500';
        case 'Green': return 'bg-emerald-400 dark:bg-emerald-500';
        case 'Blue': return 'bg-sky-400 dark:bg-sky-500';
        case 'Purple': return 'bg-purple-400 dark:bg-purple-500';
        case 'Gray': return 'bg-zinc-400 dark:bg-zinc-500';
        default: return 'bg-white';
    }
}

const themeClasses = computed(() => {
    // Top-Only Color Logic: Container is neutral, only borders/shadows might hint color
    switch (localData.value.color) {
        case 'Yellow': return 'border-yellow-200/50 dark:border-yellow-500/10 shadow-2xl shadow-yellow-500/5'; 
        case 'Red': return 'border-red-200/50 dark:border-red-500/10 shadow-2xl shadow-red-500/5';
        case 'Green': return 'border-emerald-200/50 dark:border-green-500/10 shadow-2xl shadow-emerald-500/5';
        case 'Blue': return 'border-sky-200/50 dark:border-sky-500/10 shadow-2xl shadow-sky-500/5';
        case 'Purple': return 'border-purple-200/50 dark:border-purple-500/10 shadow-2xl shadow-purple-500/5';
        case 'Gray': return 'border-black/5 dark:border-white/5 shadow-2xl shadow-black/10';
        default: return 'shadow-2xl';
    }
});

const topGradientClasses = computed(() => {
    // Premium faint gradient wash from top
    switch (localData.value.color) {
        case 'Yellow': return 'bg-gradient-to-b from-yellow-200/40 via-yellow-100/10 to-transparent dark:from-yellow-900/30 dark:to-transparent'; 
        case 'Red': return 'bg-gradient-to-b from-red-200/40 via-red-100/10 to-transparent dark:from-red-900/30 dark:to-transparent';
        case 'Green': return 'bg-gradient-to-b from-emerald-200/40 via-emerald-100/10 to-transparent dark:from-emerald-900/30 dark:to-transparent';
        case 'Blue': return 'bg-gradient-to-b from-sky-200/40 via-sky-100/10 to-transparent dark:from-sky-900/30 dark:to-transparent';
        case 'Purple': return 'bg-gradient-to-b from-purple-200/40 via-purple-100/10 to-transparent dark:from-purple-900/30 dark:to-transparent';
        case 'Gray': return 'bg-gradient-to-b from-gray-200/40 via-gray-100/10 to-transparent dark:from-zinc-800/50 dark:to-transparent';
        default: return 'opacity-0';
    }
});

function formatDisplayDate(dateStr: string) {
    if (!dateStr) return '';
    return format(new Date(dateStr), 'MMM d, HH:mm');
}

function formatTime(iso: string) {
    return format(new Date(iso), 'HH:mm');
}

function save() {
    const payload: any = { ...localData.value };
    if (dates.value.due) payload.due_at = new Date(dates.value.due).toISOString();
    
    // Safety
    if (!payload.title && !payload.content) {
        // Maybe allow empty? Better warn?
        // Let's allow creating untitled empty notes for now (speed).
        if (!payload.title) payload.title = 'Untitled Note';
    }

    if (props.isNew) {
        payload.tags = payload.tags || [];
        payload.visibility = 'Private'; 
    }

    // Ensure tags array exists
    if (!localData.value.tags) localData.value.tags = [];

    emit('save', payload);
}

// --- Tag Logic ---
const tagInput = ref('');
const showSuggestions = ref(false);

const suggestedTags = computed(() => {
    const input = tagInput.value.toLowerCase().replace(/^#/, '');
    const currentTags = localData.value.tags || [];
    
    return store.uniqueTags.filter(t => {
        const matchesInput = t.name.toLowerCase().includes(input);
        const notSelected = !currentTags.includes(t.name);
        return matchesInput && notSelected;
    }).slice(0, 5); // Limit to 5 suggestions
});

function addTag(name: string) {
    if (!localData.value.tags) localData.value.tags = [];
    if (!localData.value.tags.includes(name)) {
        localData.value.tags.push(name);
    }
    tagInput.value = '';
    showSuggestions.value = false;
}

function addTagFromInput() {
    const raw = tagInput.value.trim().replace(/^#/, '');
    if (raw) {
        addTag(raw);
    }
}

function removeTag(name: string) {
    if (localData.value.tags) {
        localData.value.tags = localData.value.tags.filter(t => t !== name);
    }
}

function handleBackspace(e: KeyboardEvent) {
    if (tagInput.value === '' && localData.value.tags && localData.value.tags.length > 0) {
        // Remove last tag
        localData.value.tags.pop();
    }
}

function handleBlur() {
    setTimeout(() => {
        showSuggestions.value = false;
    }, 200);
}
</script>

<style scoped>
/* Scoped transitions */
.editorscale-enter-active,
.editorscale-leave-active {
  transition: opacity 0.2s ease, transform 0.2s cubic-bezier(0.16, 1, 0.3, 1);
}

.editorscale-enter-from,
.editorscale-leave-to {
  opacity: 0;
  transform: scale(0.95) translateY(10px);
}

/* Custom Scrollbar for Textarea/Body */
.custom-scrollbar::-webkit-scrollbar {
  width: 4px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background-color: rgba(0,0,0,0.1);
  border-radius: 4px;
}
</style>
