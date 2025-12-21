import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { useRouter } from 'vue-router';

// In a real app, this would decode the JWT to check expiration
// import { jwtDecode } from 'jwt-decode';

export const useAuthStore = defineStore('auth', () => {
  const token = ref<string | null>(localStorage.getItem('aether_token'));
  const user = ref<any | null>(null);
  const router = useRouter();

  const isAuthenticated = computed(() => !!token.value);

  function login(newToken: string, userData: any) {
    token.value = newToken;
    user.value = userData;
    localStorage.setItem('aether_token', newToken);
    // Setup Axios headers here or via interceptor
  }

  function logout() {
    token.value = null;
    user.value = null;
    localStorage.removeItem('aether_token');
    router.push('/login');
  }

  return { token, user, isAuthenticated, login, logout };
});

