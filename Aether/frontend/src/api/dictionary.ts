import axios from 'axios';

export interface Meaning {
    partOfSpeech: string;
    definitions: {
        definition: string;
        example?: string;
    }[];
}

export interface DictionaryEntry {
    word: string;
    phonetic?: string;
    meanings: Meaning[];
    translation?: string;
    source: string;
}

/** ENG-06: Query pipeline result from unified search endpoint */
export interface QueryPipelineResult {
    word: string;
    lemma?: string;
    dictionary_entries: DictionaryEntry[];
    local_vocab?: any;
    suggestions: string[];
    inflections: string[];
}

/** ENG-06: Word family member */
export interface WordFamilyEntry {
    word: string;
    relation: string;
    score: number;
}

/** ENG-06: Collocation entry */
export interface CollocationEntry {
    phrase: string;
    position: 'before' | 'after';
    score: number;
}

export const dictionaryApi = {
    lookup: async (word: string, signal?: AbortSignal) => {
        const res = await axios.get<DictionaryEntry[]>('/api/dictionary/lookup', { params: { word }, signal });
        return res.data;
    },
    fuzzy: async (word: string, signal?: AbortSignal) => {
        const res = await axios.get<string[]>('/api/dictionary/fuzzy', { params: { word }, signal });
        return res.data;
    },
    searchSentences: async (query: string) => {
        const res = await axios.post('/api/vocabulary/sentences/search', { query });
        return res.data as { id: string, text: string, translation?: string }[];
    },

    // --- ENG-06: Search and Intelligence Pipeline ---

    /** Unified query pipeline: dictionary + lemma + inflections + local vocab + suggestions */
    queryPipeline: async (word: string, signal?: AbortSignal) => {
        const res = await axios.get<QueryPipelineResult>('/api/dictionary/query', { params: { word }, signal });
        return res.data;
    },

    /** Get word family: synonyms, antonyms, related words */
    wordFamily: async (word: string, signal?: AbortSignal) => {
        const res = await axios.get<WordFamilyEntry[]>('/api/dictionary/family', { params: { word }, signal });
        return res.data;
    },

    /** Get common collocations for a word */
    collocations: async (word: string, signal?: AbortSignal) => {
        const res = await axios.get<CollocationEntry[]>('/api/dictionary/collocations', { params: { word }, signal });
        return res.data;
    },
};
