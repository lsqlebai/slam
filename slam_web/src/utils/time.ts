export const toInputDateTime = (
  s: number,
  tz: 'local' | 'UTC' = 'UTC',
): string => {
  const d = new Date(s * 1000);
  const pad = (n: number) => String(n).padStart(2, '0');
  const yyyy = tz === 'UTC' ? d.getUTCFullYear() : d.getFullYear();
  const mm = pad((tz === 'UTC' ? d.getUTCMonth() : d.getMonth()) + 1);
  const dd = pad(tz === 'UTC' ? d.getUTCDate() : d.getDate());
  const hh = pad(tz === 'UTC' ? d.getUTCHours() : d.getHours());
  const mi = pad(tz === 'UTC' ? d.getUTCMinutes() : d.getMinutes());
  const ss = pad(tz === 'UTC' ? d.getUTCSeconds() : d.getSeconds());
  return `${yyyy}-${mm}-${dd}T${hh}:${mi}:${ss}`;
};

export const toDisplayDateTime = (
  s: number,
  tz: 'local' | 'UTC' = 'UTC',
): string => toInputDateTime(s, tz).replace('T', ' ');

export const fromInputDateTime = (
  v: string,
  tz: 'local' | 'UTC' = 'UTC',
): number => {
  const norm = v.replace(' ', 'T');
  const [date, time] = norm.split('T');
  const [y, m, d] = date.split('-').map(Number);
  const [hh, mm, ss] = time.split(':').map(Number);
  const dt =
    tz === 'UTC'
      ? new Date(
          Date.UTC(y || 0, (m || 1) - 1, d || 1, hh || 0, mm || 0, ss || 0),
        )
      : new Date(y || 0, (m || 1) - 1, d || 1, hh || 0, mm || 0, ss || 0);
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
