import {
  AccessTime,
  FormatListNumbered,
  LocalFireDepartment,
  Map as MapIcon,
} from '@mui/icons-material';
import { Box, Paper } from '@mui/material';
import type { Lang } from '../../i18n';
import { TEXTS } from '../../i18n';
import SportField from '../common/SportField';

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
        maxWidth: 500,
        width: '100%',
        mx: 0,
        height: { md: '100%' },
      }}
    >
      <Box
        sx={{
          display: 'grid',
          gridTemplateColumns: {
            xs: 'repeat(2, 1fr)',
            sm: 'repeat(2, 1fr)',
            md: '1fr',
          },
          columnGap: { xs: 1, sm: 2 },
          rowGap: { xs: 0.75, sm: 1 },
          height: { md: '100%' },
          alignContent: { md: 'center' },
        }}
      >
        <SportField
          icon={<AccessTime />}
          label={TEXTS[lang].home.labels.time}
          value={formatDurationHMS(durationSeconds)}
          iconColor="text.primary"
          labelColor="text.secondary"
          responsive
        />
        <SportField
          icon={<FormatListNumbered />}
          label={TEXTS[lang].home.labels.count}
          value={count}
          iconColor="text.secondary"
          labelColor="text.secondary"
          responsive
        />
        <SportField
          icon={<MapIcon />}
          label={TEXTS[lang].home.labels.distance}
          value={formatDistance(distanceMeter)}
          iconColor="text.primary"
          labelColor="text.secondary"
          noWrap
          responsive
        />
        <SportField
          icon={<LocalFireDepartment />}
          label={TEXTS[lang].home.labels.calories}
          value={calories}
          iconColor="error.main"
          labelColor="text.secondary"
          responsive
        />
      </Box>
    </Paper>
  );
}
