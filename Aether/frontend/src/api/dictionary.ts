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

export const dictionaryApi = {
    lookup: async (word: string, signal?: AbortSignal) => {
        const res = await axios.get<DictionaryEntry>('/api/dictionary/lookup', { params: { word }, signal });
        return res.data;
    },
    fuzzy: async (word: string, signal?: AbortSignal) => {
        const res = await axios.get<string[]>('/api/dictionary/fuzzy', { params: { word }, signal });
        return res.data;
    }
};
