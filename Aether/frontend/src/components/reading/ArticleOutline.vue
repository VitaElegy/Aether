<script setup lang="ts">
defineProps<{
    toc: { id: string; text: string; level: number; }[]
}>();

const scrollToHeading = (id: string) => {
    const element = document.getElementById(id);
    if (element) {
        element.scrollIntoView({ behavior: 'smooth' });
    }
};
</script>

<template>
    <div class="px-8 py-10">
        <nav v-if="toc.length > 0" class="flex flex-col gap-4">
            <a v-for="(item, idx) in toc" :key="idx" href="#"
                class="text-xs font-bold text-ink/40 hover:text-accent transition-all leading-tight border-l-2 border-transparent hover:border-accent pl-4 py-1 block"
                :class="{ 'ml-4': item.level === 2, 'ml-8': item.level === 3 }"
                @click.prevent="scrollToHeading(item.id)">
                {{ item.text }}
            </a>
        </nav>
        <div v-else class="text-xs text-ink/20 italic font-mono px-4">
            No structure detected.
        </div>
    </div>
</template>
