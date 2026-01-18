<template>
  <div class="meta-drawer-container">
    <transition name="slide-fade">
      <div v-if="visible" class="meta-drawer">
        <!-- Header -->
        <div class="drawer-header">
            <h2 class="serif-font text-2xl font-bold text-ink">Analysis Details</h2>
            <button class="close-btn" @click="$emit('close')">
                <i class="ri-close-line text-xl"></i>
            </button>
        </div>

        <div class="divider"></div>

        <div class="content custom-scrollbar">
            <!-- Publishing Status -->
             <div class="section">
                <label class="section-label">Status</label>
                <div class="flex items-center justify-between bg-ash/30 p-3 rounded-md">
                    <span class="text-sm font-medium" :class="statusClass">{{ form.status }}</span>
                    <button 
                        v-if="form.status === 'Draft'"
                        @click="$emit('publish')"
                        class="px-4 py-1.5 bg-ink text-paper text-xs font-bold uppercase tracking-widest hover:bg-neutral-800 transition-colors"
                    >
                        Mark Done
                    </button>
                    <button 
                        v-else
                        @click="$emit('unpublish')"
                        class="px-4 py-1.5 border border-ink text-ink text-xs font-bold uppercase tracking-widest hover:bg-ash transition-colors"
                    >
                        Reopen
                    </button>
                </div>
            </div>

            <!-- Background Image -->
            <div class="section">
                <label class="section-label">Cover Image</label>
                <div class="flex gap-2">
                    <input
                        v-model="form.background"
                        class="flex-1 bg-transparent border-b border-neutral-200 py-2 text-sm font-serif focus:outline-none focus:border-ink placeholder:text-neutral-300"
                        placeholder="Image URL..."
                    />
                     <div v-if="form.background" class="w-10 h-10 rounded border border-neutral-200 overflow-hidden flex-shrink-0">
                        <img :src="form.background" class="w-full h-full object-cover" />
                    </div>
                </div>
            </div>

            <!-- References -->
            <div class="section">
                <label class="section-label">References</label>
                <div class="flex flex-col gap-2">
                    <div v-for="(ref, idx) in form.references" :key="idx" class="flex items-center gap-2 bg-ash/20 p-2 rounded">
                        <div class="flex-1 min-w-0">
                            <div class="text-xs font-bold truncate">{{ ref.title }}</div>
                            <div class="text-[10px] text-neutral-400 truncate">{{ ref.url }}</div>
                        </div>
                        <button @click="removeReference(idx)" class="text-neutral-400 hover:text-red-500">
                            <i class="ri-delete-bin-line"></i>
                        </button>
                    </div>
                    
                    <!-- Add New -->
                    <div class="flex flex-col gap-2 mt-2 p-2 border border-dashed border-neutral-200 rounded">
                        <input v-model="newRef.title" placeholder="Title" class="bg-transparent text-xs border-b border-neutral-100 focus:border-ink outline-none py-1"/>
                        <input v-model="newRef.url" placeholder="URL" class="bg-transparent text-xs border-b border-neutral-100 focus:border-ink outline-none py-1"/>
                        <button 
                            @click="addReference" 
                            :disabled="!newRef.title || !newRef.url"
                            class="text-[10px] uppercase font-bold text-ink/60 hover:text-ink disabled:opacity-30 self-end mt-1"
                        >
                            + Add Reference
                        </button>
                    </div>
                </div>
            </div>
            
             <!-- Stats -->
            <div class="mt-auto pt-8 border-t border-neutral-200">
                <label class="section-label mb-4">Statistics</label>
                <div class="grid grid-cols-2 gap-4">
                    <div class="bg-ash/20 p-3 rounded">
                        <div class="text-xl font-bold text-ink serif-font">{{ stats.chars }}</div>
                        <div class="text-[10px] text-neutral-400 uppercase tracking-widest">Characters</div>
                    </div>
                    <div class="bg-ash/20 p-3 rounded">
                        <div class="text-xl font-bold text-ink serif-font">{{ stats.words }}</div>
                        <div class="text-[10px] text-neutral-400 uppercase tracking-widest">Words</div>
                    </div>
                </div>
            </div>
        </div>
      </div>
    </transition>
    <div v-if="visible" class="backdrop" @click="$emit('close')"></div>
  </div>
</template>

<script setup lang="ts">
import { computed, reactive } from 'vue';

const props = defineProps<{
    visible: boolean;
    form: {
        status: string;
        background: string;
        references: Array<{ title: string; url: string }>;
    };
    stats: {
        chars: number;
        words: number;
    };
}>();

const emit = defineEmits(['close', 'publish', 'unpublish']);

const statusClass = computed(() => {
    switch(props.form.status) {
        case 'Published': return 'text-green-600';
        case 'Draft': return 'text-neutral-500';
        default: return 'text-ink';
    }
});

const newRef = reactive({ title: '', url: '' });

const addReference = () => {
    if (newRef.title && newRef.url) {
        props.form.references.push({ ...newRef });
        newRef.title = '';
        newRef.url = '';
    }
};

const removeReference = (idx: number) => {
    props.form.references.splice(idx, 1);
};
</script>

<style scoped>
.meta-drawer-container {
    position: fixed;
    top: 0;
    right: 0;
    height: 100vh;
    z-index: 2000;
}

.backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0,0,0,0.2);
    backdrop-filter: blur(2px);
    z-index: 1999;
}

.meta-drawer {
    position: relative;
    z-index: 2001;
    width: 360px;
    height: 100%;
    background: #F9F7F1; /* Warm Ivory (Paper) */
    box-shadow: -5px 0 30px rgba(0,0,0,0.08);;
    display: flex;
    flex-direction: column;
    padding: 2rem;
    border-left: 1px solid rgba(0,0,0,0.05);
}

/* Animations */
.slide-fade-enter-active,
.slide-fade-leave-active {
  transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}

.slide-fade-enter-from,
.slide-fade-leave-to {
  transform: translateX(100%);
  opacity: 0;
}

/* Typography */
.serif-font {
    font-family: "Playfair Display", "Times New Roman", serif;
}

.drawer-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1.5rem;
}

.close-btn {
    background: none;
    border: none;
    cursor: pointer;
    color: #999;
    transition: color 0.2s;
}
.close-btn:hover {
    color: #333;
}

.divider {
    height: 1px;
    background: rgba(0,0,0,0.08);
    margin-bottom: 2rem;
}

.content {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 2rem;
}

.section {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
}

.section-label {
    font-size: 0.65rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.15em;
    color: #999;
}

/* Scrollbar */
.custom-scrollbar::-webkit-scrollbar { width: 4px; }
.custom-scrollbar::-webkit-scrollbar-thumb { background: #dcdcdc; border-radius: 2px; }
.custom-scrollbar::-webkit-scrollbar-track { background: transparent; }
</style>
