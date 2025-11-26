import { http } from './http';

export async function register(
  name: string,
  password: string,
  nickname: string,
): Promise<boolean> {
  const res = await http.post('/user/register', { name, password, nickname });
  return Boolean(res.data?.success);
}

export async function login(name: string, password: string): Promise<boolean> {
  const res = await http.post('/user/login', { name, password });
  return Boolean(res.data?.success);
}

export type UserInfo = { nickname: string; avatar: string };

export async function info(): Promise<UserInfo> {
  const res = await http.get('/user/info');
  return {
    nickname: String(res.data?.nickname || ''),
    avatar: String(res.data?.avatar || ''),
  };
}

export async function logout(): Promise<boolean> {
  const res = await http.post('/user/logout');
  return Boolean(res.data?.success);
}

// removed getAvatar: avatar is provided by /user/info

export async function uploadAvatar(fileOrBase64: File | string): Promise<string> {
  const fd = new FormData();
  if (typeof fileOrBase64 === 'string') {
    fd.append('avatar', fileOrBase64);
  } else {
    fd.append('file', fileOrBase64);
  }
  const res = await http.post('/user/avatar/upload', fd, {
    headers: { 'Content-Type': 'multipart/form-data' },
    timeout: 60_000,
  });
  if (res.data?.success && typeof res.data?.avatar === 'string') {
    return res.data.avatar as string;
  }
  throw new Error('上传失败');
}
