import { FormatListNumbered } from '@mui/icons-material';
import { Box } from '@mui/material';
import type { Lang } from '../../i18n';
import type { Sport } from '../../services/sport';
import SportListTitle from '../common/SportListTitle';
import SportList from '../sport/SportList';
import StatsBarCard from './StatsBarCard';
import SummaryStats from './SummaryStats';

export default function StatsFilterSection({
  lang,
  totals,
  title,
  data,
  details,
  sports,
  barMaxWidth,
  hideZero,
}: {
  lang: Lang;
  totals: {
    duration: number;
    calories: number;
    count: number;
    distanceMeter: number;
  };
  title: string;
  data: { label: string; value: number }[];
  details?: Record<
    string,
    { duration: number; count: number; calories: number }
  >;
  sports: Sport[];
  barMaxWidth?: number;
  hideZero?: boolean;
}) {
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
          }
        }
        return res;
      })()
    : data;
  return (
    <Box sx={{ mt: 0 }}>
      <Box
        sx={{
          display: 'grid',
          gridTemplateColumns: { xs: '1fr', md: 'repeat(2, minmax(0, 500px))' },
          columnGap: 2,
          rowGap: 2,
          alignItems: 'stretch',
          justifyItems: { xs: 'center', md: 'start' },
          justifyContent: { xs: 'center', md: 'start' },
          width: { xs: '100%', md: 'fit-content' },
          maxWidth: { xs: 500, md: 'none' },
          mx: { xs: 'auto', md: 0 },
        }}
      >
        <Box sx={{ width: '100%' }}>
          <SummaryStats
            lang={lang}
            durationSeconds={totals.duration}
            calories={totals.calories}
            count={totals.count}
            distanceMeter={totals.distanceMeter}
          />
        </Box>
        <Box sx={{ width: '100%' }}>
          <StatsBarCard
            lang={lang}
            title={title}
            data={filtered}
            details={details}
            barMaxWidth={bw}
            maxBarHeight={120}
          />
        </Box>
      </Box>
      <SportListTitle
        icon={<FormatListNumbered />}
        label={lang === 'zh' ? '运动记录' : 'Sport Records'}
        labelVariant="subtitle1"
        labelWeight={700}
        iconColor="text.primary"
        labelColor="text.primary"
        align="center"
        containerSx={{
          mt: 2,
          mb: { xs: 0, md: 0 },
          py: { xs: 0.25, md: 0.25 },
        }}
      />
      <Box sx={{ mt: { xs: 0.25, md: 0.25 } }}>
        <SportList lang={lang} items={sports} />
      </Box>
    </Box>
  );
}
