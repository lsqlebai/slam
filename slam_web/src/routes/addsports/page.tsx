import { useNavigate } from '@modern-js/runtime/router';
import { Add, Close, Edit, Psychology } from '@mui/icons-material';
import {
  Box,
  Button,
  CircularProgress,
  Container,
  Dialog,
  DialogContent,
  IconButton,
  Stack,
  Typography,
} from '@mui/material';
import { useRef, useState } from 'react';
import PageBase from '../../components/PageBase';
import { useToast } from '../../components/PageBase';
import PageHeader from '../../components/common/PageHeader';
import { TEXTS } from '../../i18n';
import { recognizeImages } from '../../services/sport';
import { useLangStore } from '../../stores/lang';

function AddSportsInner() {
  const { lang } = useLangStore();
  const [images, setImages] = useState<{ file: File; url: string }[]>([]);
  const [recognizing, setRecognizing] = useState(false);
  const [preview, setPreview] = useState<string | null>(null);
  const fileInputRef = useRef<HTMLInputElement | null>(null);
  const navigate = useNavigate();
  const { showError, showSuccess } = useToast();

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const fs = e.target.files;
    if (!fs || fs.length === 0) return;
    const next: { file: File; url: string }[] = [];
    for (let i = 0; i < fs.length; i++) {
      const f = fs.item(i);
      if (f) next.push({ file: f, url: URL.createObjectURL(f) });
    }
    setImages(prev => prev.concat(next));
    e.target.value = '';
  };

  const handleRecognize = async () => {
    if (images.length === 0) {
      showError('请先选择图片');
      return;
    }
    setRecognizing(true);
    try {
      const fd = new FormData();
      for (const { file } of images) {
        fd.append('image', file);
      }
      const resp = await recognizeImages(fd);
      if (!resp.success || !resp.data) {
        const em = String(resp.error?.message || '');
        const isTimeout =
          /timeout|超时/i.test(em) || resp.error?.code === 'TIMEOUT';
        showError(
          isTimeout
            ? TEXTS[lang].addsports.aiTimeoutBusy
            : em || TEXTS[lang].addsports.aiFail,
        );
        return;
      }
      const calories = resp.data.calories || 0;
      const segments = Array.isArray(resp.data.tracks)
        ? resp.data.tracks.length
        : 0;
      const aiToast =
        lang === 'zh'
          ? `恭喜你AI识别成功，运动总消耗 ${calories} Kcal，有${segments}段分段数据哦`
          : `AI recognition succeeded. Total calories ${calories} Kcal, with ${segments} segments.`;
      navigate('/sport/detail', { state: { sport: resp.data, aiToast } });
    } catch (e: unknown) {
      const raw = e instanceof Error ? e.message : String(e);
      const isTimeout = /timeout|超时|ECONNABORTED/i.test(String(raw));
      showError(
        isTimeout
          ? TEXTS[lang].addsports.aiTimeoutBusy
          : raw || TEXTS[lang].addsports.aiFail,
      );
    } finally {
      setRecognizing(false);
    }
  };

  return (
    <Box
      sx={{
        minHeight: '100dvh',
        display: 'flex',
        flexDirection: 'column',
      }}
    >
      <PageHeader
        headTitle={TEXTS[lang].addsports.headTitle}
        title={TEXTS[lang].addsports.title}
      />
      <Box
        sx={{
          bgcolor: 'grey.50',
          overflowX: { xs: 'visible', md: 'auto' },
          py: 2,
          flex: 1,
          display: 'flex',
          flexDirection: 'column',
          minHeight: { md: 'calc(100dvh - 56px - 64px)' },
        }}
      >
        <Container
          maxWidth={false}
          sx={{ px: { xs: 1, md: 2 }, height: '100%' }}
        >
          <Stack spacing={2} sx={{ width: '100%', height: '100%' }}>
            <input
              ref={fileInputRef}
              type="file"
              accept="image/*"
              multiple
              onChange={handleFileChange}
              style={{ display: 'none' }}
            />
            <Box
              sx={{
                display: 'flex',
                flexDirection: { xs: 'column', md: 'row' },
                gap: { xs: 1, md: 2 },
                alignItems: { md: 'stretch' },
                width: { xs: '100%', md: 'max-content' },
                height: { md: '100%' },
              }}
            >
              <Box
                sx={{
                  width: { xs: '100%', md: 'max-content' },
                  order: { xs: 1, md: 1 },
                }}
              >
                <Box
                  sx={{
                    display: 'flex',
                    flexDirection: { xs: 'column', md: 'row' },
                    gap: 1,
                  }}
                >
                  {images.map((img, idx) => (
                    <Box
                      key={`${idx}-${img.url}`}
                      sx={{
                        borderRadius: 1,
                        overflow: 'hidden',
                        cursor: 'pointer',
                        bgcolor: 'grey.100',
                        border: '1px solid',
                        borderColor: 'divider',
                        display: 'flex',
                        justifyContent: 'center',
                        position: 'relative',
                        maxHeight: { md: 800 },
                        width: { xs: '100%', md: 512 },
                        flex: { md: '0 0 auto' },
                      }}
                      onClick={() => setPreview(img.url)}
                    >
                      <IconButton
                        aria-label="remove-image"
                        onClick={e => {
                          e.stopPropagation();
                          URL.revokeObjectURL(img.url);
                          setImages(prev => prev.filter((_, i) => i !== idx));
                        }}
                        sx={{
                          position: 'absolute',
                          top: 6,
                          right: 6,
                          backgroundColor: 'rgba(0,0,0,0.35)',
                          color: '#fff',
                          '&:hover': { backgroundColor: 'rgba(0,0,0,0.5)' },
                          zIndex: 1,
                        }}
                      >
                        <Close fontSize="small" />
                      </IconButton>
                      <Box
                        component="img"
                        src={img.url}
                        alt=""
                        loading="lazy"
                        sx={{
                          width: '100%',
                          height: 'auto',
                          display: 'block',
                          maxHeight: { md: 800 },
                        }}
                      />
                    </Box>
                  ))}
                </Box>
              </Box>
              <Box
                sx={{
                  width: { xs: '100%', md: 300 },
                  order: { xs: 2, md: 2 },
                  display: 'flex',
                  height: { md: '100%' },
                }}
              >
                <Box
                  sx={{
                    borderRadius: 1,
                    bgcolor: 'grey.100',
                    border: '1px solid',
                    borderColor: 'divider',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    cursor: 'pointer',
                    px: 2,
                    height: { xs: 120, md: 'calc(100% - 96px)' },
                    my: { md: 6 },
                    maxHeight: { md: 800 },
                    flex: 1,
                  }}
                  onClick={() => fileInputRef.current?.click()}
                >
                  <Stack direction="row" spacing={1} alignItems="center">
                    <IconButton aria-label="add-image" color="primary">
                      <Add />
                    </IconButton>
                    <Typography variant="body2" color="text.secondary">
                      {TEXTS[lang].addsports.pickImages}
                    </Typography>
                  </Stack>
                </Box>
              </Box>
            </Box>
          </Stack>
        </Container>
      </Box>
      <Dialog
        open={Boolean(preview)}
        onClose={() => setPreview(null)}
        maxWidth="md"
        fullWidth
      >
        <DialogContent sx={{ p: 0, position: 'relative' }}>
          <IconButton
            aria-label="close-preview"
            onClick={() => setPreview(null)}
            sx={{
              position: 'absolute',
              top: 8,
              left: 8,
              backgroundColor: 'rgba(0,0,0,0.35)',
              color: '#fff',
              '&:hover': { backgroundColor: 'rgba(0,0,0,0.5)' },
              zIndex: 1,
            }}
          >
            <Close />
          </IconButton>
          {preview && (
            <Box
              component="img"
              src={preview}
              alt=""
              sx={{ width: '100%', height: 'auto', display: 'block' }}
            />
          )}
        </DialogContent>
      </Dialog>
      <Box
        sx={{
          position: 'fixed',
          left: 0,
          right: 0,
          bottom: 'calc(env(safe-area-inset-bottom) + 20px)',
          display: 'flex',
          justifyContent: 'center',
          px: 2,
          pt: 2,
          bgcolor: '#fff',
          borderTop: '1px solid',
          borderColor: 'divider',
          boxShadow: '0 -2px 8px rgba(0,0,0,0.04)',
        }}
      >
        <Box sx={{ display: 'flex', gap: 2, width: '100%', maxWidth: 480 }}>
          <Button
            variant="outlined"
            startIcon={<Edit />}
            onClick={() => navigate('/sport/detail')}
            fullWidth
          >
            {TEXTS[lang].addsports.manualButton}
          </Button>
          <Button
            variant="contained"
            startIcon={<Psychology />}
            disabled={images.length === 0 || recognizing}
            onClick={handleRecognize}
            fullWidth
          >
            {TEXTS[lang].addsports.aiButton}
          </Button>
        </Box>
      </Box>
      <Dialog open={recognizing} maxWidth="xs" fullWidth>
        <DialogContent>
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 2, py: 1 }}>
            <CircularProgress size={20} />
            <Typography variant="body2">
              {TEXTS[lang].addsports.aiLoading}
            </Typography>
          </Box>
        </DialogContent>
      </Dialog>
    </Box>
  );
}

export default function AddSportsPage() {
  return (
    <PageBase>
      <AddSportsInner />
    </PageBase>
  );
}
