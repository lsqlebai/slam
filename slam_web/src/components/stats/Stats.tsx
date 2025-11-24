import { Box, Tab, Tabs, Typography } from '@mui/material';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { TEXTS } from '../../i18n';
import type { Lang } from '../../i18n';
import { type StatSummary, getSportStats } from '../../services/sport';
import StatsSection from './StatsSection';
import SummaryStats from './SummaryStats';
import TypeBucketsChart from './TypeBucketsChart';

export default function Stats({ lang }: { lang: Lang }) {
  const tabs = useMemo(
    () => [
      { key: 'week', label: TEXTS[lang].home.statsTabs.week },
      { key: 'month', label: TEXTS[lang].home.statsTabs.month },
      { key: 'year', label: TEXTS[lang].home.statsTabs.year },
      { key: 'total', label: TEXTS[lang].home.statsTabs.total },
    ],
    [lang],
  );

  const [tabIndex, setTabIndex] = useState(2);
  const [selectedYear, setSelectedYear] = useState<number>(
    new Date().getFullYear(),
  );
  const [summary, setSummary] = useState<StatSummary | null>(null);
  const now = new Date();
  const [selectedMonth, setSelectedMonth] = useState<number>(
    now.getMonth() + 1,
  );
  const [selectedWeek, setSelectedWeek] = useState<{
    year: number;
    week: number;
  }>(() => {
    const d = new Date();
    const w = getISOWeek(d);
    return { year: w.year, week: w.week };
  });

  const loadYearStats = useCallback(async (y: number) => {
    const s = await getSportStats('year', y);
    setSummary(s);
  }, []);

  const loadMonthStats = useCallback(async (y: number, m: number) => {
    const s = await getSportStats('month', y, m);
    setSummary(s);
  }, []);

  const loadWeekStats = useCallback(async (y: number, w: number) => {
    const s = await getSportStats('week', y, undefined, w);
    setSummary(s);
  }, []);

  const loadTotalStats = useCallback(async (y: number) => {
    const s = await getSportStats('total', y);
    setSummary(s);
  }, []);

  useEffect(() => {
    if (tabIndex === 2) loadYearStats(selectedYear);
  }, [loadYearStats, selectedYear, tabIndex]);

  useEffect(() => {
    if (tabIndex === 1) loadMonthStats(selectedYear, selectedMonth);
  }, [loadMonthStats, selectedYear, selectedMonth, tabIndex]);

  useEffect(() => {
    if (tabIndex === 0) loadWeekStats(selectedWeek.year, selectedWeek.week);
  }, [loadWeekStats, selectedWeek, tabIndex]);

  useEffect(() => {
    if (tabIndex === 3) loadTotalStats(selectedYear);
  }, [loadTotalStats, selectedYear, tabIndex]);

  const years = useMemo(() => {
    const now = new Date().getFullYear();
    const fallbackStart = now - 9;
    const earliest = summary?.earliest_year ?? fallbackStart;
    const start = Math.min(earliest, now);
    const len = Math.max(1, now - start + 1);
    return Array.from({ length: len }, (_, i) => now - i);
  }, [summary]);

  const monthsDesc = useMemo(() => {
    const cm = new Date().getMonth() + 1;
    return Array.from({ length: cm }, (_, i) => cm - i);
  }, []);

  const lastWeeks = useMemo(() => {
    const arr: { year: number; week: number; label: string }[] = [];
    const pad2 = (n: number) => String(n).padStart(2, '0');
    const rangeLabel = (date: Date) => {
      const monday = new Date(date);
      monday.setDate(date.getDate() - ((date.getDay() + 6) % 7));
      const sunday = new Date(monday);
      sunday.setDate(monday.getDate() + 6);
      const s = `${pad2(monday.getMonth() + 1)}/${pad2(monday.getDate())}`;
      const e = `${pad2(sunday.getMonth() + 1)}/${pad2(sunday.getDate())}`;
      return `${s}-${e}`;
    };
    let d = new Date();
    for (let i = 0; i < 12; i++) {
      const w = getISOWeek(d);
      const label = rangeLabel(d);
      arr.push({ year: w.year, week: w.week, label });
      d = new Date(d.getTime() - 7 * 24 * 3600 * 1000);
    }
    return arr;
  }, []);

  useEffect(() => {
    if (!years.includes(selectedYear)) setSelectedYear(years[0]);
  }, [years, selectedYear]);

  const monthCalories = useMemo(() => {
    const arr = Array.from({ length: 12 }, () => 0);
    const bs = summary?.buckets || [];
    for (const b of bs) {
      const idx = Math.max(1, Math.min(12, b.date)) - 1;
      arr[idx] = b.calories || 0;
    }
    return arr;
  }, [summary]);

  const totals = useMemo(() => {
    return {
      duration: summary?.total_duration_second || 0,
      calories: summary?.total_calories || 0,
      count: summary?.total_count || 0,
      distanceMeter: summary?.total_distance_meter || 0,
    };
  }, [summary]);

  const locale = lang === 'zh' ? 'zh-CN' : 'en-US';

  function getISOWeek(date: Date): { year: number; week: number } {
    const d = new Date(
      Date.UTC(date.getFullYear(), date.getMonth(), date.getDate()),
    );
    const dayNum = d.getUTCDay() || 7;
    d.setUTCDate(d.getUTCDate() + 4 - dayNum);
    const yearStart = new Date(Date.UTC(d.getUTCFullYear(), 0, 1));
    const weekNo = Math.ceil(
      ((d.getTime() - yearStart.getTime()) / 86400000 + 1) / 7,
    );
    return { year: d.getUTCFullYear(), week: weekNo };
  }

  const daysInSelectedMonth = useMemo(() => {
    const y = selectedYear;
    const m = selectedMonth;
    return new Date(y, m, 0).getDate();
  }, [selectedYear, selectedMonth]);

  const monthDailyCalories = useMemo(() => {
    const arr = Array.from({ length: daysInSelectedMonth }, () => 0);
    for (const b of summary?.buckets || []) {
      const d = Math.max(1, Math.min(daysInSelectedMonth, b.date));
      arr[d - 1] = b.calories || 0;
    }
    return arr.map((v, idx) => ({ day: idx + 1, value: v }));
  }, [summary, daysInSelectedMonth]);

  const monthDailyDetails = useMemo(() => {
    const arr = Array.from({ length: daysInSelectedMonth }, () => ({
      duration: 0,
      count: 0,
      calories: 0,
    }));
    for (const b of summary?.buckets || []) {
      const d = Math.max(1, Math.min(daysInSelectedMonth, b.date));
      arr[d - 1] = {
        duration: b.duration || 0,
        count: b.count || 0,
        calories: b.calories || 0,
      };
    }
    const map: Record<
      string,
      { duration: number; count: number; calories: number }
    > = {};
    for (let i = 0; i < arr.length; i++) map[String(i + 1)] = arr[i];
    return map;
  }, [summary, daysInSelectedMonth]);

  const weekdayLabels = useMemo(() => {
    const now = new Date();
    const monday = new Date(now);
    monday.setDate(now.getDate() - ((now.getDay() + 6) % 7));
    return Array.from({ length: 7 }, (_, i) =>
      new Intl.DateTimeFormat(locale, { weekday: 'short' }).format(
        new Date(monday.getFullYear(), monday.getMonth(), monday.getDate() + i),
      ),
    );
  }, [locale]);

  const weekDailyCalories = useMemo(() => {
    const arr = Array.from({ length: 7 }, () => 0);
    for (const b of summary?.buckets || []) {
      const d = Math.max(1, Math.min(7, b.date));
      arr[d - 1] = b.calories || 0;
    }
    return arr.map((v, idx) => ({
      day: idx + 1,
      value: v,
      label: weekdayLabels[idx],
    }));
  }, [summary, weekdayLabels]);

  const weekDailyDetails = useMemo(() => {
    const arr = Array.from({ length: 7 }, () => ({
      duration: 0,
      count: 0,
      calories: 0,
    }));
    for (const b of summary?.buckets || []) {
      const d = Math.max(1, Math.min(7, b.date));
      arr[d - 1] = {
        duration: b.duration || 0,
        count: b.count || 0,
        calories: b.calories || 0,
      };
    }
    const map: Record<
      string,
      { duration: number; count: number; calories: number }
    > = {};
    for (let i = 0; i < 7; i++) map[weekdayLabels[i]] = arr[i];
    return map;
  }, [summary, weekdayLabels]);

  return (
    <Box sx={{ px: 2, pt: 1, pb: 2 }}>
      <Box sx={{ overflow: 'hidden', touchAction: 'pan-y' }}>
        <Tabs
          value={tabIndex}
          onChange={(_, v) => setTabIndex(v)}
          variant="fullWidth"
          sx={{
            minHeight: 44,
            '& .MuiTab-root': {
              minHeight: 44,
              minWidth: 0,
            },
          }}
        >
          {tabs.map(t => (
            <Tab
              key={t.key}
              label={t.label}
              disableRipple
              disableTouchRipple
              sx={{ textTransform: 'none' }}
            />
          ))}
        </Tabs>
      </Box>
      {tabIndex === 1 && (
        <Box sx={{ mt: 2 }}>
          <Box
            sx={{
              display: 'flex',
              gap: 2,
              overflowX: 'auto',
              pb: 1,
              scrollbarWidth: 'none',
              msOverflowStyle: 'none',
              '&::-webkit-scrollbar': { display: 'none', height: 0 },
            }}
          >
            {monthsDesc.map(m => (
              <Typography
                key={m}
                onClick={() => setSelectedMonth(m)}
                sx={{
                  flexShrink: 0,
                  cursor: 'pointer',
                  color:
                    m === selectedMonth ? 'primary.main' : 'text.secondary',
                  fontWeight: m === selectedMonth ? 700 : 500,
                }}
              >
                {lang === 'zh' ? `${m}月` : `M${String(m).padStart(2, '0')}`}
              </Typography>
            ))}
          </Box>
          <StatsSection
            lang={lang}
            totals={totals}
            title={`${selectedYear}-${String(selectedMonth).padStart(2, '0')}`}
            data={monthDailyCalories.map(md => ({
              label: String(md.day),
              value: md.value,
            }))}
            details={monthDailyDetails}
            barMaxWidth={20}
            hideZero
            sports={summary?.sports || []}
          />
        </Box>
      )}
      {tabIndex === 0 && (
        <Box sx={{ mt: 2 }}>
          <Box
            sx={{
              display: 'flex',
              gap: 2,
              overflowX: 'auto',
              pb: 1,
              scrollbarWidth: 'none',
              msOverflowStyle: 'none',
              '&::-webkit-scrollbar': { display: 'none', height: 0 },
            }}
          >
            {lastWeeks.map(w => (
              <Typography
                key={`${w.year}-${w.week}`}
                onClick={() => setSelectedWeek({ year: w.year, week: w.week })}
                sx={{
                  flexShrink: 0,
                  cursor: 'pointer',
                  color:
                    w.year === selectedWeek.year && w.week === selectedWeek.week
                      ? 'primary.main'
                      : 'text.secondary',
                  fontWeight:
                    w.year === selectedWeek.year && w.week === selectedWeek.week
                      ? 700
                      : 500,
                }}
              >
                {w.label}
              </Typography>
            ))}
          </Box>
          <StatsSection
            lang={lang}
            totals={totals}
            title={
              lastWeeks.find(
                w =>
                  w.year === selectedWeek.year && w.week === selectedWeek.week,
              )?.label || `${String(selectedWeek.week).padStart(2, '0')}`
            }
            data={weekDailyCalories.map(wd => ({
              label: wd.label,
              value: wd.value,
            }))}
            details={weekDailyDetails}
            sports={summary?.sports || []}
          />
        </Box>
      )}
      {tabIndex === 2 && (
        <Box sx={{ mt: 2 }}>
          <Box
            sx={{
              display: 'flex',
              gap: 2,
              overflowX: 'auto',
              pb: 1,
              scrollbarWidth: 'none',
              msOverflowStyle: 'none',
              '&::-webkit-scrollbar': { display: 'none', height: 0 },
            }}
          >
            {years.map(y => (
              <Typography
                key={y}
                onClick={() => setSelectedYear(y)}
                sx={{
                  flexShrink: 0,
                  cursor: 'pointer',
                  color: y === selectedYear ? 'primary.main' : 'text.secondary',
                  fontWeight: y === selectedYear ? 700 : 500,
                }}
              >
                {y}
              </Typography>
            ))}
          </Box>
          <StatsSection
            lang={lang}
            totals={totals}
            title={`${selectedYear}`}
            data={monthCalories.map((v, idx) => ({
              label: String(idx + 1),
              value: v,
            }))}
            details={(() => {
              const arr = Array.from({ length: 12 }, () => ({
                duration: 0,
                count: 0,
                calories: 0,
              }));
              for (const b of summary?.buckets || []) {
                const idx = Math.max(1, Math.min(12, b.date)) - 1;
                arr[idx] = {
                  duration: b.duration || 0,
                  count: b.count || 0,
                  calories: b.calories || 0,
                };
              }
              const map: Record<
                string,
                { duration: number; count: number; calories: number }
              > = {};
              for (let i = 0; i < 12; i++) map[String(i + 1)] = arr[i];
              return map;
            })()}
            sports={summary?.sports || []}
          />
        </Box>
      )}
      {tabIndex === 3 && (
        <Box sx={{ mt: 2 }}>
          <Box sx={{ mb: 2 }}>
          <SummaryStats
            lang={lang}
            durationSeconds={summary?.total_duration_second || 0}
            calories={summary?.total_calories || 0}
            count={summary?.total_count || 0}
            distanceMeter={summary?.total_distance_meter || 0}
          />
          </Box>
          <TypeBucketsChart lang={lang} buckets={summary?.type_buckets || []} />
        </Box>
      )}
    </Box>
  );
}
