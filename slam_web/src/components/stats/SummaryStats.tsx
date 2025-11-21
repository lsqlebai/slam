import {
  AccessTime,
  FormatListNumbered,
  LocalFireDepartment,
} from '@mui/icons-material';
import { Box, Paper, Typography } from '@mui/material';
import type { Lang } from '../../i18n';
import { TEXTS } from '../../i18n';

export default function SummaryStats({
  lang,
  durationSeconds,
  calories,
  count,
}: {
  lang: Lang;
  durationSeconds: number;
  calories: number;
  count: number;
}) {
  const formatDurationHMS = (s: number) => {
    const h = Math.floor(s / 3600);
    const m = Math.floor((s % 3600) / 60);
    const sec = s % 60;
    const pad = (n: number) => String(n).padStart(2, '0');
    return `${pad(h)}:${pad(m)}:${pad(sec)}`;
  };

  return (
    <Paper
      sx={{
        p: 1.5,
        borderRadius: 2,
        boxShadow: '0 2px 10px rgba(0,0,0,0.08)',
        minHeight: 32,
      }}
    >
      <Box
        sx={{
          display: 'grid',
          gridTemplateColumns: 'repeat(3, 1fr)',
          columnGap: 2,
          rowGap: 1,
        }}
      >
        <Box
          sx={{
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'flex-start',
          }}
        >
          <Typography variant="caption" color="text.secondary">
            {TEXTS[lang].home.labels.time}
          </Typography>
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 0.5 }}>
            <AccessTime fontSize="small" sx={{ color: 'text.secondary' }} />
            <Typography variant="subtitle1" sx={{ fontWeight: 700 }}>
              {formatDurationHMS(durationSeconds)}
            </Typography>
          </Box>
        </Box>
        <Box
          sx={{
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
          }}
        >
          <Typography variant="caption" color="text.secondary">
            {TEXTS[lang].home.labels.calories}
          </Typography>
          <Box
            sx={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'flex-start',
              gap: 0.5,
            }}
          >
            <LocalFireDepartment
              fontSize="small"
              sx={{ color: 'text.secondary' }}
            />
            <Typography
              variant="subtitle1"
              sx={{ fontWeight: 700, textAlign: 'center', width: '100%' }}
            >
              {calories}
            </Typography>
          </Box>
        </Box>
        <Box
          sx={{
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'flex-end',
          }}
        >
          <Typography variant="caption" color="text.secondary">
            {TEXTS[lang].home.labels.count}
          </Typography>
          <Box
            sx={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'flex-start',
              gap: 0.5,
            }}
          >
            <FormatListNumbered
              fontSize="small"
              sx={{ color: 'text.secondary' }}
            />
            <Typography
              variant="subtitle1"
              sx={{ fontWeight: 700, textAlign: 'right', width: '100%' }}
            >
              {count}
            </Typography>
          </Box>
        </Box>
      </Box>
    </Paper>
  );
}
