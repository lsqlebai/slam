import { Box, Tab, Tabs, Typography } from '@mui/material';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { TEXTS } from '../../i18n';
import type { Lang } from '../../i18n';
import { type StatSummary, getSportStats } from '../../services/sport';
import StatsFilterSection from './StatsFilterSection';
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

  const abortRef = useRef<AbortController | null>(null);

  const loadStats = useCallback(
    async (
      kind: 'year' | 'month' | 'week' | 'total',
      opts: { year: number; month?: number; week?: number },
    ) => {
      abortRef.current?.abort();
      const c = new AbortController();
      abortRef.current = c;
      try {
        const s = await getSportStats(
          kind,
          opts.year,
          opts.month,
          opts.week,
          c.signal,
        );
        setSummary(s);
      } catch (e) {}
    },
    [],
  );

  useEffect(() => {
    if (tabIndex === 2) loadStats('year', { year: selectedYear });
  }, [loadStats, selectedYear, tabIndex]);

  useEffect(() => {
    if (tabIndex === 1)
      loadStats('month', { year: selectedYear, month: selectedMonth });
  }, [loadStats, selectedYear, selectedMonth, tabIndex]);

  useEffect(() => {
    if (tabIndex === 0)
      loadStats('week', {
        year: selectedWeek.year,
        week: selectedWeek.week,
      });
  }, [loadStats, selectedWeek, tabIndex]);

  useEffect(() => {
    if (tabIndex === 3) loadStats('total', { year: selectedYear });
  }, [loadStats, selectedYear, tabIndex]);

  useEffect(() => {
    return () => {
      abortRef.current?.abort();
    };
  }, []);

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

  const topSelectorItems = useMemo(() => {
    if (tabIndex === 1)
      return monthsDesc.map(m => ({
        label: lang === 'zh' ? `${m}月` : `M${String(m).padStart(2, '0')}`,
        selected: m === selectedMonth,
        onClick: () => setSelectedMonth(m),
      }));
    if (tabIndex === 0)
      return lastWeeks.map(w => ({
        label: w.label,
        selected: w.year === selectedWeek.year && w.week === selectedWeek.week,
        onClick: () => setSelectedWeek({ year: w.year, week: w.week }),
      }));
    if (tabIndex === 2)
      return years.map(y => ({
        label: String(y),
        selected: y === selectedYear,
        onClick: () => setSelectedYear(y),
      }));
    return [] as { label: string; selected: boolean; onClick: () => void }[];
  }, [
    tabIndex,
    monthsDesc,
    selectedMonth,
    lastWeeks,
    selectedWeek,
    years,
    selectedYear,
    lang,
  ]);

  const hasSelector = topSelectorItems.length > 0;

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

  const renderMonthContent = () => (
    <StatsFilterSection
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
  );

  const renderWeekContent = () => (
    <StatsFilterSection
      lang={lang}
      totals={totals}
      title={
        lastWeeks.find(
          w => w.year === selectedWeek.year && w.week === selectedWeek.week,
        )?.label || `${String(selectedWeek.week).padStart(2, '0')}`
      }
      data={weekDailyCalories.map(wd => ({ label: wd.label, value: wd.value }))}
      details={weekDailyDetails}
      sports={summary?.sports || []}
    />
  );

  const renderYearContent = () => (
    <StatsFilterSection
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
  );

  const renderTotalContent = () => (
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
  );

  return (
    <Box sx={{ px: 2, pt: 1, pb: 2 }}>
      <Box
        sx={{
          display: 'grid',
          gridTemplateColumns: {
            xs: '1fr',
            md: hasSelector
              ? 'repeat(2, minmax(0, 500px))'
              : 'minmax(0, 500px)',
          },
          gridTemplateAreas: {
            xs: hasSelector ? '"tabs" "selector"' : '"tabs"',
            md: hasSelector ? '"tabs selector"' : '"tabs"',
          },
          columnGap: 2,
          alignItems: 'stretch',
          justifyItems: { xs: 'center', md: 'start' },
          justifyContent: { xs: 'center', md: 'start' },
          width: { xs: '100%', md: 'fit-content' },
          maxWidth: { xs: 500, md: 'none' },
          mx: { xs: 'auto', md: 0 },
          pb: { md: 2 },
        }}
      >
        <Box
          sx={{
            gridArea: 'tabs',
            overflow: 'hidden',
            touchAction: 'pan-y',
            background: 'linear-gradient(180deg, #1976d2 0%, #0b5fb8 100%)',
            color: 'common.white',
            borderRadius: 1,
            border: '1px solid',
            borderColor: 'transparent',
            boxShadow:
              '0 6px 16px rgba(0,0,0,0.10), 0 2px 6px rgba(0,0,0,0.07)',
            maxWidth: 500,
            width: '100%',
            mx: { xs: 'auto', sm: 'auto', md: 0 },
          }}
        >
          <Tabs
            value={tabIndex}
            onChange={(_, v) => setTabIndex(v)}
            variant="fullWidth"
            sx={{
              minHeight: 44,
              color: 'common.white',
              '& .MuiTab-root': {
                minHeight: 44,
                minWidth: 0,
                color: 'rgba(255,255,255,0.75)',
                fontWeight: 700,
              },
              '& .MuiTab-root.Mui-selected': {
                color: 'rgba(255,255,255,0.98)',
                textShadow: '0 1px 1px rgba(0,0,0,0.25)',
              },
              '& .MuiTabs-indicator': {
                backgroundColor: 'rgba(255,255,255,0.9)',
                height: 3,
                borderRadius: 1,
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
        <Box
          sx={{
            gridArea: 'selector',
            display: hasSelector ? 'flex' : 'none',
            alignItems: 'flex-end',
            gap: 2,
            overflowX: 'auto',
            pb: 1,
            scrollbarWidth: 'none',
            msOverflowStyle: 'none',
            '&::-webkit-scrollbar': { display: 'none', height: 0 },
            maxWidth: 500,
            width: '100%',
            height: { md: '100%' },
            mt: 1,
            mb: 0,
            mx: { xs: 'auto', sm: 'auto', md: 0 },
          }}
        >
          {topSelectorItems.map(it => (
            <Typography
              key={it.label}
              onClick={it.onClick}
              sx={{
                flexShrink: 0,
                cursor: 'pointer',
                color: it.selected ? 'primary.main' : 'text.secondary',
                fontWeight: it.selected ? 700 : 500,
              }}
            >
              {it.label}
            </Typography>
          ))}
        </Box>
      </Box>
      {tabIndex === 1 && renderMonthContent()}
      {tabIndex === 0 && renderWeekContent()}
      {tabIndex === 2 && renderYearContent()}
      {tabIndex === 3 && renderTotalContent()}
    </Box>
  );
}
