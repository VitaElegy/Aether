<template>
    <div class="h-full flex flex-col gap-6 p-1 overflow-y-auto">
        <!-- Top Metrics -->
        <div class="grid grid-cols-4 gap-4">
            <div class="bg-white/50 border border-ash/20 p-4 rounded-xl flex items-center justify-between">
                <div>
                    <div class="text-[10px] font-bold uppercase tracking-widest text-ink/40">Total Findings</div>
                    <div class="text-2xl font-black font-serif text-ink">{{ metrics.total }}</div>
                </div>
                <div class="w-10 h-10 rounded-full bg-ash flex items-center justify-center text-ink/40">
                    <i class="ri-bug-line text-xl"></i>
                </div>
            </div>
            <div class="bg-red-50/50 border border-red-100 p-4 rounded-xl flex items-center justify-between">
                <div>
                    <div class="text-[10px] font-bold uppercase tracking-widest text-red-400">Critical</div>
                    <div class="text-2xl font-black font-serif text-red-600">{{ metrics.critical }}</div>
                </div>
                <div class="w-10 h-10 rounded-full bg-red-100 flex items-center justify-center text-red-500">
                    <i class="ri-alarm-warning-line text-xl"></i>
                </div>
            </div>
             <div class="bg-blue-50/50 border border-blue-100 p-4 rounded-xl flex items-center justify-between">
                <div>
                    <div class="text-[10px] font-bold uppercase tracking-widest text-blue-400">Triage Pending</div>
                    <div class="text-2xl font-black font-serif text-blue-600">{{ metrics.triage }}</div>
                </div>
                <div class="w-10 h-10 rounded-full bg-blue-100 flex items-center justify-center text-blue-500">
                    <i class="ri-inbox-archive-line text-xl"></i>
                </div>
            </div>
            <div class="bg-green-50/50 border border-green-100 p-4 rounded-xl flex items-center justify-between">
                <div>
                    <div class="text-[10px] font-bold uppercase tracking-widest text-green-400">Fixed</div>
                    <div class="text-2xl font-black font-serif text-green-600">{{ metrics.fixed }}</div>
                </div>
                <div class="w-10 h-10 rounded-full bg-green-100 flex items-center justify-center text-green-500">
                    <i class="ri-checkbox-circle-line text-xl"></i>
                </div>
            </div>
        </div>

        <div class="flex-1 grid grid-cols-1 lg:grid-cols-3 gap-6 min-h-0">
            <!-- Left: Module Cards (War Room Status) -->
            <div class="lg:col-span-1 flex flex-col gap-4">
                <div class="flex items-center justify-between">
                    <h3 class="text-lg font-bold font-serif text-ink">Component Status</h3>
                    <button class="text-xs font-bold text-accent hover:underline">+ Add Module</button>
                </div>
                
                <div class="space-y-3">
                    <div v-for="mod in modules" :key="mod.name" class="bg-white border border-ash/20 p-4 rounded-xl hover:border-accent/40 transition-colors group cursor-pointer">
                        <div class="flex justify-between items-start mb-2">
                            <h4 class="font-bold text-ink">{{ mod.name }}</h4>
                            <span 
                                class="text-[10px] px-2 py-0.5 rounded-full font-bold uppercase tracking-wider"
                                :class="getStatusClass(mod.status)"
                            >
                                {{ mod.status }}
                            </span>
                        </div>
                        <div class="w-full bg-ash/30 h-1.5 rounded-full overflow-hidden mb-3">
                            <div class="h-full bg-accent" :style="{width: mod.progress + '%'}"></div>
                        </div>
                        <div class="flex items-center justify-between text-xs text-ink/40">
                            <span class="flex items-center gap-1"><i class="ri-bug-fill"></i> {{ mod.bugs }} Bugs</span>
                            <span class="font-mono">Last audit: {{ mod.lastAudit }}</span>
                        </div>
                    </div>
                </div>
            </div>

            <!-- Right: Code Heatmap -->
            <div class="lg:col-span-2 flex flex-col bg-paper-2 rounded-xl border border-ash/20 overflow-hidden">
                <div class="p-4 border-b border-ash/20 flex justify-between items-center bg-white/50">
                    <div class="flex items-center gap-2">
                        <i class="ri-fire-line text-orange-500"></i>
                        <h3 class="font-bold font-serif text-ink">Code Heatmap</h3>
                    </div>
                    <div class="flex gap-2 text-[10px] font-bold uppercase tracking-wider text-ink/40">
                        <span class="flex items-center gap-1"><div class="w-2 h-2 bg-green-400 rounded-sm"></div> Safe</span>
                        <span class="flex items-center gap-1"><div class="w-2 h-2 bg-yellow-400 rounded-sm"></div> Risk</span>
                        <span class="flex items-center gap-1"><div class="w-2 h-2 bg-red-400 rounded-sm"></div> Critical</span>
                    </div>
                </div>
                
                <!-- Mock File Tree Heatmap -->
                <div class="flex-1 p-4 overflow-y-auto font-mono text-sm space-y-1">
                    <div v-for="node in fileTree" :key="node.path" class="flex items-center gap-2 hover:bg-black/5 p-1 rounded cursor-pointer group">
                        <!-- Indentation -->
                        <div :style="{width: (node.level * 20) + 'px'}"></div>
                        
                        <!-- Icon -->
                        <i v-if="node.type === 'folder'" class="ri-folder-3-fill text-yellow-500/80"></i>
                        <i v-else class="ri-file-code-line text-ink/40"></i>
                        
                        <!-- Name -->
                        <span class="text-ink/80 group-hover:text-ink">{{ node.name }}</span>
                        
                        <!-- Heatmap Indicator -->
                        <div class="ml-auto flex items-center gap-3">
                            <div v-if="node.vulns > 0" class="h-1.5 rounded-full min-w-[60px]" :class="getHeatmapColor(node.vulns)"></div>
                            <span v-if="node.vulns > 0" class="text-xs font-bold" :class="getTextColor(node.vulns)">{{ node.vulns }}</span>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { vrkbApi } from '@/api/vrkb';
import { useVrkbStore } from '@/stores/vrkb';

const store = useVrkbStore();
const modules = ref<any[]>([]);
const fileTree = ref<any[]>([]);
const metrics = ref<any>({ total: 0, critical: 0, triage: 0, fixed: 0 });

onMounted(async () => {
    if (store.currentProject) {
        try {
            const stats = await vrkbApi.getProjectStats(store.currentProject.id);
            metrics.value = stats.metrics;
            modules.value = stats.modules;
            fileTree.value = stats.heatmap;
        } catch (e) {
            console.error("Failed to load stats", e);
        }
    }
});

const getStatusClass = (status: string) => {
    switch(status) {
        case 'Fuzzing': return 'bg-purple-100 text-purple-600';
        case 'Audit': return 'bg-blue-100 text-blue-600';
        case 'Fixed': return 'bg-green-100 text-green-600';
        default: return 'bg-ash text-ink/40';
    }
};

const getHeatmapColor = (count: number) => {
    if (count >= 5) return 'bg-red-500 w-24';
    if (count >= 3) return 'bg-orange-400 w-16';
    return 'bg-yellow-400 w-8';
};

const getTextColor = (count: number) => {
    if (count >= 5) return 'text-red-600';
    if (count >= 3) return 'text-orange-500';
    return 'text-yellow-600';
};
</script>
