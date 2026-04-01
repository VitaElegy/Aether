/**
 * Sentence Anchoring 2.0 — Text Anchoring Utilities
 *
 * Provides exact hash, normalized hash, and content hash functions
 * for the sentence anchoring repair workflow.
 */

/**
 * Simple DJB2 hash function for string content.
 * Sufficient for sentence collision avoidance in a single article context.
 */
export function generateContentHash(text: string): string {
    let hash = 5381;
    const cleanText = text.trim();

    for (let i = 0; i < cleanText.length; i++) {
        hash = ((hash << 5) + hash) + cleanText.charCodeAt(i); /* hash * 33 + c */
    }

    return (hash >>> 0).toString(16);
}

/**
 * Generate a normalized hash that is resilient to whitespace and case changes.
 * Used as the second tier in the anchoring repair chain.
 */
export function generateNormalizedHash(text: string): string {
    const normalized = text.split(/\s+/).join(' ').toLowerCase().trim();
    return generateContentHash(normalized);
}

/**
 * Resolution method enum matching backend ResolutionMethod.
 */
export type ResolutionMethod =
    | { type: 'Exact' }
    | { type: 'Normalized' }
    | { type: 'Fuzzy'; similarity: number }
    | { type: 'Unresolved' }
    | { type: 'New' };

/**
 * Sentence data from the backend sentence map.
 */
export interface SentenceData {
    uuid: string;
    hash: string;
    normalized_hash: string;
    text: string;
    start_idx: number;
    local_id: number;
    global_sentence_id?: string;
    metadata?: Record<string, any>;
    resolution: ResolutionMethod;
}

/**
 * Anchoring diagnostics from the last parse run.
 */
export interface AnchoringDiagnostics {
    exact_matches: number;
    normalized_matches: number;
    fuzzy_matches: number;
    unresolved: number;
    total_sentences: number;
}

/**
 * Migrates a value from an old hash key to a new hash key in a map.
 * Used when a sentence is edited.
 */
export function migrateSentenceMap(
    map: Record<string, any>,
    oldHash: string,
    newHash: string
): Record<string, any> {
    const newMap = { ...map };

    if (newMap[oldHash]) {
        newMap[newHash] = { ...newMap[oldHash] };
        delete newMap[oldHash];
    }

    return newMap;
}

/**
 * Extract unresolved sentences from a sentence map.
 */
export function getUnresolvedSentences(sentenceMap: Record<string, SentenceData>): SentenceData[] {
    return Object.values(sentenceMap).filter(
        s => s.resolution?.type === 'Unresolved'
    );
}

/**
 * Compute simple Levenshtein distance between two strings.
 * Used for client-side fuzzy matching in the rebind UI.
 */
export function levenshteinDistance(a: string, b: string): number {
    const matrix: number[][] = [];

    for (let i = 0; i <= b.length; i++) {
        matrix[i] = [i];
    }
    for (let j = 0; j <= a.length; j++) {
        matrix[0][j] = j;
    }

    for (let i = 1; i <= b.length; i++) {
        for (let j = 1; j <= a.length; j++) {
            if (b.charAt(i - 1) === a.charAt(j - 1)) {
                matrix[i][j] = matrix[i - 1][j - 1];
            } else {
                matrix[i][j] = Math.min(
                    matrix[i - 1][j - 1] + 1, // substitution
                    matrix[i][j - 1] + 1,       // insertion
                    matrix[i - 1][j] + 1         // deletion
                );
            }
        }
    }

    return matrix[b.length][a.length];
}

/**
 * Compute similarity ratio (0-1) between two strings.
 */
export function stringSimilarity(a: string, b: string): number {
    const maxLen = Math.max(a.length, b.length);
    if (maxLen === 0) return 1;
    return 1 - levenshteinDistance(a, b) / maxLen;
}
