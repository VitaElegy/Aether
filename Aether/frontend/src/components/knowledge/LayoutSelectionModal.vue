<script setup lang="ts">
import { LAYOUTS, fetchLayouts } from '@/registries/read_layout_registry';
import { type LayoutTemplate } from '@/api/template';
import { computed, onMounted, watch } from 'vue';

const props = defineProps<{
    modelValue: string;
    visible: boolean;
}>();

// Force refresh whenever modal opens
watch(() => props.visible, (val) => {
    if (val) {
        fetchLayouts();
    }
});

const emit = defineEmits(['update:modelValue', 'update:visible', 'confirm']);

const selectLayout = (layout: LayoutTemplate) => {
    emit('update:modelValue', layout.id);
};

const handleConfirm = () => {
    emit('confirm');
    emit('update:visible', false);
};

const getThumbnailClass = (layout: LayoutTemplate) => {
    return layout.thumbnail || 'bg-ash/10';
};

const ICON_MAP: Record<string, string> = {
    default: 'ri-article-line',
    math_v3: 'ri-function-line',
    math_v1: 'ri-compass-3-line',
    english_v1: 'ri-book-read-line',
    vrkb: 'ri-bug-line',
    memo: 'ri-sticky-note-line',
    admin_system: 'ri-settings-line'
};

const getIconForRenderer = (id: string) => ICON_MAP[id] || 'ri-layout-grid-line';
</script>

<template>
    <transition
        enter-active-class="transition duration-200 ease-out"
        enter-from-class="opacity-0"
        enter-to-class="opacity-100"
        leave-active-class="transition duration-150 ease-in"
        leave-from-class="opacity-100"
        leave-to-class="opacity-0"
    >
        <div v-if="visible" class="fixed inset-0 z-[60] flex items-center justify-center bg-black/60 backdrop-blur-sm p-4" @click.self="$emit('update:visible', false)">
            <div class="bg-surface rounded-xl border border-ink/5 shadow-2xl w-full max-w-5xl h-[80vh] flex flex-col overflow-hidden">
                
                <!-- Header -->
                <div class="flex items-center justify-between p-6 border-b border-ink/5 bg-paper">
                    <div>
                        <h2 class="text-2xl font-bold font-serif">Select Layout</h2>
                        <p class="text-ink/50 text-sm mt-1">Choose a renderer optimized for your content type.</p>
                    </div>
                    <button @click="$emit('update:visible', false)" class="text-ink/40 hover:text-ink transition-colors">
                        <i class="ri-close-line text-2xl"></i>
                    </button>
                </div>

                <!-- Body (Scrollable) -->
                <div class="flex-1 overflow-y-auto p-6 bg-ash/5">
                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                        <div v-for="layout in LAYOUTS" :key="layout.id"
                            @click="selectLayout(layout)"
                            class="group relative bg-paper rounded-lg border-2 overflow-hidden cursor-pointer transition-all duration-300 flex flex-col h-full"
                            :class="modelValue === layout.id ? 'border-accent ring-2 ring-accent/20' : 'border-transparent hover:border-ink/10 hover:shadow-lg'">
                            
                            <!-- Checkmark Badge -->
                            <div v-if="modelValue === layout.id" class="absolute top-3 right-3 z-10 bg-accent text-white w-6 h-6 rounded-full flex items-center justify-center text-sm shadow-md">
                                <i class="ri-check-line"></i>
                            </div>

                            <!-- Thumbnail Preview -->
                            <div class="aspect-video w-full rounded-t-sm overflow-hidden relative bg-ash/10">
                                <!-- Image Logic -->
                                <img v-if="layout.thumbnail && (layout.thumbnail.startsWith('http') || layout.thumbnail.startsWith('/'))" 
                                    :src="layout.thumbnail" 
                                    class="w-full h-full object-cover transition-transform duration-700 group-hover:scale-105"
                                    :style="{ objectPosition: `center ${layout.config?.cover_offset_y || 50}%` }"
                                />
                                
                                <!-- Default Identity (CSS Class + Icon) -->
                                <div v-else class="w-full h-full relative" :class="getThumbnailClass(layout)">
                                    <!-- Icon Centered -->
                                    <div class="absolute inset-0 flex items-center justify-center">
                                        <!-- Different styling if it's a dark background? We can assume opacity-20 mixed with text color works for most -->
                                        <i :class="[getIconForRenderer(layout.renderer_id), 'text-6xl opacity-40 mix-blend-overlay']"></i>
                                    </div>
                                </div>
                            </div>

                            <!-- Content -->
                            <div class="p-5 flex-1 flex flex-col">
                                <div class="flex items-start justify-between mb-2">
                                    <h3 class="font-bold text-lg" :class="modelValue === layout.id ? 'text-accent' : 'text-ink'">{{ layout.title }}</h3>
                                </div>
                                
                                <p class="text-sm text-ink/60 mb-4 leading-relaxed flex-1">
                                    {{ layout.description }}
                                </p>

                                <div class="flex flex-wrap gap-2 mt-auto">
                                    <span v-for="tag in layout.tags" :key="tag" 
                                        class="px-2 py-1 bg-ash/10 rounded text-xs font-medium text-ink/50 uppercase tracking-wider">
                                        {{ tag }}
                                    </span>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                <!-- Footer -->
                <div class="p-6 border-t border-ink/5 bg-paper flex justify-end gap-3">
                    <button @click="$emit('update:visible', false)" 
                        class="px-6 py-2.5 rounded-lg text-sm font-bold text-ink/60 hover:bg-ash/10 transition-colors">
                        Cancel
                    </button>
                    <button @click="handleConfirm" 
                        class="px-8 py-2.5 rounded-lg text-sm font-bold text-paper bg-ink hover:bg-accent transition-all shadow-lg shadow-accent/20">
                        Confirm Selection
                    </button>
                </div>
            </div>
        </div>
    </transition>
</template>


