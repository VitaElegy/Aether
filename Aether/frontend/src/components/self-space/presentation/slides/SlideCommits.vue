<template>
  <SlideBase>
    <div class="commits-content">
      <h2 class="title">版本历史 (Commits)</h2>
      <p class="hint">当前演示文稿的实时版本记录</p>
      
      <div class="timeline-container" v-if="loading">
         <div class="loading">加载中...</div>
      </div>
      
      <div class="timeline-container" v-else-if="error">
        <div class="error">{{ error }}</div>
      </div>

      <div class="timeline-container scrollable" v-else>
        <div class="timeline">
          <div v-for="(commit, index) in commits" :key="commit.id" class="timeline-item">
            <!-- Left Side: Date/Time -->
            <div class="timeline-meta">
               <span class="version-tag">{{ commit.version }}</span>
               <span class="date">{{ formatDate(commit.created_at) }}</span>
            </div>

            <!-- Center: Dot & Line -->
            <div class="timeline-marker">
              <div class="dot" :class="{ 'latest': index === 0 }"></div>
              <div class="line" v-if="index !== commits.length - 1"></div>
            </div>

            <!-- Right Side: Content -->
            <div class="timeline-content">
              <div class="commit-card">
                 <div class="header">
                   <div class="author">
                     <t-icon name="user-circle" /> {{ commit.editor_id }}
                   </div>
                   <div class="hash">#{{ commit.id.substring(0, 8) }}</div>
                 </div>
                 <div class="message">{{ commit.reason }}</div>
                 <div class="changes-info" v-if="index > 0">
                    <t-tag theme="primary" variant="light" size="small">Git Commit</t-tag>
                 </div>
              </div>
            </div>
          </div>
        </div>
      </div>

    </div>
  </SlideBase>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import SlideBase from '../SlideBase.vue'
import { useContent } from '@/composables/useContent' // Assuming we can use this or axios directly
import axios from 'axios'

interface Commit {
  id: string
  version: string
  title: string
  created_at: string
  reason?: string
  editor_id?: string
  hash?: string
  short_hash?: string
}

const commits = ref<Commit[]>([])
const loading = ref(true)
const error = ref('')

const fetchHistory = async () => {
    try {
        const res = await axios.get('/api/system/git-log');
        
        // Map Git Log to UI Model
        commits.value = res.data.map((c: any) => ({
            id: c.hash,
            version: c.short_hash,
            title: c.message, // Use message as title
            created_at: c.date,
            reason: c.message,
            editor_id: c.author,
            short_hash: c.short_hash
        }));
        loading.value = false
    } catch (e) {
        console.error("Failed to load git log", e);
        error.value = "Failed to load project history"
        loading.value = false
    }
}

const formatDate = (isoStr: string) => {
    const d = new Date(isoStr)
    return d.toLocaleString('zh-CN', { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' })
}

onMounted(() => {
    fetchHistory()
})
</script>

<style scoped>
.commits-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  height: 100%;
  width: 100%;
  padding-top: 4rem;
}

.title {
  font-size: 2.5rem;
  font-weight: 700;
  margin-bottom: 0.5rem;
  color: white;
}

.hint {
    color: rgba(255,255,255,0.5);
    margin-bottom: 3rem;
}

.timeline-container {
    width: 800px;
    max-height: 60vh;
    padding: 1rem;
}

.scrollable {
    overflow-y: auto;
    scrollbar-width: thin;
    scrollbar-color: rgba(255,255,255,0.2) transparent;
}

/* Timeline Styles */
.timeline {
    position: relative;
    padding: 1rem 0;
}

.timeline-item {
    display: flex;
    margin-bottom: 0;
    min-height: 100px;
}

/* Left: Meta */
.timeline-meta {
    width: 120px;
    text-align: right;
    padding-right: 1.5rem;
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    padding-top: 0.5rem;
}

.version-tag {
    background: #4f46e5;
    color: white;
    padding: 2px 8px;
    border-radius: 4px;
    font-size: 0.8rem;
    font-weight: bold;
    margin-bottom: 0.3rem;
}

.date {
    color: rgba(255,255,255,0.5);
    font-size: 0.8rem;
}

/* Center: Marker */
.timeline-marker {
    width: 20px;
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
}

.dot {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: rgba(255,255,255,0.3);
    border: 2px solid #1e1e1e;
    z-index: 2;
    margin-top: 0.6rem;
    transition: all 0.3s;
}

.dot.latest {
    background: #4f46e5;
    box-shadow: 0 0 10px rgba(79, 70, 229, 0.5);
    width: 16px;
    height: 16px;
    margin-top: 0.5rem;
}

.line {
    width: 2px;
    flex-grow: 1;
    background: rgba(255,255,255,0.1);
    margin-top: 5px;
}

/* Right: Content */
.timeline-content {
    flex: 1;
    padding-left: 1.5rem;
    padding-bottom: 2rem;
}

.commit-card {
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 12px;
    padding: 1rem;
    transition: transform 0.2s;
}

.commit-card:hover {
    background: rgba(255, 255, 255, 0.08);
    transform: translateX(5px);
}

.header {
    display: flex;
    justify-content: space-between;
    margin-bottom: 0.5rem;
    font-size: 0.9rem;
    color: rgba(255,255,255,0.7);
}

.author {
    display: flex;
    align-items: center;
    gap: 0.5rem;
}

.hash {
    font-family: monospace;
    opacity: 0.5;
}

.message {
    font-size: 1.1rem;
    color: white;
    line-height: 1.4;
}

.changes-info {
    margin-top: 0.8rem;
}

/* Loading/Error */
.loading, .error {
    text-align: center;
    color: rgba(255,255,255,0.5);
    font-size: 1.2rem;
    margin-top: 2rem;
}
</style>
