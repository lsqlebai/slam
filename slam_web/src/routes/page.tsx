import { Helmet } from '@modern-js/runtime/head';
import { BarChart, DirectionsRun, Settings } from '@mui/icons-material';
import {
  BottomNavigation,
  BottomNavigationAction,
  Box,
  List,
} from '@mui/material';
import { useEffect, useState } from 'react';
import PageBase from '../components/PageBase';
import HomeHeader from '../components/home/HomeHeader';
import SettingsPage from '../components/home/Settings';
import SidebarNavItem from '../components/home/SidebarNavItem';
import Sporting from '../components/home/Sporting';
import Stats from '../components/stats/Stats';
import useAndroidDoubleBackExit from '../hooks/useAndroidDoubleBackExit';
import { TEXTS } from '../i18n';
import { useLangStore } from '../stores/lang';
import { useUserStore } from '../stores/user';
import './home.css';

function HomeInner() {
  const { lang } = useLangStore();
  const [activeTab, setActiveTab] = useState(0);
  const { user, refresh } = useUserStore();
  useAndroidDoubleBackExit(TEXTS[lang].home.pressAgainExit);

  useEffect(() => {
    (async () => {
      try {
        await refresh();
      } catch {}
    })();
  }, [refresh]);

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
    if (activeTab === 0) return TEXTS[lang].home.motion;
    if (activeTab === 1) return TEXTS[lang].home.stats;
    return TEXTS[lang].home.settings;
  })();

  const [scrolling] = useState(false);

  const content = (
    <Box
      className={'scroll-auto'}
      sx={{
        px: 2,
        pb: {
          xs: 'env(safe-area-inset-bottom)',
          sm: activeTab === 2 ? 10 : 2,
        },
        flex: 1,
        minHeight: 0,
        overflowY: 'auto',
      }}
    >
      {activeTab === 0 && <Sporting lang={lang} />}
      {activeTab === 1 && <Stats lang={lang} />}
      {activeTab === 2 && <SettingsPage lang={lang} />}
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
      <HomeHeader
        title={title}
        greeting={greeting}
        sep={sep}
        user={user}
        variant="mobile"
      />
      <Box
        sx={{
          flex: 1,
          minHeight: 0,
          display: 'flex',
          flexDirection: { xs: 'column', sm: 'row' },
        }}
      >
        <Box
          sx={{
            width: 92,
            display: { xs: 'none', sm: 'flex' },
            flexDirection: 'column',
            justifyContent: 'flex-end',
            alignItems: 'stretch',
            p: 0,
            bgcolor: '#fff',
            boxShadow: '2px 0 10px rgba(0,0,0,0.08)',
            borderRight: '1px solid rgba(0,0,0,0.12)',
          }}
        >
          <List disablePadding>
            <SidebarNavItem
              selected={activeTab === 0}
              onClick={() => setActiveTab(0)}
              icon={<DirectionsRun />}
              label={TEXTS[lang].home.motion}
            />
            <SidebarNavItem
              selected={activeTab === 1}
              onClick={() => setActiveTab(1)}
              icon={<BarChart />}
              label={TEXTS[lang].home.stats}
            />
            <SidebarNavItem
              selected={activeTab === 2}
              onClick={() => setActiveTab(2)}
              icon={<Settings />}
              label={TEXTS[lang].home.settings}
            />
          </List>
        </Box>
        <Box
          sx={{
            flex: 1,
            minHeight: 0,
            display: 'flex',
            flexDirection: 'column',
          }}
        >
          <HomeHeader
            title={title}
            greeting={greeting}
            sep={sep}
            user={user}
            variant="desktop"
          />
          {content}
        </Box>
      </Box>
      <Box
        sx={{
          borderTop: '1px solid rgba(0,0,0,0.12)',
          bgcolor: '#fff',
          width: '100%',
          display: { xs: 'block', sm: 'none' },
          flexShrink: 0,
          minHeight: 'calc(64px + env(safe-area-inset-bottom))',
        }}
      >
        <BottomNavigation
          value={activeTab}
          onChange={(_, newValue) => setActiveTab(newValue)}
          showLabels
          sx={{ height: 64 }}
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
