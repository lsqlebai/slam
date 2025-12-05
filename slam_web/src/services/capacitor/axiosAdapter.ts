import { CapacitorHttp } from '@capacitor/core';
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

export interface CapacitorMultipartPart {
  key: string;
  value: string; // 普通字段：原始字符串；文件字段：base64（不带 data:xxx 前缀）
  filename?: string;
  type?: string; // MIME type，比如 image/jpeg
}

/**
 * 把 Web 原生 FormData 转成 @capacitor-community/http 需要的 multipart 数组
 */
export async function formDataToMultipart(
  formData: FormData,
): Promise<CapacitorMultipartPart[]> {
  const parts: CapacitorMultipartPart[] = [];

  const tasks: Promise<void>[] = [];

  formData.forEach((value, key) => {
    if (value instanceof Blob) {
      const file = value as File;
      tasks.push(
        (async () => {
          parts.push({
            key: 'base64',
            filename: file.name,
            type: file.type,
            value: await blobToBase64(file),
          });
        })(),
      );
    } else {
      parts.push({ key, value: String(value) });
    }
  });

  await Promise.all(tasks);

  return parts;
}

/**
 * Blob -> base64（仅返回纯 base64 字符串，不包含 `data:...;base64,` 前缀）
 */
function blobToBase64(blob: Blob): Promise<string> {
  return new Promise(resolve => {
    const reader = new FileReader();
    reader.onloadend = () => {
      const result = reader.result as string;
      // result 形如：data:image/jpeg;base64,/9j/4AAQSk...
      const commaIndex = result.indexOf(',');
      if (commaIndex >= 0) {
        resolve(result.substring(commaIndex + 1)); // 去掉前缀
      } else {
        resolve(result);
      }
    };
    reader.readAsDataURL(blob);
  });
}

export const axiosAndroidAdapter = async (
  config: InternalAxiosRequestConfig,
) => {
  const request = {
    url: buildURL(config),
    method: config.method,
    headers: config.headers,
    params: config.params ? config.params : {},
    data: config.data,
  };

  console.log('lsq:3000 data: ', request.data);
  if (request.data) console.log('lsq 3000 multipart: ', request.data.multipart);
  if (!request.url) {
    throw new Error('请求 URL 不能为空');
  }
  // if (request.data instanceof FormData) {
  //   request.data = await formDataToMultipart(request.data);
  //   request.headers.clear();
  // }
  const response = await CapacitorHttp.request({
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
