<template>
  <SlideBase>
    <div class="authors-content">
      <h2 class="title">Collaborators</h2>
      
      <div class="cards-container">
        <!-- Elegy -->
        <div class="author-card human">
          <div class="avatar-wrapper">
             <img :src="elegyAvatar" alt="Elegy" class="avatar-image" />
          </div>
          <div class="info">
            <h3 class="name">Elegy</h3>
            <p class="role">Project Lead & Architect</p>
            <p class="desc">Vision, Core Design, Implementation</p>
          </div>
        </div>

        <!-- Gemini3Pro -->
        <div class="author-card ai">
          <div class="avatar-wrapper">
             <t-icon name="code" class="avatar-icon" /> <!-- Using a generic tech/code icon -->
          </div>
          <div class="info">
            <h3 class="name">Gemini 3 Pro</h3>
            <p class="role">AI Co-Pilot</p>
            <p class="desc">Code Generation, Debugging, Documentation</p>
          </div>
        </div>
      </div>
    </div>
  </SlideBase>
</template>

<script setup lang="ts">
import SlideBase from '../SlideBase.vue'
import elegyAvatar from '@/assets/avatars/elegy_avatar.jpg'
</script>

<style scoped>
.authors-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  width: 100%;
}

.title {
  font-size: 3rem;
  font-weight: 700;
  margin-bottom: 4rem;
  background: linear-gradient(135deg, #fff 0%, #a5b4fc 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  animation: fadeInDown 1s ease-out;
}

.cards-container {
  display: flex;
  gap: 4rem;
  perspective: 1000px;
}

.author-card {
  width: 300px;
  background: rgba(255, 255, 255, 0.05);
  backdrop-filter: blur(10px);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 20px;
  padding: 3rem 2rem;
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  transition: all 0.4s cubic-bezier(0.175, 0.885, 0.32, 1.275);
  animation: fadeInUp 1s ease-out backwards;
}

.author-card:hover {
  transform: translateY(-10px) scale(1.02);
  background: rgba(255, 255, 255, 0.1);
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.4);
}

.author-card.human {
  animation-delay: 0.2s;
}

.author-card.ai {
  animation-delay: 0.4s;
}

.avatar-wrapper {
  width: 100px;
  height: 100px;
  border-radius: 50%;
  background: linear-gradient(135deg, rgba(255,255,255,0.1), rgba(255,255,255,0.05));
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 2rem;
  box-shadow: 0 8px 16px rgba(0,0,0,0.2);
}

.avatar-icon {
  font-size: 3rem;
  color: rgba(255, 255, 255, 0.9);
}

.avatar-image {
  width: 100%;
  height: 100%;
  border-radius: 50%;
  object-fit: cover;
}

.info .name {
  font-size: 1.8rem;
  font-weight: 600;
  color: white;
  margin: 0 0 0.5rem 0;
}

.info .role {
  font-size: 1rem;
  color: #a5b4fc;
  font-weight: 500;
  margin: 0 0 1rem 0;
  text-transform: uppercase;
  letter-spacing: 1px;
}

.info .desc {
  font-size: 0.9rem;
  color: rgba(255, 255, 255, 0.6);
  line-height: 1.5;
  margin: 0;
}

@keyframes fadeInDown {
  from { opacity: 0; transform: translateY(-30px); }
  to { opacity: 1; transform: translateY(0); }
}

@keyframes fadeInUp {
  from { opacity: 0; transform: translateY(30px); }
  to { opacity: 1; transform: translateY(0); }
}
</style>
