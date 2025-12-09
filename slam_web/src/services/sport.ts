import { Capacitor } from '@capacitor/core';
import { emitError } from '../utils/notify';
import { http } from './http';

export type Swimming = {
  main_stroke: string;
  stroke_avg: number;
  swolf_avg: number;
};

// 参考后端 `Running` 结构定义
export type Running = {
  speed_avg: number;
  cadence_avg: number;
  stride_length_avg: number;
  steps_total: number;
  pace_min: string;
  pace_max: string;
};

// 与后端 `SportExtra` (serde untagged) 对齐：无显式类型标签的联合
export type SportExtra = Swimming | Running;

export type Track = {
  distance_meter: number;
  duration_second: number;
  pace_average: string;
  extra?: SportExtra;
};

export type Sport = {
  id: number;
  type: string;
  start_time: number;
  calories: number;
  distance_meter: number;
  duration_second: number;
  heart_rate_avg: number;
  heart_rate_max: number;
  pace_average: string;
  extra?: SportExtra;
  tracks: Track[];
};

// 运动类型枚举与判断（基于字符串）
export enum SportType {
  Unknown = 'Unknown',
  Swimming = 'Swimming',
  Running = 'Running',
  Cycling = 'Cycling',
}

export function getSportType(typeOrSport: string | Sport | undefined): SportType {
  const raw = typeof typeOrSport === 'string' ? typeOrSport : typeOrSport?.type;
  const key = String(raw || '').toLowerCase();
  if (key.includes('swim')) return SportType.Swimming;
  if (key.includes('run')) return SportType.Running;
  if (key.includes('cycle') || key.includes('bike')) return SportType.Cycling;
  return SportType.Unknown;
}

export function isSwimmingType(type: string | undefined): boolean {
  return getSportType(type) === SportType.Swimming;
}
export function isRunningType(type: string | undefined): boolean {
  return getSportType(type) === SportType.Running;
}

export async function listSports(page = 0, size = 20): Promise<Sport[]> {
  const res = await http.get('/sport/list', {
    params: { page, size },
    headers: { 'X-Silent-Error': '1' },
  });
  return Array.isArray(res.data) ? res.data : [];
}

export type AIError = { code: string; message: string; details?: string };
export type AIResponse<T> = {
  success: boolean;
  data?: T;
  error?: AIError;
  request_id: string;
};

export async function recognizeImages(
  formData: FormData,
): Promise<AIResponse<Sport>> {
  const requesst = formData;
  // if (Capacitor.getPlatform() === 'android' && Capacitor.isNativePlatform()) {
  //   const tmpFormData: FormData = new FormData();
  //   formData.forEach((value, key) => {
  //     if (key === 'image') {
  //       tmpFormData.append('base64', value);
  //     } else {
  //       tmpFormData.append(key, value);
  //     }
  //   });
  //   requesst = tmpFormData;
  // }
  const res = await http.post('/ai/image-parse', requesst, {
    headers: { 'Content-Type': 'multipart/form-data' },
    timeout: 300_000,
  });
  return res.data as AIResponse<Sport>;
}

export async function insertSport(sport: Sport): Promise<boolean> {
  const res = await http.post('/sport/insert', sport);
  const data = res.data as { success?: boolean; error?: string };
  if (data?.success) return true;
  const msg = String(data?.error || '提交失败');
  emitError(msg);
  throw new Error(msg);
}

export async function updateSport(sport: Sport): Promise<boolean> {
  const res = await http.post('/sport/update', sport);
  const data = res.data as { success?: boolean; error?: string };
  if (data?.success) return true;
  const msg = String(data?.error || '提交失败');
  emitError(msg);
  throw new Error(msg);
}

export async function deleteSport(id: number): Promise<boolean> {
  const res = await http.post('/sport/delete', { id });
  const data = res.data as { success?: boolean; error?: string };
  if (data?.success) return true;
  const msg = String(data?.error || '删除失败');
  emitError(msg);
  throw new Error(msg);
}

export async function importSportsCsv(
  file: File,
  vendor: string,
): Promise<boolean> {
  const form = new FormData();
  form.append('file', file);
  form.append('vendor', vendor);
  const res = await http.post('/sport/import', form, {
    headers: { 'Content-Type': 'multipart/form-data' },
    timeout: 120_000,
  });
  const data = res.data as { success?: boolean; error?: string };
  if (data?.success) return true;
  const msg = String(data?.error || '上传失败');
  emitError(msg);
  throw new Error(msg);
}

export type StatBucket = {
  date: number;
  duration: number;
  calories: number;
  count: number;
};
export type TypeBucket = {
  type: string;
  duration: number;
  calories: number;
  count: number;
  distance_meter: number;
};
export type StatSummary = {
  buckets: StatBucket[];
  type_buckets: TypeBucket[];
  total_count: number;
  total_calories: number;
  total_duration_second: number;
  total_distance_meter: number;
  sports: Sport[];
  earliest_year?: number;
};

export async function getSportStats(
  kind: 'year' | 'month' | 'week' | 'total',
  year: number,
  month?: number,
  week?: number,
  signal?: AbortSignal,
): Promise<StatSummary> {
  const res = await http.get('/sport/stats', {
    params: { kind, year, month, week },
    signal,
    headers: { 'X-Silent-Error': '1' },
  });
  return res.data as StatSummary;
}
