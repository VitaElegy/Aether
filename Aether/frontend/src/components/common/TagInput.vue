<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue';
import { tagsApi } from '../../api/tags';

const props = defineProps<{
    modelValue: string; // Comma separated string "tech, rust"
    placeholder?: string;
}>();

const emit = defineEmits(['update:modelValue']);

const availableTags = ref<string[]>([]);
const inputValue = ref('');
const showSuggestions = ref(false);
const inputRef = ref<HTMLInputElement | null>(null);

// Parse initial value into array
const currentTags = computed(() => {
    return props.modelValue.split(',').map(t => t.trim()).filter(t => t);
});

onMounted(async () => {
    try {
        availableTags.value = await tagsApi.list();
    } catch (e) {
        console.error("Failed to load tags", e);
    }
});

const filteredSuggestions = computed(() => {
    const input = inputValue.value.toLowerCase();
    // Filter tags that contain input AND are not already selected
    return availableTags.value.filter(t =>
        (!input || t.toLowerCase().includes(input)) &&
        !currentTags.value.includes(t)
    ).slice(0, 10); // Limit to 10
});

const addTag = (tag: string) => {
    const newTags = [...currentTags.value, tag];
    emit('update:modelValue', newTags.join(', '));
    inputValue.value = '';
    showSuggestions.value = false;
    inputRef.value?.focus();
};

const removeTag = (tagToRemove: string) => {
    const newTags = currentTags.value.filter(t => t !== tagToRemove);
    emit('update:modelValue', newTags.join(', '));
};

const handleInputStart = () => {
    showSuggestions.value = true;
};

const handleInputBlur = () => {
    // Delay hiding to allow click event on suggestion
    setTimeout(() => {
        showSuggestions.value = false;
    }, 200);
};

const handleEnter = () => {
    if (inputValue.value) {
        // If exact match in suggestions, prioritize that (case sensitive check usually good, or just take input)
        // Just take input as new tag
        addTag(inputValue.value.trim());
    }
};

const handleBackspace = () => {
    if (!inputValue.value && currentTags.value.length > 0) {
        removeTag(currentTags.value[currentTags.value.length - 1]);
    }
};

</script>

<template>
    <div class="relative w-full">
        <div
            class="flex flex-wrap items-center gap-2 bg-paper border border-ink/10 rounded px-2 py-1.5 focus-within:border-accent transition-colors min-h-[42px]">
            <!-- Tags -->
            <span v-for="tag in currentTags" :key="tag"
                class="bg-surface text-ink text-xs px-2 py-1 rounded-full flex items-center gap-1 border border-ink/5">
                {{ tag }}
                <button @click.stop="removeTag(tag)" class="hover:text-red-500 text-ink/40">
                    <i class="ri-close-line"></i>
                </button>
            </span>

            <!-- Input -->
            <input ref="inputRef" v-model="inputValue" @focus="handleInputStart" @blur="handleInputBlur"
                @keydown.enter.prevent="handleEnter" @keydown.backspace="handleBackspace"
                @keydown.tab.prevent="filteredSuggestions.length > 0 ? addTag(filteredSuggestions[0]) : null"
                :placeholder="currentTags.length === 0 ? (placeholder || 'Add tags...') : ''"
                class="flex-1 bg-transparent border-none outline-none text-sm min-w-[80px]" />
        </div>

        <!-- Suggestions Dropdown -->
        <div v-if="showSuggestions && filteredSuggestions.length > 0"
            class="absolute z-50 left-0 right-0 mt-1 bg-paper border border-ink/10 rounded-lg shadow-xl max-h-48 overflow-y-auto tags-dropdown">
            <div v-for="tag in filteredSuggestions" :key="tag" @mousedown.prevent="addTag(tag)"
                class="px-3 py-2 text-sm hover:bg-surface cursor-pointer flex items-center gap-2">
                <i class="ri-hashtag text-ink/40 text-xs"></i> {{ tag }}
            </div>
        </div>
    </div>
</template>

<style scoped>
.tags-dropdown::-webkit-scrollbar {
    width: 4px;
}

.tags-dropdown::-webkit-scrollbar-thumb {
    background: rgba(0, 0, 0, 0.1);
    border-radius: 4px;
}
</style>
