import { useLocation, useNavigate } from '@modern-js/runtime/router';
import {
  Box,
  Button,
  Container,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  Typography,
} from '@mui/material';
import { useEffect, useState } from 'react';
import PageBase, { useToast } from '../../../components/PageBase';
import PageHeader from '../../../components/common/PageHeader';
import SportBasicInfo from '../../../components/sport/SportBasicInfo';
import SportExtraInfo from '../../../components/sport/SportExtraInfo';
import SportTracks from '../../../components/sport/SportTracks';
import { TEXTS } from '../../../i18n';
import {
  type Sport,
  type Swimming,
  type Track,
  deleteSport,
  insertSport,
  updateSport,
} from '../../../services/sport';
import { useLangStore } from '../../../stores/lang';
// time utils are used inside child components

function SubmitInner() {
  const { lang } = useLangStore();
  const navigate = useNavigate();
  const { showError, showSuccess } = useToast();
  const location = useLocation();
  type LocationState = { sport?: Sport; readonly?: boolean } | null;
  const initial: Sport | null =
    (location.state as LocationState)?.sport ?? null;
  const readonly: boolean = Boolean(
    (location.state as LocationState)?.readonly,
  );
  const fromDetail =
    'readonly' in
    ((location.state as unknown as Record<string, unknown>) || {});
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [sport, setSport] = useState<Sport>(
    () =>
      initial ?? {
        id: 0,
        type: 'Unknown',
        start_time: Math.floor(Date.now() / 1000),
        calories: 0,
        distance_meter: 0,
        duration_second: 0,
        heart_rate_avg: 0,
        heart_rate_max: 0,
        pace_average: '0',
        extra: {
          main_stroke: 'unknown',
          stroke_avg: 0,
          swolf_avg: 0,
        } as Swimming,
        tracks: [] as Track[],
      },
  );

  useEffect(() => {
    const st = (location.state as Record<string, unknown>) || {};
    const ai = st.aiToast as string | undefined;
    if (typeof ai === 'string' && ai) {
      showSuccess(ai);
      navigate('.', { state: { sport }, replace: true });
    }
  }, [location.state, showSuccess, navigate, sport]);

  const update = (patch: Partial<Sport>) =>
    setSport(prev => ({ ...prev, ...patch }));
  const updateExtra = (patch: Partial<Swimming>) => {
    const base = sport.extra ?? {
      main_stroke: 'unknown',
      stroke_avg: 0,
      swolf_avg: 0,
    };
    update({ extra: { ...base, ...patch } });
  };

  const handleSubmit = async () => {
    try {
      const ok =
        sport.id > 0 ? await updateSport(sport) : await insertSport(sport);
      if (ok) {
        showSuccess(TEXTS[lang].addsports.submitSuccess);
        navigate('/');
      } else {
        showError(TEXTS[lang].addsports.submitFail);
      }
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      showError(msg || TEXTS[lang].addsports.submitFail);
    }
  };

  const handleCancel = () => {
    if (fromDetail) {
      navigate('.', { state: { sport, readonly: true }, replace: true });
    } else {
      navigate('/addsports');
    }
  };

  const handleEdit = () => {
    navigate('.', { state: { sport, readonly: false }, replace: true });
  };

  const handleDeleteConfirm = async () => {
    try {
      if (!sport.id || sport.id <= 0) {
        showError(TEXTS[lang].addsports.deleteFail);
        return;
      }
      const ok = await deleteSport(sport.id);
      if (ok) {
        showSuccess(TEXTS[lang].addsports.deleteSuccess);
        navigate('/');
      } else {
        showError(TEXTS[lang].addsports.deleteFail);
      }
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      showError(msg || TEXTS[lang].addsports.deleteFail);
    } finally {
      setDeleteDialogOpen(false);
    }
  };

  return (
    <Box
      className="submit-wrapper"
      sx={{
        minHeight: '100dvh',
        display: 'flex',
        flexDirection: 'column',
        bgcolor: 'grey.100',
      }}
    >
      <PageHeader
        headTitle={TEXTS[lang].addsports.headTitle}
        title={TEXTS[lang].addsports.title}
      />
      <Container
        maxWidth="md"
        sx={{
          display: { xs: 'flex', md: 'grid' },
          flexDirection: { xs: 'column' },
          gridTemplateColumns: { md: '1fr 1fr' },
          justifyItems: { md: 'center' },
          alignItems: { xs: 'center', md: 'start' },
          gap: 3,
          pt: { xs: 2, sm: 3 },
          flex: 1,
          minHeight: 0,
          pb: 'calc(env(safe-area-inset-bottom) + 92px)',
        }}
      >
        <Box
          sx={{
            display: 'flex',
            flexDirection: 'column',
            gap: 3,
            width: '100%',
            maxWidth: 500,
            justifySelf: 'center',
          }}
        >
          <SportBasicInfo
            lang={lang}
            sport={sport}
            readonly={readonly}
            update={update}
          />

          <SportExtraInfo
            lang={lang}
            sport={sport}
            readonly={readonly}
            updateExtra={updateExtra}
          />
        </Box>

        <Box sx={{ width: '100%', maxWidth: 500, justifySelf: 'center' }}>
          <SportTracks
            lang={lang}
            sport={sport}
            readonly={readonly}
            update={update}
          />
        </Box>
      </Container>

      <Box
        sx={{
          position: 'fixed',
          left: 0,
          right: 0,
          bottom: 0,
          height: 'calc(env(safe-area-inset-bottom) + 72px)',
          display: 'flex',
          alignItems: 'flex-end',
          justifyContent: 'center',
          px: 2,
          bgcolor: 'background.paper',
          boxShadow: '0 -8px 20px rgba(0,0,0,0.12)',
          backdropFilter: 'saturate(160%) blur(8px)',
          zIndex: 1300,
        }}
      >
        {readonly ? (
          <Box
            sx={{
              display: 'flex',
              gap: 2,
              width: '100%',
              maxWidth: 380,
              mb: 'calc(env(safe-area-inset-bottom) + 20px)',
            }}
          >
            <Button variant="outlined" onClick={handleEdit} fullWidth>
              {TEXTS[lang].addsports.detailEdit}
            </Button>
            <Button
              variant="contained"
              color="error"
              onClick={() => setDeleteDialogOpen(true)}
              fullWidth
            >
              {TEXTS[lang].addsports.detailDelete}
            </Button>
          </Box>
        ) : (
          <Box
            sx={{
              display: 'flex',
              gap: 2,
              width: '100%',
              maxWidth: 380,
              mb: 'calc(env(safe-area-inset-bottom) + 20px)',
            }}
          >
            <Button variant="outlined" onClick={handleCancel} fullWidth>
              {TEXTS[lang].register.cancel}
            </Button>
            <Button variant="contained" onClick={handleSubmit} fullWidth>
              {TEXTS[lang].addsports.detailConfirm}
            </Button>
          </Box>
        )}
      </Box>

      <Dialog
        open={deleteDialogOpen}
        onClose={() => setDeleteDialogOpen(false)}
        maxWidth="xs"
        fullWidth
      >
        <DialogTitle>{TEXTS[lang].addsports.deleteConfirmTitle}</DialogTitle>
        <DialogContent>
          <Typography>{TEXTS[lang].addsports.deleteConfirmMessage}</Typography>
        </DialogContent>
        <DialogActions>
          <Button variant="outlined" onClick={() => setDeleteDialogOpen(false)}>
            {TEXTS[lang].register.cancel}
          </Button>
          <Button
            variant="contained"
            color="error"
            onClick={handleDeleteConfirm}
          >
            {TEXTS[lang].addsports.detailDelete}
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
}

export default function SubmitPage() {
  return (
    <PageBase>
      <SubmitInner />
    </PageBase>
  );
}
