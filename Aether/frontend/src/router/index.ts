import { createRouter, createWebHistory } from 'vue-router';
import { useAuthStore } from '../stores/auth';

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/login',
      name: 'login',
      component: () => import('../views/LoginView.vue'),
      meta: { transition: 'fade' }
    },
    {
      path: '/',
      name: 'home',
      component: () => import('../views/HomeView.vue'),
      meta: { transition: 'fade' }
    },
    {
      path: '/search',
      name: 'search',
      component: () => import('../views/SearchView.vue'),
      meta: { transition: 'fade' }
    },
    {
      path: '/editor/:id?',
      name: 'editor',
      component: () => import('../views/EditorView.vue'),
      meta: { requiresAuth: true, transition: 'zoom' }
    },
    {
      path: '/article/:id',
      name: 'article',
      component: () => import('../views/ReadView.vue'),
      meta: { transition: 'slide-right' }
    },
    {
      path: '/profile/:id',
      name: 'profile',
      component: () => import('../views/UserProfileView.vue'),
      meta: { transition: 'slide-left' }
    },
    {
      path: '/settings',
      name: 'settings',
      component: () => import('../views/SettingsView.vue'),
      meta: { requiresAuth: true, transition: 'fade' }
    },
    {
      path: '/content/:id/history',
      name: 'history',
      component: () => import('../views/HistoryView.vue'),
      meta: { requiresAuth: true, transition: 'slide-right' }
    },
    {
      path: '/content/:id/version/:version',
      name: 'version',
      component: () => import('../views/VersionView.vue'),
      meta: { requiresAuth: true, transition: 'slide-right' }
    },
    {
      path: '/space',
      name: 'space',
      component: () => import('../views/SelfSpaceView.vue'),
      meta: { transition: 'fade' }
    },
    {
      path: '/kb/:id',
      name: 'knowledge-base',
      component: () => import('../views/KnowledgeBaseDetail.vue'),
      meta: { transition: 'slide-right' }
    },
    {
      path: '/kb/:id/tree',
      name: 'kb-tree-detail',
      component: () => import('../views/MathTreeDetailView.vue'),
      meta: { transition: 'fade' }
    },
    {
      path: '/space/presentation',
      name: 'space-presentation',
      component: () => import('../components/self-space/presentation/PresentationContainer.vue'),
      meta: { transition: 'fade' }
    },
    // Wildcard redirect for safety
    {
      path: '/:pathMatch(.*)*',
      redirect: '/'
    }
  ],
});

router.beforeEach((to, from, next) => {
  const authStore = useAuthStore();

  if (to.meta.requiresAuth && !authStore.isAuthenticated) {
    next('/login');
  } else if (to.name === 'login' && authStore.isAuthenticated) {
    next('/'); // Don't let them see login if already authed
  } else {
    next();
  }
});

export default router;
