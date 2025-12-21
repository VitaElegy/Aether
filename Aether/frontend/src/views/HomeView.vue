<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { useAuthStore } from '../stores/auth';
import { MessagePlugin, Layout, Header, Content, Footer, Button } from 'tdesign-vue-next';
import DynamicRenderer from '../components/DynamicRenderer.vue';

const router = useRouter();
const authStore = useAuthStore();

const handleLogout = () => {
  authStore.logout();
};

// Mock Data (In real app, fetch from API)
const posts = ref([
  {
    id: 1,
    type: 'Markdown',
    data: { content: '# Hello Aether\nWelcome to the future of content.' }
  },
  {
    id: 2,
    type: 'CodeSnippet',
    data: { language: 'rust', code: 'fn main() { println!("Hello World"); }' }
  }
]);

</script>

<template>
  <t-layout class="main-layout">
    <t-header class="header">
      <div class="logo">AETHER</div>
      <div class="user-info">
        <span>{{ authStore.user?.name || 'Guest' }}</span>
        <t-button variant="text" theme="danger" @click="handleLogout">Logout</t-button>
      </div>
    </t-header>
    <t-content class="content">
      <div class="feed">
        <div v-for="post in posts" :key="post.id" class="post-card">
          <DynamicRenderer :type="post.type" :data="post.data" />
        </div>
      </div>
    </t-content>
  </t-layout>
</template>

<style scoped>
.main-layout {
  height: 100vh;
  background: var(--td-bg-color-page);
}
.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 24px;
  background: var(--td-bg-color-container);
  box-shadow: var(--td-shadow-1);
}
.logo {
  font-family: 'Fira Code', monospace;
  font-weight: bold;
  font-size: 1.5rem;
}
.content {
  padding: 24px;
  max-width: 800px;
  margin: 0 auto;
}
.post-card {
  margin-bottom: 24px;
  padding: 24px;
  background: var(--td-bg-color-container);
  border-radius: var(--td-radius-medium);
  box-shadow: var(--td-shadow-1);
}
</style>

