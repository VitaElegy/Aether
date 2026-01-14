<script setup lang="ts">
import { ref } from 'vue';
import TopNavBar from '@/components/TopNavBar.vue';

interface Props {
    title: string;
    loading: boolean;
    canEdit: boolean;
}

const props = defineProps<Props>();
const emit = defineEmits(['create-article', 'open-settings']);
</script>

<template>
    <div class="h-screen w-full flex flex-col bg-paper transition-colors duration-500 overflow-hidden">
        <!-- Dashboard Header -->
        <TopNavBar>
            <template #center>
                <div class="flex items-center gap-2 opacity-50 hover:opacity-100 transition-opacity">
                    <span class="text-xs font-black uppercase tracking-[0.2em]">{{ title || 'Knowledge Base' }}</span>
                </div>
            </template>
            
            <template #right>
                <div class="flex items-center gap-4">
                    <button v-if="canEdit" @click="$emit('create-article')"
                        class="text-xs font-black uppercase tracking-widest text-accent hover:brightness-125 transition-all flex items-center gap-2">
                        <i class="ri-add-line text-lg"></i>
                        <span>New Entry</span>
                    </button>
                    
                    <button v-if="canEdit" @click="$emit('open-settings')"
                        class="text-ink/20 hover:text-accent transition-colors">
                        <i class="ri-settings-4-line text-xl"></i>
                    </button>
                </div>
            </template>
        </TopNavBar>

        <!-- Dynamic Visualization Header (Slot) -->
        <div class="flex-shrink-0 relative z-10">
            <slot name="header"></slot>
        </div>

        <!-- Main Content Area (Slot) -->
        <main class="flex-1 overflow-hidden relative z-0">
             <slot name="content"></slot>
        </main>
        
        <!-- Settings Modal (Slot) -->
        <slot name="settings"></slot>
    </div>
</template>
