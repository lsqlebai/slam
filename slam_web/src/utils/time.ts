export const toInputDateTime = (s: number): string => {
  const d = new Date(s * 1000);
  const pad = (n: number) => String(n).padStart(2, '0');
  const y = d.getFullYear();
  const m = pad(d.getMonth() + 1);
  const day = pad(d.getDate());
  const hh = pad(d.getHours());
  const mm = pad(d.getMinutes());
  const ss = pad(d.getSeconds());
  return `${y}-${m}-${day}T${hh}:${mm}:${ss}`;
};

export const toDisplayDateTime = (s: number): string =>
  toInputDateTime(s).replace('T', ' ');

export const fromInputDateTime = (v: string): number => {
  const norm = v.replace(' ', 'T');
  const [date, time] = norm.split('T');
  const [y, m, d] = date.split('-').map(Number);
  const [hh, mm, ss] = time.split(':').map(Number);
  const dt = new Date(y, (m || 1) - 1, d || 1, hh || 0, mm || 0, ss || 0);
  return Math.floor(dt.getTime() / 1000);
};

export const toHMS = (s: number): string => {
  const hh = Math.floor(s / 3600);
  const mm = Math.floor((s % 3600) / 60);
  const ss = s % 60;
  const pad = (n: number) => String(n).padStart(2, '0');
  return `${pad(hh)}:${pad(mm)}:${pad(ss)}`;
};

export const fromHMS = (v: string): number => {
  const parts = v.split(':').map(p => Number.parseInt(p || '0'));
  const [hh, mm, ss] = [parts[0] || 0, parts[1] || 0, parts[2] || 0];
  return hh * 3600 + mm * 60 + ss;
};

export const toLocaleDateTime = (s: number, locale: string): string =>
  new Intl.DateTimeFormat(locale, {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    hour12: false,
  }).format(new Date(s * 1000));

export const toLocaleDate = (s: number, locale: string): string =>
  new Intl.DateTimeFormat(locale, {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
  }).format(new Date(s * 1000));
