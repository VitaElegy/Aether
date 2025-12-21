import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { useRouter } from 'vue-router';
import axios from 'axios';

// In a real app, this would decode the JWT to check expiration
// import { jwtDecode } from 'jwt-decode';

export interface User {
  id: string;
  username: string;
  email: string;
  display_name?: string;
  bio?: string;
  avatar_url?: string;
  permissions: number;
}

export const useAuthStore = defineStore('auth', () => {
  const token = ref<string | null>(localStorage.getItem('aether_token'));
  const user = ref<User | null>(null);
  const router = useRouter();

  const isAuthenticated = computed(() => !!token.value);

  // Initialize axios interceptor
  axios.interceptors.request.use(config => {
      if (token.value) {
          config.headers.Authorization = `Bearer ${token.value}`;
      }
      return config;
  });

  function login(newToken: string, userData: any) {
    token.value = newToken;
    user.value = userData;
    localStorage.setItem('aether_token', newToken);
  }

  function logout() {
    token.value = null;
    user.value = null;
    localStorage.removeItem('aether_token');
    router.push('/login');
  }

  function parseJwt(token: string) {
    try {
      const base64Url = token.split('.')[1];
      const base64 = base64Url.replace(/-/g, '+').replace(/_/g, '/');
      const jsonPayload = decodeURIComponent(window.atob(base64).split('').map(function(c) {
          return '%' + ('00' + c.charCodeAt(0).toString(16)).slice(-2);
      }).join(''));
      return JSON.parse(jsonPayload);
    } catch (e) {
      return null;
    }
  }

  async function fetchUser() {
    if (!token.value) return;

    let userId = user.value?.id;
    if (!userId) {
      const claims = parseJwt(token.value);
      if (claims && claims.sub) {
        userId = claims.sub;
      }
    }

    if (!userId) return;

    try {
      const res = await axios.get(`/api/users/${userId}`);
      user.value = { ...user.value, ...res.data };
    } catch (e) {
      console.error("Failed to fetch user", e);
    }
  }

  async function updateUser(updates: Partial<User>) {
      if (!user.value?.id) return;
      try {
          await axios.put(`/api/users/${user.value.id}`, updates);
          user.value = { ...user.value, ...updates } as User;
      } catch (e) {
          throw e;
      }
  }

  return { token, user, isAuthenticated, login, logout, fetchUser, updateUser };
});

