import { Delete } from '@mui/icons-material';
import {
  Box,
  Button,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  Typography,
} from '@mui/material';
import { useState } from 'react';
import { TEXTS } from '../../i18n';
import type { Track as TrackType } from '../../services/sport';
import { getSportType } from '../../services/sport';
import { getExtraConfigByType, groupByLayout, makeUniformLayout, type FieldConfig } from './ExtraConfig';

export default function TrackItem({
  lang,
  idx,
  t,
  readonly,
  sportType,
  onEdit,
  onDelete,
}: {
  lang: 'zh' | 'en';
  idx: number;
  t: TrackType;
  readonly: boolean;
  sportType?: string;
  onEdit: (idx: number) => void;
  onDelete: (idx: number) => void;
}) {
  const [confirmOpen, setConfirmOpen] = useState(false);
  const sportTypeEnum = getSportType(sportType);
  const fields: FieldConfig[] = getExtraConfigByType(lang, sportTypeEnum);
  const layout = makeUniformLayout(fields.length, 4);
  const rows = groupByLayout(fields, layout);

  return (
    <Box
      sx={{
        display: 'grid',
        gridTemplateColumns: '1fr',
        columnGap: 0,
        rowGap: 1,
        alignItems: 'start',
        borderTop: idx === 0 ? 'none' : '1px solid',
        borderColor: 'divider',
        mt: idx === 0 ? 0 : 1,
        pt: idx === 0 ? 0 : 1,
      }}
    >
      <Box
        onClick={readonly ? undefined : () => onEdit(idx)}
        sx={{
          px: 1,
          py: 1,
          borderRadius: 1,
          cursor: readonly ? 'default' : 'pointer',
          '&:hover': readonly ? undefined : { bgcolor: 'action.hover' },
          transition: 'background-color 0.2s ease',
        }}
      >
        <Box
          sx={{
            display: 'grid',
            gridTemplateColumns: 'repeat(4, 1fr)',
            columnGap: 2,
            rowGap: 0.5,
          }}
        >
          <Box
            sx={{
              minWidth: 0,
              display: 'flex',
              flexDirection: 'column',
              justifyContent: 'center',
            }}
          >
            <Typography
              variant="subtitle1"
              noWrap
              sx={{ fontWeight: 800, color: 'primary.main' }}
            >
              {lang === 'zh' ? `第${idx + 1}段` : `Segment ${idx + 1}`}
            </Typography>
          </Box>
          <Box sx={{ minWidth: 0 }}>
            <Typography variant="caption" color="text.secondary" noWrap>
              {TEXTS[lang].addsports.submitDistanceLabel}
            </Typography>
            <Typography variant="body1" noWrap sx={{ fontWeight: 600 }}>
              {t.distance_meter} m
            </Typography>
          </Box>
          <Box sx={{ minWidth: 0 }}>
            <Typography variant="caption" color="text.secondary" noWrap>
              {TEXTS[lang].addsports.submitDurationLabel}
            </Typography>
            <Typography variant="body1" noWrap sx={{ fontWeight: 600 }}>
              {t.duration_second >= 0
                ? `${String(Math.floor(t.duration_second / 3600)).padStart(2, '0')}:${String(Math.floor((t.duration_second % 3600) / 60)).padStart(2, '0')}:${String(t.duration_second % 60).padStart(2, '0')}`
                : ''}
            </Typography>
          </Box>
          <Box sx={{ minWidth: 0 }}>
            <Typography variant="caption" color="text.secondary" noWrap>
              {TEXTS[lang].addsports.submitPaceLabel}
            </Typography>
            <Typography variant="body1" noWrap sx={{ fontWeight: 600 }}>
              {t.pace_average}
            </Typography>
          </Box>
          {rows.length > 0 && t.extra
            ? rows
                .flat()
                .map(cfg => {
                  const raw = (t.extra as any)?.[cfg.key];
                  const value = raw ?? cfg.default;
                  const display =
                    cfg.kind === 'select' && cfg.options
                      ? (cfg.options.find(o => o.value === value)?.label ?? value)
                      : value;
                  return (
                    <Box key={`extra-${cfg.key}`} sx={{ minWidth: 0 }}>
                      <Typography variant="caption" color="text.secondary" noWrap>
                        {cfg.label}
                      </Typography>
                      <Typography variant="body1" noWrap sx={{ fontWeight: 600 }}>
                        {String(display)}
                      </Typography>
                    </Box>
                  );
                })
            : null}
          <Box
            sx={{
              minWidth: 0,
              display: 'flex',
              alignItems: 'flex-end',
              justifyContent: 'flex-end',
              gridColumn: '4',
            }}
          >
            {readonly ? null : (
              <Button
                variant="outlined"
                color="error"
                size="small"
                startIcon={<Delete />}
                onClick={e => {
                  e.stopPropagation();
                  setConfirmOpen(true);
                }}
                sx={{
                  whiteSpace: 'nowrap',
                  fontSize: 12,
                  lineHeight: 1.6,
                  minWidth: 0,
                  px: 1,
                  '& .MuiButton-startIcon': { mr: 0.5 },
                }}
              >
                {TEXTS[lang].addsports.submitTrackDelete}
              </Button>
            )}
          </Box>
        </Box>
      </Box>
      <Dialog
        open={confirmOpen}
        onClose={() => setConfirmOpen(false)}
        maxWidth="xs"
        fullWidth
      >
        <DialogTitle>{TEXTS[lang].addsports.deleteConfirmTitle}</DialogTitle>
        <DialogContent>
          <Typography variant="body2" color="text.secondary">
            {TEXTS[lang].addsports.deleteConfirmMessage}
          </Typography>
        </DialogContent>
        <DialogActions>
          <Button variant="outlined" onClick={() => setConfirmOpen(false)}>
            {TEXTS[lang].register.cancel}
          </Button>
          <Button
            variant="contained"
            color="error"
            onClick={() => {
              onDelete(idx);
              setConfirmOpen(false);
            }}
          >
            {TEXTS[lang].addsports.submitTrackDelete}
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
}
