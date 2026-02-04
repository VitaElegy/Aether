<template>
  <Transition name="fade" appear>
    <div v-if="isOpen" class="fixed inset-0 z-[100] flex items-center justify-center p-4">
        <!-- Backdrop -->
        <div 
            class="absolute inset-0 bg-neutral-900/60 backdrop-blur-sm transition-opacity" 
            @click="close"
        ></div>

        <!-- Writer Card -->
        <div 
            class="relative w-full max-w-xl bg-paper dark:bg-zinc-900 rounded-3xl shadow-2xl overflow-hidden border border-border/50 ring-1 ring-black/5 transform transition-all duration-300"
            :class="{ 'scale-100 opacity-100': isOpen, 'scale-95 opacity-0': !isOpen }"
        >
            <!-- Header (Minimal) -->
            <div class="px-6 py-4 flex items-center justify-between border-b border-border/40">
                <div class="flex items-center gap-2">
                    <div class="w-2 h-2 rounded-full bg-primary animate-pulse"></div>
                    <span class="text-xs font-bold uppercase tracking-widest text-text-tertiary">New Memo</span>
                </div>
                
                <div class="flex gap-2">
                     <button @click="minimize" class="p-1.5 hover:bg-surface-2 rounded-full text-text-tertiary transition-colors">
                        <i class="ri-subtract-line"></i>
                    </button>
                    <button @click="close" class="p-1.5 hover:bg-surface-2 rounded-full text-text-tertiary transition-colors">
                        <i class="ri-close-line text-lg"></i>
                    </button>
                </div>
            </div>

            <!-- Editor -->
            <div class="p-6">
                <!-- Title (Optional, small) -->
                 <input 
                    v-model="localData.title"
                    type="text" 
                    placeholder="Title (Optional)" 
                    class="w-full bg-transparent border-none p-0 text-lg font-bold text-text-primary placeholder:text-text-quaternary focus:ring-0 mb-3"
                    @keydown.enter.prevent="contentArea?.focus()"
                />

                <!-- Main Content -->
                <textarea 
                    ref="contentArea"
                    v-model="localData.content"
                    placeholder="What's on your mind?" 
                    class="w-full min-h-[200px] max-h-[40vh] bg-transparent border-none p-0 resize-none text-base leading-relaxed text-text-secondary placeholder:text-text-quaternary focus:ring-0 font-serif custom-scrollbar"
                    autofocus
                ></textarea>

                <!-- Tags & Quick Actions -->
                <div class="mt-4 flex flex-wrap gap-2">
                     <div 
                        v-for="tag in localData.tags" 
                        :key="tag" 
                        class="bg-surface-2 text-text-secondary px-2 py-0.5 rounded-full text-xs flex items-center gap-1 cursor-pointer hover:bg-red-500/10 hover:text-red-500 transition-colors group"
                        @click="removeTag(tag)"
                    >
                        <span>#{{ tag }}</span>
                        <i class="ri-close-line text-[10px] opacity-0 group-hover:opacity-100"></i>
                    </div>
                    
                    <input 
                        v-model="tagInput" 
                        @keydown.enter.prevent="addTagFromInput" 
                        @keydown.backspace="handleBackspace"
                        type="text"
                        placeholder="#tag" 
                        class="bg-transparent border-none p-0 text-xs text-text-tertiary focus:ring-0 w-20"
                    />
                </div>
            </div>

            <!-- Footer -->
            <div class="px-6 py-4 bg-surface-1 border-t border-border/40 flex items-center justify-between">
                <div class="flex items-center gap-3">
                    <!-- Color Picker (Mini) -->
                    <div class="flex -space-x-1 hover:space-x-1 transition-all duration-300 p-1 rounded-full hover:bg-surface-2">
                         <button 
                            v-for="c in ['Yellow', 'Red', 'Green', 'Blue', 'Purple']" 
                            :key="c"
                            @click="localData.color = c"
                            class="w-4 h-4 rounded-full border border-surface-0 ring-1 ring-border/20 transition-transform hover:scale-125 hover:z-10"
                            :class="colorBg(c)"
                         ></button>
                    </div>
                </div>

                <div class="flex items-center gap-3">
                    <span class="text-[10px] text-text-quaternary uppercase tracking-wider">
                        {{ localData.content?.length || 0 }} chars
                    </span>
                    <button 
                        @click="save"
                        class="px-5 py-2 bg-primary text-on-primary rounded-lg text-sm font-bold shadow-lg shadow-primary/20 hover:shadow-primary/40 hover:-translate-y-0.5 transition-all flex items-center gap-2"
                        :disabled="!localData.content && !localData.title"
                        :class="{ 'opacity-50 cursor-not-allowed': !localData.content && !localData.title }"
                    >
                        <span>Capture</span>
                        <i class="ri-send-plane-fill"></i>
                    </button>
                </div>
            </div>
        </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import type { Memo } from '@/stores/memos';

const props = defineProps<{
  modelValue: boolean; // v-model for visibility
}>();

const emit = defineEmits(['update:modelValue', 'save']);

const isOpen = computed({
    get: () => props.modelValue,
    set: (v) => emit('update:modelValue', v),
});

const localData = ref<Partial<Memo>>({
  title: '',
  content: '',
  status: 'Todo',
  priority: 'P2',
  color: 'Yellow',
  tags: [],
});

const tagInput = ref('');
const contentArea = ref<HTMLTextAreaElement | null>(null);

function close() {
    isOpen.value = false;
}

function minimize() {
    isOpen.value = false;
    // Potentially save draft logic here in future
}

function save() {
    if (!localData.value.content && !localData.value.title) return;
    
    // Default title if empty
    const payload = { ...localData.value };
    if (!payload.title) {
        // Extract first few words? Or just Leave generic
        const content = payload.content || '';
        payload.title = content.slice(0, 20) + (content.length > 20 ? '...' : '') || 'Quick Note';
    }

    // Default tag "Today" or derived date?
    // Let's keep it clean.
    
    emit('save', payload);
    
    // Reset
    localData.value = {
        title: '',
        content: '',
        status: 'Todo',
        priority: 'P2',
        color: 'Yellow',
        tags: [],
    };
    close();
}

function colorBg(c: string) {
    switch(c) {
        case 'Yellow': return 'bg-yellow-400';
        case 'Red': return 'bg-red-400';
        case 'Green': return 'bg-emerald-400';
        case 'Blue': return 'bg-sky-400';
        case 'Purple': return 'bg-purple-400';
        default: return 'bg-zinc-400';
    }
}

// Tag Logic
function addTagFromInput() {
    const raw = tagInput.value.trim().replace(/^#/, '');
    if (raw) {
        if (!localData.value.tags) localData.value.tags = [];
        if (!localData.value.tags.includes(raw)) localData.value.tags.push(raw);
        tagInput.value = '';
    }
}

function removeTag(tag: string) {
    if (localData.value.tags) {
        localData.value.tags = localData.value.tags.filter(t => t !== tag);
    }
}

function handleBackspace() {
    if (!tagInput.value && localData.value.tags?.length) {
        localData.value.tags.pop();
    }
}

// Auto-focus when opened
watch(isOpen, (v) => {
    if (v) {
        setTimeout(() => {
            contentArea.value?.focus();
        }, 100);
    }
});
</script>

<style scoped>
.bounce-enter-active {
  animation: bounce-in 0.5s;
}
.bounce-leave-active {
  animation: bounce-in 0.5s reverse;
}
@keyframes bounce-in {
  0% { transform: scale(0); }
  50% { transform: scale(1.05); }
  100% { transform: scale(1); }
}

.custom-scrollbar::-webkit-scrollbar {
  width: 4px;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background-color: var(--color-border);
  border-radius: 4px;
}
</style>
