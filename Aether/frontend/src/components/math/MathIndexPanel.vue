<script setup lang="ts">
import { ref } from 'vue';

// Mock Data for now
const chapters = [
    { id: '1', title: '1. Introduction to Groups', active: true },
    { id: '2', title: '2. Subgroups and Cosets', active: false },
    { id: '3', title: '3. Homomorphisms', active: false },
    { id: '4', title: '4. Group Actions', active: false },
    { id: '5', title: '5. The Sylow Theorems', active: false },
];
</script>

<template>
    <div class="h-full flex flex-col p-8 font-serif border-r border-ink/5">
        <h3 class="text-xs font-black uppercase tracking-[0.2em] text-ink/40 mb-8 pl-4">Table of Contents</h3>
        
        <nav class="flex flex-col gap-1">
            <button 
                v-for="chapter in chapters" 
                :key="chapter.id"
                class="group flex items-baseline gap-4 px-4 py-3 text-left transition-all hover:bg-ink/5 rounded-sm"
                :class="{ 'opacity-100': chapter.active, 'opacity-60 hover:opacity-100': !chapter.active }"
            >
                <span class="text-xs font-mono text-ink/30 group-hover:text-accent transition-colors">{{ chapter.id }}</span>
                <span class="text-sm font-serif text-ink group-hover:translate-x-1 transition-transform duration-300">{{ chapter.title }}</span>
            </button>
        </nav>

        <div class="mt-auto pt-8 border-t border-ink/5">
             <button class="flex items-center gap-3 text-ink/40 hover:text-ink text-xs uppercase tracking-widest transition-colors group">
                <i class="ri-settings-line group-hover:rotate-90 transition-transform"></i>
                <span>Visual Settings</span>
            </button>
        </div>
    </div>
</template>
