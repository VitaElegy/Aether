<template>
    <div class="p-8 max-w-7xl mx-auto animate-fade-in">
        <div class="mb-8">
            <h1 class="text-2xl font-bold text-ink font-serif mb-1">System Logs</h1>
            <p class="text-ink/60 text-sm">Review recent system activity and git history.</p>
        </div>

        <div v-if="loading" class="space-y-4">
            <div v-for="i in 5" :key="i" class="h-16 bg-ash/10 rounded animate-pulse"></div>
        </div>
        
        <div v-else class="bg-surface rounded-xl border border-ink/5 overflow-hidden">
            <div class="overflow-x-auto">
                <table class="w-full text-left text-sm">
                    <thead class="bg-ash/5 text-ink/40 text-[10px] uppercase font-bold tracking-wider">
                        <tr>
                            <th class="px-6 py-3 font-medium">Hash</th>
                            <th class="px-6 py-3 font-medium">Message</th>
                            <th class="px-6 py-3 font-medium">Author</th>
                            <th class="px-6 py-3 font-medium">Date</th>
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-ink/5">
                        <tr v-for="commit in commits" :key="commit.hash" class="hover:bg-ash/5 transition-colors">
                            <td class="px-6 py-4 font-mono text-xs text-accent">{{ commit.short_hash }}</td>
                            <td class="px-6 py-4 text-ink font-medium">{{ commit.message }}</td>
                            <td class="px-6 py-4 text-ink/60">{{ commit.author }}</td>
                            <td class="px-6 py-4 text-ink/40 whitespace-nowrap">{{ new Date(commit.date).toLocaleString() }}</td>
                        </tr>
                    </tbody>
                </table>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { systemApi, type GitCommit } from '@/api/system';
import { MessagePlugin } from 'tdesign-vue-next';

const commits = ref<GitCommit[]>([]);
const loading = ref(true);

const fetchLogs = async () => {
    loading.value = true;
    try {
        const { data } = await systemApi.getGitLog();
        commits.value = data;
    } catch (e) {
        MessagePlugin.error('Failed to load system logs');
    } finally {
        loading.value = false;
    }
};

onMounted(fetchLogs);
</script>
