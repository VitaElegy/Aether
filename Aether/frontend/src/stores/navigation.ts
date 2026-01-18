import { defineStore } from 'pinia';
import { ref } from 'vue';

export const useNavigationStore = defineStore('navigation', () => {
    // State to track if a module has taken over a section
    const hasCustomCenter = ref(false);
    const hasCustomRight = ref(false);

    // Actions to register/unregister
    // Components should call these on mount/unmount if they use the Teleport portals
    function setCustomCenter(active: boolean) {
        hasCustomCenter.value = active;
    }

    function setCustomRight(active: boolean) {
        hasCustomRight.value = active;
    }

    // Reset all on module switch usually handled by unmounts, 
    // but a global reset is useful for Router guards.
    function reset() {
        hasCustomCenter.value = false;
        hasCustomRight.value = false;
    }

    return {
        hasCustomCenter,
        hasCustomRight,
        setCustomCenter,
        setCustomRight,
        reset
    };
});
