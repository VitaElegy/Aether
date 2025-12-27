<script setup lang="ts">
import { onMounted, ref } from 'vue';

interface Props {
    language: string;
    code: string;
}

const props = defineProps<Props>();
const copied = ref(false);

const copy = async () => {
    await navigator.clipboard.writeText(props.code);
    copied.value = true;
    setTimeout(() => copied.value = false, 2000);
};
</script>

<template>
    <div class="my-6 border border-ash/50 bg-ash/30 p-4 rounded-sm relative group">
        <div class="absolute top-2 right-2 opacity-0 group-hover:opacity-100 transition-opacity">
            <button @click="copy" class="text-xs uppercase font-bold tracking-widest text-ink/40 hover:text-ink">
                {{ copied ? 'Copied' : 'Copy' }}
            </button>
        </div>
        <div class="text-[10px] uppercase tracking-widest text-ink/40 mb-2 border-b border-ash/50 pb-2 inline-block">
            {{ language }}</div>
        <pre class="font-mono text-sm overflow-x-auto text-ink"><code>{{ code }}</code></pre>
    </div>
</template>
