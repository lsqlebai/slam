import { Capacitor } from '@capacitor/core';
import axios from 'axios';
import { axiosAndroidAdapter } from './axiosCapacitorAdapter';

export const http = axios.create({
  baseURL: '/api',
  withCredentials: true,
  timeout: 15000,
});

if (Capacitor.getPlatform() === 'android' && Capacitor.isNativePlatform()) {
  const apiBase = process.env.MODERN_PUBLIC_API_BASE;
  if (apiBase && /^https?:\/\//.test(apiBase)) {
    http.defaults.baseURL = apiBase;
  }
  http.defaults.adapter = axiosAndroidAdapter;
}

function redirectToLogin() {
  if (typeof window === 'undefined') return;
  const p = window.location?.pathname || '';
  if (p !== '/login') window.location.replace('/login');
}

http.interceptors.response.use(
  res => {
    const status = res?.status;
    if (status === 401) {
      redirectToLogin();
      return Promise.reject(new Error('未登录或会话过期'));
    }
    return res;
  },
  err => {
    const status = err?.response?.status ?? err?.status;
    if (status === 401) {
      redirectToLogin();
      return Promise.reject(new Error('未登录或会话过期'));
    }
    const message = err?.response?.data?.error || err.message || '网络错误';
    return Promise.reject(new Error(message));
  },
);
