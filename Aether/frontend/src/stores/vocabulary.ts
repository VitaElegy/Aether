import { defineStore } from 'pinia';
import { ref } from 'vue';
import axios from 'axios';

export type VocabularyLevel = 'A1' | 'A2' | 'B1' | 'B2' | 'C1' | 'C2' | 'Unknown';
export type MasteryStatus = 'New' | 'Learning' | 'Familiar' | 'Mastered';

export interface VocabularyExample {
    id: string;
    sentence: string;
    translation?: string;
    note?: string;
    image_url?: string;
    article_id?: string;
    sentence_uuid?: string;
    created_at: string;
    global_sentence_id?: string;
}

export interface Vocabulary {
    id: string;
    /** Canonical lemma form (e.g. "run" for "running") */
    lemma?: string;
    word: string;
    definition: string;
    translation?: string;
    phonetic?: string;
    root?: string;
    examples: VocabularyExample[];
    status: string;
    is_important: boolean;
    query_count: number;
    /** CEFR proficiency level */
    level: VocabularyLevel;
    /** User-defined tags */
    tags: string[];
    /** Mastery status for spaced repetition */
    mastery: MasteryStatus;
    /** Source KB where first encountered */
    source_kb_id?: string;
    /** Soft-delete flag */
    is_archived: boolean;
    language: string;
    created_at: string;
}

export const useVocabularyStore = defineStore('vocabulary', () => {
    const vocabularies = ref<Vocabulary[]>([]);
    const loading = ref(false);

    async function searchWord(word: string) {
        try {
            console.log(`[Store] Searching for word: ${word}`);
            const res = await axios.get(`/api/vocabulary?query=${word}&limit=1`);
            console.log('[Store] Search response:', res.data);

            const list = Array.isArray(res.data) ? res.data : (res.data?.data || []);

            if (list.length > 0) {
                return list[0];
            }
            return null;
        } catch (e) {
            console.error('[Store] Search failed', e);
            return null;
        }
    }

    async function saveVocabulary(data: Partial<Vocabulary>) {
        try {
            const res = await axios.post('/api/vocabulary', data);
            return res.data.id;
        } catch (e) {
            console.error(e);
            throw e;
        }
    }

    async function addExample(vocabId: string, example: any) {
        try {
            await axios.post(`/api/vocabulary/${vocabId}/examples`, example);
        } catch (e) {
            console.error(e);
            throw e;
        }
    }

    async function fetchVocabulary(params: { limit?: number; offset?: number; query?: string; kb_id?: string }) {
        try {
            loading.value = true;
            const res = await axios.get('/api/vocabulary', { params });
            return res.data;
        } catch (e) {
            console.error('[Store] Fetch vocabulary failed', e);
            throw e;
        } finally {
            loading.value = false;
        }
    }

    // --- ENG-03: Batch Operations ---

    async function batchTag(ids: string[], tags: string[], mode: 'add' | 'set' = 'add') {
        const res = await axios.post('/api/vocabulary/batch-tag', { ids, tags, mode });
        return res.data;
    }

    async function batchImportance(ids: string[], isImportant: boolean) {
        const res = await axios.post('/api/vocabulary/batch-importance', { ids, is_important: isImportant });
        return res.data;
    }

    async function batchArchive(ids: string[]) {
        const res = await axios.post('/api/vocabulary/batch-archive', { ids });
        return res.data;
    }

    async function batchRestore(ids: string[]) {
        const res = await axios.post('/api/vocabulary/batch-restore', { ids });
        return res.data;
    }

    async function mergeDuplicates(primaryId: string, duplicateIds: string[]) {
        const res = await axios.post('/api/vocabulary/merge', {
            primary_id: primaryId,
            duplicate_ids: duplicateIds,
        });
        return res.data;
    }

    // Aesthetic Preferences
    const fontMode = ref<'serif' | 'sans'>('serif');

    return {
        vocabularies,
        loading,
        fontMode,
        searchWord,
        saveVocabulary,
        addExample,
        fetchVocabulary,
        // ENG-03 batch operations
        batchTag,
        batchImportance,
        batchArchive,
        batchRestore,
        mergeDuplicates,
    };
});
