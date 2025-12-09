import { Add, Timeline } from '@mui/icons-material';
import { Box, Fab, Paper, Stack, Typography } from '@mui/material';
import { useState } from 'react';
import { TEXTS } from '../../i18n';
import type { Sport, Track } from '../../services/sport';
import { getSportType, SportType } from '../../services/sport';
import { updateSport } from '../../services/sport';
import { fromHMS, toHMS } from '../../utils/time';
import { useToast } from '../PageBase';
import TrackItem from './TrackItem';
import TrackDialog from './TrackDialog';
import { getDefaultTrackByType } from './ExtraConfig';

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
  const sportTypeEnum = getSportType(sport.type);
  const defaultTrack: Track = getDefaultTrackByType(sportTypeEnum);
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
                key={`${idx}-${t.distance_meter}-${t.duration_second}-${t.pace_average}`}
                lang={lang}
                idx={idx}
                t={t}
                readonly={readonly}
                sportType={sport.type}
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
      <TrackDialog
        lang={lang}
        open={trackDialogOpen}
        trackDraft={trackDraft}
        sportType={sportTypeEnum}
        onChange={updateTrackDraft}
        onCancel={() => {
          setTrackDialogOpen(false);
          setEditingIndex(null);
        }}
        onSubmit={() => {
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
      />
    </Stack>
  );
}
