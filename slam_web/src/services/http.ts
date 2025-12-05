import { Capacitor } from '@capacitor/core';
import axios from 'axios';
import { emitHttpError } from '../utils/notify';
import { axiosAndroidAdapter } from './capacitor/axiosAdapter';
import { initHttp } from './capacitor/cookie';

export const http = axios.create({
  baseURL: '/api',
  withCredentials: true,
  timeout: 15000,
});

if (Capacitor.getPlatform() === 'android' && Capacitor.isNativePlatform()) {
  // 🟢 关键：在 App 启动时恢复所有 cookie
  // 🔥 request 拦截器： 所有请求都等 init 完成
  // http.interceptors.request.use(async config => {
  //   await initHttp(); // 等 cookie 恢复完成
  //   return config;
  // });

  const apiBase = process.env.MODERN_PUBLIC_API_BASE;
  if (apiBase && /^https?:\/\//.test(apiBase)) {
    http.defaults.baseURL = apiBase;
  }
  //http.defaults.adapter = axiosAndroidAdapter;
}

function redirectToLogin() {
  if (typeof window === 'undefined') return;
  const p = window.location?.pathname || '';
  if (p !== '/login') window.location.replace('/login');
}

// 🔥 response 拦截器：处理 401 错误
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
    emitHttpError(err);
    const message = err?.response?.data?.error || err.message || '网络错误';
    return Promise.reject(new Error(message));
  },
);
