import { useNavigate } from '@modern-js/runtime/router';
import { UploadFile } from '@mui/icons-material';
import { Box, Button, Divider, MenuItem, Select, Typography, Avatar } from '@mui/material';
import { useEffect, useRef, useState } from 'react';
import AvatarEditor from './AvatarEditor';
import { LANGUAGE_NAMES, TEXTS, saveLang } from '../../i18n';
import type { Lang } from '../../i18n';
import { importSportsCsv } from '../../services/sport';
import { logout, uploadAvatar } from '../../services/user';
import { useUserStore } from '../../stores/user';
import { useToast } from '../PageBase';

export default function Settings({
  lang,
  onLangChange,
}: { lang: Lang; onLangChange?: (l: Lang) => void }) {
  const { showSuccess, showError } = useToast();
  const navigate = useNavigate();
  const fileInputRef = useRef<HTMLInputElement | null>(null);
  const [uploading, setUploading] = useState(false);
  const avatarInputRef = useRef<HTMLInputElement | null>(null);
  const [avatarUploading, setAvatarUploading] = useState(false);
  const { user, refresh, updateAvatarLocal } = useUserStore();
  const [editorOpen, setEditorOpen] = useState(false);
  const [editorUrl, setEditorUrl] = useState<string | null>(null);
  useEffect(() => {
    refresh().catch(() => {});
  }, [refresh]);
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
            ref={avatarInputRef}
            type="file"
            accept="image/*"
            style={{ display: 'none' }}
            onChange={async e => {
              const f = e.target.files?.[0];
              if (!f) return;
              const url = URL.createObjectURL(f);
              setEditorUrl(url);
              setEditorOpen(true);
            }}
          />
          <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
            <Typography variant="subtitle1" sx={{ fontWeight: 700 }}>
              头像
            </Typography>
            <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
              <Avatar
                src={user?.avatar || undefined}
                sx={{ width: 48, height: 48, cursor: avatarUploading ? 'not-allowed' : 'pointer' }}
                onClick={() => {
                  if (!avatarUploading) avatarInputRef.current?.click();
                }}
              />
            </Box>
          </Box>
        </Box>

        <AvatarEditor
          open={editorOpen}
          imageUrl={editorUrl}
          cropSize={128}
          editorSize={300}
          onCancel={() => {
            setEditorOpen(false);
            if (editorUrl) { URL.revokeObjectURL(editorUrl); }
          }}
          onConfirm={async b64 => {
            try {
              setAvatarUploading(true);
              const saved = await uploadAvatar(b64);
              updateAvatarLocal(saved);
              setEditorOpen(false);
              if (editorUrl) { URL.revokeObjectURL(editorUrl); }
              setEditorUrl(null);
              showSuccess('头像已更新');
            } catch (err: unknown) {
              const msg = err instanceof Error ? err.message : String(err);
              showError(msg || '上传失败');
            } finally {
              setAvatarUploading(false);
            }
          }}
        />

        <Divider sx={{ my: 0 }} />

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
          <Box
            sx={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
            }}
          >
            <Typography variant="subtitle1" sx={{ fontWeight: 700 }}>
              {TEXTS[lang].home.miImport}
            </Typography>
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

        <Box
          sx={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
          }}
        >
          <Typography variant="subtitle1" sx={{ fontWeight: 700 }}>
            {TEXTS[lang].home.language}
          </Typography>
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
