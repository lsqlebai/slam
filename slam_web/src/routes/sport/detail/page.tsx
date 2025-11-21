import { Helmet } from '@modern-js/runtime/head';
import { useLocation, useNavigate } from '@modern-js/runtime/router';
import {
  Add,
  ArrowBack,
  Delete,
  Info,
  Pool,
  Timeline,
} from '@mui/icons-material';
import {
  Box,
  Button,
  Container,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  Divider,
  Fab,
  IconButton,
  MenuItem,
  Paper,
  Stack,
  TextField,
  Typography,
} from '@mui/material';
import { useEffect, useState } from 'react';
import PageBase, { useToast } from '../../../components/PageBase';
import { TEXTS, getSavedLang } from '../../../i18n';
import {
  type Sport,
  type Swimming,
  type Track,
  insertSport,
} from '../../../services/sport';

function SubmitInner() {
  const [lang, setLang] = useState<'zh' | 'en'>('zh');
  const navigate = useNavigate();
  const { showError, showSuccess } = useToast();
  const location = useLocation();
  type LocationState = { sport?: Sport; readonly?: boolean } | null;
  const initial: Sport | null =
    (location.state as LocationState)?.sport ?? null;
  const readonly: boolean = Boolean(
    (location.state as LocationState)?.readonly,
  );
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
  const defaultTrack: Track = {
    distance_meter: 0,
    duration_second: 0,
    pace_average: '0',
    extra: { main_stroke: 'unknown', stroke_avg: 0, swolf_avg: 0 },
  };
  const [trackDialogOpen, setTrackDialogOpen] = useState(false);
  const [trackDraft, setTrackDraft] = useState<Track>(defaultTrack);
  const [editingIndex, setEditingIndex] = useState<number | null>(null);

  useEffect(() => {
    setLang(getSavedLang());
  }, []);

  const update = (patch: Partial<Sport>) =>
    setSport(prev => ({ ...prev, ...patch }));
  const updateExtra = (patch: Partial<Swimming>) =>
    update({ extra: { ...sport.extra, ...patch } });
  const updateTrack = (idx: number, patch: Partial<Track>) => {
    const next = sport.tracks.slice();
    next[idx] = { ...next[idx], ...patch } as Track;
    update({ tracks: next });
  };
  const addTrack = () => {
    setEditingIndex(null);
    setTrackDraft(defaultTrack);
    setTrackDialogOpen(true);
  };
  const removeTrack = (idx: number) =>
    update({ tracks: sport.tracks.filter((_, i) => i !== idx) });
  const updateTrackDraft = (patch: Partial<Track>) =>
    setTrackDraft(prev => ({ ...prev, ...patch }) as Track);
  const editTrack = (idx: number) => {
    setEditingIndex(idx);
    setTrackDraft(sport.tracks[idx]);
    setTrackDialogOpen(true);
  };
  const toInputDateTime = (s: number) => {
    const d = new Date(s * 1000);
    const pad = (n: number) => String(n).padStart(2, '0');
    const y = d.getFullYear();
    const m = pad(d.getMonth() + 1);
    const day = pad(d.getDate());
    const hh = pad(d.getHours());
    const mm = pad(d.getMinutes());
    const ss = pad(d.getSeconds());
    return `${y}-${m}-${day}T${hh}:${mm}:${ss}`;
  };
  const fromInputDateTime = (v: string) => {
    const [date, time] = v.split('T');
    const [y, m, d] = date.split('-').map(Number);
    const [hh, mm, ss] = time.split(':').map(Number);
    const dt = new Date(y, (m || 1) - 1, d || 1, hh || 0, mm || 0, ss || 0);
    return Math.floor(dt.getTime() / 1000);
  };
  const toHMS = (s: number) => {
    const hh = Math.floor(s / 3600);
    const mm = Math.floor((s % 3600) / 60);
    const ss = s % 60;
    const pad = (n: number) => String(n).padStart(2, '0');
    return `${pad(hh)}:${pad(mm)}:${pad(ss)}`;
  };
  const fromHMS = (v: string) => {
    const parts = v.split(':').map(p => Number.parseInt(p || '0'));
    const [hh, mm, ss] = [parts[0] || 0, parts[1] || 0, parts[2] || 0];
    return hh * 3600 + mm * 60 + ss;
  };

  const handleSubmit = async () => {
    try {
      const ok = await insertSport(sport);
      if (ok) {
        showSuccess('提交成功');
        navigate('/');
      } else {
        showError('提交失败');
      }
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      showError(msg || '提交失败');
    }
  };

  return (
    <Box
      className="submit-wrapper"
      sx={{ pb: 'calc(env(safe-area-inset-bottom) + 136px)' }}
    >
      <Helmet>
        <title>{TEXTS[lang].addsports.headTitle}</title>
      </Helmet>
      <Box
        sx={{
          position: 'sticky',
          top: 0,
          zIndex: 1300,
          bgcolor: '#fff',
          pt: 'calc(env(safe-area-inset-top) + 12px)',
          pl: 1,
          pr: 2,
          py: 1,
          minHeight: 56,
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
        }}
      >
        <IconButton
          aria-label="back"
          onClick={() => navigate(-1)}
          sx={{ color: 'text.primary' }}
        >
          <ArrowBack fontSize="large" />
        </IconButton>
        <Typography
          variant="h6"
          noWrap
          sx={{ textAlign: 'right', fontWeight: 600 }}
        >
          {TEXTS[lang].addsports.title}
        </Typography>
      </Box>
      <Divider sx={{ mb: 1 }} />
      <Container
        maxWidth="md"
        sx={{
          display: 'flex',
          alignItems: 'flex-start',
          justifyContent: 'center',
        }}
      >
        <Stack spacing={3} sx={{ width: '100%', maxWidth: 840, pl: 1 }}>
          <Stack spacing={1} sx={{ mt: 0 }}>
            <Stack
              direction="row"
              spacing={1}
              alignItems="center"
              sx={{ pl: 1 }}
            >
              <Info fontSize="small" />
              <Typography variant="subtitle1">
                {TEXTS[lang].addsports.submitBasicTitle}
              </Typography>
            </Stack>
            <Paper elevation={3} sx={{ p: 2, borderRadius: 2 }}>
              <Stack spacing={2} sx={{ pb: 2 }}>
                <Stack direction="row" spacing={2}>
                  <TextField
                    variant="standard"
                    select
                    label={TEXTS[lang].addsports.submitTypeLabel}
                    value={sport.type}
                    onChange={e => update({ type: e.target.value })}
                    disabled={readonly}
                    fullWidth
                  >
                    {[
                      {
                        label: TEXTS[lang].addsports.optUnknown,
                        value: 'Unknown',
                      },
                      {
                        label: TEXTS[lang].addsports.optSwimming,
                        value: 'Swimming',
                      },
                      {
                        label: TEXTS[lang].addsports.optRunning,
                        value: 'Running',
                      },
                      {
                        label: TEXTS[lang].addsports.optCycling,
                        value: 'Cycling',
                      },
                    ].map(opt => (
                      <MenuItem key={opt.value} value={opt.value}>
                        {opt.label}
                      </MenuItem>
                    ))}
                  </TextField>
                  <TextField
                    variant="standard"
                    label={TEXTS[lang].addsports.submitStartTimeLabel}
                    type="datetime-local"
                    value={toInputDateTime(sport.start_time)}
                    slotProps={{ htmlInput: { step: 1 } }}
                    onChange={e =>
                      update({ start_time: fromInputDateTime(e.target.value) })
                    }
                    disabled={readonly}
                    fullWidth
                  />
                </Stack>
                <Stack direction="row" spacing={2}>
                  <TextField
                    variant="standard"
                    label={TEXTS[lang].addsports.submitCaloriesLabel}
                    type="number"
                    value={sport.calories === 0 ? '' : sport.calories}
                    onChange={e =>
                      update({
                        calories: Number.parseInt(e.target.value || '0'),
                      })
                    }
                    disabled={readonly}
                    fullWidth
                  />
                  <TextField
                    variant="standard"
                    label={TEXTS[lang].addsports.submitDistanceLabel}
                    type="number"
                    value={
                      sport.distance_meter === 0 ? '' : sport.distance_meter
                    }
                    onChange={e =>
                      update({
                        distance_meter: Number.parseInt(e.target.value || '0'),
                      })
                    }
                    disabled={readonly}
                    fullWidth
                  />
                </Stack>
                <Stack direction="row" spacing={2}>
                  <TextField
                    variant="standard"
                    label={TEXTS[lang].addsports.submitDurationLabel}
                    type="time"
                    value={toHMS(sport.duration_second)}
                    slotProps={{ htmlInput: { step: 1 } }}
                    onChange={e =>
                      update({ duration_second: fromHMS(e.target.value) })
                    }
                    disabled={readonly}
                    fullWidth
                  />
                  <TextField
                    variant="standard"
                    label={TEXTS[lang].addsports.submitPaceLabel}
                    value={sport.pace_average}
                    onChange={e => update({ pace_average: e.target.value })}
                    disabled={readonly}
                    fullWidth
                  />
                </Stack>
                <Stack direction="row" spacing={2}>
                  <TextField
                    variant="standard"
                    label={TEXTS[lang].addsports.submitHRAvgLabel}
                    type="number"
                    value={
                      sport.heart_rate_avg === 0 ? '' : sport.heart_rate_avg
                    }
                    onChange={e =>
                      update({
                        heart_rate_avg: Number.parseInt(e.target.value || '0'),
                      })
                    }
                    disabled={readonly}
                    fullWidth
                  />
                  <TextField
                    variant="standard"
                    label={TEXTS[lang].addsports.submitHRMaxLabel}
                    type="number"
                    value={
                      sport.heart_rate_max === 0 ? '' : sport.heart_rate_max
                    }
                    onChange={e =>
                      update({
                        heart_rate_max: Number.parseInt(e.target.value || '0'),
                      })
                    }
                    disabled={readonly}
                    fullWidth
                  />
                </Stack>
              </Stack>
            </Paper>
          </Stack>

          <Stack spacing={1} sx={{ mt: 4 }}>
            <Stack
              direction="row"
              spacing={1}
              alignItems="center"
              sx={{ pl: 1 }}
            >
              <Pool fontSize="small" />
              <Typography variant="subtitle1">
                {TEXTS[lang].addsports.submitSwimTitle}
              </Typography>
            </Stack>
            <Paper elevation={3} sx={{ p: 2, borderRadius: 2 }}>
              <Stack spacing={2} sx={{ pb: 2 }}>
                <Stack direction="row" spacing={2}>
                  <TextField
                    variant="standard"
                    select
                    label={TEXTS[lang].addsports.submitStrokeLabel}
                    value={sport.extra.main_stroke}
                    onChange={e => updateExtra({ main_stroke: e.target.value })}
                    disabled={readonly}
                    fullWidth
                  >
                    {[
                      {
                        label: TEXTS[lang].addsports.strokeUnknown,
                        value: 'unknown',
                      },
                      {
                        label: TEXTS[lang].addsports.strokeFreestyle,
                        value: 'freestyle',
                      },
                      {
                        label: TEXTS[lang].addsports.strokeButterfly,
                        value: 'butterfly',
                      },
                      {
                        label: TEXTS[lang].addsports.strokeBreaststroke,
                        value: 'breaststroke',
                      },
                      {
                        label: TEXTS[lang].addsports.strokeBackstroke,
                        value: 'backstroke',
                      },
                      {
                        label: TEXTS[lang].addsports.strokeMedley,
                        value: 'medley',
                      },
                    ].map(opt => (
                      <MenuItem key={opt.value} value={opt.value}>
                        {opt.label}
                      </MenuItem>
                    ))}
                  </TextField>
                  <TextField
                    variant="standard"
                    label={TEXTS[lang].addsports.submitStrokeAvgLabel}
                    type="number"
                    value={
                      sport.extra.stroke_avg === 0 ? '' : sport.extra.stroke_avg
                    }
                    onChange={e =>
                      updateExtra({
                        stroke_avg: Number.parseInt(e.target.value || '0'),
                      })
                    }
                    disabled={readonly}
                    fullWidth
                  />
                  <TextField
                    variant="standard"
                    label={TEXTS[lang].addsports.submitSwolfAvgLabel}
                    type="number"
                    value={
                      sport.extra.swolf_avg === 0 ? '' : sport.extra.swolf_avg
                    }
                    onChange={e =>
                      updateExtra({
                        swolf_avg: Number.parseInt(e.target.value || '0'),
                      })
                    }
                    disabled={readonly}
                    fullWidth
                  />
                </Stack>
              </Stack>
            </Paper>
          </Stack>

          <Stack spacing={1} sx={{ mt: 4 }}>
            <Stack
              direction="row"
              spacing={1}
              alignItems="center"
              sx={{ pl: 1 }}
            >
              <Timeline fontSize="small" />
              <Typography variant="subtitle1" noWrap>
                {TEXTS[lang].addsports.submitTracksTitle}
              </Typography>
              <Box sx={{ flex: 1 }} />
              {readonly ? null : (
                <Fab
                  size="small"
                  aria-label="add-track"
                  onClick={addTrack}
                  sx={{
                    bgcolor: 'success.main',
                    color: 'common.white',
                    width: 28,
                    height: 28,
                    minWidth: 28,
                    minHeight: 28,
                    borderRadius: '50%',
                    p: 0,
                    boxShadow: '0 2px 8px rgba(0,0,0,0.24)',
                    '&:hover': {
                      bgcolor: 'success.dark',
                      boxShadow: '0 3px 12px rgba(0,0,0,0.28)',
                    },
                    '&:active': { boxShadow: '0 1px 6px rgba(0,0,0,0.25)' },
                  }}
                >
                  <Add sx={{ fontSize: 16 }} />
                </Fab>
              )}
            </Stack>
            <Paper elevation={3} sx={{ p: 2, borderRadius: 2 }}>
              {sport.tracks.length === 0 ? (
                <Typography
                  variant="body2"
                  color="text.secondary"
                  sx={{ textAlign: 'center', py: 4 }}
                >
                  无分段数据
                </Typography>
              ) : (
                <Stack spacing={1}>
                  {sport.tracks.map((t, idx) => {
                    const strokeLabel = (v: string) => {
                      switch (v) {
                        case 'freestyle':
                          return TEXTS[lang].addsports.strokeFreestyle;
                        case 'butterfly':
                          return TEXTS[lang].addsports.strokeButterfly;
                        case 'breaststroke':
                          return TEXTS[lang].addsports.strokeBreaststroke;
                        case 'backstroke':
                          return TEXTS[lang].addsports.strokeBackstroke;
                        case 'medley':
                          return TEXTS[lang].addsports.strokeMedley;
                        default:
                          return TEXTS[lang].addsports.strokeUnknown;
                      }
                    };
                    return (
                      <Box
                        key={`${t.distance_meter}-${t.duration_second}-${t.pace_average}-${t.extra.main_stroke}`}
                        sx={{
                          display: 'grid',
                          gridTemplateColumns: '1fr auto',
                          gridTemplateRows: 'auto auto',
                          columnGap: 2,
                          rowGap: 1,
                          alignItems: 'start',
                        }}
                      >
                        <Box
                          onClick={readonly ? undefined : () => editTrack(idx)}
                          sx={{
                            gridColumn: 1,
                            gridRow: '1 / span 2',
                            px: 1,
                            py: 1,
                            borderRadius: 1,
                            cursor: readonly ? 'default' : 'pointer',
                            '&:hover': readonly
                              ? undefined
                              : { bgcolor: 'action.hover' },
                            transition: 'background-color 0.2s ease',
                          }}
                        >
                          <Box
                            sx={{
                              display: 'grid',
                              gridTemplateColumns: 'repeat(3, 1fr)',
                              columnGap: 2,
                              rowGap: 1,
                            }}
                          >
                            <Box sx={{ minWidth: 0 }}>
                              <Typography
                                variant="caption"
                                color="text.secondary"
                                noWrap
                              >
                                {TEXTS[lang].addsports.submitDistanceLabel}
                              </Typography>
                              <Typography
                                variant="body1"
                                noWrap
                                sx={{ fontWeight: 600 }}
                              >
                                {t.distance_meter} m
                              </Typography>
                            </Box>
                            <Box sx={{ minWidth: 0 }}>
                              <Typography
                                variant="caption"
                                color="text.secondary"
                                noWrap
                              >
                                {TEXTS[lang].addsports.submitDurationLabel}
                              </Typography>
                              <Typography
                                variant="body1"
                                noWrap
                                sx={{ fontWeight: 600 }}
                              >
                                {toHMS(t.duration_second)}
                              </Typography>
                            </Box>
                            <Box sx={{ minWidth: 0 }}>
                              <Typography
                                variant="caption"
                                color="text.secondary"
                                noWrap
                              >
                                {TEXTS[lang].addsports.submitPaceLabel}
                              </Typography>
                              <Typography
                                variant="body1"
                                noWrap
                                sx={{ fontWeight: 600 }}
                              >
                                {t.pace_average}
                              </Typography>
                            </Box>
                            <Box sx={{ minWidth: 0 }}>
                              <Typography
                                variant="caption"
                                color="text.secondary"
                                noWrap
                              >
                                {TEXTS[lang].addsports.submitStrokeLabel}
                              </Typography>
                              <Typography
                                variant="body1"
                                noWrap
                                sx={{ fontWeight: 600 }}
                              >
                                {strokeLabel(t.extra.main_stroke)}
                              </Typography>
                            </Box>
                            <Box sx={{ minWidth: 0 }}>
                              <Typography
                                variant="caption"
                                color="text.secondary"
                                noWrap
                              >
                                {TEXTS[lang].addsports.submitStrokeAvgLabel}
                              </Typography>
                              <Typography
                                variant="body1"
                                noWrap
                                sx={{ fontWeight: 600 }}
                              >
                                {t.extra.stroke_avg}
                              </Typography>
                            </Box>
                            <Box sx={{ minWidth: 0 }}>
                              <Typography
                                variant="caption"
                                color="text.secondary"
                                noWrap
                              >
                                {TEXTS[lang].addsports.submitSwolfAvgLabel}
                              </Typography>
                              <Typography
                                variant="body1"
                                noWrap
                                sx={{ fontWeight: 600 }}
                              >
                                {t.extra.swolf_avg}
                              </Typography>
                            </Box>
                          </Box>
                        </Box>
                        {readonly ? null : (
                          <IconButton
                            aria-label="delete"
                            onClick={e => {
                              e.stopPropagation();
                              removeTrack(idx);
                            }}
                            sx={{
                              gridColumn: 2,
                              gridRow: 2,
                              width: 40,
                              border: '1px solid',
                              borderColor: 'error.main',
                              color: 'error.main',
                              borderRadius: 1,
                              display: 'flex',
                              alignItems: 'center',
                              justifyContent: 'center',
                              '&:hover': {
                                bgcolor: 'error.light',
                                color: '#fff',
                              },
                            }}
                          >
                            <Delete />
                          </IconButton>
                        )}
                      </Box>
                    );
                  })}
                </Stack>
              )}
            </Paper>
          </Stack>
          <Dialog
            open={trackDialogOpen}
            onClose={() => {
              setTrackDialogOpen(false);
              setEditingIndex(null);
            }}
            maxWidth="md"
            fullWidth
          >
            <DialogTitle>{TEXTS[lang].addsports.submitTrackAdd}</DialogTitle>
            <DialogContent>
              <Stack spacing={2} sx={{ pt: 1 }}>
                <TextField
                  variant="standard"
                  label={TEXTS[lang].addsports.submitDistanceLabel}
                  type="number"
                  value={
                    trackDraft.distance_meter === 0
                      ? ''
                      : trackDraft.distance_meter
                  }
                  onChange={e =>
                    updateTrackDraft({
                      distance_meter: Number.parseInt(e.target.value || '0'),
                    })
                  }
                  fullWidth
                />
                <TextField
                  variant="standard"
                  label={TEXTS[lang].addsports.submitDurationLabel}
                  type="time"
                  value={toHMS(trackDraft.duration_second)}
                  slotProps={{ htmlInput: { step: 1 } }}
                  onChange={e =>
                    updateTrackDraft({
                      duration_second: fromHMS(e.target.value),
                    })
                  }
                  fullWidth
                />
                <TextField
                  variant="standard"
                  label={TEXTS[lang].addsports.submitPaceLabel}
                  value={trackDraft.pace_average}
                  onChange={e =>
                    updateTrackDraft({ pace_average: e.target.value })
                  }
                  fullWidth
                />
                <TextField
                  variant="standard"
                  select
                  label={TEXTS[lang].addsports.submitStrokeLabel}
                  value={trackDraft.extra.main_stroke}
                  onChange={e =>
                    updateTrackDraft({
                      extra: {
                        ...trackDraft.extra,
                        main_stroke: e.target.value,
                      },
                    })
                  }
                  fullWidth
                >
                  {[
                    {
                      label: TEXTS[lang].addsports.strokeUnknown,
                      value: 'unknown',
                    },
                    {
                      label: TEXTS[lang].addsports.strokeFreestyle,
                      value: 'freestyle',
                    },
                    {
                      label: TEXTS[lang].addsports.strokeButterfly,
                      value: 'butterfly',
                    },
                    {
                      label: TEXTS[lang].addsports.strokeBreaststroke,
                      value: 'breaststroke',
                    },
                    {
                      label: TEXTS[lang].addsports.strokeBackstroke,
                      value: 'backstroke',
                    },
                    {
                      label: TEXTS[lang].addsports.strokeMedley,
                      value: 'medley',
                    },
                  ].map(opt => (
                    <MenuItem key={opt.value} value={opt.value}>
                      {opt.label}
                    </MenuItem>
                  ))}
                </TextField>
                <TextField
                  variant="standard"
                  label={TEXTS[lang].addsports.submitStrokeAvgLabel}
                  type="number"
                  value={
                    trackDraft.extra.stroke_avg === 0
                      ? ''
                      : trackDraft.extra.stroke_avg
                  }
                  onChange={e =>
                    updateTrackDraft({
                      extra: {
                        ...trackDraft.extra,
                        stroke_avg: Number.parseInt(e.target.value || '0'),
                      },
                    })
                  }
                  fullWidth
                />
                <TextField
                  variant="standard"
                  label={TEXTS[lang].addsports.submitSwolfAvgLabel}
                  type="number"
                  value={
                    trackDraft.extra.swolf_avg === 0
                      ? ''
                      : trackDraft.extra.swolf_avg
                  }
                  onChange={e =>
                    updateTrackDraft({
                      extra: {
                        ...trackDraft.extra,
                        swolf_avg: Number.parseInt(e.target.value || '0'),
                      },
                    })
                  }
                  fullWidth
                />
              </Stack>
            </DialogContent>
            <DialogActions>
              <Button
                variant="outlined"
                onClick={() => {
                  setTrackDialogOpen(false);
                  setEditingIndex(null);
                }}
              >
                {TEXTS[lang].register.cancel}
              </Button>
              <Button
                variant="contained"
                onClick={() => {
                  if (editingIndex !== null) {
                    const next = sport.tracks.slice();
                    next[editingIndex] = trackDraft;
                    update({ tracks: next });
                  } else {
                    update({ tracks: [...sport.tracks, trackDraft] });
                  }
                  setTrackDialogOpen(false);
                  setTrackDraft(defaultTrack);
                  setEditingIndex(null);
                }}
              >
                {TEXTS[lang].addsports.submitButton}
              </Button>
            </DialogActions>
          </Dialog>
        </Stack>
      </Container>

      {readonly ? null : (
        <Box
          sx={{
            position: 'fixed',
            left: 0,
            right: 0,
            bottom: 'calc(env(safe-area-inset-bottom) + 20px)',
            display: 'flex',
            justifyContent: 'center',
            px: 2,
          }}
        >
          <Box sx={{ display: 'flex', gap: 2, width: '100%', maxWidth: 380 }}>
            <Button variant="contained" onClick={handleSubmit} fullWidth>
              {TEXTS[lang].addsports.submitButton}
            </Button>
          </Box>
        </Box>
      )}
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
