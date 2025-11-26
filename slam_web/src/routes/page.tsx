import { Helmet } from '@modern-js/runtime/head';
import { useNavigate } from '@modern-js/runtime/router';
import { BarChart, DirectionsRun, Settings } from '@mui/icons-material';
import {
  Avatar,
  BottomNavigation,
  BottomNavigationAction,
  Box,
  Divider,
  Typography,
} from '@mui/material';
import { useEffect, useRef, useState } from 'react';
import PageBase from '../components/PageBase';
import SettingsPage from '../components/home/Settings';
import Sporting from '../components/home/Sporting';
import Stats from '../components/stats/Stats';
import { TEXTS, getSavedLang } from '../i18n';
import { useUserStore } from '../stores/user';
import './home.css';

function HomeInner() {
  const [lang, setLang] = useState<'zh' | 'en'>('zh');
  const [value, setValue] = useState(0);
  const { user, refresh } = useUserStore();
  const navigate = useNavigate();

  useEffect(() => {
    setLang(getSavedLang());
    (async () => {
      try {
        const ok = await refresh();
        if (!ok) throw new Error('unauth');
      } catch {
        navigate('/login');
      }
    })();
  }, [navigate, refresh]);

  const hour = new Date().getHours();
  const gKey = (() => {
    if (hour >= 5 && hour < 11) return 'morning';
    if (hour >= 11 && hour < 13) return 'noon';
    if (hour >= 13 && hour < 18) return 'afternoon';
    return 'evening';
  })();
  const greeting =
    TEXTS[lang].home.greetings[
      gKey as keyof (typeof TEXTS)['zh']['home']['greetings']
    ];
  const sep = lang === 'zh' ? '，' : ', ';
  const title = (() => {
    if (value === 0) return TEXTS[lang].home.motion;
    if (value === 1) return TEXTS[lang].home.stats;
    return TEXTS[lang].home.settings;
  })();

  const contentRef = useRef<HTMLDivElement | null>(null);
  const [scrolling, setScrolling] = useState(false);
  const scrollTimerRef = useRef<number | null>(null);

  const content = (
    <Box
      ref={contentRef}
      className={scrolling ? 'scroll-auto scrolling' : 'scroll-auto'}
      onScroll={() => {
        if (scrollTimerRef.current) {
          window.clearTimeout(scrollTimerRef.current);
        }
        setScrolling(true);
        scrollTimerRef.current = window.setTimeout(() => {
          setScrolling(false);
        }, 800);
      }}
      sx={{
        px: 2,
        pb: 'calc(env(safe-area-inset-bottom) + 72px)',
        flex: 1,
        minHeight: 0,
        overflowY: 'auto',
      }}
    >
      {value === 0 && <Sporting lang={lang} />}
      {value === 1 && <Stats lang={lang} />}
      {value === 2 && <SettingsPage lang={lang} onLangChange={setLang} />}
    </Box>
  );

  return (
    <Box
      sx={{
        height: '100dvh',
        display: 'flex',
        flexDirection: 'column',
        overflow: 'hidden',
      }}
      className="home-bg"
    >
      <Helmet>
        <title>{TEXTS[lang].home.headTitle}</title>
        <meta
          name="viewport"
          content="width=device-width, initial-scale=1, viewport-fit=cover"
        />
      </Helmet>
      <Box
        sx={{
          pt: 'calc(env(safe-area-inset-top) + 12px)',
          px: 2,
          pb: 1,
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
        }}
      >
        <Typography variant="h6" noWrap sx={{ minWidth: 0 }}>
          {title}
        </Typography>
        {user?.nickname && (
          <Box
            sx={{ display: 'flex', alignItems: 'center', gap: 1, minWidth: 0 }}
          >
            <Typography
              variant="subtitle1"
              color="text.primary"
              noWrap
              sx={{ textAlign: 'right', minWidth: 0 }}
            >
              {greeting}
              {sep}
              {user?.nickname}
            </Typography>
            <Avatar
              src={user?.avatar || undefined}
              sx={{ width: 32, height: 32 }}
            />
          </Box>
        )}
      </Box>
      <Divider sx={{ mb: 1 }} />
      {content}
      <Box
        sx={{
          position: 'fixed',
          left: 0,
          right: 0,
          bottom: 0,
          borderTop: '1px solid rgba(0,0,0,0.12)',
          bgcolor: '#fff',
          zIndex: 1300,
          width: '100%',
        }}
      >
        <BottomNavigation
          value={value}
          onChange={(_, newValue) => setValue(newValue)}
          showLabels
          sx={{ height: 64, paddingBottom: 'env(safe-area-inset-bottom)' }}
        >
          <BottomNavigationAction
            label={TEXTS[lang].home.motion}
            icon={<DirectionsRun />}
          />
          <BottomNavigationAction
            label={TEXTS[lang].home.stats}
            icon={<BarChart />}
          />
          <BottomNavigationAction
            label={TEXTS[lang].home.settings}
            icon={<Settings />}
          />
        </BottomNavigation>
      </Box>
    </Box>
  );
}

export default function HomePage() {
  return (
    <PageBase>
      <HomeInner />
    </PageBase>
  );
}
