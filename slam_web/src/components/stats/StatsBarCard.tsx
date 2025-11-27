import { Box, Stack, Tooltip, Typography } from '@mui/material';
import { useRef, useState } from 'react';
import { TEXTS } from '../../i18n';
import type { Lang } from '../../i18n';

export default function StatsBarCard({
  lang,
  title,
  data,
  details,
  barMaxWidth,
  maxBarHeight = 120,
}: {
  lang: Lang;
  title: string;
  data: { label: string; value: number }[];
  details?: Record<
    string,
    { duration: number; count: number; calories: number }
  >;
  barMaxWidth?: number;
  maxBarHeight?: number;
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
  const bw = barMaxWidth ?? 24;

  return (
    <Stack
      spacing={1}
      sx={{
        p: 1,
        borderRadius: 2,
        bgcolor: '#fff',
        boxShadow: '0 2px 10px rgba(0,0,0,0.08)',
        maxWidth: 500,
        width: '100%',
        mx: 0,
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
        {data.map(d => {
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
          const tip = (
            <Stack spacing={0.5}>
              <Typography variant="caption">{`${durationLabel} ${duration}`}</Typography>
              <Typography variant="caption">{`${countLabel} ${count}`}</Typography>
              <Typography variant="caption">{`${caloriesLabel} ${calories} Kcal`}</Typography>
            </Stack>
          );
          return (
            <Tooltip
              key={d.label}
              title={tip}
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
  );
}

const formatDurationHMS = (s: number) => {
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  const sec = s % 60;
  const pad = (n: number) => String(n).padStart(2, '0');
  return `${pad(h)}:${pad(m)}:${pad(sec)}`;
};
