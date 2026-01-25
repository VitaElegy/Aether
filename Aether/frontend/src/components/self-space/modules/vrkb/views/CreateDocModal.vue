<template>
    <div v-if="isOpen" class="fixed inset-0 z-50 flex items-center justify-center p-4">
        <!-- Backdrop -->
        <div class="absolute inset-0 bg-ink/20 backdrop-blur-sm" @click="close"></div>

        <!-- Modal -->
        <div class="relative w-full max-w-sm bg-white rounded-xl shadow-2xl border border-ash/20 overflow-hidden flex flex-col animate-scale-in">
            <div class="p-6">
                <h3 class="font-bold font-serif text-ink text-lg mb-6">Create New</h3>
                
                <!-- Type Selection -->
                <div class="flex bg-ash/20 p-1 rounded-lg mb-6">
                    <button 
                        @click="type = 'file'"
                        class="flex-1 py-1.5 text-xs font-bold uppercase tracking-wider rounded-md transition-all flex items-center justify-center gap-2"
                        :class="type === 'file' ? 'bg-white text-accent shadow-sm' : 'text-ink/40 hover:text-ink/60'"
                    >
                        <i class="ri-file-text-line text-lg"></i> Page
                    </button>
                    <button 
                        @click="type = 'folder'"
                        class="flex-1 py-1.5 text-xs font-bold uppercase tracking-wider rounded-md transition-all flex items-center justify-center gap-2"
                        :class="type === 'folder' ? 'bg-white text-yellow-500 shadow-sm' : 'text-ink/40 hover:text-ink/60'"
                    >
                        <i class="ri-folder-3-fill text-lg"></i> Folder
                    </button>
                </div>

                <!-- Name Input -->
                <div class="space-y-2 mb-6">
                    <label class="text-xs font-bold uppercase tracking-wider text-ink/40">Name</label>
                    <input 
                        ref="nameInput"
                        v-model="name" 
                        type="text" 
                        class="w-full px-4 py-2 bg-ash/30 rounded-lg text-sm font-medium text-ink focus:outline-none focus:ring-2 focus:ring-accent/20 transition-all placeholder-ink/30"
                        :placeholder="type === 'file' ? 'e.g. System Architecture' : 'e.g. Archive'"
                        @keydown.enter="create"
                    >
                </div>

                <!-- Actions -->
                <div class="flex items-center justify-end gap-3">
                    <button @click="close" class="px-4 py-2 text-xs font-bold text-ink/60 hover:bg-ash/50 rounded-lg transition-colors">Cancel</button>
                    <button 
                        @click="create" 
                        class="px-6 py-2 bg-accent text-white rounded-lg text-xs font-bold shadow-lg shadow-accent/20 hover:bg-accent/90 transition-all flex items-center gap-2"
                        :disabled="!name.trim()"
                    >
                        Create {{ type === 'file' ? 'Page' : 'Folder' }}
                    </button>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, watch, nextTick } from 'vue';

const props = defineProps<{
    isOpen: boolean
}>();

const emit = defineEmits(['close', 'create']);

const type = ref<'file' | 'folder'>('file');
const name = ref('');
const nameInput = ref<HTMLInputElement | null>(null);

const close = () => {
    emit('close');
};

const create = () => {
    if (!name.value.trim()) return;
    emit('create', { name: name.value, type: type.value });
    name.value = '';
    type.value = 'file'; // Reset
};

watch(() => props.isOpen, async (newVal) => {
    if (newVal) {
        await nextTick();
        nameInput.value?.focus();
    }
});
</script>
