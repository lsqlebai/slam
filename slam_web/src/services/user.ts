import { http } from './http';

export async function register(
  name: string,
  password: string,
): Promise<boolean> {
  const res = await http.post('/user/register', { name, password });
  return Boolean(res.data?.success);
}

export async function login(name: string, password: string): Promise<boolean> {
  const res = await http.post('/user/login', { name, password });
  return Boolean(res.data?.success);
}
