<script setup lang="ts">
import { ref, reactive } from 'vue';
import { useRouter } from 'vue-router';
import axios from 'axios';
import { MessagePlugin, Button, Input, Select, Textarea, TagInput } from 'tdesign-vue-next';
import { ArrowLeftIcon, SaveIcon, UploadIcon } from 'tdesign-icons-vue-next';
import DynamicRenderer from '../components/DynamicRenderer.vue';

const router = useRouter();

const form = reactive({
  title: '',
  body: '',
  category: '',
  visibility: 'Public',
  tags: [] as string[]
});

const visibilityOptions = [
  { label: 'Public (Everyone)', value: 'Public' },
  { label: 'Internal (Logged In)', value: 'Internal' },
  { label: 'Private (Only Me)', value: 'Private' }
];

const handleImport = () => {
  const input = document.createElement('input');
  input.type = 'file';
  input.accept = '.md,.markdown';
  input.onchange = (e: any) => {
    const file = e.target.files[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = (ev) => {
      form.body = ev.target?.result as string;
      // Try to extract title from first line if empty
      if (!form.title && form.body.startsWith('# ')) {
          const firstLine = form.body.split('\n')[0];
          form.title = firstLine.replace('# ', '').trim();
      }
      MessagePlugin.success('Markdown loaded');
    };
    reader.readAsText(file);
  };
  input.click();
};

const handlePublish = async () => {
  try {
    await axios.post('/api/content', {
        title: form.title,
        body: form.body,
        tags: form.tags,
        category: form.category || null,
        visibility: form.visibility
    });
    MessagePlugin.success('Published successfully');
    router.push('/');
  } catch (err) {
    MessagePlugin.error('Failed to publish');
  }
};
</script>

<template>
  <div class="editor-container">
    <header class="editor-header">
      <div class="left">
        <t-button variant="text" shape="circle" @click="router.back()">
          <template #icon><arrow-left-icon /></template>
        </t-button>
        <span class="page-title">New Entry</span>
      </div>
      <div class="right">
        <t-button variant="outline" @click="handleImport">
          <template #icon><upload-icon /></template>
          Import MD
        </t-button>
        <t-button theme="primary" @click="handlePublish">
          <template #icon><save-icon /></template>
          Publish
        </t-button>
      </div>
    </header>

    <div class="editor-layout">
       <!-- Meta Column -->
       <aside class="meta-pane">
          <div class="form-group">
             <label>Visibility</label>
             <t-select v-model="form.visibility" :options="visibilityOptions" class="glass-select" />
          </div>
          <div class="form-group">
             <label>Category</label>
             <t-input v-model="form.category" placeholder="e.g. Tech/Rust" class="glass-input" />
          </div>
          <div class="form-group">
             <label>Tags</label>
             <t-tag-input v-model="form.tags" placeholder="Enter tags..." class="glass-input" />
          </div>
       </aside>

       <!-- Main Editor -->
       <main class="editor-main">
          <t-input
            v-model="form.title"
            placeholder="Title"
            size="large"
            class="title-input"
          />
          <div class="split-view">
             <textarea
                v-model="form.body"
                class="markdown-editor"
                placeholder="Write your thoughts..."
             ></textarea>
             <div class="preview-pane">
                <DynamicRenderer type="Markdown" :data="{ content: form.body }" />
             </div>
          </div>
       </main>
    </div>
  </div>
</template>

<style scoped>
.editor-container {
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: var(--td-bg-color-page);
  color: #fff;
}

.editor-header {
  height: 64px;
  border-bottom: 1px solid rgba(255,255,255,0.05);
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 24px;
}

.left, .right {
  display: flex;
  align-items: center;
  gap: 16px;
}

.page-title {
  font-size: 18px;
  font-weight: 600;
}

.editor-layout {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.meta-pane {
  width: 300px;
  padding: 24px;
  border-right: 1px solid rgba(255,255,255,0.05);
  display: flex;
  flex-direction: column;
  gap: 24px;
  background: rgba(0,0,0,0.2);
}

.form-group label {
  display: block;
  margin-bottom: 8px;
  font-size: 13px;
  color: rgba(255,255,255,0.5);
  text-transform: uppercase;
}

.editor-main {
  flex: 1;
  display: flex;
  flex-direction: column;
}

/* Custom Input Styles to match theme */
:deep(.glass-input), :deep(.glass-select) {
  --td-bg-color-container: rgba(255,255,255,0.05);
  --td-border-level-2-color: rgba(255,255,255,0.1);
}

:deep(.title-input) {
  --td-bg-color-container: transparent;
  --td-border-level-2-color: transparent;
  --td-input-text-color: #fff;
  --td-input-placeholder-color: rgba(255,255,255,0.2);
  --td-component-size-l: 80px;
  font-size: 32px;
  font-weight: 700;
}

:deep(.title-input:hover), :deep(.title-input:focus-within) {
  --td-bg-color-container: transparent;
  border-color: transparent !important;
  box-shadow: none !important;
}

.split-view {
  flex: 1;
  display: flex;
  border-top: 1px solid rgba(255,255,255,0.05);
}

.markdown-editor {
  flex: 1;
  background: transparent;
  border: none;
  border-right: 1px solid rgba(255,255,255,0.05);
  color: #fff;
  padding: 24px;
  font-family: 'SFMono-Regular', Consolas, monospace;
  font-size: 14px;
  line-height: 1.6;
  resize: none;
  outline: none;
}

.preview-pane {
  flex: 1;
  padding: 24px;
  overflow-y: auto;
  background: rgba(255,255,255,0.02);
}
</style>

