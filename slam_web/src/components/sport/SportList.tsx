import { useNavigate } from '@modern-js/runtime/router';
import {
  AccessTime,
  AltRoute,
  AvTimer,
  DirectionsBike,
  DirectionsRun,
  DirectionsWalk,
  HelpOutline,
  LocalFireDepartment,
  Pool,
} from '@mui/icons-material';
import { Box, Card, CardContent, Stack } from '@mui/material';
import type { Lang } from '../../i18n';
import { TEXTS } from '../../i18n';
import type { Sport } from '../../services/sport';
import SportField from '../common/SportField';

export default function SportList({
  lang,
  items,
  onItemClick,
}: {
  lang: Lang;
  items: Sport[];
  onItemClick?: (s: Sport) => void;
}) {
  const navigate = useNavigate();

  const formatDurationHMS = (s: number) => {
    const h = Math.floor(s / 3600);
    const m = Math.floor((s % 3600) / 60);
    const sec = s % 60;
    const pad = (n: number) => String(n).padStart(2, '0');
    return `${pad(h)}:${pad(m)}:${pad(sec)}`;
  };
  const formatDateTimeFull = (t: number) => {
    const d = new Date(t * 1000);
    const pad = (n: number) => String(n).padStart(2, '0');
    const y = d.getFullYear();
    const mo = pad(d.getMonth() + 1);
    const da = pad(d.getDate());
    const h = pad(d.getHours());
    const mi = pad(d.getMinutes());
    const s = pad(d.getSeconds());
    return `${y}-${mo}-${da} ${h}:${mi}:${s}`;
  };

  const TypeLabelFor = (t: string) => {
    const key = t.toLowerCase();
    if (key.includes('swim')) return TEXTS[lang].addsports.optSwimming;
    if (key.includes('run')) return TEXTS[lang].addsports.optRunning;
    if (key.includes('bike') || key.includes('cycl'))
      return TEXTS[lang].addsports.optCycling;
    if (key.includes('unknown')) return TEXTS[lang].addsports.optUnknown;
    return t;
  };

  return (
    <Box
      sx={{
        display: 'grid',
        gridTemplateColumns: {
          xs: '1fr',
          md: 'repeat(2, minmax(0, 500px))',
          xl: 'repeat(3, minmax(0, 500px))',
        },
        columnGap: { xs: 0, sm: 0, md: 2, lg: 2, xl: 2 },
        rowGap: { xs: 1, sm: 1.5, md: 2, lg: 2.5, xl: 3 },
        justifyContent: { xs: 'center', md: 'start' },
      }}
    >
      {items.map(s => (
        <Card
          key={`${s.id}-${s.start_time}`}
          variant="outlined"
          sx={{
            borderRadius: 2,
            bgcolor: '#fff',
            boxShadow: '0 2px 10px rgba(0,0,0,0.08)',
            transition: 'box-shadow 0.2s ease, transform 0.2s ease',
            '&:hover': {
              boxShadow: '0 6px 20px rgba(0,0,0,0.16)',
              transform: 'translateY(-2px)',
            },
            cursor: 'pointer',
            width: '100%',
            maxWidth: 500,
            justifySelf: { xs: 'center', md: 'start' },
            aspectRatio: '5 / 1',
          }}
          onClick={() =>
            onItemClick
              ? onItemClick(s)
              : navigate('/sport/detail', {
                  state: { sport: s, readonly: true },
                })
          }
        >
          <CardContent
            sx={{
              pt: 1.5,
              pb: 1.5,
              display: 'flex',
              alignItems: 'center',
              height: '100%',
              '&.MuiCardContent-root:last-child': { paddingBottom: 1.5 },
            }}
          >
            <Stack spacing={0.75} sx={{ width: '100%' }}>
              <Box
                sx={{
                  display: 'grid',
                  gridTemplateColumns: '1fr 3fr',
                  columnGap: 2,
                  alignItems: 'center',
                }}
              >
                <Box
                  sx={{ display: 'flex', alignItems: 'center', minWidth: 0 }}
                >
                  <SportField
                    icon={(() => {
                      const key = s.type.toLowerCase();
                      if (key.includes('run')) return <DirectionsRun />;
                      if (key.includes('swim')) return <Pool />;
                      if (key.includes('bike') || key.includes('cycle'))
                        return <DirectionsBike />;
                      if (key.includes('walk') || key.includes('hike'))
                        return <DirectionsWalk />;
                      return <HelpOutline />;
                    })()}
                    label={TypeLabelFor(s.type)}
                    labelVariant="subtitle1"
                    labelColor="text.primary"
                    labelWeight={600}
                    colon={false}
                    responsive
                  />
                </Box>
                <Box
                  sx={{
                    display: 'flex',
                    alignItems: 'center',
                    minWidth: 0,
                    justifyContent: 'flex-end',
                  }}
                >
                  <SportField
                    icon={<AccessTime />}
                    value={formatDateTimeFull(s.start_time)}
                    valueVariant="body2"
                    valueColor="text.secondary"
                    iconColor="text.secondary"
                    align="center"
                    responsive
                  />
                </Box>
              </Box>
              <Box
                sx={{
                  display: 'grid',
                  gridTemplateColumns: 'repeat(3, 1fr)',
                  columnGap: 2,
                  mt: 0,
                  mb: 1,
                }}
              >
                <Box
                  sx={{
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'flex-start',
                    minWidth: 0,
                  }}
                >
                  <SportField
                    icon={<AvTimer />}
                    value={formatDurationHMS(s.duration_second)}
                    valueVariant="body2"
                    valueColor="text.secondary"
                    iconColor="text.secondary"
                    align="center"
                    responsive
                  />
                </Box>
                <Box
                  sx={{
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    minWidth: 0,
                  }}
                >
                  <SportField
                    icon={<AltRoute />}
                    value={`${s.distance_meter} m`}
                    valueVariant="body2"
                    valueColor="text.secondary"
                    iconColor="text.secondary"
                    align="center"
                    responsive
                  />
                </Box>
                <Box
                  sx={{
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'flex-end',
                    minWidth: 0,
                  }}
                >
                  <SportField
                    icon={<LocalFireDepartment />}
                    value={`${s.calories} kcal`}
                    valueVariant="body2"
                    valueColor="text.secondary"
                    iconColor="text.secondary"
                    align="center"
                    responsive
                  />
                </Box>
              </Box>
            </Stack>
          </CardContent>
        </Card>
      ))}
    </Box>
  );
}
