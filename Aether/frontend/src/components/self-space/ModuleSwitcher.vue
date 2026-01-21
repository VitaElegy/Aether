<script setup lang="ts">
import { useRouter } from 'vue-router';

import type { SelfSpacePlugin } from '../../core/plugin';

defineProps<{
    activeModule: string;
    modules: SelfSpacePlugin[];
}>();

const emit = defineEmits<{
    (e: 'switch', module: string): void;
}>();

const router = useRouter();

</script>

<template>
    <div class="fixed bottom-8 left-1/2 -translate-x-1/2 z-50">
        <div
            class="flex items-center gap-2 bg-paper/80 backdrop-blur-xl border border-ash/50 px-2 py-2 rounded-full shadow-2xl transition-all hover:scale-105 duration-300">
            <button v-for="mod in modules" :key="mod.id" @click="emit('switch', mod.id)"
                class="relative w-12 h-12 rounded-full flex items-center justify-center transition-all duration-300 group"
                :class="activeModule === mod.id ? 'bg-ink text-paper' : 'hover:bg-ash/50 text-ink/50 hover:text-ink'"
                :title="mod.dock.label">
                <i :class="[mod.dock.icon, 'text-xl']"></i>

                <!-- Tooltip -->
                <span
                    class="absolute -top-10 left-1/2 -translate-x-1/2 bg-ink text-paper text-[10px] uppercase font-bold tracking-widest px-2 py-1 rounded opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none whitespace-nowrap">
                    {{ mod.dock.label }}
                </span>
            </button>

            <!-- Separator -->
            <div class="w-px h-6 bg-ash/50 mx-2"></div>

            <!-- Exit Button -->
            <button @click="router.push('/')"
                class="w-10 h-10 rounded-full flex items-center justify-center text-red-400 hover:bg-red-50 hover:text-red-500 transition-colors"
                title="Exit Space">
                <i class="ri-logout-box-r-line"></i>
            </button>
        </div>
    </div>
</template>
