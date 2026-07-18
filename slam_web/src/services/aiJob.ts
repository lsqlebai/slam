import { http } from './http';
import type { Sport } from './sport';

export type AIJobStatus =
  | 'queued'
  | 'running'
  | 'ready'
  | 'failed'
  | 'submitted';

export type AIJobAsset = {
  id: string;
  mime: string;
  position: number;
  created_at: number;
  deleted_at?: number;
};

export type AIJob = {
  id: string;
  status: AIJobStatus;
  result?: Sport;
  error_code?: string;
  error_message?: string;
  attempts: number;
  next_attempt_at?: number;
  submitted_sport_id?: number;
  created_at: number;
  started_at?: number;
  finished_at?: number;
  submitted_at?: number;
  assets: AIJobAsset[];
};

export async function createAIJob(formData: FormData): Promise<AIJob> {
  const response = await http.post('/ai/jobs', formData, {
    headers: { 'Content-Type': 'multipart/form-data' },
    timeout: 120_000,
  });
  return response.data as AIJob;
}

export async function listAIJobs(page = 0, size = 50): Promise<AIJob[]> {
  const response = await http.get('/ai/jobs', {
    params: { page, size },
    headers: { 'X-Silent-Error': '1' },
  });
  return Array.isArray(response.data) ? response.data : [];
}

export async function getAIJob(id: string): Promise<AIJob> {
  const response = await http.get(`/ai/jobs/${encodeURIComponent(id)}`);
  return response.data as AIJob;
}

export async function retryAIJob(id: string): Promise<void> {
  await http.post(`/ai/jobs/${encodeURIComponent(id)}/retry`);
}

export async function getAIAssetBlob(
  id: string,
  kind: 'thumbnail' | 'content',
): Promise<Blob> {
  const response = await http.get(
    `/ai/assets/${encodeURIComponent(id)}/${kind}`,
    { responseType: 'blob', headers: { 'X-Silent-Error': '1' } },
  );
  return response.data as Blob;
}
