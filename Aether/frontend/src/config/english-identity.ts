/**
 * English Module Identity & Capability Map
 *
 * Consolidates the legacy renderer IDs (english_v1, vocabulary, article-analysis)
 * into a single canonical identity with a capability-based tab system.
 */

export const ENGLISH_RENDERER_ID = 'english_v1' as const;

/** All legacy renderer IDs that resolve to the English module. */
export const ENGLISH_ALIASES = [
  'english',
  'english_v1',
  'english_v1_std',
  'vocabulary',
  'vocabulary_std',
  'article_analysis',
  'article-analysis',
  'english_analysis',
  'english analysis',
] as const;

export type EnglishCapability = 'vocabulary' | 'articles' | 'search' | 'portability';

export interface EnglishTabMode {
  id: EnglishCapability;
  label: string;
  icon: string;
  default: boolean;
}

export interface EnglishShellLaunchRule {
  source: string;
  targetTab: EnglishCapability;
  description: string;
}

/** The canonical tab mode contract for the English shell. */
export const ENGLISH_TAB_MODES: EnglishTabMode[] = [
  { id: 'vocabulary', label: 'Words', icon: 'ri-book-2-line', default: false },
  { id: 'articles', label: 'Articles', icon: 'ri-article-line', default: true },
  { id: 'search', label: 'Search', icon: 'ri-search-line', default: false },
  { id: 'portability', label: 'Import/Export', icon: 'ri-upload-cloud-line', default: false },
];

/** Shell launch rules: how to open the English module from different contexts. */
export const ENGLISH_LAUNCH_RULES: EnglishShellLaunchRule[] = [
  { source: 'kb_open', targetTab: 'articles', description: 'Opening an English KB defaults to article workspace' },
  { source: 'vocabulary_direct', targetTab: 'vocabulary', description: 'Direct vocabulary link opens the vocabulary tab' },
  { source: 'article_analysis', targetTab: 'articles', description: 'Legacy article-analysis renderer opens articles tab' },
  { source: 'search_query', targetTab: 'search', description: 'Opening from a search context activates search tab' },
];

/** Returns true if the given renderer ID resolves to the English module. */
export function isEnglishRenderer(rendererId?: string | null): boolean {
  if (!rendererId) return false;
  const normalized = rendererId.trim().toLowerCase().replace(/[\s_]+/g, '_');
  return ENGLISH_ALIASES.some(alias => {
    const normalizedAlias = alias.replace(/[\s_-]+/g, '_').toLowerCase();
    return normalized === normalizedAlias;
  });
}

/** Resolves a legacy renderer ID to the canonical English renderer ID if applicable. */
export function resolveEnglishRenderer(rendererId?: string | null): string | null {
  if (isEnglishRenderer(rendererId)) return ENGLISH_RENDERER_ID;
  return rendererId ?? null;
}

/** Given a launch source context, returns the target tab. Falls back to default. */
export function resolveEnglishLaunchTab(source?: string): EnglishCapability {
  if (source) {
    const rule = ENGLISH_LAUNCH_RULES.find(r => r.source === source);
    if (rule) return rule.targetTab;
  }
  return ENGLISH_TAB_MODES.find(t => t.default)?.id ?? 'articles';
}

/**
 * Resolves tab restore from persisted state.
 * If the persisted tab ID is valid, returns it. Otherwise returns the default.
 */
export function resolveEnglishTabRestore(persistedTab?: string): EnglishCapability {
  if (persistedTab && ENGLISH_TAB_MODES.some(t => t.id === persistedTab)) {
    return persistedTab as EnglishCapability;
  }
  return resolveEnglishLaunchTab();
}
