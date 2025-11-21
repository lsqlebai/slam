import { useNavigate } from '@modern-js/runtime/router';
import {
  Box,
  Button,
  MenuItem,
  Select,
  Typography,
  Divider,
} from '@mui/material';
import { UploadFile } from '@mui/icons-material';
import { LANGUAGE_NAMES, TEXTS, saveLang } from '../../i18n';
import type { Lang } from '../../i18n';
import { logout } from '../../services/user';
import { useToast } from '../PageBase';
import { importSportsCsv } from '../../services/sport';
import { useRef, useState } from 'react';

export default function Settings({
  lang,
  onLangChange,
}: { lang: Lang; onLangChange?: (l: Lang) => void }) {
  const { showSuccess, showError } = useToast();
  const navigate = useNavigate();
  const fileInputRef = useRef<HTMLInputElement | null>(null);
  const [uploading, setUploading] = useState(false);
  return (
    <Box
      sx={{
        px: 2,
        py: 2,
        display: 'flex',
        flexDirection: 'column',
        gap: 16,
      }}
    >
      
      <Box sx={{ display: 'flex', flexDirection: 'column', gap: 1 }}>
        <Box>
          <input
            ref={fileInputRef}
            type="file"
            accept=".csv,text/csv"
            style={{ display: 'none' }}
            onChange={async e => {
              const f = e.target.files?.[0];
              if (!f) return;
              try {
                setUploading(true);
                const ok = await importSportsCsv(f, 'xiaomi');
                if (ok) {
                  showSuccess('上传成功');
                } else {
                  showError('上传失败');
                }
              } catch (err: unknown) {
                const msg = err instanceof Error ? err.message : String(err);
                showError(msg || '上传失败');
              } finally {
                setUploading(false);
                e.target.value = '';
              }
            }}
          />
          <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
            <Typography variant="subtitle1" sx={{ fontWeight: 700 }}>{TEXTS[lang].home.miImport}</Typography>
            <Button
              variant="outlined"
              startIcon={<UploadFile />}
              onClick={() => fileInputRef.current?.click()}
              disabled={uploading}
            >
              {TEXTS[lang].home.xiaomiSports}
            </Button>
          </Box>
        </Box>

        <Divider sx={{ my: 0 }} />

        <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
          <Typography variant="subtitle1" sx={{ fontWeight: 700 }}>{TEXTS[lang].home.language}</Typography>
          <Select
            value={lang}
            onChange={e => {
              const next = (e.target.value as Lang) || 'zh';
              saveLang(next);
              onLangChange?.(next);
            }}
            size="small"
            sx={{
              minWidth: 160,
              '& .MuiSelect-select': { textAlign: 'right' },
              '& .MuiOutlinedInput-notchedOutline': { display: 'none' },
            }}
            MenuProps={{
              PaperProps: {
                sx: { '& li': { textAlign: 'right' } },
              },
            }}
          >
            <MenuItem value="zh">{LANGUAGE_NAMES.zh}</MenuItem>
            <MenuItem value="en">{LANGUAGE_NAMES.en}</MenuItem>
          </Select>
        </Box>
      </Box>
      

      <Box
        sx={{
          position: 'fixed',
          left: 0,
          right: 0,
          bottom: 'calc(64px + env(safe-area-inset-bottom) + 20px)',
          display: 'flex',
          justifyContent: 'center',
          px: 2,
        }}
      >
        <Button
          variant="contained"
          color="error"
          onClick={async () => {
            try {
              const ok = await logout();
              if (ok) {
                showSuccess(TEXTS[lang].home.logout);
                navigate('/login');
              } else {
                showError('退出失败');
              }
            } catch (e: unknown) {
              const msg = e instanceof Error ? e.message : String(e);
              showError(msg || '退出失败');
            }
          }}
        >
          {TEXTS[lang].home.logout}
        </Button>
      </Box>
    </Box>
  );
}
