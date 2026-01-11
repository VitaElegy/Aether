<template>
  <div class="presentation-container" tabindex="0" @keydown="handleKeydown" ref="container">
    <Transition :name="transitionName" mode="out-in">
      <component
        :is="currentSlideComponent"
        :key="currentSlideIndex"
        :page-number="currentSlideIndex + 1"
        :total-pages="slides.length"
      />
    </Transition>

    <div class="controls">
      <div class="control-btn" @click="prevSlide" :class="{ disabled: currentSlideIndex === 0 }">
        <t-icon name="chevron-left" />
      </div>
      <div class="control-btn" @click="nextSlide" :class="{ disabled: currentSlideIndex === slides.length - 1 }">
        <t-icon name="chevron-right" />
      </div>
    </div>
    
    <div class="exit-btn" @click="exitPresentation">
        <t-icon name="close" />
    </div>

    <div class="page-indicator">
        {{ currentSlideIndex + 1 }} / {{ slides.length }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, nextTick } from 'vue'
import { useRouter } from 'vue-router'
import SlideBase from './SlideBase.vue'
// Import slides
import SlideCover from './slides/SlideCover.vue'
import SlideIntro from './slides/SlideIntro.vue'
import SlideConcepts from './slides/SlideConcepts.vue'
import SlideTechStack from './slides/SlideTechStack.vue'
import SlideFeatures from './slides/SlideFeatures.vue'
import SlideAesthetics from './slides/SlideAesthetics.vue'
import SlideDesign from './slides/SlideDesign.vue'
import SlideAlgorithms from './slides/SlideAlgorithms.vue'
import SlideDevProcess from './slides/SlideDevProcess.vue'
import SlideRoadmap from './slides/SlideRoadmap.vue'
import SlideAuthors from './slides/SlideAuthors.vue'
import SlideClosing from './slides/SlideClosing.vue'

const router = useRouter()
const container = ref<HTMLElement | null>(null)

// Register slides here
const slides = [
  SlideCover,
  SlideIntro,
  SlideConcepts,
  SlideFeatures,
  SlideAesthetics,
  SlideTechStack,
  SlideDesign,
  SlideAlgorithms,
  SlideDevProcess,
  SlideRoadmap,
  SlideAuthors,
  SlideClosing
]

const currentSlideIndex = ref(0)
const direction = ref('next')

const currentSlideComponent = computed(() => {
  return slides[currentSlideIndex.value] || SlideBase
})

const transitionName = computed(() => {
  return direction.value === 'next' ? 'slide-left' : 'slide-right'
})

const nextSlide = () => {
  if (currentSlideIndex.value < slides.length - 1) {
    direction.value = 'next'
    currentSlideIndex.value++
  }
}

const prevSlide = () => {
  if (currentSlideIndex.value > 0) {
    direction.value = 'prev'
    currentSlideIndex.value--
  }
}

const exitPresentation = () => {
  router.push('/self-space')
}

const handleKeydown = (e: KeyboardEvent) => {
  if (e.key === 'ArrowRight' || e.key === 'Space') {
    nextSlide()
  } else if (e.key === 'ArrowLeft') {
    prevSlide()
  } else if (e.key === 'Escape') {
    exitPresentation()
  }
}

onMounted(() => {
  nextTick(() => {
    container.value?.focus()
  })
})
</script>

<style scoped>
.presentation-container {
  width: 100vw;
  height: 100vh;
  position: fixed;
  top: 0;
  left: 0;
  z-index: 2000; /* High z-index to cover everything */
  background: black;
  outline: none;
}

.controls {
  position: absolute;
  bottom: 2rem;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  gap: 1rem;
  z-index: 10;
  opacity: 0;
  transition: opacity 0.3s;
}

.presentation-container:hover .controls {
  opacity: 1;
}

.control-btn {
  width: 3rem;
  height: 3rem;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(10px);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  color: white;
  transition: all 0.2s;
}

.control-btn:hover:not(.disabled) {
  background: rgba(255, 255, 255, 0.2);
  transform: scale(1.1);
}

.control-btn.disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.exit-btn {
    position: absolute;
    top: 2rem;
    right: 2rem;
    width: 3rem;
    height: 3rem;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.05);
    backdrop-filter: blur(5px);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    color: rgba(255, 255, 255, 0.7);
    z-index: 10;
    transition: all 0.2s;
}

.exit-btn:hover {
    background: rgba(255, 255, 255, 0.2);
    color: white;
}

/* Transitions */
.slide-left-enter-active,
.slide-left-leave-active,
.slide-right-enter-active,
.slide-right-leave-active {
  transition: all 0.5s cubic-bezier(0.25, 0.1, 0.25, 1);
}

.slide-left-enter-from {
  transform: translateX(100%);
  opacity: 0;
}
.slide-left-leave-to {
  transform: translateX(-20%);
  opacity: 0;
}

.slide-right-leave-to {
  transform: translateX(20%);
  opacity: 0;
}

.page-indicator {
    position: absolute;
    bottom: 2rem;
    right: 3rem;
    color: rgba(255, 255, 255, 0.4);
    font-size: 1.2rem;
    font-family: monospace;
    z-index: 20;
    pointer-events: none;
}
</style>
