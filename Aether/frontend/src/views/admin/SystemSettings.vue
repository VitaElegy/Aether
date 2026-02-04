<template>
    <div class="p-8 max-w-4xl mx-auto">
        <div class="mb-8">
            <h2 class="text-2xl font-bold font-serif text-ink">System Configuration</h2>
            <p class="text-ink/60 mt-1">Manage global system parameters and limits.</p>
        </div>

        <div class="bg-surface rounded-xl border border-ink/5 shadow-sm overflow-hidden">
            <!-- Loading State -->
            <div v-if="loading" class="p-8 flex justify-center">
                <div class="w-8 h-8 rounded-full border-4 border-accent border-t-transparent animate-spin"></div>
            </div>

            <!-- Content -->
            <div v-else class="p-6 space-y-8">
                
                <!-- Section: Storage Limits -->
                <div class="space-y-4">
                    <div class="flex items-center gap-3 pb-2 border-b border-ink/5">
                        <i class="ri-hard-drive-2-line text-accent text-xl"></i>
                        <h3 class="font-bold text-ink">Storage Limits</h3>
                    </div>

                    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                        <div class="space-y-2">
                            <label class="block text-sm font-bold text-ink/70">Max Upload Size (MB)</label>
                            <div class="relative">
                                <input type="number" v-model.number="settings.max_upload_size_mb" 
                                    class="w-full bg-paper border border-ink/10 rounded-lg px-4 py-2 text-ink focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent transition-all"
                                    min="1" max="500">
                                <span class="absolute right-4 top-1/2 -translate-y-1/2 text-xs font-bold text-ink/30 pointer-events-none">MB</span>
                            </div>
                            <p class="text-xs text-ink/40">Maximum size for a single file upload. Server hard limit is 500MB.</p>
                        </div>
                    </div>
                </div>

            </div>

            <!-- Actions -->
            <div class="bg-ash/5 px-6 py-4 flex items-center justify-end gap-3 border-t border-ink/5">
                <span v-if="successMessage" class="text-green-600 text-sm font-bold animate-fade-in mr-4">
                    <i class="ri-checkbox-circle-line align-bottom"></i> {{ successMessage }}
                </span>
                
                <button @click="fetchSettings" class="px-4 py-2 hover:bg-ink/5 text-ink/60 rounded-lg text-sm font-bold transition-colors">
                    Reset
                </button>
                <button @click="saveSettings" :disabled="saving" class="px-6 py-2 bg-accent hover:bg-accent-hover text-white rounded-lg text-sm font-bold shadow-lg shadow-accent/20 transition-all flex items-center gap-2">
                    <span v-if="saving" class="w-4 h-4 rounded-full border-2 border-white/50 border-t-white animate-spin"></span>
                    <span>{{ saving ? 'Saving...' : 'Save Changes' }}</span>
                </button>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { systemApi, type SystemSettings } from '@/api/system';

const loading = ref(true);
const saving = ref(false);
const successMessage = ref('');

const settings = ref<SystemSettings>({
    max_upload_size_mb: 5
});

const fetchSettings = async () => {
    loading.value = true;
    try {
        const res = await systemApi.getSettings();
        settings.value = res.data;
    } catch (e) {
        console.error("Failed to load settings", e);
    } finally {
        loading.value = false;
    }
};

const saveSettings = async () => {
    saving.value = true;
    successMessage.value = '';
    try {
        await systemApi.updateSettings(settings.value);
        successMessage.value = 'Settings saved successfully!';
        setTimeout(() => { successMessage.value = ''; }, 3000);
    } catch (e) {
        console.error("Failed to save settings", e);
        alert("Failed to save settings: " + (e as any).message);
    } finally {
        saving.value = false;
    }
};

onMounted(() => {
    fetchSettings();
});
</script>
