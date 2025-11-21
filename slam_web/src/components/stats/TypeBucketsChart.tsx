import { Box, Stack, Typography } from '@mui/material';
import { BarChart } from '@mui/x-charts/BarChart';
import {
  ChartsTooltipContainer,
  type ChartsTooltipProps,
  useItemTooltip,
} from '@mui/x-charts/ChartsTooltip';
import type { Lang } from '../../i18n';
import { TEXTS } from '../../i18n';
import type { TypeBucket } from '../../services/sport';

export default function TypeBucketsChart({
  lang,
  buckets,
  barMaxWidth,
}: {
  lang: Lang;
  buckets: TypeBucket[];
  barMaxWidth?: number;
}) {
  const dataset = buckets.map(b => ({
    label: typeLabelFor(lang, b.type),
    calories: b.calories,
    duration: b.duration,
    count: b.count,
  }));
  const CustomTooltip = (props: ChartsTooltipProps) => {
    const item = useItemTooltip<'bar'>();
    const idx = item?.identifier?.dataIndex ?? -1;
    const it = idx >= 0 ? dataset[idx] : undefined;
    const caloriesTitle = TEXTS[lang].home.labels.calories;
    const timeTitle = TEXTS[lang].home.labels.time;
    const countTitle = TEXTS[lang].home.labels.count;
    return (
      <ChartsTooltipContainer {...props} trigger="item" anchor="pointer">
        {item && it ? (
          <Box
            sx={{
              bgcolor: 'background.paper',
              borderRadius: 1,
              boxShadow: '0 2px 10px rgba(0,0,0,0.12)',
              border: '1px solid',
              borderColor: 'divider',
              p: 1,
            }}
          >
            <Stack spacing={0.5}>
              <Typography variant="caption" sx={{ fontWeight: 700 }}>
                {it.label}
              </Typography>
              <Typography variant="caption">
                {caloriesTitle}{' '}
                <Box
                  component="span"
                  sx={{ fontWeight: 800, color: 'text.primary' }}
                >
                  {item.formattedValue ?? `${it.calories} Kcal`}
                </Box>
              </Typography>
              <Typography variant="caption">
                {timeTitle}{' '}
                <Box
                  component="span"
                  sx={{ fontWeight: 800, color: 'text.primary' }}
                >
                  {formatDurationHMS(it.duration)}
                </Box>
              </Typography>
              <Typography variant="caption">
                {countTitle}{' '}
                <Box
                  component="span"
                  sx={{ fontWeight: 800, color: 'text.primary' }}
                >
                  {it.count}
                </Box>
              </Typography>
            </Stack>
          </Box>
        ) : null}
      </ChartsTooltipContainer>
    );
  };

  return (
    <Stack
      spacing={1}
      sx={{
        p: 1,
        pl: 0,
        borderRadius: 2,
        bgcolor: '#fff',
        boxShadow: '0 2px 10px rgba(0,0,0,0.08)',
      }}
    >
      <Box sx={{ height: 240 }}>
        <BarChart
          dataset={dataset}
          layout="horizontal"
          yAxis={[
            {
              scaleType: 'band',
              dataKey: 'label',
              position: 'left',
              tickLabelStyle: { fontSize: 10 },
              width: 30,
              barGapRatio: 0.2,
              categoryGapRatio: 0.6,
              tickLabelPlacement: 'middle',
            },
          ]}
          xAxis={[
            { label: TEXTS[lang].home.labels.calories, position: 'bottom' },
          ]}
          series={[
            {
              dataKey: 'calories',
              valueFormatter: (v: number | null, ctx?: unknown) => {
                const dataIndex = (ctx as { dataIndex?: number } | undefined)
                  ?.dataIndex;
                const idx = (dataIndex ?? -1) as number;
                const it = dataset[idx];
                const val = v ?? 0;
                return `${val} Kcal`;
              },
            },
          ]}
          height={240}
          slots={{ tooltip: CustomTooltip }}
          slotProps={{ tooltip: { position: 'top' } }}
        />
      </Box>
    </Stack>
  );
}

function typeLabelFor(lang: Lang, t: string): string {
  const key = t.toLowerCase();
  if (key.includes('swim')) return TEXTS[lang].addsports.optSwimming;
  if (key.includes('run')) return TEXTS[lang].addsports.optRunning;
  if (key.includes('bike') || key.includes('cycl'))
    return TEXTS[lang].addsports.optCycling;
  if (key.includes('unknown')) return TEXTS[lang].addsports.optUnknown;
  return t;
}

function formatDurationHMS(s: number): string {
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  const sec = s % 60;
  const pad = (n: number) => String(n).padStart(2, '0');
  return `${pad(h)}:${pad(m)}:${pad(sec)}`;
}
