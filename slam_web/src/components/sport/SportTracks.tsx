import { Add, Timeline } from '@mui/icons-material';
import {
  Box,
  Button,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  Fab,
  MenuItem,
  Paper,
  Stack,
  TextField,
  Typography,
} from '@mui/material';
import { useState } from 'react';
import { TEXTS } from '../../i18n';
import type { Sport, Track } from '../../services/sport';
import { updateSport } from '../../services/sport';
import { fromHMS, toHMS } from '../../utils/time';
import { useToast } from '../PageBase';
import TrackItem from './TrackItem';

export default function SportTracks({
  lang,
  sport,
  readonly,
  update,
}: {
  lang: 'zh' | 'en';
  sport: Sport;
  readonly: boolean;
  update: (patch: Partial<Sport>) => void;
}) {
  const { showSuccess } = useToast();
  const addsports = TEXTS[lang].addsports as (typeof TEXTS)['zh']['addsports'];
  const noTracksText = addsports.noTracksData;
  const defaultTrack: Track = {
    distance_meter: 0,
    duration_second: 0,
    pace_average: '0',
    extra: { main_stroke: 'unknown', stroke_avg: 0, swolf_avg: 0 },
  };
  const [trackDialogOpen, setTrackDialogOpen] = useState(false);
  const [trackDraft, setTrackDraft] = useState<Track>(defaultTrack);
  const [editingIndex, setEditingIndex] = useState<number | null>(null);
  const updateTrackDraft = (patch: Partial<Track>) =>
    setTrackDraft(prev => ({ ...prev, ...patch }) as Track);
  return (
    <Stack spacing={1} sx={{ maxWidth: 500, width: '100%' }}>
      <Stack direction="row" spacing={1} alignItems="center" sx={{ pl: 1 }}>
        <Timeline fontSize="small" />
        <Typography variant="subtitle1" noWrap>
          {TEXTS[lang].addsports.submitTracksTitle}
        </Typography>
        <Box sx={{ flex: 1 }} />
        {readonly ? null : (
          <Fab
            size="small"
            aria-label="add-track"
            onClick={() => {
              setEditingIndex(null);
              setTrackDraft(defaultTrack);
              setTrackDialogOpen(true);
            }}
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
            {noTracksText}
          </Typography>
        ) : (
          <Stack spacing={1}>
            {sport.tracks.map((t, idx) => (
              <TrackItem
                key={`${idx}-${t.distance_meter}-${t.duration_second}-${t.pace_average}-${t.extra.main_stroke}`}
                lang={lang}
                idx={idx}
                t={t}
                readonly={readonly}
                onEdit={() => {
                  setEditingIndex(idx);
                  setTrackDraft(sport.tracks[idx]);
                  setTrackDialogOpen(true);
                }}
                onDelete={async (delIdx: number) => {
                  const nextTracks = sport.tracks.filter(
                    (_, i) => i !== delIdx,
                  );
                  const nextSport = { ...sport, tracks: nextTracks } as Sport;
                  if (nextSport.id && nextSport.id > 0) {
                    const ok = await updateSport(nextSport);
                    if (ok) {
                      update({ tracks: nextTracks });
                      showSuccess(TEXTS[lang].addsports.deleteSuccess);
                    }
                  } else {
                    update({ tracks: nextTracks });
                    showSuccess(TEXTS[lang].addsports.deleteSuccess);
                  }
                }}
              />
            ))}
          </Stack>
        )}
      </Paper>
      <Dialog
        open={trackDialogOpen}
        onClose={() => {
          setTrackDialogOpen(false);
          setEditingIndex(null);
        }}
        fullWidth
        PaperProps={{ sx: { maxWidth: 440 } }}
      >
        <DialogTitle>{TEXTS[lang].addsports.submitTrackAdd}</DialogTitle>
        <DialogContent>
          <Stack spacing={2} sx={{ pt: 1 }}>
            <TextField
              variant="standard"
              label={TEXTS[lang].addsports.submitDistanceLabel}
              type="number"
              value={
                trackDraft.distance_meter === 0 ? '' : trackDraft.distance_meter
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
                updateTrackDraft({ duration_second: fromHMS(e.target.value) })
              }
              fullWidth
            />
            <TextField
              variant="standard"
              label={TEXTS[lang].addsports.submitPaceLabel}
              value={trackDraft.pace_average}
              onChange={e => updateTrackDraft({ pace_average: e.target.value })}
              fullWidth
            />
            <TextField
              variant="standard"
              select
              label={TEXTS[lang].addsports.submitStrokeLabel}
              value={trackDraft.extra.main_stroke}
              onChange={e =>
                updateTrackDraft({
                  extra: { ...trackDraft.extra, main_stroke: e.target.value },
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
                { label: TEXTS[lang].addsports.strokeMedley, value: 'medley' },
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
  );
}
