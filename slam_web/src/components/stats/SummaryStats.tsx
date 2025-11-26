import {
  AccessTime,
  FormatListNumbered,
  LocalFireDepartment,
  Map as MapIcon,
} from '@mui/icons-material';
import { Box, Paper, Typography } from '@mui/material';
import type { Lang } from '../../i18n';
import { TEXTS } from '../../i18n';

export default function SummaryStats({
  lang,
  durationSeconds,
  calories,
  count,
  distanceMeter,
}: {
  lang: Lang;
  durationSeconds: number;
  calories: number;
  count: number;
  distanceMeter: number;
}) {
  const formatDurationHMS = (s: number) => {
    const h = Math.floor(s / 3600);
    const m = Math.floor((s % 3600) / 60);
    const sec = s % 60;
    const pad = (n: number) => String(n).padStart(2, '0');
    return `${pad(h)}:${pad(m)}:${pad(sec)}`;
  };
  const formatDistance = (m: number) =>
    m >= 1000 ? `${(m / 1000).toFixed(2)} km` : `${m} m`;

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
          gridTemplateColumns: 'repeat(2, 1fr)',
          columnGap: 2,
          rowGap: 1,
        }}
      >
        <Box sx={{ display: 'flex', alignItems: 'center' }}>
          <AccessTime
            fontSize="small"
            sx={{ color: 'text.primary', mr: 0.5 }}
          />
          <Typography variant="caption" color="text.secondary" sx={{ mr: 0.5 }}>
            {TEXTS[lang].home.labels.time}:
          </Typography>
          <Typography variant="subtitle1" sx={{ fontWeight: 700 }}>
            {formatDurationHMS(durationSeconds)}
          </Typography>
        </Box>
        <Box sx={{ display: 'flex', alignItems: 'center' }}>
          <FormatListNumbered
            fontSize="small"
            sx={{ color: 'text.secondary', mr: 0.5 }}
          />
          <Typography variant="caption" color="text.secondary" sx={{ mr: 0.5 }}>
            {TEXTS[lang].home.labels.count}:
          </Typography>
          <Typography variant="subtitle1" sx={{ fontWeight: 700 }}>
            {count}
          </Typography>
        </Box>
        <Box sx={{ display: 'flex', alignItems: 'center', flexWrap: 'nowrap' }}>
          <MapIcon fontSize="small" sx={{ color: 'text.primary', mr: 0.5 }} />
          <Typography
            variant="caption"
            color="text.secondary"
            sx={{ mr: 0.5 }}
            noWrap
          >
            {TEXTS[lang].home.labels.distance}:
          </Typography>
          <Typography variant="subtitle1" sx={{ fontWeight: 700 }} noWrap>
            {formatDistance(distanceMeter)}
          </Typography>
        </Box>
        <Box sx={{ display: 'flex', alignItems: 'center' }}>
          <LocalFireDepartment
            fontSize="small"
            sx={{ color: 'error.main', mr: 0.5 }}
          />
          <Typography variant="caption" color="text.secondary" sx={{ mr: 0.5 }}>
            {TEXTS[lang].home.labels.calories}:
          </Typography>
          <Typography variant="subtitle1" sx={{ fontWeight: 700 }}>
            {calories}
          </Typography>
        </Box>
      </Box>
    </Paper>
  );
}
