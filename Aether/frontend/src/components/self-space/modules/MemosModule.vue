<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { knowledgeApi } from '@/api/knowledge';

const props = defineProps<{
    kbId?: string;
    headless?: boolean;
}>();

const currentKb = ref<any>(null);
const loading = ref(false);

const loadMemoSpace = async (id: string) => {
    loading.value = true;
    try {
        currentKb.value = await knowledgeApi.get(id);
    } catch (e) {
        console.error(e);
    } finally {
        loading.value = false;
    }
};

onMounted(() => {
    if (props.kbId) loadMemoSpace(props.kbId);
});

watch(() => props.kbId, (newVal) => {
    if (newVal) {
        loadMemoSpace(newVal);
    } else {
        currentKb.value = null;
    }
});
</script>

<template>
    <div class="h-full flex flex-col relative bg-yellow-50/50 text-ink">
        <div v-if="loading" class="absolute inset-0 flex items-center justify-center">
            <span class="animate-pulse">Loading Memo Space...</span>
        </div>
        
        <div v-else-if="currentKb" class="p-8">
            <div class="flex items-center gap-4 mb-8">
                <div class="w-12 h-12 rounded-full bg-yellow-400 flex items-center justify-center text-white shadow-lg shadow-yellow-200">
                    <i class="ri-sticky-note-fill text-2xl"></i>
                </div>
                <div>
                    <h2 class="text-2xl font-bold tracking-tight">{{ currentKb.title }}</h2>
                    <p class="text-xs font-mono uppercase tracking-widest opacity-40">Quick Memos</p>
                </div>
            </div>

            <!-- Placeholder for Memo Grid -->
            <div class="grid grid-cols-2 md:grid-cols-3 gap-4 opacity-50">
                 <div class="aspect-square bg-yellow-200/50 rounded-xl border border-yellow-300 border-dashed flex items-center justify-center">
                    <span class="text-sm font-bold text-yellow-600">New Memo</span>
                 </div>
                 <!-- Simulation of content -->
                 <div class="aspect-square bg-white rounded-xl shadow-sm p-4 rotate-1 border border-ink/5">
                    <p class="font-handwriting text-lg">Don't forget to buy milk!</p>
                 </div>
                 <div class="aspect-square bg-white rounded-xl shadow-sm p-4 -rotate-2 border border-ink/5">
                    <p class="font-handwriting text-lg">Meeting at 3PM</p>
                 </div>
            </div>
        </div>

        <div v-else class="h-full flex flex-col items-center justify-center text-center animate-in fade-in zoom-in-95 duration-500">
            <div class="w-24 h-24 rounded-full bg-ash/20 flex items-center justify-center mb-6 text-neutral-300">
                <i class="ri-sticky-note-line text-4xl"></i>
            </div>
            <h2 class="text-2xl font-bold tracking-tight mb-2">Quick Memos</h2>
            <p class="text-neutral-400 font-mono text-xs uppercase tracking-widest max-w-md mx-auto leading-relaxed">
                select a space to view memos
            </p>
        </div>
    </div>
</template>
