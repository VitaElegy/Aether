<script setup lang="ts">
import { ref } from 'vue';
import BlockWrapper from '../components/blocks/BlockWrapper.vue';

// Mock Data representing what we'd get from the Backend
const blocks = ref([
    {
        id: '1',
        type: 'axiom',
        revision: 1,
        payload: {
            label: 'ZFC-1',
            content: 'Extensionality: $\\forall x \\forall y (\\forall z (z \\in x \\iff z \\in y) \\implies x = y)$'
        }
    },
    {
        id: '2',
        type: 'definition',
        revision: 1,
        payload: {
            term: 'Subset',
            content: 'A set $A$ is a subset of $B$ ($A \\subseteq B$) if every element of $A$ is in $B$.'
        }
    },
    {
        id: '3',
        type: 'theorem',
        revision: 1,
        payload: {
            label: 'Theorem 1.1',
            content: 'Every set is a subset of itself.'
        }
    },
    {
        id: '4',
        type: 'proof',
        revision: 1,
        payload: {
            steps: '1. Let $x$ be an arbitrary element of $A$. \n2. Then trivially $x \\in A$. \n3. Therefore, $A \\subseteq A$.',
            qcd_symbol: 'Q.E.D.'
        }
    },
    // TEST CASE: Quarantine (Missing 'content' which might cause render error or schema violation)
    // Actually, our Schema requires content. If payload is wrong, the component might throw.
    // Let's induce a crash by passing a type that exists but payload that causes logic error,
    // Or just a completely unknown type to test the Registry Fallback/Error.
    {
        id: '5',
        type: 'axiom',
        revision: 1,
        payload: {
             // Missing 'content', 'marked' might throw or return undefined
             label: 'Broken Axiom'
        }
    },
    {
        id: '6',
        type: 'unknown_type',
        revision: 1,
        payload: {}
    }
]);
</script>

<template>
    <div class="min-h-screen bg-gray-50 p-12">
        <div class="max-w-3xl mx-auto bg-white shadow-lg rounded-xl p-8">
            <h1 class="text-3xl font-serif font-bold text-gray-900 border-b pb-4 mb-8">
                Math KB V2 Architecture Demo
            </h1>
            
            <div class="space-y-2">
                <BlockWrapper 
                    v-for="block in blocks" 
                    :key="block.id" 
                    :block="block" 
                />
            </div>
        </div>
    </div>
</template>
