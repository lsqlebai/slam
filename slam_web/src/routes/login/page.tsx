import { Helmet } from '@modern-js/runtime/head';
import { Link as RouterLink } from '@modern-js/runtime/router';
import { Settings } from '@mui/icons-material';
import {
  Box,
  Button,
  Container,
  IconButton,
  Menu,
  MenuItem,
  Paper,
  Stack,
  TextField,
  Typography,
} from '@mui/material';
import { useEffect, useState } from 'react';
import './login.css';
import PageBase, { useToast } from '../../components/PageBase';
import { LANGUAGE_NAMES, TEXTS, getSavedLang, saveLang } from '../../i18n';
import { login as loginRequest } from '../../services/user';

export default function LoginPage() {
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [lang, setLang] = useState<'zh' | 'en'>('zh');
  const [menuAnchor, setMenuAnchor] = useState<null | HTMLElement>(null);
  const { showError, showSuccess } = useToast();

  const handleLogin = async () => {
    if (!username || !password) {
      showError('请填写用户名和密码');
      return;
    }
    try {
      const ok = await loginRequest(username, password);
      if (ok) {
        showSuccess('登录成功');
      } else {
        showError('登录失败');
      }
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      showError(msg || '登录失败');
    }
  };

  useEffect(() => {
    setLang(getSavedLang());
  }, []);

  return (
    <PageBase>
      <Box className="login-wrapper login-bg">
        <Helmet>
          <title>{TEXTS[lang].login.headTitle}</title>
        </Helmet>
        <Container maxWidth="sm" sx={{ display: 'flex', alignItems: 'center' }}>
          <Paper elevation={3} sx={{ p: 3, width: '100%', maxWidth: 420 }}>
            <Stack spacing={2}>
              <Typography variant="h5" align="center">
                {TEXTS[lang].login.title}
              </Typography>
              <TextField
                label={TEXTS[lang].login.username}
                value={username}
                onChange={e => setUsername(e.target.value)}
                fullWidth
                autoComplete="username"
              />
              <TextField
                label={TEXTS[lang].login.password}
                type="password"
                value={password}
                onChange={e => setPassword(e.target.value)}
                fullWidth
                autoComplete="current-password"
              />
              <Box sx={{ display: 'flex', gap: 2 }}>
                <Button variant="contained" onClick={handleLogin} fullWidth>
                  {TEXTS[lang].login.login}
                </Button>
                <Button
                  component={RouterLink}
                  to="/register"
                  variant="outlined"
                  fullWidth
                >
                  {TEXTS[lang].login.register}
                </Button>
              </Box>
            </Stack>
          </Paper>
          <Box sx={{ position: 'fixed', right: 16, bottom: 16 }}>
            <IconButton
              aria-label="settings"
              onClick={e => setMenuAnchor(e.currentTarget)}
              sx={{
                color: '#fff',
                backgroundColor: 'rgba(0,0,0,0.35)',
                border: '1px solid rgba(255,255,255,0.6)',
                '&:hover': { backgroundColor: 'rgba(0,0,0,0.5)' },
              }}
            >
              <Settings />
            </IconButton>
            <Menu
              anchorEl={menuAnchor}
              open={Boolean(menuAnchor)}
              onClose={() => setMenuAnchor(null)}
              anchorOrigin={{ vertical: 'top', horizontal: 'right' }}
              transformOrigin={{ vertical: 'bottom', horizontal: 'right' }}
            >
              <MenuItem
                onClick={() => {
                  setLang('zh');
                  saveLang('zh');
                  setMenuAnchor(null);
                }}
              >
                {LANGUAGE_NAMES.zh}
              </MenuItem>
              <MenuItem
                onClick={() => {
                  setLang('en');
                  saveLang('en');
                  setMenuAnchor(null);
                }}
              >
                {LANGUAGE_NAMES.en}
              </MenuItem>
            </Menu>
          </Box>
        </Container>
      </Box>
    </PageBase>
  );
}
