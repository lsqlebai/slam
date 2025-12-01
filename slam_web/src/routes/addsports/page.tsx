import { useNavigate } from '@modern-js/runtime/router';
import { Close, Edit, Psychology } from '@mui/icons-material';
import {
  Box,
  Button,
  CircularProgress,
  Container,
  Dialog,
  DialogContent,
  IconButton,
  Typography,
} from '@mui/material';
import { useRef, useState } from 'react';
import PageBase from '../../components/PageBase';
import { useToast } from '../../components/PageBase';
import ImagePickerCard from '../../components/common/ImagePickerCard';
import ImagePreviewDialog from '../../components/common/ImagePreviewDialog';
import PageHeader from '../../components/common/PageHeader';
import { TEXTS } from '../../i18n';
import { recognizeImages } from '../../services/sport';
import { useLangStore } from '../../stores/lang';

function AddSportsInner() {
  const { lang } = useLangStore();
  const [images, setImages] = useState<{ file: File; url: string }[]>([]);
  const [recognizing, setRecognizing] = useState(false);
  const [preview, setPreview] = useState<string | null>(null);
  const navigate = useNavigate();
  const { showError, showSuccess } = useToast();

  const handleFilesSelected = (fs: FileList | null) => {
    if (!fs || fs.length === 0) return;
    const next: { file: File; url: string }[] = [];
    for (let i = 0; i < fs.length; i++) {
      const f = fs.item(i);
      if (f) next.push({ file: f, url: URL.createObjectURL(f) });
    }
    setImages(prev => prev.concat(next));
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
          flexDirection: { xs: 'column', md: 'row' },
          gap: { xs: 1, md: 2 },
          alignItems: { md: 'stretch' },
          width: { xs: '100%', md: 'max-content' },
          minWidth: { md: '100%' },
          height: { md: '100%' },
          px: { xs: 2, md: 2 },
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
        <ImagePickerCard lang={lang} onFilesSelected={handleFilesSelected} />
      </Box>
      <ImagePreviewDialog
        open={Boolean(preview)}
        imageUrl={preview}
        onClose={() => setPreview(null)}
      />
      <Box
        sx={{
          pb: 2,
          display: 'flex',
          justifyContent: 'center',
          px: 2,
          pt: 2,
          bgcolor: '#fff',
          borderTop: '1px solid',
          borderColor: 'divider',
          boxShadow: '0 -2px 8px rgba(0,0,0,0.04)',
          minHeight: 36,
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
