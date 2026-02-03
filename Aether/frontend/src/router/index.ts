import { createRouter, createWebHistory } from 'vue-router';
import { useAuthStore } from '../stores/auth';
import { useNavigationStackStore } from '../stores/navigationStack';

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
      path: '/space/:kbId?',
      name: 'space',
      component: () => import('../views/SelfSpaceView.vue'),
      meta: { transition: 'fade' }
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
    {
      path: '/demo/math-v2',
      name: 'math-v2-demo',
      component: () => import('../views/MathV2Demo.vue'),
      meta: { transition: 'fade' }
    },
    // [REMOVED] /admin/templates - Moved to internal navigation within AdminDashboard
    // This enforces the Desktop Shell architecture.
    // Wildcard redirect for safety
    {
      path: '/:pathMatch(.*)*',
      redirect: '/'
    }
  ],
});

router.beforeEach(async (to, from, next) => {
  const authStore = useAuthStore();
  const navStore = useNavigationStackStore();

  // 1. Initialize Nav Store if needed
  if (!navStore.isLoaded) {
    await navStore.init();
  }

  // 2. Determine Module ID
  let moduleId = 'global';
  const path = to.path;

  if (path.startsWith('/space')) {
    moduleId = 'space';
  } else if (path.startsWith('/editor')) {
    moduleId = 'editor';
  } else if (path.startsWith('/article') || path === '/') {
    moduleId = 'library';
  } else if (path.startsWith('/search')) {
    moduleId = 'search';
  } else if (path.startsWith('/settings')) {
    moduleId = 'settings';
  } else if (path.startsWith('/profile')) {
    moduleId = 'profile';
  }

  // 3. Set Active Module
  navStore.setActiveModule(moduleId);

  // 4. Update Stack (Smart Trace)
  // We only track "Content" pages, ignoring auth/login pages for the stack
  if (to.name !== 'login') {
    const stack = navStore.stacks[moduleId] || [];
    const prevItem = stack[stack.length - 2];

    if (prevItem && prevItem === to.fullPath) {
      // Detected "Back" navigation (User went to previous item)
      // Pop the top to unwind
      navStore.popRoute(moduleId);
    } else {
      // Forward navigation -> Push
      navStore.pushRoute(moduleId, to.fullPath);
    }
  }

  // 5. Auth Middleware
  if (to.meta.requiresAuth && !authStore.isAuthenticated) {
    next('/login');
  } else if (to.name === 'login' && authStore.isAuthenticated) {
    next('/'); // Don't let them see login if already authed
  } else {
    next();
  }
});

export default router;
