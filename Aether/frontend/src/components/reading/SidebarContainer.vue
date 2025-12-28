<script setup lang="ts">
import { computed, watch } from 'vue';

const props = defineProps<{
    position: 'left' | 'right';
    mode: 'docked' | 'floating';
    isOpen: boolean;
    width?: string; // e.g. 'w-80'
    title?: string;
}>();

const emit = defineEmits(['close']);

const widthClass = computed(() => props.width || 'w-80');

const containerClasses = computed(() => {
    const base = `flex flex-col bg-paper border-ash/50 transition-all duration-300 ease-in-out z-40`;
    const pos = props.position === 'left' ? 'border-r' : 'border-l';

    // Floating mode
    if (props.mode === 'floating') {
        const absolutePos = props.position === 'left' ? 'left-0' : 'right-0';
        // Add shadow for floating
        return `${base} fixed top-0 bottom-0 ${absolutePos} ${widthClass.value} shadow-2xl border-none ${pos}`;
    }

    // Docked mode
    // We use negative margins or width transition to collapse
    return `${base} relative h-full flex-shrink-0 ${widthClass.value} ${pos}`;
});

// Helper to guess pixel width for margin logic if needed,
// strictly generic 'w-80' (20rem = 320px) is standard.
const extractWidth = (w: string) => '20rem';

// Debug log for Sidebar State
watch(() => props.isOpen, (v) => console.log(`[SidebarContainer ${props.position}] Open:`, v));

const visibilityStyles = computed(() => {
    if (props.mode === 'docked') {
        const widthVal = '20rem'; // Explicit width matching w-80
        return {
            marginLeft: props.position === 'left' && !props.isOpen ? `-${widthVal}` : '0',
            marginRight: props.position === 'right' && !props.isOpen ? `-${widthVal}` : '0',
            opacity: props.isOpen ? 1 : 0.5,
            // Transition width for smooth collapse
            width: props.isOpen ? widthVal : '0px',
            overflow: 'hidden'
        };
    }

    // Floating
    return {
        transform: props.isOpen
            ? 'translateX(0)'
            : `translateX(${props.position === 'left' ? '-100%' : '100%'})`
    };
});

</script>

<template>
    <aside :class="containerClasses" :style="visibilityStyles">
        <!-- Header -->
        <div class="flex items-center justify-between px-6 py-4 border-b border-ash/50 h-16 flex-shrink-0 gap-4">
            <span class="text-[10px] font-black uppercase tracking-[0.2em] text-ink/40 truncate select-none"
                :title="title">{{ title }}</span>
            <button @click="emit('close')" class="text-ink/40 hover:text-accent transition-colors flex-shrink-0">
                <i class="ri-close-line text-lg"></i>
            </button>
        </div>

        <!-- Content -->
        <div class="flex-1 overflow-y-auto custom-scrollbar">
            <slot></slot>
        </div>
    </aside>
</template>
