<script setup lang="ts">
import { ref, reactive } from 'vue';
import { useRouter } from 'vue-router';
import { useAuthStore } from '../stores/auth';
import { MessagePlugin, Button, Input, Form, FormItem, Card } from 'tdesign-vue-next';
import { UserIcon, LockOnIcon, MailIcon, ArrowRightIcon } from 'tdesign-icons-vue-next';
import axios from 'axios';

const router = useRouter();
const authStore = useAuthStore();

const isLogin = ref(true);
const loading = ref(false);

const form = reactive({
  username: '',
  password: '',
  email: ''
});

const toggleMode = () => {
  isLogin.value = !isLogin.value;
  form.username = '';
  form.password = '';
  form.email = '';
};

const handleSubmit = async () => {
  loading.value = true;
  try {
    if (isLogin.value) {
      const res = await axios.post('/api/auth/login', {
        username: form.username,
        password: form.password
      });
      authStore.login('mock-token', { name: form.username });
      MessagePlugin.success('Welcome back, Student.');
      router.push('/');
    } else {
      await axios.post('/api/auth/register', {
        username: form.username,
        email: form.email,
        password: form.password
      });
      MessagePlugin.success('Registration complete. Please sign in.');
      isLogin.value = true;
    }
  } catch (err: any) {
    MessagePlugin.error(err.response?.data?.error || 'Login failed');
  } finally {
    loading.value = false;
  }
};
</script>

<template>
  <div class="school-login-container">
    <div class="notebook-spine"></div>

    <div class="content-wrapper">
      <div class="login-card-stack">
        <!-- Decorative cards behind -->
        <div class="card-layer layer-2"></div>
        <div class="card-layer layer-1"></div>

        <!-- Main Card -->
        <div class="main-card">
          <div class="card-header">
            <div class="school-logo">
              <span class="logo-icon">🎓</span>
            </div>
            <h1>{{ isLogin ? 'Student Portal' : 'New Enrollment' }}</h1>
            <p class="subtitle">Aether Academy of Code</p>
          </div>

          <t-form :data="form" @submit="handleSubmit" class="school-form" label-align="top">
            <t-form-item name="username">
              <t-input
                v-model="form.username"
                placeholder="Student ID / Username"
                size="large"
                class="ruled-input"
              >
                <template #prefix-icon><user-icon /></template>
              </t-input>
            </t-form-item>

            <t-form-item name="email" v-if="!isLogin">
              <t-input
                v-model="form.email"
                placeholder="Email Address"
                size="large"
                class="ruled-input"
              >
                <template #prefix-icon><mail-icon /></template>
              </t-input>
            </t-form-item>

            <t-form-item name="password">
              <t-input
                v-model="form.password"
                type="password"
                placeholder="Password"
                size="large"
                class="ruled-input"
              >
                <template #prefix-icon><lock-on-icon /></template>
              </t-input>
            </t-form-item>

            <div class="form-actions">
              <t-button theme="primary" type="submit" block size="large" :loading="loading" class="submit-btn">
                {{ isLogin ? 'Sign In' : 'Enroll Now' }}
                <template #suffix><arrow-right-icon /></template>
              </t-button>
            </div>
          </t-form>

          <div class="footer-note">
            <span @click="toggleMode" class="link-text">
              {{ isLogin ? 'New student? Apply here.' : 'Already enrolled? Log in.' }}
            </span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.school-login-container {
  min-height: 100vh;
  width: 100vw;
  background-color: #F0F4F8;
  background-image:
    linear-gradient(#E1E8ED 1px, transparent 1px),
    linear-gradient(90deg, #E1E8ED 1px, transparent 1px);
  background-size: 20px 20px;
  display: flex;
  justify-content: center;
  align-items: center;
  position: relative;
  overflow: hidden;
}

/* Notebook Spine Effect */
.notebook-spine {
  position: absolute;
  left: 40px;
  top: 0;
  bottom: 0;
  width: 60px;
  background: #EF4444; /* Spiral Red or Spine Red */
  border-right: 4px solid #B91C1C;
  box-shadow: 5px 0 15px rgba(0,0,0,0.1);
  z-index: 10;
  display: flex;
  flex-direction: column;
  align-items: center;
  padding-top: 40px;
}

.notebook-spine::before {
  content: '';
  position: absolute;
  top: 0;
  bottom: 0;
  left: 10px;
  width: 2px;
  background: rgba(255,255,255,0.2);
}

.content-wrapper {
  margin-left: 80px; /* Offset for spine */
  z-index: 20;
}

.login-card-stack {
  position: relative;
  width: 420px;
}

.card-layer {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: #fff;
  border-radius: 16px;
  border: 1px solid #CBD5E1;
  box-shadow: 0 4px 6px rgba(0,0,0,0.05);
}

.layer-2 {
  transform: rotate(-3deg);
  z-index: 1;
}

.layer-1 {
  transform: rotate(2deg);
  z-index: 2;
}

.main-card {
  position: relative;
  background: #fff;
  padding: 40px;
  border-radius: 16px;
  border: 1px solid #CBD5E1;
  box-shadow: 0 10px 25px rgba(0,0,0,0.1);
  z-index: 3;
}

.card-header {
  text-align: center;
  margin-bottom: 32px;
}

.school-logo {
  width: 64px;
  height: 64px;
  background: #EFF6FF;
  border-radius: 50%;
  margin: 0 auto 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 32px;
  border: 2px solid #DBEAFE;
}

h1 {
  font-family: 'Georgia', serif;
  font-size: 24px;
  color: #1E3A8A;
  margin: 0;
  font-weight: 700;
}

.subtitle {
  color: #64748B;
  font-size: 14px;
  margin-top: 4px;
}

/* Form Styles */
:deep(.ruled-input) {
  --td-bg-color-container: #F8FAFC;
  border: none !important;
  border-bottom: 2px solid #E2E8F0 !important;
  border-radius: 0 !important;
  box-shadow: none !important;
  background: transparent !important;
}

:deep(.ruled-input:focus-within) {
  border-bottom-color: #2563EB !important;
}

:deep(.t-input__inner) {
  font-family: 'Courier New', monospace; /* Typewriter feel */
  font-weight: 600;
}

.submit-btn {
  margin-top: 16px;
  font-weight: 700;
  letter-spacing: 0.5px;
}

.footer-note {
  margin-top: 24px;
  text-align: center;
  border-top: 1px dashed #E2E8F0;
  padding-top: 16px;
}

.link-text {
  color: #2563EB;
  font-weight: 600;
  cursor: pointer;
  text-decoration: underline;
  text-decoration-style: wavy; /* Fun touch */
  text-decoration-thickness: 1px;
}

.link-text:hover {
  color: #1E40AF;
}

@media (max-width: 600px) {
  .notebook-spine { display: none; }
  .content-wrapper { margin-left: 0; }
  .login-card-stack { width: 90vw; }
}
</style>
