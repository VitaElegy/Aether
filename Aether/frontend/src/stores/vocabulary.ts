import { defineStore } from 'pinia';
import { ref } from 'vue';
import axios from 'axios';

export interface VocabularyExample {
    id: string;
    sentence: string;
    translation?: string;
    note?: string;
    image_url?: string;
    article_id?: string;
    sentence_uuid?: string;
    created_at: string;
}

export interface Vocabulary {
    id: string;
    word: string;
    definition: string;
    translation?: string;
    phonetic?: string;
    examples: VocabularyExample[];
    status: string;
    is_important: boolean;
    query_count: number;
}

export const useVocabularyStore = defineStore('vocabulary', () => {
    const vocabularies = ref<Vocabulary[]>([]);
    const loading = ref(false);

    async function searchWord(word: string) {
        // Find local first? Or always fetch from API?
        // Basic list fetch:
        try {
            const res = await axios.get(`/api/vocabulary?query=${word}&limit=1`);
            if (res.data && res.data.length > 0 && res.data[0].word.toLowerCase() === word.toLowerCase()) {
                return res.data[0];
            }
            return null;
        } catch (e) {
            console.error(e);
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
            // Invalidate or update local cache if we had one
        } catch (e) {
            console.error(e);
            throw e;
        }
    }

    // Aesthetic Preferences
    const fontMode = ref<'serif' | 'sans'>('serif');

    return {
        vocabularies,
        loading,
        fontMode,
        searchWord,
        saveVocabulary,
        addExample
    };
});
