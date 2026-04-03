import { ref, shallowRef, type Ref } from 'vue';
import {
    assetsApi,
    type AssetNode,
    type AssetType,
    type ListAssetsParams,
} from '@/api/assets';

export type PickerMode = 'modal' | 'split';

export interface AssetPickerOptions {
    mode?: PickerMode;
    multiple?: boolean;
    acceptTypes?: AssetType[];
    initialQuery?: string;
}

export interface AssetPickerResult {
    assets: AssetNode[];
    cancelled: boolean;
}

type SelectCallback = (result: AssetPickerResult) => void;

const isOpen = ref(false);
const mode = ref<PickerMode>('modal');
const multiple = ref(false);
const acceptTypes = ref<AssetType[]>([]);
const selectedAssets = shallowRef<AssetNode[]>([]);
const searchQuery = ref('');
const assets = shallowRef<AssetNode[]>([]);
const loading = ref(false);
const recentAssets = shallowRef<AssetNode[]>([]);

let _onSelectCallback: SelectCallback | null = null;
let _resolvePromise: ((result: AssetPickerResult) => void) | null = null;

async function fetchPickerAssets(params?: ListAssetsParams) {
    loading.value = true;
    try {
        const queryParams: ListAssetsParams = {
            limit: 100,
            sort_by: 'newest',
            ...params,
        };

        if (searchQuery.value.trim()) {
            queryParams.q = searchQuery.value.trim();
        }
        if (acceptTypes.value.length === 1) {
            queryParams.asset_type = acceptTypes.value[0];
        }

        const response = await assetsApi.list(queryParams);
        assets.value = response.items;
    } catch (error) {
        console.error('[AssetPicker] Failed to fetch assets', error);
        assets.value = [];
    } finally {
        loading.value = false;
    }
}

function openPicker(options: AssetPickerOptions = {}): Promise<AssetPickerResult> {
    mode.value = options.mode ?? 'modal';
    multiple.value = options.multiple ?? false;
    acceptTypes.value = options.acceptTypes ?? [];
    searchQuery.value = options.initialQuery ?? '';
    selectedAssets.value = [];
    isOpen.value = true;

    void fetchPickerAssets();

    return new Promise<AssetPickerResult>((resolve) => {
        _resolvePromise = resolve;
    });
}

function closePicker() {
    isOpen.value = false;
    const result: AssetPickerResult = {
        assets: [],
        cancelled: true,
    };
    if (_onSelectCallback) {
        _onSelectCallback(result);
    }
    if (_resolvePromise) {
        _resolvePromise(result);
        _resolvePromise = null;
    }
    _onSelectCallback = null;
}

function confirmSelection() {
    const result: AssetPickerResult = {
        assets: [...selectedAssets.value],
        cancelled: false,
    };
    isOpen.value = false;
    if (_onSelectCallback) {
        _onSelectCallback(result);
    }
    if (_resolvePromise) {
        _resolvePromise(result);
        _resolvePromise = null;
    }
    _onSelectCallback = null;

    // Update recent assets (prepend, deduplicate, cap at 20)
    const newRecent = [...selectedAssets.value];
    const existingIds = new Set(newRecent.map((a) => a.id));
    for (const a of recentAssets.value) {
        if (!existingIds.has(a.id)) {
            newRecent.push(a);
            existingIds.add(a.id);
        }
    }
    recentAssets.value = newRecent.slice(0, 20);
}

function toggleAssetSelection(asset: AssetNode) {
    if (!multiple.value) {
        selectedAssets.value = [asset];
        return;
    }

    const existing = selectedAssets.value.find((a) => a.id === asset.id);
    if (existing) {
        selectedAssets.value = selectedAssets.value.filter((a) => a.id !== asset.id);
    } else {
        selectedAssets.value = [...selectedAssets.value, asset];
    }
}

function isAssetSelected(assetId: string): boolean {
    return selectedAssets.value.some((a) => a.id === assetId);
}

function onSelect(callback: SelectCallback) {
    _onSelectCallback = callback;
}

async function searchAssets(query: string) {
    searchQuery.value = query;
    await fetchPickerAssets();
}

async function filterByType(assetType?: AssetType) {
    if (assetType) {
        acceptTypes.value = [assetType];
    } else {
        acceptTypes.value = [];
    }
    await fetchPickerAssets();
}

/**
 * Reset all global state. Intended for test isolation — call in beforeEach.
 * @internal
 */
function _resetForTesting() {
    isOpen.value = false;
    mode.value = 'modal';
    multiple.value = false;
    acceptTypes.value = [];
    selectedAssets.value = [];
    searchQuery.value = '';
    assets.value = [];
    loading.value = false;
    recentAssets.value = [];
    _onSelectCallback = null;
    _resolvePromise = null;
}

export function useAssetPicker() {
    return {
        // State
        isOpen: isOpen as Readonly<Ref<boolean>>,
        mode: mode as Readonly<Ref<PickerMode>>,
        multiple: multiple as Readonly<Ref<boolean>>,
        acceptTypes: acceptTypes as Readonly<Ref<AssetType[]>>,
        selectedAssets: selectedAssets as Readonly<Ref<AssetNode[]>>,
        searchQuery: searchQuery as Readonly<Ref<string>>,
        assets: assets as Readonly<Ref<AssetNode[]>>,
        loading: loading as Readonly<Ref<boolean>>,
        recentAssets: recentAssets as Readonly<Ref<AssetNode[]>>,

        // Actions
        openPicker,
        closePicker,
        confirmSelection,
        toggleAssetSelection,
        isAssetSelected,
        onSelect,
        searchAssets,
        filterByType,
        fetchPickerAssets,
        _resetForTesting,
    };
}
