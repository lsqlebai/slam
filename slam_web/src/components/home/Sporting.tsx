import { useNavigate } from '@modern-js/runtime/router';
import { Add, BarChart } from '@mui/icons-material';
import { Box, Card, CardContent, Fab, Stack, Typography } from '@mui/material';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { TEXTS } from '../../i18n';
import type { Lang } from '../../i18n';
import { type Sport, listSports } from '../../services/sport';
import SportCalendar from '../common/SportCalendar';
import SportListTitle from '../common/SportListTitle';
import SportList from '../sport/SportList';

export default function Sporting({ lang }: { lang: Lang }) {
  const [items, setItems] = useState<Sport[]>([]);
  const [page, setPage] = useState(0);
  const [loading, setLoading] = useState(false);
  const [hasMore, setHasMore] = useState(true);
  const sentinelRef = useRef<HTMLDivElement | null>(null);
  const navigate = useNavigate();

  const loadPage = useCallback(async (p: number) => {
    setLoading(true);
    try {
      const data = await listSports(p, 20);
      if (p === 0) setItems(data);
      else setItems(prev => prev.concat(data));
      setHasMore(data.length >= 20);
      setPage(p);
      return data;
    } catch {
      setHasMore(false);
    } finally {
      setLoading(false);
    }
  }, []);

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

  const now = new Date();
  const [displayYear, setDisplayYear] = useState(now.getFullYear());
  const [displayMonth, setDisplayMonth] = useState(now.getMonth());
  const prevYear = displayMonth === 0 ? displayYear - 1 : displayYear;
  const prevMonth = displayMonth === 0 ? 11 : displayMonth - 1;
  const isNowMonth =
    displayYear === now.getFullYear() && displayMonth === now.getMonth();
  const hasPrevInItems = useMemo(() => {
    for (const it of items) {
      const d = new Date(it.start_time * 1000);
      if (d.getFullYear() === prevYear && d.getMonth() === prevMonth)
        return true;
    }
    return false;
  }, [items, prevYear, prevMonth]);
  const disablePrev = !hasPrevInItems && !hasMore;
  const disableNext = isNowMonth;
  const handlePrev = useCallback(async () => {
    if (loading) return;
    const py = prevYear;
    const pm = prevMonth;
    const hasPrev = hasPrevInItems;
    if (hasPrev) {
      setDisplayYear(py);
      setDisplayMonth(pm);
      return;
    }
    if (hasMore) {
      const data = await loadPage(page + 1);
      const merged = items.concat(data ?? []);
      const found = merged.some(it => {
        const d = new Date(it.start_time * 1000);
        return d.getFullYear() === py && d.getMonth() === pm;
      });
      if (found) {
        setDisplayYear(py);
        setDisplayMonth(pm);
      }
    }
  }, [
    hasPrevInItems,
    hasMore,
    loadPage,
    page,
    items,
    prevYear,
    prevMonth,
    loading,
  ]);
  const handleNext = useCallback(() => {
    if (disableNext) return;
    const ny = displayMonth === 11 ? displayYear + 1 : displayYear;
    const nm = displayMonth === 11 ? 0 : displayMonth + 1;
    setDisplayYear(ny);
    setDisplayMonth(nm);
  }, [disableNext, displayMonth, displayYear]);
  const activeDays = useMemo(() => {
    const s = new Set<number>();
    for (const it of items) {
      const d = new Date(it.start_time * 1000);
      if (d.getFullYear() === displayYear && d.getMonth() === displayMonth) {
        s.add(d.getDate());
      }
    }
    return s;
  }, [items, displayYear, displayMonth]);

  return (
    <Box sx={{ px: 0, py: 0 }}>
      <Box sx={{ px: 2, pb: 1, mt: 2 }}>
        <Box
          sx={{
            display: 'grid',
            gridTemplateColumns: { xs: '1fr', md: 'auto 1fr' },
            columnGap: { xs: 0, md: 2 },
            alignItems: 'stretch',
          }}
        >
          <SportCalendar
            year={displayYear}
            month={displayMonth}
            lang={lang}
            activeDays={activeDays}
            disablePrev={disablePrev}
            disableNext={disableNext}
            onPrev={handlePrev}
            onNext={handleNext}
          />
          <Card
            onClick={() => navigate('/addsports')}
            sx={{
              display: { xs: 'none', sm: 'none', md: 'block' },
              height: '100%',
              borderRadius: 3,
              bgcolor: 'rgba(0,0,0,0.02)',
              boxShadow: '0 3px 12px rgba(0,0,0,0.10)',
              transition: 'box-shadow 0.2s ease, transform 0.2s ease',
              '&:hover': {
                boxShadow: '0 4px 14px rgba(0,0,0,0.10)',
                transform: 'translateY(-2px)',
              },
              cursor: 'pointer',
              border: 'none',
              width: '100%',
              maxWidth: 500,
              justifySelf: { md: 'start' },
              '&:hover .add-circle': {
                borderColor: 'primary.main',
                boxShadow: '0 6px 16px rgba(0,0,0,0.12)',
              },
              '&:hover .add-plus-line': {
                bgcolor: 'primary.main',
              },
            }}
          >
            <CardContent
              sx={{
                height: '100%',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                '&.MuiCardContent-root:last-child': { paddingBottom: 2 },
              }}
            >
              <Stack spacing={1.25} alignItems="center">
                <Box
                  sx={{
                    position: 'relative',
                    width: { xs: 56, sm: 64, md: 72 },
                    height: { xs: 56, sm: 64, md: 72 },
                    border: '2px dashed',
                    borderColor: 'divider',
                    borderRadius: '50%',
                    boxShadow: '0 3px 10px rgba(0,0,0,0.12)',
                  }}
                  className="add-circle"
                >
                  <Box
                    sx={{
                      position: 'absolute',
                      left: '50%',
                      top: '50%',
                      transform: 'translate(-50%, -50%)',
                      width: '60%',
                      height: '3px',
                      bgcolor: 'divider',
                    }}
                    className="add-plus-line"
                  />
                  <Box
                    sx={{
                      position: 'absolute',
                      left: '50%',
                      top: '50%',
                      transform: 'translate(-50%, -50%)',
                      width: '3px',
                      height: '60%',
                      bgcolor: 'divider',
                    }}
                    className="add-plus-line"
                  />
                </Box>
                <Typography
                  variant="subtitle1"
                  sx={{ fontWeight: 400, color: 'text.secondary' }}
                >
                  {TEXTS[lang].addsports.title}
                </Typography>
              </Stack>
            </CardContent>
          </Card>
        </Box>
      </Box>
      {groups.map(g => (
        <Box key={g.key} sx={{ px: 2, pb: 2 }}>
          <SportListTitle
            icon={<BarChart />}
            label={g.month}
            labelVariant="subtitle2"
            iconColor="text.primary"
            labelColor="text.primary"
            gap={1}
          />
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
          right: { xs: 12, sm: 16, md: 20, lg: 24, xl: 32 },
          bottom: {
            xs: 'calc(64px + env(safe-area-inset-bottom) + 12px)',
            sm: 16,
            md: 20,
            lg: 24,
            xl: 32,
          },
          zIndex: 1350,
          display: { xs: 'block', sm: 'block', md: 'none' },
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
