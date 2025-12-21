<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useAuthStore } from '../stores/auth';
import { ArrowLeftIcon, MapIcon, MailIcon } from 'tdesign-icons-vue-next';
import { Button, Avatar, Tag, Divider } from 'tdesign-vue-next';

const route = useRoute();
const router = useRouter();
const authStore = useAuthStore();

// Mock user data fetch
const userId = route.params.id as string;
const user = ref({
  id: userId,
  name: userId === 'me' ? (authStore.user?.name || 'Explorer') : 'Traveler',
  role: 'Architect of Voids',
  bio: 'Documenting the digital landscape, one commit at a time. Searching for the perfect abstraction.',
  location: 'Sector 7G',
  joined: 'Oct 2023',
  stats: {
    entries: 42,
    collections: 5,
    miles: 890
  }
});

const isMe = userId === 'me' || userId === authStore.user?.id;
</script>

<template>
  <div class="profile-container">
    <div class="profile-card">
      <header class="profile-header">
        <t-button variant="text" shape="circle" class="back-btn" @click="router.back()">
          <template #icon><arrow-left-icon /></template>
        </t-button>
        <div class="header-ornament">IDENTITY CARD</div>
      </header>

      <div class="profile-body">
        <div class="avatar-section">
          <div class="avatar-frame">
             <img :src="`https://api.dicebear.com/7.x/notionists/svg?seed=${user.name}`" alt="User" />
          </div>
          <div class="rank-badge">LVL. {{ Math.floor(user.stats.entries / 10) + 1 }}</div>
        </div>

        <div class="info-section">
          <h1 class="user-name">{{ user.name }}</h1>
          <div class="user-meta">
            <span class="role">{{ user.role }}</span>
            <span class="location"><map-icon /> {{ user.location }}</span>
          </div>

          <div class="bio-text">
            "{{ user.bio }}"
          </div>

          <div class="stats-row">
            <div class="stat-item">
              <span class="stat-val">{{ user.stats.entries }}</span>
              <span class="stat-label">Entries</span>
            </div>
            <div class="stat-divider"></div>
            <div class="stat-item">
              <span class="stat-val">{{ user.stats.collections }}</span>
              <span class="stat-label">Collections</span>
            </div>
            <div class="stat-divider"></div>
            <div class="stat-item">
              <span class="stat-val">{{ user.stats.miles }}</span>
              <span class="stat-label">Miles</span>
            </div>
          </div>
        </div>
      </div>

      <footer class="profile-footer" v-if="!isMe">
        <t-button theme="primary" variant="outline" class="action-btn">
          <mail-icon /> Send Signal
        </t-button>
      </footer>
    </div>
  </div>
</template>

<style scoped>
.profile-container {
  min-height: 100vh;
  display: flex;
  justify-content: center;
  align-items: center;
  background: rgba(0,0,0,0.6); /* Overlay effect if modal, or standalone page bg */
  padding: 20px;
}

.profile-card {
  width: 500px;
  background: #F5F1E6;
  background-image: url('https://www.transparenttextures.com/patterns/cream-paper.png');
  box-shadow: 0 10px 30px rgba(0,0,0,0.3);
  border-radius: 8px;
  overflow: hidden;
  position: relative;
  border: 1px solid #D7CCC8;
}

.profile-card::before {
  content: '';
  position: absolute;
  top: 10px;
  bottom: 10px;
  left: 10px;
  right: 10px;
  border: 2px double #D7CCC8;
  pointer-events: none;
}

.profile-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 24px 24px 0;
  position: relative;
  z-index: 2;
}

.back-btn {
  color: #5D4037;
}

.header-ornament {
  font-family: 'Inter', sans-serif;
  font-size: 10px;
  letter-spacing: 2px;
  color: #A1887F;
  border: 1px solid #A1887F;
  padding: 2px 8px;
  border-radius: 10px;
}

.profile-body {
  padding: 32px 40px;
  text-align: center;
  position: relative;
  z-index: 2;
}

.avatar-section {
  position: relative;
  display: inline-block;
  margin-bottom: 24px;
}

.avatar-frame {
  width: 120px;
  height: 120px;
  border-radius: 50%;
  border: 4px solid #fff;
  box-shadow: 0 4px 12px rgba(0,0,0,0.1);
  overflow: hidden;
  background: #EAE5D9;
}

.avatar-frame img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.rank-badge {
  position: absolute;
  bottom: 0;
  right: 0;
  background: #8B4513;
  color: #fff;
  font-family: 'Inter', sans-serif;
  font-size: 10px;
  font-weight: 700;
  padding: 4px 8px;
  border-radius: 10px;
  border: 2px solid #fff;
}

.user-name {
  font-family: 'Crimson Pro', serif;
  font-size: 2.5rem;
  color: #3E2723;
  margin: 0 0 8px;
  line-height: 1;
}

.user-meta {
  display: flex;
  justify-content: center;
  gap: 16px;
  font-size: 0.9rem;
  color: #8D6E63;
  margin-bottom: 24px;
}

.user-meta span {
  display: flex;
  align-items: center;
  gap: 4px;
}

.role {
  text-transform: uppercase;
  letter-spacing: 1px;
  font-weight: 600;
}

.bio-text {
  font-family: 'Caveat', cursive;
  font-size: 1.4rem;
  color: #5D4037;
  line-height: 1.4;
  margin-bottom: 32px;
}

.stats-row {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 24px;
  border-top: 1px solid #D7CCC8;
  border-bottom: 1px solid #D7CCC8;
  padding: 16px 0;
}

.stat-item {
  display: flex;
  flex-direction: column;
}

.stat-val {
  font-family: 'Crimson Pro', serif;
  font-size: 1.5rem;
  font-weight: 700;
  color: #3E2723;
  line-height: 1;
}

.stat-label {
  font-size: 0.7rem;
  text-transform: uppercase;
  letter-spacing: 1px;
  color: #A1887F;
  margin-top: 4px;
}

.stat-divider {
  width: 1px;
  height: 24px;
  background: #D7CCC8;
}

.profile-footer {
  padding: 0 40px 32px;
  text-align: center;
  position: relative;
  z-index: 2;
}

.action-btn {
  width: 100%;
  border-color: #8B4513;
  color: #8B4513;
}

.action-btn:hover {
  background: rgba(139, 69, 19, 0.05);
}
</style>

