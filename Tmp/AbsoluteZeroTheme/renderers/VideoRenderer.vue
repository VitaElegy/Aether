<script setup lang="ts">
import { computed } from 'vue';

interface Props {
  url: string;
  title?: string;
}

const props = withDefaults(defineProps<Props>(), {
  title: '',
});

const videoSrc = computed(() => {
  if (props.url.includes('youtube.com') || props.url.includes('youtu.be')) {
    const videoId = props.url.split('v=')[1] || props.url.split('/').pop();
    return `https://www.youtube.com/embed/${videoId}`;
  }
  return props.url;
});

const isYouTube = computed(() => props.url.includes('youtube') || props.url.includes('youtu.be'));
</script>

<template>
  <div class="my-8">
    <div class="aspect-video bg-ash w-full overflow-hidden">
      <iframe v-if="isYouTube" :src="videoSrc" class="w-full h-full" frameborder="0" allowfullscreen></iframe>
      <video v-else :src="videoSrc" controls class="w-full h-full object-cover"></video>
    </div>
    <div v-if="title" class="mt-2 text-xs font-mono text-neutral-400 text-center uppercase tracking-widest">{{ title }}</div>
  </div>
</template>
