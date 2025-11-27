import { Helmet } from '@modern-js/runtime/head';
import { Link as RouterLink, useNavigate } from '@modern-js/runtime/router';
import { Settings } from '@mui/icons-material';
import {
  Box,
  Container,
  Divider,
  IconButton,
  Menu,
  MenuItem,
  Paper,
  Stack,
  Typography,
} from '@mui/material';
import { useRef, useState } from 'react';
import { useLangStore } from '../../stores/lang';
import './login.css';
import PageBase, { useToast } from '../../components/PageBase';
import ResponsiveButton from '../../components/common/ResponsiveButton';
import ResponsiveInput from '../../components/common/ResponsiveInput';

function LoginInner() {
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const { lang, setLang } = useLangStore();
  const [menuAnchor, setMenuAnchor] = useState<null | HTMLElement>(null);
  const { showError, showSuccess } = useToast();
  const navigate = useNavigate();
  const passwordInputRef = useRef<HTMLInputElement | null>(null);

  const handleLogin = async () => {
    if (!username || !password) {
      showError('请填写用户名和密码');
      return;
    }
    try {
      const ok = await loginRequest(username, password);
      if (ok) {
        showSuccess('登录成功');
        navigate('/');
      } else {
        showError('登录失败');
      }
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      showError(msg || '登录失败');
    }
  };

  return (
    <Box className="login-wrapper login-bg">
      <Helmet>
        <title>{TEXTS[lang].login.headTitle}</title>
      </Helmet>
      <Container
        maxWidth="sm"
        sx={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          px: { xs: 2, sm: 3 },
        }}
      >
        <Paper
          elevation={3}
          sx={{
            p: { xs: 3, md: 4 },
            borderRadius: { xs: '12px', md: '16px' },
            width: '100%',
            maxWidth: { xs: 360, sm: 420, md: 480 },
          }}
        >
          <Stack spacing={2}>
            <Typography
              variant="h5"
              align="center"
              sx={{ fontSize: { xs: '1.25rem', md: '1.375rem' } }}
            >
              {TEXTS[lang].login.title}
            </Typography>
            <ResponsiveInput
              label={TEXTS[lang].login.username}
              value={username}
              onChange={e => setUsername(e.target.value)}
              fullWidth
              autoComplete="username"
              onKeyDown={e => {
                if (e.key === 'Enter') {
                  const ne = e.nativeEvent as unknown as {
                    isComposing?: boolean;
                  } & { keyCode?: number };
                  if (ne.isComposing || e.keyCode === 229) return;
                  e.preventDefault();
                  passwordInputRef.current?.focus();
                }
              }}
            />
            <ResponsiveInput
              label={TEXTS[lang].login.password}
              type="password"
              value={password}
              onChange={e => setPassword(e.target.value)}
              fullWidth
              autoComplete="current-password"
              inputRef={passwordInputRef}
              onKeyDown={e => {
                if (e.key === 'Enter') {
                  const ne = e.nativeEvent as unknown as {
                    isComposing?: boolean;
                  } & { keyCode?: number };
                  if (ne.isComposing || e.keyCode === 229) return;
                  handleLogin();
                }
              }}
            />
            <Box
              sx={{
                display: 'flex',
                gap: { xs: 1.5, md: 2 },
                flexDirection: { xs: 'column', sm: 'row' },
              }}
            >
              <ResponsiveButton
                variant="contained"
                onClick={handleLogin}
                fullWidth
              >
                {TEXTS[lang].login.login}
              </ResponsiveButton>
              <ResponsiveButton
                component={RouterLink}
                to="/register"
                variant="outlined"
                fullWidth
              >
                {TEXTS[lang].login.register}
              </ResponsiveButton>
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
              className="menu-item-compact"
              onClick={() => {
                setLang('zh');
                setMenuAnchor(null);
              }}
            >
              {LANGUAGE_NAMES.zh}
            </MenuItem>
            <Divider />
            <MenuItem
              className="menu-item-compact"
              onClick={() => {
                setLang('en');
                setMenuAnchor(null);
              }}
            >
              {LANGUAGE_NAMES.en}
            </MenuItem>
          </Menu>
        </Box>
      </Container>
    </Box>
  );
}
import { LANGUAGE_NAMES, TEXTS } from '../../i18n';
import { login as loginRequest } from '../../services/user';

export default function LoginPage() {
  return (
    <PageBase>
      <LoginInner />
    </PageBase>
  );
}
