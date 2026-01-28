

<template>
    <div class="fixed bottom-8 left-1/2 -translate-x-1/2 z-50 flex flex-col items-center gap-2">
        
        <!-- Sub-Dock for Groups -->
        <Transition
            enter-active-class="transition duration-200 ease-out"
            enter-from-class="opacity-0 translate-y-4 scale-90"
            enter-to-class="opacity-100 translate-y-0 scale-100"
            leave-active-class="transition duration-150 ease-in"
            leave-from-class="opacity-100 translate-y-0 scale-100"
            leave-to-class="opacity-0 translate-y-4 scale-90"
        >
            <div v-if="activeGroup" 
                 class="bg-paper/90 backdrop-blur-xl border border-ash/50 px-2 py-2 rounded-full shadow-xl flex items-center gap-2 mb-2">
                <button v-for="sub in activeGroup.children" :key="sub.id" @click="selectSubModule(sub)"
                    class="relative w-10 h-10 rounded-full flex items-center justify-center transition-all duration-300 group hover:bg-ash/20"
                    :class="activeModule === sub.id ? 'bg-accent text-white' : 'text-ink/60'"
                    :title="sub.dock.label">
                    <i :class="[sub.dock.icon, 'text-lg']"></i>
                     <span
                        class="absolute -top-8 left-1/2 -translate-x-1/2 bg-ink text-paper text-[10px] uppercase font-bold tracking-widest px-2 py-1 rounded opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none whitespace-nowrap">
                        {{ sub.dock.label }}
                    </span>
                </button>
            </div>
        </Transition>

        <!-- Main Dock -->
        <div
            class="flex items-center gap-2 bg-paper/80 backdrop-blur-xl border border-ash/50 px-2 py-2 rounded-full shadow-2xl transition-all hover:scale-105 duration-300">
            <button v-for="mod in modules" :key="mod.id" 
                @click.stop="handleClick(mod)"
                class="relative w-12 h-12 rounded-full flex items-center justify-center transition-all duration-300 group"
                :class="isModuleActive(mod) ? 'bg-ink text-paper' : 'hover:bg-ash/50 text-ink/50 hover:text-ink'"
                :title="mod.dock.label">
                
                <!-- Group Indicator -->
                <div v-if="mod.children && mod.children.length > 1" 
                     class="absolute top-0 right-0 w-3 h-3 bg-accent text-[8px] font-bold text-white rounded-full flex items-center justify-center z-10 border border-paper pointer-events-none">
                    {{ mod.children.length }}
                </div>

                <!-- Close Button (Hover) -->
                <button v-if="!mod.pinned && !mod.children"
                    @click.stop="emit('close', mod.id)"
                    class="absolute -top-1 -right-1 w-4 h-4 rounded-full bg-red-500 text-white flex items-center justify-center text-[10px] opacity-0 group-hover:opacity-100 transition-opacity z-20 hover:scale-110 shadow-sm"
                    title="Close">
                    <i class="ri-close-line"></i>
                </button>

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

<script setup lang="ts">
import { useRouter } from 'vue-router';
import { ref } from 'vue';

const props = defineProps<{
    activeModule: string;
    modules: any[]; 
}>();

const emit = defineEmits<{
    (e: 'switch', module: string): void;
    (e: 'close', module: string): void;
}>();

const router = useRouter();

// Derived State for Grouping
const activeGroup = ref<any | null>(null);

const handleClick = (mod: any) => {
    if (mod.children && mod.children.length > 0) {
        // Toggle Group
        activeGroup.value = activeGroup.value?.id === mod.id ? null : mod;
    } else {
        // Direct Switch
        activeGroup.value = null;
        emit('switch', mod.id);
    }
};

const selectSubModule = (sub: any) => {
    emit('switch', sub.id);
};

const isModuleActive = (mod: any) => {
    if (props.activeModule === mod.id) return true;
    if (mod.children && mod.children.some((c: any) => c.id === props.activeModule)) return true;
    return false;
};
</script>
