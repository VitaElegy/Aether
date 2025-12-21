import { createRouter, createWebHistory } from 'vue-router';
import { useAuthStore } from '../stores/auth';

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/login',
      name: 'login',
      component: () => import('../views/LoginView.vue'),
    },
    {
      path: '/',
      name: 'home',
      component: () => import('../views/HomeView.vue'), // Assuming this exists or will exist
      meta: { requiresAuth: true }
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

