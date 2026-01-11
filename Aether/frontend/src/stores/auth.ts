import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { useRouter } from 'vue-router';
import axios from 'axios';

// In a real app, this would decode the JWT to check expiration
// import { jwtDecode } from 'jwt-decode';

// Permissions Constants (Mirrors Backend)
export const Permissions = {
  READ_PUBLIC: 1 << 0,
  COMMENT: 1 << 1,

  CREATE_POST: 1 << 4,
  EDIT_POST: 1 << 5,
  DELETE_POST: 1 << 6,

  MEMO_READ: 1 << 8,
  MEMO_WRITE: 1 << 9,

  TODO_READ: 1 << 12,
  TODO_WRITE: 1 << 13,

  ADMIN: 1 << 30, // JS Bitwise operations are 32-bit safe usually, 1<<63 is tricky in JS numbers.
  // Ideally we should use BigInt for 64-bit flags if we go that high.
  // For now 1<<30 is safe max for bitwise ops in some envs, but let's stick to BigInt roughly or just simple numbers.
  // Actually, JS bitwise operators truncate to 32 bits.
  // So 1 << 63 is 0.
  // WE MUST USE BigInt for 64-bit permissions compatibility if backend uses u64.
};

export interface User {
  id: string;
  username: string;
  email: string;
  display_name?: string;
  bio?: string;
  avatar_url?: string;
  permissions: number; // This comes as a number from JSON. If > 2^53 it loses precision.
  // AuthClaims.perms is u64.
  // Rust serializers (serde_json) might serialize u64 as number or stirng depending on settings.
  // It's safer to treat as number for low bits, but if we need high bits we must ensure backend sends string.
  // For this demo let's assume low bits.
  experience?: ExperienceItem[];
}

export interface ExperienceItem {
  id: string;
  title: string;
  organization: string;
  start_date: string; // YYYY-MM
  end_date?: string; // YYYY-MM or 'Present'
  description?: string;
  link?: string;
}

export const useAuthStore = defineStore('auth', () => {
  const token = ref<string | null>(localStorage.getItem('aether_token'));
  const user = ref<User | null>(null);
  const router = useRouter();

  const isAuthenticated = computed(() => !!token.value);

  // Helper to check permissions securely
  function hasPermission(requiredPerm: number) {
    if (!user.value) return false;
    // Note: JS Bitwise works on 32-bit signed integers.
    // If we use high bits (>31), we need BigInt logic.
    // For extensibility demo (bits 8-13), standard bitwise is fine.
    return (user.value.permissions & requiredPerm) === requiredPerm;
  }


  // Initialize axios interceptors
  axios.interceptors.request.use(config => {
    if (token.value) {
      config.headers.Authorization = `Bearer ${token.value}`;
    }
    return config;
  });

  axios.interceptors.response.use(
    (response) => response,
    (error) => {
      if (error.response && error.response.status === 401) {
        // Token expired or invalid
        logout();
      }
      return Promise.reject(error);
    }
  );

  function login(newToken: string, userData: any) {
    token.value = newToken;
    user.value = userData;
    localStorage.setItem('aether_token', newToken);
  }

  function logout() {
    token.value = null;
    user.value = null;
    localStorage.removeItem('aether_token');

    // SAFETY: useRouter() only works inside components.
    // If this store is initialized in a router guard or interceptor context, router might be undefined or fail.
    try {
      if (router) {
        router.push('/login');
      } else {
        throw new Error("Router instance not found");
      }
    } catch (e) {
      // Fallback for non-component context
      window.location.href = '/login';
    }
  }

  function parseJwt(token: string) {
    try {
      const base64Url = token.split('.')[1];
      const base64 = base64Url.replace(/-/g, '+').replace(/_/g, '/');
      const jsonPayload = decodeURIComponent(window.atob(base64).split('').map(function (c) {
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

  return { token, user, isAuthenticated, login, logout, fetchUser, updateUser, hasPermission };
});

