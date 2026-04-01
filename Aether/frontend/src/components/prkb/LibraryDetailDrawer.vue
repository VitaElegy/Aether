<template>
  <t-drawer
    :visible="visible"
    @update:visible="$emit('update:visible', $event)"
    size="600px"
    :header="paper?.title || 'Paper Details'"
    :footer="false"
    @close="$emit('close')"
  >
    <div v-if="paper" class="space-y-6 pb-8">
      <!-- Title & State -->
      <div>
        <h2 class="text-xl font-serif font-bold text-gray-900 leading-tight mb-2">{{ paper.title }}</h2>
        <div class="flex items-center gap-2 flex-wrap">
          <span class="px-2 py-0.5 text-xs font-bold rounded" :class="stateClass">{{ paper.state }}</span>
          <span v-if="paper.venue" class="px-2 py-0.5 text-xs bg-blue-50 text-blue-700 rounded">{{ paper.venue.name }}</span>
          <span class="text-xs text-gray-400">{{ displayYear }}</span>
          <span class="px-2 py-0.5 text-xs rounded" :class="pdfStatusClass">PDF: {{ paper.pdf_status }}</span>
        </div>
      </div>

      <!-- Authors -->
      <div>
        <h4 class="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-2">Authors</h4>
        <div class="flex flex-wrap gap-2">
          <span v-for="author in paper.authors" :key="author.id"
            class="px-2 py-1 bg-gray-100 text-gray-700 rounded text-sm">
            {{ author.name }}
          </span>
        </div>
      </div>

      <!-- Abstract -->
      <div>
        <h4 class="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-2">Abstract</h4>
        <p class="text-sm text-gray-700 leading-relaxed">{{ paper.abstract_text }}</p>
      </div>

      <!-- Tags -->
      <div>
        <h4 class="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-2">Tags</h4>
        <div class="flex flex-wrap gap-1 mb-2">
          <span v-for="tag in paper.tags" :key="tag"
            class="px-2 py-0.5 bg-indigo-50 text-indigo-700 rounded text-xs cursor-pointer hover:bg-indigo-100"
            @click="removeTag(tag)"
          >
            {{ tag }} <i class="ri-close-line ml-1"></i>
          </span>
        </div>
        <div class="flex gap-2">
          <input v-model="newTag" placeholder="Add tag..." class="text-sm border rounded px-2 py-1 flex-1" @keydown.enter="addTag" />
          <button @click="addTag" class="px-3 py-1 text-xs bg-indigo-600 text-white rounded hover:bg-indigo-700">Add</button>
        </div>
      </div>

      <!-- Signals -->
      <div v-if="paper.signals">
        <h4 class="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-2">Signals</h4>
        <div class="grid grid-cols-2 gap-3">
          <div class="bg-gray-50 rounded p-3">
            <div class="text-lg font-mono font-bold text-gray-900">{{ paper.signals.citation_count }}</div>
            <div class="text-xs text-gray-500">Citations</div>
          </div>
          <div class="bg-gray-50 rounded p-3">
            <div class="text-lg font-mono font-bold text-gray-900">{{ paper.signals.github_stars }}</div>
            <div class="text-xs text-gray-500">GitHub Stars</div>
          </div>
          <div v-if="paper.signals.venue_tier" class="bg-gray-50 rounded p-3">
            <div class="text-lg font-mono font-bold text-gray-900">{{ paper.signals.venue_tier }}</div>
            <div class="text-xs text-gray-500">Venue Tier</div>
          </div>
          <div v-if="paper.signals.custom_importance" class="bg-gray-50 rounded p-3">
            <div class="text-lg font-mono font-bold text-gray-900">{{ paper.signals.custom_importance }}/5</div>
            <div class="text-xs text-gray-500">Importance</div>
          </div>
        </div>
        <!-- Custom importance setter -->
        <div class="mt-3 flex items-center gap-2">
          <span class="text-xs text-gray-500">Set importance:</span>
          <button v-for="n in 5" :key="n"
            @click="$emit('update-signals', paper.id, { custom_importance: n })"
            class="w-7 h-7 rounded text-sm transition-colors"
            :class="paper.signals.custom_importance === n ? 'bg-amber-400 text-white font-bold' : 'bg-gray-100 text-gray-600 hover:bg-gray-200'"
          >{{ n }}</button>
        </div>
      </div>

      <!-- Notes -->
      <div>
        <h4 class="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-2">Notes</h4>
        <textarea
          v-model="localNotes"
          placeholder="Add your notes here..."
          class="w-full border rounded p-3 text-sm min-h-[120px] focus:outline-none focus:ring-1 focus:ring-blue-400"
          @blur="saveNotes"
        ></textarea>
      </div>

      <!-- PDF Status -->
      <div>
        <h4 class="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-2">PDF</h4>
        <div class="flex items-center gap-3">
          <span class="px-2 py-1 rounded text-xs" :class="pdfStatusClass">{{ paper.pdf_status }}</span>
          <a v-if="paper.pdf_url" :href="paper.pdf_url" target="_blank"
            class="text-xs text-blue-600 hover:underline flex items-center gap-1">
            <i class="ri-external-link-line"></i> Open PDF
          </a>
          <button v-if="paper.pdf_status === 'not_attached' && paper.pdf_url"
            @click="$emit('queue-pdf', paper.id)"
            class="text-xs px-3 py-1 bg-blue-600 text-white rounded hover:bg-blue-700">
            Download PDF
          </button>
        </div>
      </div>

      <!-- Actions -->
      <div class="border-t pt-4 flex items-center gap-2 flex-wrap">
        <button @click="$emit('toggle-read', paper.id, !paper.is_read)"
          class="px-3 py-1.5 text-xs rounded border transition-colors"
          :class="paper.is_read ? 'bg-green-50 border-green-200 text-green-700' : 'border-gray-300 text-gray-600 hover:bg-gray-50'"
        >
          <i :class="paper.is_read ? 'ri-checkbox-circle-fill' : 'ri-checkbox-blank-circle-line'" class="mr-1"></i>
          {{ paper.is_read ? 'Read' : 'Mark Read' }}
        </button>
        <a v-if="paper.url" :href="paper.url" target="_blank"
          class="px-3 py-1.5 text-xs rounded border border-gray-300 text-gray-600 hover:bg-gray-50">
          <i class="ri-external-link-line mr-1"></i> Open Source
        </a>
        <button @click="copyCitation" class="px-3 py-1.5 text-xs rounded border border-gray-300 text-gray-600 hover:bg-gray-50">
          <i class="ri-file-copy-line mr-1"></i> Copy Citation
        </button>
        <button @click="$emit('export-bib', paper.id)" class="px-3 py-1.5 text-xs rounded border border-gray-300 text-gray-600 hover:bg-gray-50">
          <i class="ri-download-line mr-1"></i> Export BibTeX
        </button>
      </div>
    </div>
  </t-drawer>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { MessagePlugin } from 'tdesign-vue-next';
import type { Paper } from '@/stores/prkb';

const props = defineProps<{ paper: Paper | null; visible: boolean }>();
const emit = defineEmits(['close', 'update:visible', 'toggle-read', 'update-signals', 'queue-pdf', 'export-bib', 'update-tags', 'update-notes']);

const newTag = ref('');
const localNotes = ref('');

watch(() => props.paper, (p) => {
  localNotes.value = p?.notes || '';
}, { immediate: true });

const stateClass = computed(() => {
  switch (props.paper?.state) {
    case 'Inbox': return 'bg-blue-100 text-blue-700';
    case 'Reading': return 'bg-amber-100 text-amber-700';
    case 'Done': return 'bg-green-100 text-green-700';
    case 'Trash': return 'bg-red-100 text-red-700';
    default: return 'bg-gray-100 text-gray-600';
  }
});

const pdfStatusClass = computed(() => {
  switch (props.paper?.pdf_status) {
    case 'downloaded': case 'indexed': return 'bg-green-100 text-green-700';
    case 'queued': return 'bg-blue-100 text-blue-700';
    case 'failed': return 'bg-red-100 text-red-700';
    default: return 'bg-gray-100 text-gray-600';
  }
});

const displayYear = computed(() => {
  if (!props.paper?.publish_date) return '';
  return new Date(props.paper.publish_date).getFullYear();
});

const addTag = () => {
  if (!newTag.value.trim() || !props.paper) return;
  const tags = [...props.paper.tags, newTag.value.trim()];
  emit('update-tags', props.paper.id, tags);
  newTag.value = '';
};

const removeTag = (tag: string) => {
  if (!props.paper) return;
  const tags = props.paper.tags.filter(t => t !== tag);
  emit('update-tags', props.paper.id, tags);
};

const saveNotes = () => {
  if (!props.paper) return;
  if (localNotes.value !== (props.paper.notes || '')) {
    emit('update-notes', props.paper.id, localNotes.value);
  }
};

const copyCitation = () => {
  if (!props.paper) return;
  const authors = props.paper.authors.map(a => a.name).join(', ');
  const year = new Date(props.paper.publish_date).getFullYear();
  const venue = props.paper.venue?.name || '';
  const citation = `${authors}. "${props.paper.title}". ${venue}${venue ? ', ' : ''}${year}.`;
  navigator.clipboard.writeText(citation);
  MessagePlugin.success('Citation copied');
};
</script>
