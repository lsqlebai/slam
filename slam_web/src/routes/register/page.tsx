import { Helmet } from '@modern-js/runtime/head';
import { Link as RouterLink, useNavigate } from '@modern-js/runtime/router';
import { Settings, Visibility, VisibilityOff } from '@mui/icons-material';
import {
  Box,
  Button,
  Container,
  FormControl,
  IconButton,
  InputAdornment,
  InputLabel,
  Menu,
  MenuItem,
  Paper,
  Select,
  Stack,
  TextField,
  Typography,
} from '@mui/material';
import { useEffect, useState } from 'react';
import './register.css';
import PageBase, { useToast } from '../../components/PageBase';

function RegisterInner() {
  const [username, setUsername] = useState('');
  const [nickname, setNickname] = useState('');
  const [password, setPassword] = useState('');
  const [confirm, setConfirm] = useState('');
  const { showError, showSuccess } = useToast();
  const [showPassword, setShowPassword] = useState(false);
  const [showConfirm, setShowConfirm] = useState(false);
  const [lang, setLang] = useState<'zh' | 'en'>('zh');
  const [menuAnchor, setMenuAnchor] = useState<null | HTMLElement>(null);
  const navigate = useNavigate();

  useEffect(() => {
    setLang(getSavedLang());
  }, []);

  const handleSubmit = async () => {
    if (!username || !nickname || !password || !confirm) {
      showError(TEXTS[lang].register.errorFill);
      return;
    }
    if (password.length < 6) {
      showError(TEXTS[lang].register.errorLength);
      return;
    }
    if (password !== confirm) {
      showError(TEXTS[lang].register.errorMismatch);
      return;
    }
    try {
      const ok = await registerRequest(username, password, nickname);
      if (ok) {
        showSuccess(TEXTS[lang].register.success);
        navigate('/');
      } else {
        showError('注册失败');
      }
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      showError(msg || '注册失败');
    }
  };

  return (
    <Box className="login-wrapper register-bg">
      <Helmet>
        <title>{TEXTS[lang].register.headTitle}</title>
      </Helmet>
      <Container
        maxWidth="sm"
        sx={{ display: 'flex', alignItems: 'center', justifyContent: 'center' }}
      >
        <Paper elevation={3} sx={{ p: 3, width: '100%', maxWidth: 420 }}>
          <Stack spacing={2}>
            <Typography variant="h5" align="center">
              {TEXTS[lang].register.title}
            </Typography>
            <TextField
              label={TEXTS[lang].register.username}
              value={username}
              onChange={e => setUsername(e.target.value)}
              fullWidth
              autoComplete="username"
            />
            <TextField
              label={TEXTS[lang].register.nickname}
              value={nickname}
              onChange={e => setNickname(e.target.value)}
              fullWidth
              autoComplete="nickname"
            />
            <TextField
              label={TEXTS[lang].register.password}
              type={showPassword ? 'text' : 'password'}
              value={password}
              onChange={e => setPassword(e.target.value)}
              fullWidth
              autoComplete="new-password"
              inputProps={{ minLength: 6 }}
              InputProps={{
                endAdornment: (
                  <InputAdornment position="end">
                    <IconButton
                      aria-label="切换密码可见性"
                      onClick={() => setShowPassword(v => !v)}
                      edge="end"
                    >
                      {showPassword ? <VisibilityOff /> : <Visibility />}
                    </IconButton>
                  </InputAdornment>
                ),
              }}
            />
            <TextField
              label={TEXTS[lang].register.confirm}
              type={showConfirm ? 'text' : 'password'}
              value={confirm}
              onChange={e => setConfirm(e.target.value)}
              fullWidth
              autoComplete="new-password"
              inputProps={{ minLength: 6 }}
              InputProps={{
                endAdornment: (
                  <InputAdornment position="end">
                    <IconButton
                      aria-label="切换确认密码可见性"
                      onClick={() => setShowConfirm(v => !v)}
                      edge="end"
                    >
                      {showConfirm ? <VisibilityOff /> : <Visibility />}
                    </IconButton>
                  </InputAdornment>
                ),
              }}
            />
            <Box sx={{ display: 'flex', gap: 2 }}>
              <Button variant="contained" onClick={handleSubmit} fullWidth>
                {TEXTS[lang].register.submit}
              </Button>
              <Button
                component={RouterLink}
                to="/login"
                variant="outlined"
                fullWidth
              >
                {TEXTS[lang].register.cancel}
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
  );
}

import { LANGUAGE_NAMES, TEXTS, getSavedLang, saveLang } from '../../i18n';
import { register as registerRequest } from '../../services/user';

export default function RegisterPage() {
  return (
    <PageBase>
      <RegisterInner />
    </PageBase>
  );
}
