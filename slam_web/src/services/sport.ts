import { http } from './http';

export type Swimming = {
  main_stroke: string;
  stroke_avg: number;
  swolf_avg: number;
};

export type Track = {
  distance_meter: number;
  duration_second: number;
  pace_average: string;
  extra: Swimming;
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
  extra: Swimming;
  tracks: Track[];
};

export async function listSports(page = 0, size = 20): Promise<Sport[]> {
  const res = await http.get('/sport/list', { params: { page, size } });
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
  const res = await http.post('/ai/image-parse', formData, {
    headers: { 'Content-Type': 'multipart/form-data' },
    timeout: 300_000,
  });
  return res.data as AIResponse<Sport>;
}

export async function insertSport(sport: Sport): Promise<boolean> {
  const res = await http.post('/sport/insert', sport);
  const data = res.data as { success?: boolean };
  return !!data?.success;
}

export async function updateSport(sport: Sport): Promise<boolean> {
  const res = await http.post('/sport/update', sport);
  const data = res.data as { success?: boolean };
  return !!data?.success;
}

export async function deleteSport(id: number): Promise<boolean> {
  const res = await http.post('/sport/delete', { id });
  const data = res.data as { success?: boolean };
  return !!data?.success;
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
  const data = res.data as { success?: boolean };
  return !!data?.success;
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
};
export type StatSummary = {
  buckets: StatBucket[];
  type_buckets: TypeBucket[];
  total_count: number;
  total_calories: number;
  total_duration_second: number;
  sports: Sport[];
  earliest_year?: number;
};

export async function getSportStats(
  kind: 'year' | 'month' | 'week' | 'total',
  year: number,
  month?: number,
  week?: number,
): Promise<StatSummary> {
  const res = await http.get('/sport/stats', {
    params: { kind, year, month, week },
  });
  return res.data as StatSummary;
}
