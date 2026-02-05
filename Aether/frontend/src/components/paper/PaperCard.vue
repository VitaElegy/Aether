<template>
  <div class="group relative bg-white border border-gray-200 rounded-sm hover:border-blue-400 hover:shadow-md transition-all duration-200 h-full flex flex-col">
    <!-- Main Click Area -->
    <a :href="paper.pdf_url || paper.url" target="_blank" class="block p-4 flex-1" @click.stop>
      <!-- Header: Title & Meta -->
      <div class="mb-2">
        <h3 class="text-base font-bold text-gray-900 leading-snug font-serif group-hover:text-blue-700 mb-1">
          {{ paper.title }}
        </h3>
        <div class="flex items-center gap-2 text-xs text-gray-500 font-mono">
           <span v-if="displayVenue" class="px-1.5 py-0.5 bg-gray-100 text-gray-700 rounded-sm uppercase tracking-wide text-[10px] font-bold">
             {{ displayVenue }}
           </span>
           <span v-if="displayYear" class="text-gray-400 font-sans">{{ displayYear }}</span>
        </div>
      </div>

      <!-- Authors -->
      <div class="mb-3 text-xs text-gray-600 truncate font-medium">
        {{ displayAuthors }}
      </div>

      <!-- Abstract -->
      <div class="text-[13px] text-gray-500 leading-relaxed line-clamp-3 font-sans mb-2">
        {{ displayAbstract }}
      </div>

      <!-- Keywords -->
      <div v-if="displayKeywords" class="flex flex-wrap gap-1 mb-2">
         <span v-for="kw in displayKeywords" :key="kw" class="text-[10px] text-gray-500 bg-gray-50 border border-gray-100 px-1 rounded">
            {{ kw }}
         </span>
      </div>
    </a>

    <!-- Footer: Stats & Actions -->
    <div class="px-3 py-2 border-t border-gray-50 bg-gray-50/50 flex justify-between items-center mt-auto">
      <!-- Left: Signals -->
      <div class="flex items-center gap-3">
        <!-- GitHub -->
         <a v-if="paper.github_repo || displayStars" 
            :href="paper.github_repo" 
            target="_blank"
            class="flex items-center gap-1 text-xs text-gray-600 hover:text-black transition-colors"
            title="GitHub Repository"
            @click.stop
         >
            <i class="ri-github-fill text-sm"></i>
            <span v-if="displayStars" class="font-mono font-bold">{{ formatNumber(displayStars) }}</span>
         </a>

         <!-- Citations -->
         <div v-if="displayCitations" class="flex items-center gap-1 text-xs text-gray-400" title="Citations">
            <i class="ri-double-quotes-l"></i>
            <span class="font-mono">{{ formatNumber(displayCitations) }}</span>
         </div>
      </div>

      <!-- Right: Actions -->
      <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
        <!-- Inbox Actions -->
        <template v-if="!isLibraryItem">
             <button 
                class="p-1.5 text-gray-400 hover:text-green-600 hover:bg-green-50 rounded transition-colors"
                title="Accept to Library"
                @click.stop="$emit('save', paper)"
             >
                <i class="ri-check-line text-lg"></i>
             </button>
             <button 
                class="p-1.5 text-gray-400 hover:text-red-600 hover:bg-red-50 rounded transition-colors"
                title="Dismiss"
                @click.stop="$emit('trash', paper)"
             >
                <i class="ri-delete-bin-line text-lg"></i>
             </button>
        </template>

        <!-- Library Actions -->
        <template v-else>
            <!-- Read Toggle -->
            <button 
                class="p-1.5 rounded transition-colors"
                :class="paper.is_read ? 'text-green-600 bg-green-50' : 'text-gray-400 hover:text-blue-600 hover:bg-blue-50'"
                :title="paper.is_read ? 'Mark as Unread' : 'Mark as Read'"
                @click.stop="$emit('update', paper.id, { is_read: !paper.is_read })"
            >
                <i :class="paper.is_read ? 'ri-checkbox-circle-fill' : 'ri-checkbox-blank-circle-line'" class="text-lg"></i>
            </button>
        </template>

        <!-- Common Actions -->
        <a 
            :href="paper.pdf_url || paper.url" 
            target="_blank"
            class="p-1.5 text-gray-400 hover:text-blue-600 hover:bg-blue-50 rounded transition-colors"
            title="Open PDF"
            @click.stop
        >
            <i class="ri-file-pdf-line text-lg"></i>
        </a>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { PropType, computed } from 'vue';

// Loose interface but with V2 types optionally
interface Author {
    name: string;
}
interface Venue {
    name: string;
}
interface Signals {
    citation_count: number;
    github_stars: number;
}

interface PaperLike {
    title: string;
    authors: string[] | Author[];
    abstract?: string;
    abstract_text?: string;
    venue?: string | Venue; 
    publication?: string;
    year?: number;
    publish_date?: string;
    github_stars?: number;
    github_repo?: string;
    signals?: Signals;
    pdf_url?: string | null;
    url?: string;
    is_saved?: boolean;
    state?: string;
    is_read?: boolean;
    id?: string;
    metadata?: {
        track?: string;
        series?: string;
        keywords?: string[];
    };
}

const props = defineProps({
  paper: {
    type: Object as PropType<PaperLike>,
    required: true
  }
});

defineEmits(['save', 'update', 'trash']);

// ... computeds ...

const displayTrack = computed(() => props.paper.metadata?.track);
const displaySeries = computed(() => props.paper.metadata?.series);
const displayKeywords = computed(() => props.paper.metadata?.keywords?.slice(0, 3)); // Show top 3

const trackColorClass = computed(() => {
    const t = (displayTrack.value || '').toLowerCase();
    if (t.includes('oral') || t.includes('spotlight')) return 'bg-red-50 text-red-600 border border-red-100';
    if (t.includes('poster')) return 'bg-blue-50 text-blue-600 border border-blue-100';
    return 'bg-gray-100 text-gray-600';
});

const isLibraryItem = computed(() => {
    return 'state' in props.paper && props.paper.state !== 'inbox'; 
});

const displayAbstract = computed(() => {
    return props.paper.abstract_text || props.paper.abstract || "No abstract available.";
});

const displayAuthors = computed(() => {
    const list = props.paper.authors || [];
    const names = list.map(a => typeof a === 'string' ? a : a.name);
    if (names.length === 0) return 'Unknown Author';
    // Show more authors in a dense line
    return names.join(', ');
});

const displayVenue = computed(() => {
    if (props.paper.venue) {
        return typeof props.paper.venue === 'string' ? props.paper.venue : props.paper.venue.name;
    }
    return props.paper.publication;
});

const displayYear = computed(() => {
    if (props.paper.year) return props.paper.year;
    if (props.paper.publish_date) {
        return new Date(props.paper.publish_date).getFullYear();
    }
    return '';
});

const displayStars = computed(() => {
    if (props.paper.signals?.github_stars) return props.paper.signals.github_stars;
    return props.paper.github_stars;
});

const displayCitations = computed(() => {
    return props.paper.signals?.citation_count;
});

const formatNumber = (num: number) => {
    if (num >= 1000) return (num / 1000).toFixed(1) + 'k';
    return num;
};
</script>

<style scoped>
/* Optional: specific line clamp override if tailwind plugin not configured */
.line-clamp-3 {
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
</style>
