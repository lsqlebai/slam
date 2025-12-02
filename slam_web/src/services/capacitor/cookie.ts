// cookieStore.ts
import { Http } from '@capacitor-community/http';

const COOKIE_KEY = 'capacitor_http_cookies_local'; // 存在 localStorage 的 key

type CookieItem = { key: string; value: string };
type CookieMap = Record<string, CookieItem[]>;

function getOrigin(url: string): string {
  try {
    const u = new URL(url);
    return `${u.protocol}//${u.host}`;
  } catch {
    return url;
  }
}

let initPromise: Promise<void> | null = null;

export function initHttp() {
  if (!initPromise) {
    // 只执行一次
    initPromise = restoreAllCookies().catch(err => {
      console.warn('[cookie] restore error', err);
    });
  }
  return initPromise;
}

/**
 * 保存某个 URL 的 cookie 到 localStorage
 */
export async function saveCookiesForUrl(url: string) {
  const origin = getOrigin(url);

  const { cookies } = await Http.getCookies({ url: origin });

  const stored = localStorage.getItem(COOKIE_KEY);
  const map: CookieMap = stored ? JSON.parse(stored) : {};

  map[origin] = cookies;

  localStorage.setItem(COOKIE_KEY, JSON.stringify(map));
}

/**
 * App 启动时恢复所有 cookie 到 @capacitor-community/http
 */
export async function restoreAllCookies() {
  const stored = localStorage.getItem(COOKIE_KEY);
  if (!stored) return;

  const map: CookieMap = JSON.parse(stored);

  for (const [origin, cookies] of Object.entries(map)) {
    for (const c of cookies) {
      await Http.setCookie({
        url: origin,
        key: c.key,
        value: c.value,
      });
    }
  }
}

/**
 * 清空某域的 cookie（退出登录时用）
 */
export async function clearCookiesFor(origin: string) {
  await Http.clearCookies({ url: origin });
  const stored = localStorage.getItem(COOKIE_KEY);
  if (!stored) return;

  const map: CookieMap = JSON.parse(stored);
  delete map[origin];

  localStorage.setItem(COOKIE_KEY, JSON.stringify(map));
}
