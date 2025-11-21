import { useNavigate } from '@modern-js/runtime/router';
import {
  AccessTime,
  Add,
  AltRoute,
  AvTimer,
  BarChart,
  DirectionsBike,
  DirectionsRun,
  DirectionsWalk,
  HelpOutline,
  LocalFireDepartment,
  Pool,
} from '@mui/icons-material';
import { Box, Card, CardContent, Fab, Stack, Typography } from '@mui/material';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { TEXTS } from '../../i18n';
import type { Lang } from '../../i18n';
import { type Sport, listSports } from '../../services/sport';
import SportList from '../sport/SportList';

export default function Sporting({ lang }: { lang: Lang }) {
  const [items, setItems] = useState<Sport[]>([]);
  const [page, setPage] = useState(0);
  const [loading, setLoading] = useState(false);
  const [hasMore, setHasMore] = useState(true);
  const sentinelRef = useRef<HTMLDivElement | null>(null);
  const navigate = useNavigate();

  const loadPage = useCallback(
    async (p: number) => {
      setLoading(true);
      try {
        const data = await listSports(p, 20);
        if (p === 0) setItems(data);
        else setItems(prev => prev.concat(data));
        setHasMore(data.length >= 20);
        setPage(p);
      } catch {
        if (p === 0) navigate('/login');
      } finally {
        setLoading(false);
      }
    },
    [navigate],
  );

  useEffect(() => {
    loadPage(0);
  }, [loadPage]);

  useEffect(() => {
    const el = sentinelRef.current;
    if (!el) return;
    const io = new IntersectionObserver(
      entries => {
        for (const e of entries) {
          if (e.isIntersecting && hasMore && !loading) {
            loadPage(page + 1);
          }
        }
      },
      { root: null, rootMargin: '300px 0px', threshold: 0 },
    );
    io.observe(el);
    return () => io.disconnect();
  }, [hasMore, loading, page, loadPage]);

  const groups = useMemo(() => {
    const byMonth: Record<string, Sport[]> = {};
    for (const s of items) {
      const d = new Date(s.start_time * 1000);
      const k = `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}`;
      if (!byMonth[k]) byMonth[k] = [];
      byMonth[k].push(s);
    }
    const keys = Object.keys(byMonth).sort((a, b) => (a < b ? 1 : -1));
    return keys.map(k => ({ key: k, month: k, list: byMonth[k] }));
  }, [items]);

  const locale = lang === 'zh' ? 'zh-CN' : 'en-US';
  const formatDate = (t: number) =>
    new Intl.DateTimeFormat(locale, {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
      hour12: false,
    }).format(new Date(t * 1000));
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
  const km = (m: number) => (m / 1000).toFixed(2);

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

  const now = new Date();
  const year = now.getFullYear();
  const month = now.getMonth();
  const monthTitle = new Intl.DateTimeFormat(locale, {
    year: 'numeric',
    month: 'long',
  }).format(now);
  const firstDay = new Date(year, month, 1);
  const startOffset = (firstDay.getDay() + 6) % 7;
  const daysInMonth = new Date(year, month + 1, 0).getDate();
  const mondayThisWeek = new Date(now);
  mondayThisWeek.setDate(now.getDate() - ((now.getDay() + 6) % 7));
  const weekdayLabels = Array.from({ length: 7 }, (_, i) =>
    new Intl.DateTimeFormat(locale, { weekday: 'short' }).format(
      new Date(
        mondayThisWeek.getFullYear(),
        mondayThisWeek.getMonth(),
        mondayThisWeek.getDate() + i,
      ),
    ),
  );
  const blankKeys = useMemo(
    () =>
      Array.from({ length: startOffset }, (_, i) =>
        new Date(year, month, 1 - (startOffset - i)).toISOString(),
      ),
    [startOffset, year, month],
  );
  const activeDays = useMemo(() => {
    const s = new Set<number>();
    for (const it of items) {
      const d = new Date(it.start_time * 1000);
      if (d.getFullYear() === year && d.getMonth() === month) {
        s.add(d.getDate());
      }
    }
    return s;
  }, [items, year, month]);

  return (
    <Box sx={{ px: 0, py: 0 }}>
      <Box sx={{ px: 2, pb: 1, mt: 2 }}>
        <Stack spacing={1}>
          <Box
            sx={{
              p: 1.5,
              borderRadius: 2,
              bgcolor: '#fff',
              boxShadow: '0 2px 10px rgba(0,0,0,0.08)',
            }}
          >
            <Stack
              direction="row"
              justifyContent="center"
              alignItems="center"
              spacing={0.75}
              sx={{ mb: 1 }}
            >
              <LocalFireDepartment
                fontSize="small"
                sx={{ color: 'primary.main' }}
              />
              <Typography variant="subtitle1" sx={{ fontWeight: 600 }}>
                {monthTitle}
              </Typography>
              <LocalFireDepartment
                fontSize="small"
                sx={{ color: 'primary.main' }}
              />
            </Stack>
            <Box
              sx={{
                display: 'grid',
                gridTemplateColumns: 'repeat(7, 1fr)',
                gap: 1,
              }}
            >
              {weekdayLabels.map(w => (
                <Typography
                  key={w}
                  variant="caption"
                  color="text.secondary"
                  sx={{ textAlign: 'center', fontWeight: 600 }}
                >
                  {w}
                </Typography>
              ))}
            </Box>
            <Box
              sx={{
                display: 'grid',
                gridTemplateColumns: 'repeat(7, 1fr)',
                gap: 0.75,
                mt: 1,
              }}
            >
              {blankKeys.map(k => (
                <Box key={k} />
              ))}
              {Array.from({ length: daysInMonth }, (_, i) => {
                const day = i + 1;
                const isActive = activeDays.has(day);
                return (
                  <Box
                    key={`day-${day}`}
                    sx={{
                      height: 32,
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'center',
                      borderRadius: 1,
                      bgcolor: isActive ? 'primary.light' : 'transparent',
                      color: 'text.primary',
                      fontWeight: isActive ? 600 : 500,
                    }}
                  >
                    <Typography variant="body2">{day}</Typography>
                  </Box>
                );
              })}
            </Box>
          </Box>
        </Stack>
      </Box>
      {groups.map(g => (
        <Box key={g.key} sx={{ px: 2, pb: 2 }}>
          <Stack direction="row" alignItems="center" spacing={1} sx={{ py: 1 }}>
            <BarChart fontSize="small" />
            <Typography variant="subtitle2">{g.month}</Typography>
          </Stack>
          <SportList
            lang={lang}
            items={g.list}
            onItemClick={s =>
              navigate('/sport/detail', {
                state: { sport: s, readonly: true },
              })
            }
          />
        </Box>
      ))}
      <Box ref={sentinelRef} sx={{ height: 1 }} />
      {loading && (
        <Box sx={{ display: 'flex', justifyContent: 'center', py: 2 }}>
          <Typography variant="body2" color="text.secondary">
            Loading...
          </Typography>
        </Box>
      )}
      <Box
        sx={{
          position: 'fixed',
          right: 16,
          bottom: 'calc(64px + env(safe-area-inset-bottom) + 16px)',
          zIndex: 1350,
        }}
      >
        <Fab
          color="primary"
          aria-label="add"
          onClick={() => navigate('/addsports')}
        >
          <Add />
        </Fab>
      </Box>
    </Box>
  );
}
