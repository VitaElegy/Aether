<template>
  <div class="h-full">
    <Paper_v1Layout v-model:activeTab="activeTab" @add-feed="showAddFeed = true">
      <template #tools>
         <button 
            @click="store.refreshFeeds()"
            :disabled="store.loading"
            class="flex items-center px-4 py-2 text-sm font-bold text-white bg-gray-900 border border-transparent rounded-md hover:bg-gray-800 disabled:opacity-50 transition-all shadow-sm"
         >
            <i :class="store.loading ? 'animate-spin' : ''" class="ri-refresh-line mr-2"></i>
            {{ store.loading ? 'Updating...' : 'Fetch All' }}
         </button>
      </template>
      <template #default>
        <!-- Inbox View -->
        <div v-if="activeTab === 'inbox'" class="space-y-4">
            <div v-if="store.loading" class="text-center py-10 text-gray-500">
                <i class="ri-loader-4-line animate-spin text-2xl"></i>
                <p class="mt-2">Fetching papers from ArXiv...</p>
            </div>
            
            <div v-else-if="store.inbox.length === 0" class="text-center py-12 bg-white rounded-lg border border-dashed border-gray-300">
                <p class="text-gray-500 mb-4">Inbox is empty.</p>
                <div class="flex flex-col items-center gap-2">
                    <p class="text-xs text-gray-400">Add a feed (e.g., 'cs.CV') in the sidebar to get started.</p>
                </div>
            </div>

            <div v-else class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
                <PaperCard 
                    v-for="paper in store.inbox" 
                    :key="paper.external_id || paper.id" 
                    :paper="paper" 
                    @save="store.savePaper"
                    @trash="store.trashPaper" 
                />
            </div>
        </div>

        <!-- Library View -->
        <div v-else-if="activeTab === 'library'" class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
             <div v-if="store.library.length === 0" class="col-span-full py-12 text-center text-gray-500">
                 Your library is empty. Save papers from the Inbox to see them here.
             </div>
             
             <PaperCard 
                v-for="paper in store.library" 
                :key="paper.id" 
                :paper="paper" 
                class="h-full"
                @update="store.updatePaper"
                @trash="store.trashPaper"
            />
        </div>

        <div v-else class="flex items-center justify-center h-64 text-gray-400">
            Work in Progress: {{ activeTab }}
        </div>
      </template>
    </Paper_v1Layout>

    <!-- ADD FEED DIALOG -->
    <t-dialog v-model:visible="showAddFeed" header="Add Research Feed" @confirm="confirmAddFeed">
        <t-form ref="form" :data="feedForm" label-align="top">
            <t-form-item label="Quick Add from Popular Sources">
                <t-select placeholder="Select a preset..." @change="onPresetSelect">
                    <t-option-group label="Conferences (DBLP RSS)">
                        <t-option value="CCS|https://dblp.org/db/conf/ccs/index.rss|rss" label="ACM CCS" />
                        <t-option value="S&P|https://dblp.org/db/conf/sp/index.rss|rss" label="IEEE S&P (Oakland)" />
                        <t-option value="NDSS|https://dblp.org/db/conf/ndss/index.rss|rss" label="NDSS Symposium" />
                        <t-option value="USENIX|https://dblp.org/db/conf/uss/index.rss|rss" label="USENIX Security" />
                    </t-option-group>
                    <t-option-group label="Blogs & News">
                         <t-option value="Google Project Zero|https://googleprojectzero.blogspot.com/feeds/posts/default|rss" label="Google Project Zero" />
                         <t-option value="Sec-Circus|https://sec-circus.com/feed|rss" label="Sec-Circus" />
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
import { usePrkbStore } from '@/stores/prkb'; // Ensure store is imported
import Paper_v1Layout from '../kb/layouts/Paper_v1Layout.vue';
import PaperCard from '@/components/paper/PaperCard.vue';
import { AddIcon, DeleteIcon, RefreshIcon } from 'tdesign-icons-vue-next'; // Import necessary icons for Sidebar

const store = usePrkbStore();
const activeTab = ref('inbox');
const showAddFeed = ref(false); // To match usage in template if restored, or keep simple
const feedForm = reactive({
    name: '',
    url: '',
    type: 'arxiv'
});

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

onMounted(async () => {
    await store.fetchFeeds();
    
    // Auto-seed if empty
    if (store.feeds.length === 0) {
        // Optional: auto-seed common CS feeds
        // await store.createFeed('CS AI', 'cs.AI', 'arxiv');
    }
    
    store.fetchInbox();
    store.fetchLibrary();
});
</script>
