<script setup lang="ts">
import { ref, onMounted, reactive } from 'vue';
import { useRouter } from 'vue-router';
import { useAuthStore } from '../stores/auth';
import { MessagePlugin } from 'tdesign-vue-next';

const router = useRouter();
const authStore = useAuthStore();

const terminalLines = ref<string[]>([]);
const form = reactive({
  username: '',
  password: '',
});
const step = ref<'init' | 'user' | 'pass' | 'processing'>('init');
const cursorVisible = ref(true);

// Typewriter effect helper
const typeLine = async (text: string, delay = 30) => {
  let current = '';
  terminalLines.value.push('');
  const index = terminalLines.value.length - 1;

  for (const char of text) {
    current += char;
    terminalLines.value[index] = current;
    await new Promise(r => setTimeout(r, delay));
  }
};

onMounted(async () => {
  // Blinking cursor effect
  setInterval(() => {
    cursorVisible.value = !cursorVisible.value;
  }, 500);

  // Boot sequence
  await typeLine('AETHER KERNEL v0.1.0-alpha loaded.');
  await typeLine('Initializing secure handshake protocol...');
  await new Promise(r => setTimeout(r, 400));
  await typeLine('[OK] Connection established.');
  await typeLine('[OK] Encryption: Ed25519 verified.');

  step.value = 'user';
});

const handleUserEnter = async () => {
  if (!form.username) return;
  terminalLines.value.push(`user: ${form.username}`);
  step.value = 'pass';
};

const handlePassEnter = async () => {
  if (!form.password) return;
  step.value = 'processing';
  await typeLine('Authenticating...');

  // Mock login for demo (Replace with actual API call)
  setTimeout(async () => {
    if (form.username === 'admin' && form.password === 'secret') {
        await typeLine('[SUCCESS] Access Granted.');
        await typeLine('Redirecting to mainframe...');
        authStore.login('mock-jwt-token', { name: form.username });
        setTimeout(() => router.push('/'), 1000);
    } else {
        await typeLine('[ERROR] Access Denied: Invalid credentials.');
        await typeLine('Resetting connection...');
        form.username = '';
        form.password = '';
        step.value = 'user';
    }
  }, 1500);
};

</script>

<template>
  <div class="terminal-container">
    <div class="crt-overlay"></div>
    <div class="terminal-content">
      <div v-for="(line, idx) in terminalLines" :key="idx" class="line">
        <span class="prefix" v-if="line.startsWith('[')">&gt; </span>
        {{ line }}
      </div>

      <!-- Interactive Inputs -->
      <div v-if="step === 'user'" class="input-line">
        <span class="prompt">login:</span>
        <input
          v-model="form.username"
          @keyup.enter="handleUserEnter"
          type="text"
          autofocus
          class="ghost-input"
        />
        <span class="mirror">{{ form.username }}</span>
        <span class="cursor" v-if="cursorVisible">█</span>
      </div>

      <div v-if="step === 'pass'" class="input-line">
        <span class="prompt">password:</span>
        <input
          v-model="form.password"
          @keyup.enter="handlePassEnter"
          type="password"
          class="ghost-input"
          ref="passInput"
          :autofocus="true"
        />
        <span class="mirror">{{ '*'.repeat(form.password.length) }}</span>
        <span class="cursor" v-if="cursorVisible">█</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
@import url('https://fonts.googleapis.com/css2?family=Fira+Code:wght@400;700&display=swap');

.terminal-container {
  background-color: #0a0a0a;
  color: #00ff41; /* Hacker Green */
  height: 100vh;
  width: 100vw;
  font-family: 'Fira Code', monospace;
  padding: 2rem;
  overflow: hidden;
  position: relative;
}

/* CRT Scanline Effect */
.crt-overlay {
  position: absolute;
  top: 0; left: 0; width: 100%; height: 100%;
  background: linear-gradient(rgba(18, 16, 16, 0) 50%, rgba(0, 0, 0, 0.25) 50%), linear-gradient(90deg, rgba(255, 0, 0, 0.06), rgba(0, 255, 0, 0.02), rgba(0, 0, 255, 0.06));
  background-size: 100% 2px, 3px 100%;
  pointer-events: none;
  z-index: 10;
}

.line {
  margin-bottom: 0.5rem;
  text-shadow: 0 0 5px rgba(0, 255, 65, 0.5);
}

.prompt {
  margin-right: 1rem;
  color: #fff;
}

.input-line {
  display: flex;
  align-items: center;
}

/* Ghost Input: The actual input is invisible but captures focus */
.ghost-input {
  position: absolute;
  opacity: 0;
  top: 0; left: 0;
  height: 0; width: 0;
}

.cursor {
  margin-left: 2px;
  animation: blink 1s step-end infinite;
}

.prefix {
  color: #555;
}

@keyframes blink {
  0%, 100% { opacity: 1; }
  50% { opacity: 0; }
}
</style>

