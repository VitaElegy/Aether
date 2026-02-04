<template>
  <t-card :header-bordered="true" shadow class="paper-card group cursor-pointer transition-all duration-200 hover:-translate-y-1">
      <template #header>
          <div class="flex justify-between items-start w-full gap-4">
             <div>
                 <a :href="paper.pdf_url || paper.url" target="_blank" class="text-blue-600 font-bold hover:underline text-lg leading-snug block" @click.stop>
                     {{ paper.title }}
                 </a>
                 <div class="text-sm text-gray-500 mt-2 flex items-center gap-2 flex-wrap">
                     <span class="font-medium text-gray-700">{{ (paper.authors || []).slice(0, 3).join(', ') }}{{ (paper.authors || []).length > 3 ? ' et al.' : '' }}</span>
                     <span class="text-gray-300">•</span>
                     <span>{{ displayYear }}</span>
                     <span v-if="paper.venue" class="px-2 py-0.5 bg-gray-100 text-gray-600 rounded text-xs font-medium">{{ paper.venue }}</span>
                 </div>
             </div>
             <div class="flex flex-col items-end gap-1 shrink-0">
                  <t-tag v-if="paper.is_saved" theme="success" variant="light" size="small">Saved</t-tag>
                  <div v-if="paper.github_stars" class="flex items-center text-xs text-gray-500 bg-gray-50 px-2 py-0.5 rounded border border-gray-100">
                      <i class="ri-star-fill text-yellow-400 mr-1"></i> {{ paper.github_stars }}
                  </div>
             </div>
          </div>
      </template>
      
      <div class="text-gray-600 text-sm leading-relaxed line-clamp-4 h-20">
          {{ displayAbstract }}
      </div>

      <template #actions>
          <div class="flex justify-end gap-2 p-2 opacity-100 md:opacity-0 group-hover:opacity-100 transition-opacity">
              <t-button v-if="!paper.is_saved" theme="primary" variant="text" size="small" @click.stop="$emit('save', paper)">
                  <template #icon><i class="ri-bookmark-line"></i></template> Save
              </t-button>
              <t-button theme="default" variant="text" size="small" :href="paper.pdf_url || paper.url" target="_blank" @click.stop>
                  PDF
              </t-button>
          </div>
      </template>
  </t-card>
</template>

<script setup lang="ts">
import { PropType, computed } from 'vue';

// Loose interface to accept both InboxItem and LibraryPaper
interface PaperLike {
    title: string;
    authors: string[];
    abstract?: string;
    abstract_text?: string;
    venue?: string; // Optional (ArXiv doesn't give venue easily yet)
    year?: number;
    publish_date?: string;
    github_stars?: number;
    pdf_url?: string | null;
    url?: string;
    is_saved?: boolean;
}

const props = defineProps({
  paper: {
    type: Object as PropType<PaperLike>,
    required: true
  }
});

defineEmits(['save']);

const displayAbstract = computed(() => {
    return props.paper.abstract_text || props.paper.abstract || "No abstract available.";
});

const displayYear = computed(() => {
    if (props.paper.year) return props.paper.year;
    if (props.paper.publish_date) {
        return new Date(props.paper.publish_date).getFullYear();
    }
    return 'Unknown';
});
</script>
