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

export type UserInfo = { nickname: string };

export async function info(): Promise<UserInfo> {
  const res = await http.get('/user/info');
  return { nickname: String(res.data?.nickname || '') };
}

export async function logout(): Promise<boolean> {
  const res = await http.post('/user/logout');
  return Boolean(res.data?.success);
}
