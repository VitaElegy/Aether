<template>
  <SlideBase>
    <div class="content">
      <h2 class="section-title">未来路线图</h2>
      
      <div class="timeline">
        <!-- Phase 1: Completed -->
        <div class="milestone complete">
            <div class="dot"></div>
            <div class="date">Phase 1 (Completed)</div>
            <div class="desc">Core Kernel: Hexagonal Arch, ReBAC, Versioning</div>
        </div>
        
        <!-- Phase 2: Active -->
        <div class="milestone active">
            <div class="dot"></div>
            <div class="date">Phase 2 (Current)</div>
            <div class="desc">Self Space: English Engine, FSRS, Local Dictionaries</div>
        </div>

        <!-- Phase 3: Planned -->
        <div class="milestone feature">
            <div class="dot"></div>
            <div class="date">Phase 3 (Next Step)</div>
            <div class="desc">Knowledge Graph & Structured Memos</div>
        </div>

        <!-- Phase 4: Future -->
        <div class="milestone future">
            <div class="dot"></div>
            <div class="date">Phase 4 (Future)</div>
            <div class="desc">Ecosystem: Public Sharing & Advanced Search</div>
        </div>
      </div>

    </div>
  </SlideBase>
</template>

<script setup lang="ts">
import SlideBase from '../SlideBase.vue'
</script>

<style scoped>
.content { text-align: center; }
.section-title {
  font-size: 3rem;
  margin-bottom: 4rem;
  background: linear-gradient(to right, #38bdf8, #818cf8);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.timeline {
    position: relative;
    max-width: 800px;
    margin: 0 auto;
    padding-left: 2rem;
    border-left: 2px solid rgba(255,255,255,0.1);
}

.milestone {
    position: relative;
    margin-bottom: 3rem;
    text-align: left;
    padding-left: 2rem;
    opacity: 0.5;
}

.milestone.complete, .milestone.active {
    opacity: 1;
}

.dot {
    position: absolute;
    left: -2.6rem;
    top: 0.2rem;
    width: 1rem;
    height: 1rem;
    border-radius: 50%;
    background: #333;
    border: 2px solid #555;
}

.milestone.complete .dot {
    background: #4ade80;
    border-color: #4ade80;
}

.milestone.active .dot {
    background: #38bdf8;
    border-color: #38bdf8;
    box-shadow: 0 0 10px #38bdf8;
}

.date {
    font-size: 0.9rem;
    text-transform: uppercase;
    letter-spacing: 2px;
    color: rgba(255,255,255,0.5);
    margin-bottom: 0.5rem;
}

.desc {
    font-size: 1.5rem;
    font-weight: 500;
}
</style>
