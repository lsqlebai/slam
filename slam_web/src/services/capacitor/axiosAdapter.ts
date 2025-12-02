import { Http } from '@capacitor-community/http';
import type { InternalAxiosRequestConfig } from 'axios';
import { saveCookiesForUrl } from './cookie';

function isAbsoluteURL(url?: string) {
  if (!url) return false;
  return /^https?:\/\//.test(url);
}
function combineURLs(baseURL?: string, relativeURL?: string) {
  const b = (baseURL || '').replace(/\/+$/, '');
  const r = (relativeURL || '').replace(/^\/+/, '');
  if (!b) return r;
  if (!r) return b;
  return `${b}/${r}`;
}
function buildURL(config: InternalAxiosRequestConfig) {
  const url = config.url || '';
  if (isAbsoluteURL(url)) {
    return url;
  }
  return combineURLs(config.baseURL, url);
}

export const axiosAndroidAdapter = async (
  config: InternalAxiosRequestConfig,
) => {
  const request = {
    url: buildURL(config),
    method: config.method,
    headers: config.headers,
    params: config.params ? config.params : {},
    data: config.data ? config.data : {},
  };

  if (!request.url) {
    throw new Error('请求 URL 不能为空');
  }
  const response = await Http.request({
    ...request,
    url: request.url,
    method: request.method || 'GET',
  });

  // 🟢 关键：同步 cookie 到 localStorage
  saveCookiesForUrl(request.url).catch(err =>
    console.warn('[cookie] save error', err),
  );

  return {
    data: response.data,
    status: response.status,
    statusText: '',
    headers: response.headers,
    config,
    request: {},
  };
};
