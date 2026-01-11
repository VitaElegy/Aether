<template>
  <SlideBase>
    <div class="content">
      <h1 class="thank-you">谢谢</h1>
      <div class="qa">Q & A 环节</div>
      <div class="signature">Designed by Aether Team</div>
    </div>
  </SlideBase>
</template>

<script setup lang="ts">
import SlideBase from '../SlideBase.vue'
</script>

<style scoped>
.content { text-align: center; }

.thank-you {
  font-size: 8rem;
  font-weight: 900;
  margin: 0;
  background: linear-gradient(135deg, #fff 0%, #cbd5e1 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  animation: float 6s ease-in-out infinite;
}

.qa {
    font-size: 3rem;
    margin-top: 2rem;
    color: rgba(255,255,255,0.6);
}

.signature {
    margin-top: 5rem;
    font-size: 1rem;
    opacity: 0.4;
    letter-spacing: 2px;
}

@keyframes float {
    0%, 100% { transform: translateY(0); }
    50% { transform: translateY(-20px); }
}
</style>
