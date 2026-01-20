/**
 * Simple DJB2 hash function for string content.
 * Sufficient for sentence collision avoidance in a single article context.
 */
export function generateContentHash(text: string): string {
    let hash = 5381;
    // Normalize: Trim whitespace to allow tolerance for spacing
    const cleanText = text.trim();

    for (let i = 0; i < cleanText.length; i++) {
        hash = ((hash << 5) + hash) + cleanText.charCodeAt(i); /* hash * 33 + c */
    }

    // Convert to positive hex string
    return (hash >>> 0).toString(16);
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
        // Move data
        newMap[newHash] = { ...newMap[oldHash] };
        // Clean old
        delete newMap[oldHash];
    }

    return newMap;
}
