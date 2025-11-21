import { Box, Stack, Tooltip, Typography } from '@mui/material';
import { useRef, useState } from 'react';
import { TEXTS } from '../../i18n';
import type { Lang } from '../../i18n';
import type { Sport } from '../../services/sport';
import SportList from '../sport/SportList';
import SummaryStats from './SummaryStats';

export default function StatsSection({
  lang,
  totals,
  title,
  data,
  sports,
  barMaxWidth,
  hideZero,
  details,
}: {
  lang: Lang;
  totals: { duration: number; calories: number; count: number };
  title: string;
  data: { label: string; value: number }[];
  sports: Sport[];
  barMaxWidth?: number;
  hideZero?: boolean;
  details?: Record<
    string,
    { duration: number; count: number; calories: number }
  >;
}) {
  const [hovered, setHovered] = useState<string | null>(null);
  const timerRef = useRef<number | null>(null);
  const startHoverTimer = (key: string) => {
    if (timerRef.current) window.clearTimeout(timerRef.current);
    timerRef.current = window.setTimeout(() => {
      setHovered(key);
    }, 100);
  };
  const cancelHover = (key: string) => {
    if (timerRef.current) {
      window.clearTimeout(timerRef.current);
      timerRef.current = null;
    }
    setHovered(h => (h === key ? null : h));
  };
  const max = Math.max(0, ...data.map(x => x.value));
  const maxBarHeight = 120;
  const bw = barMaxWidth ?? 24;
  const filtered = hideZero
    ? (() => {
        const res: { label: string; value: number }[] = [];
        let i = 0;
        while (i < data.length) {
          const d = data[i];
          if (d.value !== 0) {
            res.push(d);
            i++;
          } else {
            const runStart = i;
            let runLen = 0;
            while (i < data.length && data[i].value === 0) {
              runLen++;
              i++;
            }
            const groups = Math.floor(runLen / 3);
            for (let g = 0; g < groups; g++) {
              const keepIdx = runStart + g * 3;
              res.push(data[keepIdx]);
            }
            // leftover < 3 zeros are ignored
          }
        }
        return res;
      })()
    : data;

  return (
    <Box>
      <Box sx={{ mb: 2 }}>
        <SummaryStats
          lang={lang}
          durationSeconds={totals.duration}
          calories={totals.calories}
          count={totals.count}
        />
      </Box>
      <Stack
        spacing={1}
        sx={{
          p: 1,
          borderRadius: 2,
          bgcolor: '#fff',
          boxShadow: '0 2px 10px rgba(0,0,0,0.08)',
        }}
      >
        <Typography variant="subtitle1" sx={{ fontWeight: 700 }}>
          {title}
        </Typography>
        <Box
          sx={{
            display: 'flex',
            alignItems: 'flex-end',
            gap: 1,
            height: 160,
            px: 0.5,
          }}
        >
          {filtered.map(d => {
            const h =
              max > 0
                ? Math.max(4, Math.round((d.value / max) * maxBarHeight))
                : 4;
            const info = details?.[d.label];
            const durationLabel = TEXTS[lang].home.labels.time;
            const countLabel = TEXTS[lang].home.labels.count;
            const caloriesLabel = TEXTS[lang].home.labels.calories;
            const duration = formatDurationHMS(info?.duration ?? 0);
            const count = String(info?.count ?? 0);
            const calories = String(info?.calories ?? 0);
            const title = (
              <Stack spacing={0.5}>
                <Typography variant="caption">{`${durationLabel} ${duration}`}</Typography>
                <Typography variant="caption">{`${countLabel} ${count}`}</Typography>
                <Typography variant="caption">{`${caloriesLabel} ${calories} Kcal`}</Typography>
              </Stack>
            );
            return (
              <Tooltip
                key={d.label}
                title={title}
                arrow
                open={hovered === d.label}
                disableHoverListener
                disableFocusListener
                disableTouchListener
              >
                <Box
                  sx={{
                    display: 'flex',
                    flexDirection: 'column',
                    alignItems: 'center',
                    flex: 1,
                    cursor: 'pointer',
                  }}
                >
                  <Box
                    sx={{
                      width: '100%',
                      maxWidth: bw,
                      height: h,
                      bgcolor: 'primary.main',
                      borderRadius: 1,
                      transition: 'background-color 0.2s ease',
                      '&:hover': { bgcolor: 'primary.dark' },
                    }}
                    onMouseEnter={() => startHoverTimer(d.label)}
                    onMouseLeave={() => cancelHover(d.label)}
                  />
                  <Typography
                    variant="caption"
                    color="text.secondary"
                    sx={{ mt: 0.5 }}
                  >
                    {d.label}
                  </Typography>
                </Box>
              </Tooltip>
            );
          })}
        </Box>
      </Stack>
      <Typography variant="subtitle1" sx={{ fontWeight: 700, mt: 2, mb: 0.5 }}>
        {lang === 'zh' ? '运动记录' : 'Sport Records'}
      </Typography>
      <Box sx={{ mt: 1 }}>
        <SportList lang={lang} items={sports} />
      </Box>
    </Box>
  );
}
const formatDurationHMS = (s: number) => {
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  const sec = s % 60;
  const pad = (n: number) => String(n).padStart(2, '0');
  return `${pad(h)}:${pad(m)}:${pad(sec)}`;
};
