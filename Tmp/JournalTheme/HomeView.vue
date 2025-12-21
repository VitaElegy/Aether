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
  ArrowRightIcon,
  LocationIcon
} from 'tdesign-icons-vue-next';
import { Button, Tag, Avatar, Divider } from 'tdesign-vue-next';
import DynamicRenderer from '../components/DynamicRenderer.vue';

const router = useRouter();
const authStore = useAuthStore();

const handleLogout = () => {
  authStore.logout();
  router.push('/login');
};

const goToProfile = (userId: string) => {
  router.push(`/profile/${userId}`);
};

const posts = ref<any[]>([]);

onMounted(async () => {
  try {
    const res = await axios.get('/api/content');
    posts.value = res.data.map((p: any) => ({
      id: p.id,
      authorId: p.author_id,
      type: p.body.type,
      title: p.title,
      author: 'Traveler', // Mock
      date: new Date(p.created_at).toLocaleDateString('en-US', { month: 'long', day: 'numeric', year: 'numeric' }),
      weekday: new Date(p.created_at).toLocaleDateString('en-US', { weekday: 'long' }),
      tags: p.tags,
      data: p.body.type === 'Markdown' ? { content: p.body.data } : p.body.data,
      mood: ['Sunny', 'Rainy', 'Cloudy'][Math.floor(Math.random() * 3)],
      location: 'The Void'
    }));
  } catch (err) {
    console.error('Failed to fetch posts', err);
  }
});

const getMoodIcon = (mood: string) => {
  if (mood === 'Sunny') return '☀️';
  if (mood === 'Rainy') return '🌧️';
  return '☁️';
};
</script>

<template>
  <div class="journal-desk">
    <!-- The Journal Book -->
    <div class="journal-book open">

      <!-- Left Page: Cover / Navigation / Profile Snippet -->
      <aside class="page left-page">
        <div class="page-content">
          <div class="journal-header">
            <h1 class="journal-title">Chronicles</h1>
            <p class="journal-subtitle">Of Aether</p>
            <div class="divider-ornament">✻</div>
          </div>

          <div class="profile-stamp" @click="goToProfile(authStore.user?.id || 'me')">
            <div class="avatar-frame">
              <img src="https://api.dicebear.com/7.x/notionists/svg?seed=Felix" alt="User" />
            </div>
            <div class="stamp-info">
              <span class="user-name">{{ authStore.user?.name || 'Explorer' }}</span>
              <span class="user-role">Architect</span>
            </div>
          </div>

          <nav class="journal-nav">
            <div class="nav-item active">
              <span class="bullet">•</span> Journal Entries
            </div>
            <div class="nav-item">
              <span class="bullet">◦</span> Collections
            </div>
            <div class="nav-item">
              <span class="bullet">◦</span> Map
            </div>
          </nav>

          <div class="page-footer">
            <div class="search-field">
              <search-icon />
              <input type="text" placeholder="Search memories..." />
            </div>
            <button class="sign-out-link" @click="handleLogout">Close Book</button>
          </div>
        </div>
        <div class="book-spine"></div>
      </aside>

      <!-- Right Page: Content Stream -->
      <main class="page right-page">
        <div class="page-header-row">
          <div class="date-stamp">
            {{ new Date().toLocaleDateString('en-US', { month: 'short', year: 'numeric' }).toUpperCase() }}
          </div>
          <t-button theme="primary" variant="text" @click="router.push('/editor')" class="new-entry-btn">
            <add-icon /> New Entry
          </t-button>
        </div>

        <div class="entries-stream">
          <div v-for="post in posts" :key="post.id" class="journal-entry">
            <div class="entry-meta-sidebar">
              <span class="day-number">{{ new Date(post.date).getDate() }}</span>
              <span class="day-name">{{ post.weekday.substring(0,3).toUpperCase() }}</span>
              <div class="mood-icon">{{ getMoodIcon(post.mood) }}</div>
            </div>

            <div class="entry-content-area">
              <div class="entry-header">
                <div class="header-top">
                  <h2 class="entry-title">{{ post.title }}</h2>
                  <div class="location-tag">
                    <location-icon size="12px" /> {{ post.location }}
                  </div>
                </div>
                <div class="author-link" @click.stop="goToProfile(post.authorId)">
                  Written by <span class="author-name">{{ post.author }}</span>
                </div>
              </div>

              <div class="entry-body">
                 <DynamicRenderer :type="post.type" :data="post.data" />
              </div>

              <div class="entry-footer">
                <div class="tags">
                  <span v-for="tag in post.tags" :key="tag" class="handwritten-tag">#{{ tag }}</span>
                </div>
                <div class="bookmark">
                  <bookmark-icon />
                </div>
              </div>
            </div>
          </div>
        </div>
      </main>

    </div>
  </div>
</template>

<style scoped>
/* Desk Environment */
.journal-desk {
  min-height: 100vh;
  display: flex;
  justify-content: center;
  align-items: center;
  padding: 40px;
  perspective: 1500px;
  overflow: hidden;
}

/* The Book */
.journal-book {
  display: flex;
  width: 1000px;
  height: 800px;
  background: #F5F1E6;
  box-shadow:
    0 20px 50px rgba(0,0,0,0.3),
    0 0 0 2px #5D4037; /* Leather edge hint */
  border-radius: 8px;
  position: relative;
  transform-style: preserve-3d;
}

/* Common Page Styles */
.page {
  flex: 1;
  position: relative;
  padding: 48px;
  background-image:
    url('https://www.transparenttextures.com/patterns/cream-paper.png'),
    linear-gradient(to right, rgba(0,0,0,0.05) 0%, transparent 10%); /* Shadow near spine */
  overflow: hidden;
}

.left-page {
  border-radius: 6px 0 0 6px;
  border-right: 1px solid rgba(0,0,0,0.1);
  display: flex;
  flex-direction: column;
}

.right-page {
  border-radius: 0 6px 6px 0;
  overflow-y: auto;
}

/* Spine Effect */
.book-spine {
  position: absolute;
  right: 0;
  top: 0;
  bottom: 0;
  width: 40px;
  background: linear-gradient(to right,
    rgba(0,0,0,0) 0%,
    rgba(0,0,0,0.1) 40%,
    rgba(0,0,0,0.05) 60%,
    rgba(0,0,0,0) 100%
  );
  pointer-events: none;
  z-index: 10;
  transform: translateX(50%);
}

/* Left Page Content */
.journal-header {
  text-align: center;
  margin-bottom: 48px;
}

.journal-title {
  font-family: 'Cinzel', serif; /* Or similar classic font */
  font-size: 3rem;
  color: #3E2723;
  margin: 0;
  letter-spacing: 4px;
  text-transform: uppercase;
}

.journal-subtitle {
  font-family: 'Caveat', cursive;
  font-size: 1.5rem;
  color: #8D6E63;
  margin: -10px 0 16px;
}

.divider-ornament {
  font-size: 24px;
  color: #A1887F;
}

.profile-stamp {
  display: flex;
  align-items: center;
  gap: 16px;
  background: rgba(139, 69, 19, 0.05);
  padding: 12px 20px;
  border-radius: 2px;
  border: 1px dashed #D7CCC8;
  margin-bottom: 40px;
  cursor: pointer;
  transition: background 0.2s;
}

.profile-stamp:hover {
  background: rgba(139, 69, 19, 0.1);
}

.avatar-frame {
  width: 56px;
  height: 56px;
  border-radius: 50%;
  overflow: hidden;
  border: 2px solid #5D4037;
  sepia: 0.5; /* Old photo effect */
}

.avatar-frame img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.stamp-info {
  display: flex;
  flex-direction: column;
}

.user-name {
  font-weight: 700;
  color: #3E2723;
  font-size: 1.1rem;
}

.user-role {
  font-family: 'Inter', sans-serif;
  font-size: 0.75rem;
  text-transform: uppercase;
  letter-spacing: 1px;
  color: #8D6E63;
}

.journal-nav {
  display: flex;
  flex-direction: column;
  gap: 16px;
  flex: 1;
}

.nav-item {
  font-family: 'Crimson Pro', serif;
  font-size: 1.2rem;
  color: #5D4037;
  cursor: pointer;
  transition: color 0.2s;
  display: flex;
  align-items: center;
  gap: 12px;
}

.nav-item:hover, .nav-item.active {
  color: #8B4513;
  font-style: italic;
  font-weight: 600;
}

.bullet {
  color: #D7CCC8;
  font-size: 1.5rem;
  line-height: 1;
}

.page-footer {
  margin-top: auto;
}

.search-field {
  display: flex;
  align-items: center;
  border-bottom: 2px solid #D7CCC8;
  padding-bottom: 8px;
  margin-bottom: 24px;
  color: #8D6E63;
}

.search-field input {
  background: transparent;
  border: none;
  outline: none;
  font-family: 'Crimson Pro', serif;
  font-size: 1rem;
  margin-left: 12px;
  width: 100%;
  color: #3E2723;
}

.sign-out-link {
  background: none;
  border: none;
  font-family: 'Inter', sans-serif;
  text-transform: uppercase;
  font-size: 0.8rem;
  letter-spacing: 1px;
  color: #A1887F;
  cursor: pointer;
  width: 100%;
  text-align: left;
}

.sign-out-link:hover {
  color: #8B4513;
}

/* Right Page Content */
.page-header-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 40px;
  border-bottom: 1px double #D7CCC8;
  padding-bottom: 16px;
}

.date-stamp {
  font-family: 'Inter', sans-serif;
  font-weight: 700;
  letter-spacing: 2px;
  color: #A1887F;
  border: 2px solid #A1887F;
  padding: 4px 12px;
  border-radius: 4px;
  transform: rotate(-2deg);
}

.new-entry-btn {
  color: #8B4513 !important;
}

.entries-stream {
  display: flex;
  flex-direction: column;
  gap: 48px;
}

.journal-entry {
  display: flex;
  gap: 24px;
}

.entry-meta-sidebar {
  display: flex;
  flex-direction: column;
  align-items: center;
  min-width: 50px;
  padding-top: 8px;
}

.day-number {
  font-family: 'Crimson Pro', serif;
  font-size: 2rem;
  font-weight: 700;
  color: #3E2723;
  line-height: 1;
}

.day-name {
  font-family: 'Inter', sans-serif;
  font-size: 0.7rem;
  text-transform: uppercase;
  color: #8D6E63;
  margin-bottom: 12px;
}

.mood-icon {
  font-size: 1.5rem;
  opacity: 0.7;
}

.entry-content-area {
  flex: 1;
  padding-bottom: 24px;
  border-bottom: 1px dashed #D7CCC8;
}

.entry-header {
  margin-bottom: 16px;
}

.header-top {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
}

.entry-title {
  font-size: 1.8rem;
  margin: 0;
  color: #2C241B;
  font-weight: 600;
  line-height: 1.2;
}

.location-tag {
  font-family: 'Inter', sans-serif;
  font-size: 0.7rem;
  color: #A1887F;
  text-transform: uppercase;
  display: flex;
  align-items: center;
  gap: 4px;
}

.author-link {
  font-family: 'Caveat', cursive;
  font-size: 1.1rem;
  color: #8D6E63;
  margin-top: 4px;
  cursor: pointer;
  display: inline-block;
  transition: color 0.2s;
}

.author-link:hover {
  color: #8B4513;
  text-decoration: underline;
}

.entry-body {
  font-size: 1.1rem;
  color: #4E342E;
  margin-bottom: 24px;
}

.entry-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.tags {
  display: flex;
  gap: 12px;
}

.handwritten-tag {
  font-family: 'Caveat', cursive;
  font-size: 1.2rem;
  color: #8B4513;
}

.bookmark {
  color: #D7CCC8;
  cursor: pointer;
}

.bookmark:hover {
  color: #8B4513;
}

@media (max-width: 1100px) {
  .journal-book {
    width: 100%;
    height: 90vh;
    flex-direction: column;
  }

  .left-page {
    border-right: none;
    border-bottom: 1px solid rgba(0,0,0,0.1);
    flex: 0 0 auto;
    padding: 24px;
  }

  .book-spine {
    width: 100%;
    height: 20px;
    top: auto;
    bottom: 0;
    right: 0;
    left: 0;
    transform: translateY(50%);
    background: linear-gradient(to bottom, rgba(0,0,0,0), rgba(0,0,0,0.1));
  }

  .journal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 0;
  }

  .journal-title {
    font-size: 2rem;
  }

  .journal-subtitle, .divider-ornament, .journal-nav, .page-footer {
    display: none; /* Simplify for mobile/tablet */
  }

  .profile-stamp {
    margin-bottom: 0;
  }
}
</style>
