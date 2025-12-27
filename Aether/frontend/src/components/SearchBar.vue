<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from 'vue';
import { useRouter } from 'vue-router';
import axios from 'axios';

const router = useRouter();
const query = ref('');
const results = ref<any[]>([]);
const showPreview = ref(false);
let debounceTimeout: ReturnType<typeof setTimeout>;

const searchContainer = ref<HTMLElement | null>(null);

const performSearch = async () => {
    if (!query.value.trim()) {
        results.value = [];
        return;
    }

    try {
        const res = await axios.get(`/api/search?q=${encodeURIComponent(query.value)}`);
        // Dedupe and take top 5 for preview
        const uniqueData = Array.from(new Map(res.data.map((item: any) => [item.id, item])).values());
        results.value = uniqueData.slice(0, 5).map((p: any) => ({
            id: p.id,
            title: p.title,
            type: p.body.type
        }));
        showPreview.value = true;
    } catch (err) {
        console.error(err);
    }
};

watch(query, () => {
    clearTimeout(debounceTimeout);
    if (!query.value.trim()) {
        showPreview.value = false;
        return;
    }
    debounceTimeout = setTimeout(performSearch, 300);
});

const onEnter = () => {
    showPreview.value = false;
    router.push(`/search?q=${encodeURIComponent(query.value)}`);
};

const navigateTo = (id: string) => {
    showPreview.value = false;
    router.push(`/article/${id}`);
};

// Click outside to close
const handleClickOutside = (event: MouseEvent) => {
    if (searchContainer.value && !searchContainer.value.contains(event.target as Node)) {
        showPreview.value = false;
    }
};

onMounted(() => {
    document.addEventListener('click', handleClickOutside);
});

onUnmounted(() => {
    document.removeEventListener('click', handleClickOutside);
});
</script>

<template>
    <div ref="searchContainer" class="relative group hidden md:block">
        <div class="relative">
            <input v-model="query" @keyup.enter="onEnter" type="text" placeholder="SEARCH..."
                class="bg-transparent border-b border-transparent focus:border-ink outline-none text-xs font-mono uppercase tracking-widest w-24 focus:w-48 transition-all placeholder:text-ink/30 text-ink pb-1" />
            <i
                class="ri-search-line absolute right-0 top-0 text-ink/30 text-xs pointer-events-none group-focus-within:opacity-0 transition-opacity"></i>
        </div>

        <!-- Preview Dropdown -->
        <div v-if="showPreview && results.length > 0"
            class="absolute top-full left-0 w-64 bg-paper border border-ash shadow-xl rounded-sm mt-2 p-2 z-50">
            <div class="text-[10px] text-ink/40 font-mono uppercase tracking-widest mb-2 px-2">Top Matches</div>
            <div v-for="res in results" :key="res.id" @click="navigateTo(res.id)"
                class="px-2 py-2 hover:bg-ash/50 cursor-pointer transition-colors group/item">
                <div class="text-sm font-bold text-ink group-hover/item:text-ink/70 truncate">{{ res.title }}</div>
                <div class="text-[10px] text-ink/40 uppercase tracking-widest">{{ res.type }}</div>
            </div>
            <div @click="onEnter"
                class="border-t border-ash mt-2 pt-2 px-2 text-[10px] font-bold uppercase tracking-widest text-ink/50 hover:text-ink cursor-pointer text-center">
                View All Results
            </div>
        </div>
    </div>
</template>
