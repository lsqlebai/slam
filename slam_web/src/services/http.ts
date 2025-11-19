import axios from 'axios';

export const http = axios.create({
  baseURL: '/api',
  withCredentials: true,
  timeout: 15000,
});

http.interceptors.response.use(
  res => res,
  err => {
    const message = err?.response?.data?.error || err.message || '网络错误';
    return Promise.reject(new Error(message));
  },
);
