<template>
  <div class="h-full">
    <Paper_v1Layout v-model:activeTab="activeTab" @add-feed="showAddFeed = true">
      <template #tools>
        <div v-if="store.fetchProgress.active" class="flex items-center space-x-3 bg-white px-4 py-2 rounded-md shadow-sm border border-blue-200 min-w-[300px]">
          <i class="ri-loader-4-line animate-spin text-blue-500"></i>
          <div class="flex-1">
            <div class="flex justify-between text-xs text-gray-500 mb-1">
              <span class="truncate max-w-[180px]">Fetching {{ store.fetchProgress.current }}/{{ store.fetchProgress.total }}: {{ store.fetchProgress.currentFeedName }}</span>
              <span>{{ Math.round((store.fetchProgress.current / store.fetchProgress.total) * 100) }}%</span>
            </div>
            <div class="w-full bg-gray-200 rounded-full h-1.5">
              <div class="bg-blue-500 h-1.5 rounded-full transition-all duration-300" :style="{ width: `${(store.fetchProgress.current / store.fetchProgress.total) * 100}%` }"></div>
            </div>
          </div>
        </div>
        <button
          v-else
          @click="store.refreshFeeds()"
          :disabled="store.loading"
          class="flex items-center px-4 py-2 text-sm font-bold text-white bg-gray-900 border border-transparent rounded-md hover:bg-gray-800 disabled:opacity-50 transition-all shadow-sm"
        >
          <i :class="store.loading ? 'animate-spin' : ''" class="ri-refresh-line mr-2"></i>
          {{ store.loading ? 'Updating...' : 'Fetch All' }}
        </button>
      </template>

      <template #default>
        <!-- PRKB-01: Feed Control Center -->
        <div v-if="activeTab === 'feeds'" class="space-y-4">
          <FeedControlCenter
            :feeds="store.feeds"
            @toggle-enabled="store.toggleFeedEnabled"
            @manual-fetch="store.refreshFeeds"
            @test-parser="store.testFeedParser"
            @delete-feed="confirmDeleteFeed"
          />
        </div>

        <!-- PRKB-02: Inbox Triage -->
        <div v-else-if="activeTab === 'inbox'" class="space-y-4">
          <div v-if="store.loading" class="text-center py-10 text-gray-500">
            <i class="ri-loader-4-line animate-spin text-2xl"></i>
            <p class="mt-2">Fetching papers...</p>
          </div>

          <div v-else-if="store.inbox.length === 0" class="text-center py-12 bg-white rounded-lg border border-dashed border-gray-300">
            <p class="text-gray-500 mb-4">Inbox is empty.</p>
            <p class="text-xs text-gray-400">Add a feed in the sidebar to get started.</p>
          </div>

          <div v-else class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
            <InboxTriageCard
              v-for="item in store.inbox"
              :key="item.id"
              :item="item"
              @save="store.savePaper"
              @skip="store.skipInboxItem"
              @trash="store.trashPaper"
              @mark-read="store.markInboxRead"
              @set-priority="store.setInboxPriority"
              @add-note="promptNote"
            />
          </div>
        </div>

        <!-- PRKB-03/04: Library with Search -->
        <div v-else-if="activeTab === 'library'" class="space-y-4">
          <SearchFacetBar :venues="store.venues" @search="handleSearch" />

          <div v-if="store.library.length === 0" class="py-12 text-center text-gray-500">
            Your library is empty. Save papers from the Inbox to see them here.
          </div>

          <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            <PaperCard
              v-for="paper in store.library"
              :key="paper.id"
              :paper="paper"
              class="h-full cursor-pointer"
              @click.native="store.fetchPaperDetail(paper.id)"
              @update="store.updatePaper"
              @trash="store.trashPaper"
            />
          </div>
        </div>

        <!-- PRKB-05: Collections -->
        <div v-else-if="activeTab === 'collections'" class="space-y-4">
          <div class="grid grid-cols-1 lg:grid-cols-4 gap-6">
            <div class="lg:col-span-1">
              <CollectionPanel
                :collections="store.collections"
                :selected-id="selectedCollectionId"
                @select="selectCollection"
                @delete="confirmDeleteCollection"
                @create="store.createCollection"
              />
            </div>
            <div class="lg:col-span-3">
              <div v-if="!selectedCollectionId" class="text-center py-12 text-gray-400">
                Select a collection to view its papers.
              </div>
              <div v-else-if="collectionPapers.length === 0" class="text-center py-12 text-gray-400">
                This collection is empty. Add papers from the library.
              </div>
              <div v-else class="grid grid-cols-1 md:grid-cols-2 gap-4">
                <PaperCard
                  v-for="paper in collectionPapers"
                  :key="paper.id"
                  :paper="paper"
                  class="h-full"
                  @update="store.updatePaper"
                  @trash="store.trashPaper"
                />
              </div>
            </div>
          </div>
        </div>

        <!-- PRKB-08: Portability -->
        <div v-else-if="activeTab === 'portability'" class="max-w-3xl mx-auto">
          <PortabilityPanel
            ref="portabilityRef"
            @export="handleExport"
            @import="handleImport"
          />
        </div>

        <!-- Fallback -->
        <div v-else class="flex items-center justify-center h-64 text-gray-400">
          Work in Progress: {{ activeTab }}
        </div>
      </template>
    </Paper_v1Layout>

    <!-- PRKB-03: Library Detail Drawer -->
    <LibraryDetailDrawer
      :paper="store.selectedPaper"
      :visible="store.drawerVisible"
      @close="store.drawerVisible = false"
      @toggle-read="(id: string, read: boolean) => store.updatePaper(id, { is_read: read })"
      @update-tags="(id: string, tags: string[]) => store.updatePaper(id, { tags })"
      @update-notes="(id: string, notes: string) => store.updatePaper(id, { notes })"
      @update-signals="store.updateSignals"
      @queue-pdf="store.queuePdfDownload"
      @export-bib="(id: string) => store.exportPapers('bibtex', undefined, [id])"
    />

    <!-- ADD FEED DIALOG -->
    <t-dialog v-model:visible="showAddFeed" header="Add Research Feed" @confirm="confirmAddFeed">
      <t-form ref="form" :data="feedForm" label-align="top">
        <t-form-item label="Quick Add from Popular Sources">
          <t-select placeholder="Select a preset..." @change="onPresetSelect">
            <t-option-group label="Conferences (DBLP RSS)">
              <t-option value="CCS|https://dblp.org/feed/streams/conf/ccs.rss|rss" label="ACM CCS" />
              <t-option value="S&P|https://dblp.org/feed/streams/conf/sp.rss|rss" label="IEEE S&P (Oakland)" />
              <t-option value="NDSS|https://dblp.org/feed/streams/conf/ndss.rss|rss" label="NDSS Symposium" />
              <t-option value="USENIX|https://dblp.org/feed/streams/conf/uss.rss|rss" label="USENIX Security" />
            </t-option-group>
            <t-option-group label="Blogs & News">
              <t-option value="Google Project Zero|https://googleprojectzero.blogspot.com/feeds/posts/default|rss" label="Google Project Zero" />
              <t-option value="The Hacker News|https://thehackernews.com/feeds/posts/default|rss" label="The Hacker News" />
              <t-option value="Full Disclosure|https://seclists.org/rss/fulldisclosure.rss|rss" label="Full Disclosure" />
            </t-option-group>
            <t-option-group label="ArXiv Categories">
              <t-option value="ArXiv Cryptography|cs.CR|arxiv" label="Cryptography & Security (cs.CR)" />
              <t-option value="ArXiv AI|cs.AI|arxiv" label="Artificial Intelligence (cs.AI)" />
              <t-option value="ArXiv SE|cs.SE|arxiv" label="Software Engineering (cs.SE)" />
            </t-option-group>
          </t-select>
        </t-form-item>

        <t-divider>Or Custom</t-divider>

        <t-form-item label="Name" name="name">
          <t-input v-model="feedForm.name" placeholder="e.g. AI Papers" />
        </t-form-item>
        <t-form-item label="Arxiv Category / URL" name="url">
          <t-input v-model="feedForm.url" placeholder="e.g. cs.AI or https://example.com/feed.xml" />
          <div class="text-xs text-gray-500 mt-1">For Arxiv, use category code (cs.CV, cs.LG). For others, use full RSS/Atom URL.</div>
        </t-form-item>
        <t-form-item label="Type" name="type">
          <t-select v-model="feedForm.type">
            <t-option value="arxiv" label="Arxiv Category" />
            <t-option value="rss" label="RSS/Atom Feed" />
          </t-select>
        </t-form-item>
      </t-form>
    </t-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue';
import { usePrkbStore } from '@/stores/prkb';
import type { Paper } from '@/stores/prkb';
import Paper_v1Layout from '../kb/layouts/Paper_v1Layout.vue';
import PaperCard from '@/components/paper/PaperCard.vue';

// PRKB Components
import FeedControlCenter from '@/components/prkb/FeedControlCenter.vue';
import InboxTriageCard from '@/components/prkb/InboxTriageCard.vue';
import LibraryDetailDrawer from '@/components/prkb/LibraryDetailDrawer.vue';
import SearchFacetBar from '@/components/prkb/SearchFacetBar.vue';
import CollectionPanel from '@/components/prkb/CollectionPanel.vue';
import PortabilityPanel from '@/components/prkb/PortabilityPanel.vue';

const store = usePrkbStore();
const activeTab = ref('inbox');
const showAddFeed = ref(false);
const feedForm = reactive({
  name: '',
  url: '',
  type: 'arxiv'
});

// PRKB-05: Collections state
const selectedCollectionId = ref<string | undefined>(undefined);
const collectionPapers = ref<Paper[]>([]);
const portabilityRef = ref<InstanceType<typeof PortabilityPanel> | null>(null);

const onPresetSelect = (value: string) => {
  const [name, url, type] = value.split('|');
  feedForm.name = name;
  feedForm.url = url;
  feedForm.type = type;
};

const confirmAddFeed = async () => {
  if (feedForm.name && feedForm.url) {
    await store.createFeed(feedForm.name, feedForm.url, feedForm.type);
    showAddFeed.value = false;
    feedForm.name = '';
    feedForm.url = '';
    feedForm.type = 'arxiv';
  }
};

const confirmDeleteFeed = async (id: string) => {
  if (confirm('Are you sure you want to remove this feed?')) {
    await store.deleteFeed(id);
  }
};

const confirmDeleteCollection = async (id: string) => {
  if (confirm('Delete this collection? Papers will not be deleted.')) {
    await store.deleteCollection(id);
    if (selectedCollectionId.value === id) {
      selectedCollectionId.value = undefined;
      collectionPapers.value = [];
    }
  }
};

// PRKB-02: Note prompt
const promptNote = async (id: string) => {
  const note = prompt('Enter note:');
  if (note !== null) {
    await store.setInboxNote(id, note);
  }
};

// PRKB-04: Search handler
const handleSearch = async (filters: any) => {
  const params = new URLSearchParams();
  if (filters.q) params.set('q', filters.q);
  if (filters.venue_id) params.set('venue_id', filters.venue_id);
  if (filters.year) params.set('year', filters.year.toString());
  if (filters.state) params.set('state', filters.state);
  if (filters.has_pdf) params.set('has_pdf', 'true');

  store.searchQuery = filters.q || '';
  await store.fetchLibrary(filters.venue_id);
};

// PRKB-05: Collection selection
const selectCollection = async (id: string) => {
  selectedCollectionId.value = id;
  collectionPapers.value = await store.fetchCollectionPapers(id);
};

// PRKB-08: Export/Import
const handleExport = (format: string) => {
  store.exportPapers(format);
};

const handleImport = async (bibtex: string, mergeTags: boolean, mergeNotes: boolean) => {
  const result = await store.importBibtex(bibtex, mergeTags, mergeNotes);
  if (portabilityRef.value && result) {
    portabilityRef.value.setImportResult(result);
  }
};

onMounted(async () => {
  await store.fetchFeeds();
  store.fetchInbox();
  store.fetchLibrary();
  store.fetchCollections();
});
</script>
