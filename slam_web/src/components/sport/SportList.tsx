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
import { Box, Card, CardContent, Stack, Typography } from '@mui/material';
import type { Lang } from '../../i18n';
import { TEXTS } from '../../i18n';
import type { Sport } from '../../services/sport';

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

  const locale = lang === 'zh' ? 'zh-CN' : 'en-US';
  const formatDateOnly = (t: number) =>
    new Intl.DateTimeFormat(locale, {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
    }).format(new Date(t * 1000));
  const formatDurationHMS = (s: number) => {
    const h = Math.floor(s / 3600);
    const m = Math.floor((s % 3600) / 60);
    const sec = s % 60;
    const pad = (n: number) => String(n).padStart(2, '0');
    return `${pad(h)}:${pad(m)}:${pad(sec)}`;
  };

  const IconFor = (t: string) => {
    const key = t.toLowerCase();
    if (key.includes('run')) return <DirectionsRun />;
    if (key.includes('swim')) return <Pool />;
    if (key.includes('bike') || key.includes('cycle'))
      return <DirectionsBike />;
    if (key.includes('walk') || key.includes('hike')) return <DirectionsWalk />;
    if (key.includes('unknown')) return <HelpOutline />;
    return <HelpOutline />;
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
    <Stack spacing={1}>
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
              pt: 1,
              pb: 1,
              '&.MuiCardContent-root:last-child': { paddingBottom: 1.5 },
            }}
          >
            <Stack spacing={0.5}>
              <Stack direction="row" spacing={1} alignItems="center">
                {IconFor(s.type)}
                <Typography variant="subtitle1" sx={{ fontWeight: 600 }}>
                  {TypeLabelFor(s.type)}
                </Typography>
                <Box sx={{ flex: 1 }} />
                <Box
                  sx={{ display: 'flex', alignItems: 'center', minWidth: 0 }}
                >
                  <AccessTime
                    fontSize="small"
                    sx={{ color: 'text.secondary', mr: 0.5 }}
                  />
                  <Typography
                    variant="body2"
                    color="text.secondary"
                    noWrap
                    sx={{ minWidth: 0 }}
                  >
                    {formatDateOnly(s.start_time)}
                  </Typography>
                </Box>
              </Stack>
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
                  <AvTimer
                    fontSize="small"
                    sx={{ color: 'text.secondary', mr: 0.5 }}
                  />
                  <Typography
                    variant="body2"
                    color="text.secondary"
                    noWrap
                    sx={{ minWidth: 0, lineHeight: 1.1 }}
                  >
                    {formatDurationHMS(s.duration_second)}
                  </Typography>
                </Box>
                <Box
                  sx={{
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'flex-start',
                    minWidth: 0,
                  }}
                >
                  <AltRoute
                    fontSize="small"
                    sx={{ color: 'text.secondary', mr: 0.5 }}
                  />
                  <Box
                    sx={{
                      display: 'flex',
                      alignItems: 'baseline',
                      minWidth: 0,
                      gap: 0.25,
                      ml: 'auto',
                      justifyContent: 'flex-end',
                      maxWidth: '100%',
                    }}
                  >
                    <Typography
                      component="span"
                      variant="body2"
                      color="text.secondary"
                      noWrap
                      sx={{
                        minWidth: 0,
                        overflow: 'hidden',
                        textOverflow: 'ellipsis',
                        whiteSpace: 'nowrap',
                        lineHeight: 1.1,
                        textAlign: 'right',
                      }}
                    >
                      {s.distance_meter}
                    </Typography>
                    <Typography
                      component="span"
                      variant="body2"
                      color="text.secondary"
                      sx={{
                        whiteSpace: 'nowrap',
                        lineHeight: 1.1,
                        flexShrink: 0,
                      }}
                    >
                      m
                    </Typography>
                  </Box>
                </Box>
                <Box
                  sx={{
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'flex-start',
                    minWidth: 0,
                  }}
                >
                  <LocalFireDepartment
                    fontSize="small"
                    sx={{ color: 'text.secondary', mr: 0.5 }}
                  />
                  <Box
                    sx={{
                      display: 'flex',
                      alignItems: 'baseline',
                      minWidth: 0,
                      gap: 0.25,
                      ml: 'auto',
                      justifyContent: 'flex-end',
                      maxWidth: '100%',
                    }}
                  >
                    <Typography
                      component="span"
                      variant="body2"
                      color="text.secondary"
                      noWrap
                      sx={{
                        minWidth: 0,
                        overflow: 'hidden',
                        textOverflow: 'ellipsis',
                        whiteSpace: 'nowrap',
                        lineHeight: 1.1,
                        textAlign: 'right',
                      }}
                    >
                      {s.calories}
                    </Typography>
                    <Typography
                      component="span"
                      variant="body2"
                      color="text.secondary"
                      sx={{
                        whiteSpace: 'nowrap',
                        lineHeight: 1.1,
                        flexShrink: 0,
                      }}
                    >
                      kcal
                    </Typography>
                  </Box>
                </Box>
              </Box>
            </Stack>
          </CardContent>
        </Card>
      ))}
    </Stack>
  );
}
