<template>
  <SlideBase>
    <div class="content">
      <h2 class="section-title">Design Philosophy</h2>
      <p class="subtitle">极简主义 · 优雅美学 · 沉浸体验</p>

      <div class="showcase-container">
          <!-- Glass Card Effect Demo -->
          <div class="demo-card glass-effect">
              <div class="card-header">
                  <span class="dot red"></span>
                  <span class="dot yellow"></span>
                  <span class="dot green"></span>
              </div>
              <div class="card-body">
                  <h3>Glassmorphism</h3>
                  <p>采用现代化毛玻璃设计语言，背景模糊与半透明层叠加，营造纵深感。</p>
                  <button class="demo-btn">Interaction</button>
              </div>
          </div>

          <!-- Typography Demo -->
          <div class="demo-card typography">
              <h3>To Strive, To Seek, To Find</h3>
              <p class="serif">Not yield.</p>
              <div class="font-spec">
                  <span>Inter (Sans)</span>
                  <span>Merriweather (Serif)</span>
                  <span>Fira Code (Mono)</span>
              </div>
              <p class="desc">精心调配的字体排印，为长时间阅读和编码提供最舒适的视觉体验。</p>
          </div>
      </div>

      <div class="philosophy-grid">
          <div class="phial-item">
              <div class="icon">✨</div>
              <h4>Less is More</h4>
              <p>剔除一切无用的装饰，让内容成为主角。</p>
          </div>
          <div class="phial-item">
              <div class="icon">🌊</div>
              <h4>Fluid Motion</h4>
              <p>丝滑的微交互动画，让每一次点击都充满生命力。</p>
          </div>
          <div class="phial-item">
              <div class="icon">🌘</div>
              <h4>Dark Mode</h4>
              <p>原生级深色模式支持，保护视力，专注沉浸。</p>
          </div>
      </div>

    </div>
  </SlideBase>
</template>

<script setup lang="ts">
import SlideBase from '../SlideBase.vue'
</script>

<style scoped>
.content { text-align: center; max-width: 1200px; }

.section-title {
  font-size: 3.5rem;
  margin-bottom: 0.5rem;
  background: linear-gradient(to right, #fff, #94a3b8);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  font-family: 'Times New Roman', serif; /* Elegant serif for title */
  font-style: italic;
}

.subtitle {
    color: rgba(255,255,255,0.6);
    margin-bottom: 3rem;
    font-size: 1.2rem;
    letter-spacing: 4px;
    text-transform: uppercase;
}

.showcase-container {
    display: flex;
    justify-content: center;
    gap: 3rem;
    margin-bottom: 4rem;
}

.demo-card {
    width: 350px;
    height: 220px;
    border-radius: 1rem;
    padding: 1.5rem;
    text-align: left;
    transition: transform 0.3s ease;
}
.demo-card:hover { transform: translateY(-10px); }

/* Glass Effect */
.glass-effect {
    background: rgba(255, 255, 255, 0.05);
    backdrop-filter: blur(10px);
    border: 1px solid rgba(255, 255, 255, 0.1);
    box-shadow: 0 8px 32px 0 rgba(0, 0, 0, 0.37);
}
.card-header { display: flex; gap: 8px; margin-bottom: 1rem; }
.dot { width: 12px; height: 12px; border-radius: 50%; }
.red { background: #ff5f56; }
.yellow { background: #ffbd2e; }
.green { background: #27c93f; }
.card-body h3 { color: #fff; margin: 0 0 0.5rem 0; }
.card-body p { color: #aaa; font-size: 0.9rem; margin-bottom: 1rem; }
.demo-btn {
    background: rgba(255,255,255,0.1); border: 1px solid rgba(255,255,255,0.2);
    color: #fff; padding: 0.5rem 1rem; border-radius: 0.5rem; cursor: pointer;
    transition: all 0.2s;
}
.demo-btn:hover { background: #fff; color: #000; }

/* Typography */
.typography {
    background: #0f172a;
    border: 1px solid #334155;
    position: relative;
    overflow: hidden;
}
.typography::before {
    content: ''; position: absolute; top:0; left:0; width:4px; height:100%; background: #3b82f6;
}
.typography h3 { color: #fff; font-family: sans-serif; margin-top: 0; }
.typography .serif { font-family: serif; font-style: italic; font-size: 1.5rem; color: #94a3b8; margin: 0.5rem 0 1rem; }
.font-spec { display: flex; gap: 1rem; font-size: 0.7rem; color: #64748b; font-family: monospace; border-top: 1px solid #334155; padding-top: 1rem; margin-bottom: 1rem; }
.typography .desc { font-size: 0.85rem; color: #94a3b8; line-height: 1.4; margin: 0; }

.philosophy-grid {
    display: flex;
    justify-content: center;
    gap: 4rem;
}

.phial-item {
    text-align: center;
    width: 200px;
}
.phial-item .icon { font-size: 2.5rem; margin-bottom: 1rem; }
.phial-item h4 { color: #fff; margin: 0 0 0.5rem 0; font-size: 1.2rem; }
.phial-item p { color: #888; font-size: 0.9rem; line-height: 1.5; }

</style>
