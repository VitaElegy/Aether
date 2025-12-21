<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { useAuthStore } from '../stores/auth';
import axios from 'axios';
import {
  UserCircleIcon,
  SearchIcon,
  AddIcon,
  CalendarIcon,
  BookmarkIcon,
  FilterIcon
} from 'tdesign-icons-vue-next';
import { Button, Tag, Input, Row, Col } from 'tdesign-vue-next';
import DynamicRenderer from '../components/DynamicRenderer.vue';

const router = useRouter();
const authStore = useAuthStore();

const handleLogout = () => {
  authStore.logout();
  router.push('/login');
};

const posts = ref<any[]>([]);
const subjects = ref(['Math', 'Science', 'History', 'Literature', 'Coding']);

onMounted(async () => {
  try {
    const res = await axios.get('/api/content');
    posts.value = res.data.map((p: any) => ({
      id: p.id,
      type: p.body.type,
      title: p.title,
      author: 'Student',
      date: new Date(p.created_at).toLocaleDateString('en-US', { weekday: 'short', month: 'short', day: 'numeric' }),
      tags: p.tags,
      // Wrap content for renderer
      data: p.body.type === 'Markdown' ? { content: p.body.data } : p.body.data,
      // Random "Grade" or Status sticker for fun
      sticker: Math.random() > 0.7 ? 'A+' : (Math.random() > 0.5 ? 'Good' : null)
    }));
  } catch (err) {
    console.error('Failed to fetch posts', err);
  }
});
</script>

<template>
  <div class="planner-container">
    <!-- Spiral Binding Effect -->
    <div class="spiral-binding">
      <div v-for="n in 20" :key="n" class="ring"></div>
    </div>

    <!-- Sidebar: Class Schedule / Subjects -->
    <aside class="planner-sidebar">
      <div class="profile-section">
        <div class="avatar-circle">
          <span>{{ authStore.user?.name?.[0] || 'S' }}</span>
        </div>
        <div class="student-info">
          <h3>{{ authStore.user?.name || 'Student' }}</h3>
          <span class="grade-level">Grade 12 • AP CS</span>
        </div>
      </div>

      <nav class="subject-nav">
        <div class="nav-header">SUBJECTS</div>
        <div v-for="sub in subjects" :key="sub" class="subject-tab">
          <span class="color-dot"></span>
          {{ sub }}
        </div>
        <div class="subject-tab active">
          <span class="color-dot active"></span>
          All Notes
        </div>
      </nav>

      <div class="sidebar-footer">
        <button class="logout-btn" @click="handleLogout">
          Sign Out
        </button>
      </div>
    </aside>

    <!-- Main Content: Notebook Pages -->
    <main class="planner-content">
      <header class="planner-header">
        <div class="date-display">
          <calendar-icon />
          <span>{{ new Date().toLocaleDateString('en-US', { weekday: 'long', month: 'long', day: 'numeric' }) }}</span>
        </div>
        <div class="header-actions">
          <div class="search-wrapper">
            <search-icon class="search-icon" />
            <input type="text" placeholder="Find notes..." />
          </div>
          <t-button theme="primary" shape="circle" @click="router.push('/editor')" class="add-btn">
            <template #icon><add-icon /></template>
          </t-button>
        </div>
      </header>

      <div class="notebook-page">
        <div class="page-lines"></div>

        <div class="notes-grid">
          <div v-for="post in posts" :key="post.id" class="sticky-note-card">
            <!-- Tape effect -->
            <div class="tape"></div>

            <div class="note-header">
              <span class="note-date">{{ post.date }}</span>
              <div v-if="post.sticker" class="sticker">{{ post.sticker }}</div>
            </div>

            <h3 class="note-title">{{ post.title }}</h3>

            <div class="note-preview">
              <DynamicRenderer :type="post.type" :data="post.data" />
            </div>

            <div class="note-footer">
              <div class="tags">
                <span v-for="tag in post.tags" :key="tag" class="handwritten-tag">#{{ tag }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </main>
  </div>
</template>

<style scoped>
.planner-container {
  display: flex;
  min-height: 100vh;
  background: #333; /* Desk background */
  font-family: 'Patrick Hand', 'Comic Sans MS', sans-serif; /* Handwriting font fallback */
  padding: 20px;
  gap: 0;
  overflow: hidden;
}

/* Spiral Binding */
.spiral-binding {
  width: 40px;
  background: transparent;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 15px;
  padding-top: 40px;
  z-index: 10;
}

.ring {
  width: 30px;
  height: 12px;
  background: #C0C0C0;
  border-radius: 6px;
  box-shadow: 1px 1px 3px rgba(0,0,0,0.5);
  transform: rotate(-5deg);
}

/* Sidebar (Left Page) */
.planner-sidebar {
  width: 260px;
  background: #FDFBF7; /* Cream paper */
  border-radius: 12px 0 0 12px;
  padding: 32px 24px;
  display: flex;
  flex-direction: column;
  box-shadow: -5px 0 15px rgba(0,0,0,0.2);
  position: relative;
}

/* Texture overlay */
.planner-sidebar::before, .notebook-page::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-image: url('https://www.transparenttextures.com/patterns/paper.png'); /* Subtle texture if available */
  opacity: 0.5;
  pointer-events: none;
}

.profile-section {
  text-align: center;
  margin-bottom: 40px;
  border-bottom: 2px dashed #E5E7EB;
  padding-bottom: 24px;
}

.avatar-circle {
  width: 80px;
  height: 80px;
  background: #DBEAFE;
  border-radius: 50%;
  margin: 0 auto 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 32px;
  color: #1E40AF;
  border: 3px solid #fff;
  box-shadow: 0 4px 6px rgba(0,0,0,0.1);
}

.student-info h3 {
  margin: 0;
  font-family: 'Georgia', serif;
  color: #1F2937;
}

.grade-level {
  font-size: 12px;
  color: #6B7280;
  text-transform: uppercase;
  letter-spacing: 1px;
}

.subject-nav {
  flex: 1;
}

.nav-header {
  font-size: 12px;
  color: #9CA3AF;
  margin-bottom: 12px;
  letter-spacing: 1px;
  font-weight: 700;
}

.subject-tab {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  margin-bottom: 8px;
  border-radius: 8px;
  cursor: pointer;
  transition: background 0.2s;
  font-weight: 500;
  color: #4B5563;
}

.subject-tab:hover {
  background: #F3F4F6;
}

.subject-tab.active {
  background: #EFF6FF;
  color: #1E40AF;
}

.color-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: #D1D5DB;
}

.color-dot.active {
  background: #3B82F6;
}

.logout-btn {
  width: 100%;
  padding: 10px;
  border: 1px solid #E5E7EB;
  background: #fff;
  border-radius: 8px;
  cursor: pointer;
  color: #EF4444;
  font-weight: 600;
  transition: all 0.2s;
}

.logout-btn:hover {
  background: #FEF2F2;
}

/* Main Content (Right Page) */
.planner-content {
  flex: 1;
  background: #fff;
  border-radius: 0 12px 12px 0;
  padding: 0; /* No padding on container, handled inside */
  display: flex;
  flex-direction: column;
  position: relative;
  box-shadow: 5px 0 15px rgba(0,0,0,0.2);
}

.planner-header {
  height: 80px;
  border-bottom: 2px solid #E2E8F0; /* Red header line */
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 32px;
  background: #fff;
  border-radius: 0 12px 0 0;
}

.date-display {
  display: flex;
  align-items: center;
  gap: 8px;
  color: #6B7280;
  font-weight: 600;
  font-size: 18px;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 16px;
}

.search-wrapper {
  background: #F3F4F6;
  border-radius: 20px;
  padding: 6px 16px;
  display: flex;
  align-items: center;
  gap: 8px;
}

.search-wrapper input {
  border: none;
  background: transparent;
  outline: none;
  font-family: inherit;
  width: 150px;
}

.notebook-page {
  flex: 1;
  position: relative;
  background:
    linear-gradient(#E5E7EB 1px, transparent 1px);
  background-size: 100% 32px; /* Lined paper spacing */
  padding: 32px;
  overflow-y: auto;
}

.page-lines {
  position: absolute;
  top: 0;
  bottom: 0;
  left: 40px;
  width: 2px;
  background: #FECACA; /* Margin line (Red) */
  z-index: 0;
}

/* Notes Grid */
.notes-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 32px;
  padding-left: 30px; /* Offset for margin line */
  position: relative;
  z-index: 1;
}

.sticky-note-card {
  background: #FEF3C7; /* Post-it yellow default */
  padding: 24px;
  box-shadow: 2px 4px 8px rgba(0,0,0,0.1);
  position: relative;
  transform: rotate(-1deg);
  transition: transform 0.2s;
  font-family: 'Comic Sans MS', 'Chalkboard SE', sans-serif; /* Handwritten feel */
}

.sticky-note-card:hover {
  transform: scale(1.02) rotate(0deg);
  z-index: 10;
}

.sticky-note-card:nth-child(2n) {
  background: #E0F2FE; /* Blue note */
  transform: rotate(1deg);
}

.sticky-note-card:nth-child(3n) {
  background: #DCFCE7; /* Green note */
  transform: rotate(-2deg);
}

/* Tape Effect */
.tape {
  position: absolute;
  top: -10px;
  left: 50%;
  transform: translateX(-50%);
  width: 80px;
  height: 24px;
  background: rgba(255,255,255,0.4);
  box-shadow: 0 1px 2px rgba(0,0,0,0.1);
  transform: translateX(-50%) rotate(-2deg);
}

.note-header {
  display: flex;
  justify-content: space-between;
  margin-bottom: 8px;
}

.note-date {
  font-size: 12px;
  color: rgba(0,0,0,0.5);
}

.sticker {
  font-size: 24px;
  font-weight: 900;
  color: #EF4444;
  border: 2px solid #EF4444;
  border-radius: 50%;
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  transform: rotate(-15deg);
  opacity: 0.8;
}

.note-title {
  margin: 0 0 12px 0;
  font-size: 18px;
  color: #1F2937;
  border-bottom: 1px dashed rgba(0,0,0,0.2);
  padding-bottom: 4px;
}

.note-preview {
  font-size: 14px;
  line-height: 1.5;
  color: #374151;
  max-height: 150px;
  overflow: hidden;
  margin-bottom: 16px;
}

.handwritten-tag {
  display: inline-block;
  font-size: 12px;
  color: #4B5563;
  margin-right: 8px;
}

.handwritten-tag::before {
  content: '#';
  color: #9CA3AF;
}

/* Font Import (Optional, using web safe defaults mostly) */
@import url('https://fonts.googleapis.com/css2?family=Patrick+Hand&display=swap');
</style>
