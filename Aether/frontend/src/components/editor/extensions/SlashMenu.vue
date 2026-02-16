<script setup lang="ts">
import { ref, watch, onMounted } from 'vue';
import { Icon } from 'tdesign-vue-next';

const props = defineProps<{
    items: any[];
    command: (item: any) => void;
    editor: any;
}>();

const selectedIndex = ref(0);

watch(() => props.items, () => {
    selectedIndex.value = 0;
});

const onKeyDown = ({ event }: { event: KeyboardEvent }) => {
    if (event.key === 'ArrowUp') {
        upHandler();
        return true;
    }
    if (event.key === 'ArrowDown') {
        downHandler();
        return true;
    }
    if (event.key === 'Enter') {
        enterHandler();
        return true;
    }
    return false;
};

const upHandler = () => {
    selectedIndex.value = ((selectedIndex.value + props.items.length) - 1) % props.items.length;
};

const downHandler = () => {
    selectedIndex.value = (selectedIndex.value + 1) % props.items.length;
};

const enterHandler = () => {
    selectItem(selectedIndex.value);
};

const selectItem = (index: number) => {
    const item = props.items[index];
    if (item) {
        props.command(item);
    }
};

defineExpose({
    onKeyDown,
});
</script>

<template>
    <div class="bg-white dark:bg-zinc-800 rounded-lg shadow-xl border border-gray-200 dark:border-zinc-700 overflow-hidden min-w-[200px] py-1">
        <div class="px-3 py-2 text-xs font-semibold text-gray-400 uppercase tracking-wider border-b border-gray-100 dark:border-zinc-700/50 mb-1">
            Basic Blocks
        </div>
        <button
            v-for="(item, index) in items"
            :key="index"
            class="w-full text-left px-3 py-2 text-sm flex items-center gap-3 transition-colors"
            :class="{ 'bg-blue-50 dark:bg-zinc-700 text-blue-600 dark:text-blue-400': index === selectedIndex, 'text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-zinc-700/50': index !== selectedIndex }"
            @click="selectItem(index)"
        >
            <div class="w-6 h-6 rounded flex items-center justify-center bg-gray-100 dark:bg-zinc-700 border border-gray-200 dark:border-zinc-600 shrink-0">
                 <!-- Icon Placeholder if item.icon is string, assume tdesign icon name or similar -->
                 <!-- If item.icon is component, usage depends on implementation. -->
                 <!-- Assuming primitive icon name for now -->
                 <i :class="item.iconClass || 'ri-text'" class="text-sm"></i>
            </div>
            <div class="flex flex-col">
                <span class="font-medium">{{ item.title }}</span>
                <span class="text-[10px] text-gray-400 truncate max-w-[150px]">{{ item.description }}</span>
            </div>
        </button>
    </div>
</template>
