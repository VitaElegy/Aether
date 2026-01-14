<script setup lang="ts">
import { ref, onMounted } from 'vue';

const theorems = [
    {
        title: "Bolzano-Weierstrass Theorem",
        statement: "Every bounded sequence in ℝⁿ has a convergent subsequence."
    },
    {
        title: "Heine-Borel Theorem",
        statement: "A subset of Euclidean space is compact if and only if it is closed and bounded."
    },
    {
        title: "Fundamental Theorem of Algebra",
        statement: "Every non-zero, single-variable, degree n polynomial with complex coefficients has, counted with multiplicity, exactly n complex roots."
    },
    {
        title: "Euler's Identity",
        statement: "e^(iπ) + 1 = 0"
    }
];

const current = ref(theorems[0]);
const isAnimating = ref(false);

const shuffle = () => {
    isAnimating.value = true;
    setTimeout(() => {
        let idx;
        do {
             idx = Math.floor(Math.random() * theorems.length);
        } while (theorems[idx] === current.value);
        current.value = theorems[idx];
        isAnimating.value = false;
    }, 300); // Wait for exit animation
};

onMounted(() => {
    // Initial shuffle
    shuffle();
});
</script>

<template>
    <div class="w-full h-full flex flex-col relative group">
        <div class="flex items-center justify-between mb-2">
            <span class="text-[10px] font-black uppercase tracking-widest text-ink/40">Theorem of the Day</span>
            <button @click="shuffle" class="text-ink/20 hover:text-accent transition-colors">
                <i class="ri-refresh-line"></i>
            </button>
        </div>
        
        <div class="flex-1 bg-paper border border-ink/5 rounded-xl p-6 flex flex-col items-center justify-center text-center relative overflow-hidden hover:border-accent/20 transition-colors">
            <!-- Decorative Background -->
            <div class="absolute inset-0 bg-accent/5 opacity-0 group-hover:opacity-100 transition-opacity"></div>
            
            <Transition name="fade-scale" mode="out-in">
                <div :key="current.title" class="relative z-10">
                    <h3 class="text-lg font-black font-serif mb-4">{{ current.title }}</h3>
                    <p class="text-sm font-mono text-ink/70 italic">
                        "{{ current.statement }}"
                    </p>
                </div>
            </Transition>
        </div>
    </div>
</template>

<style scoped>
.fade-scale-enter-active,
.fade-scale-leave-active {
  transition: all 0.3s ease;
}

.fade-scale-enter-from {
  opacity: 0;
  transform: scale(0.95);
}

.fade-scale-leave-to {
  opacity: 0;
  transform: scale(1.05);
}
</style>
