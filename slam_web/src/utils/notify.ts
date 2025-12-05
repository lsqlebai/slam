import axios from 'axios';
import type { AxiosError } from 'axios';

type Listener = (msg: string) => void;

let listeners: Listener[] = [];
let lastMsg = '';
let lastAt = 0;

export function onError(fn: Listener) {
  listeners.push(fn);
  return () => {
    listeners = listeners.filter(l => l !== fn);
  };
}

export function emitError(msg: string) {
  const now = Date.now();
  if (msg === lastMsg && now - lastAt < 3000) return;
  lastMsg = msg;
  lastAt = now;
  for (const l of listeners) l(msg);
}

export function emitHttpError(err: unknown) {
  const e = err as
    | AxiosError
    | {
        status?: number;
        code?: string;
        message?: string;
        config?: { headers?: Record<string, string | number> };
        response?: { status?: number; data?: { error?: string } };
      };
  const status: number | undefined = e?.response?.status ?? e?.status;
  if (status === 401) return;
  if (axios.isCancel?.(err)) return;
  const silentHeader =
    e?.config?.headers?.['X-Silent-Error'] ||
    e?.config?.headers?.['x-silent-error'];
  const is5xx = typeof status === 'number' && status >= 500 && status < 600;
  if (String(silentHeader) === '1' && !is5xx) return;
  if (
    typeof navigator !== 'undefined' &&
    navigator &&
    navigator.onLine === false
  ) {
    emitError('网络不可用');
    return;
  }
  const raw: string =
    ((): string | undefined => {
      const d = e?.response?.data;
      if (d && typeof d === 'object') {
        const m = (d as Record<string, unknown>).error;
        if (typeof m === 'string') return m;
      }
      return undefined;
    })() ||
    e?.message ||
    '网络错误';
  const isTimeoutStatus = status === 504;
  const isTimeout =
    isTimeoutStatus ||
    /timeout|超时|ECONNABORTED/i.test(String(raw)) ||
    e?.code === 'ECONNABORTED';
  const msg = isTimeout
    ? '请求超时，请稍后重试'
    : is5xx
      ? '服务繁忙，请稍后重试'
      : raw;
  emitError(String(msg || '网络错误'));
}
