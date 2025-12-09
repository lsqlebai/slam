import {
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  Button,
  Stack,
  TextField,
  MenuItem,
} from '@mui/material';
import { TEXTS } from '../../i18n';
import type { Track } from '../../services/sport';
import { SportType } from '../../services/sport';
import { fromHMS, toHMS } from '../../utils/time';
import { getExtraConfigByType, groupByLayout, type FieldConfig, type LayoutConfig, getDefaultExtraByType } from './ExtraConfig';

export default function TrackDialog({
  lang,
  open,
  trackDraft,
  sportType,
  onChange,
  onCancel,
  onSubmit,
}: {
  lang: 'zh' | 'en';
  open: boolean;
  trackDraft: Track;
  sportType: SportType;
  onChange: (patch: Partial<Track>) => void;
  onCancel: () => void;
  onSubmit: () => void;
}) {
  const fields: FieldConfig[] = getExtraConfigByType(lang, sportType);
  const EXTRA_LAYOUT_BY_TYPE: Record<SportType, number[]> = {
    [SportType.Swimming]: [3],
    [SportType.Running]: [2, 2, 2],
    [SportType.Cycling]: [],
    [SportType.Unknown]: [],
  };
  const layout: LayoutConfig = { rowFieldCounts: EXTRA_LAYOUT_BY_TYPE[sportType] ?? [] };
  const rows = groupByLayout(fields, layout);
  return (
    <Dialog
      open={open}
      onClose={onCancel}
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
            value={trackDraft.distance_meter === 0 ? '' : trackDraft.distance_meter}
            onChange={e =>
              onChange({
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
            onChange={e => onChange({ duration_second: fromHMS(e.target.value) })}
            fullWidth
          />
          <TextField
            variant="standard"
            label={TEXTS[lang].addsports.submitPaceLabel}
            value={trackDraft.pace_average}
            onChange={e => onChange({ pace_average: e.target.value })}
            fullWidth
          />
          {rows.length > 0 ? (
            <Stack spacing={2}>
              {rows.map((row, rIdx) => (
                <Stack key={rIdx} direction="row" spacing={2}>
                  {row.map(cfg => {
                    const rawVal = (trackDraft.extra as any)?.[cfg.key];
                    const baseVal = rawVal ?? cfg.default;
                    const value = cfg.kind === 'number' ? (baseVal === 0 ? '' : baseVal) : (baseVal ?? '');
                    if (cfg.kind === 'select') {
                      return (
                        <TextField
                          key={cfg.key}
                          variant="standard"
                          select
                          label={cfg.label}
                          value={value}
                          onChange={e => {
                            const base = (trackDraft.extra ?? getDefaultExtraByType(sportType) ?? {}) as any;
                            onChange({ extra: { ...base, [cfg.key]: e.target.value } });
                          }}
                          fullWidth
                        >
                          {(cfg.options || []).map(opt => (
                            <MenuItem key={opt.value} value={opt.value}>
                              {opt.label}
                            </MenuItem>
                          ))}
                        </TextField>
                      );
                    }
                    const type = cfg.kind === 'number' ? 'number' : 'text';
                    return (
                      <TextField
                        key={cfg.key}
                        variant="standard"
                        label={cfg.label}
                        type={type}
                        value={value}
                        onChange={e => {
                          const parsed = cfg.kind === 'number'
                            ? (cfg.parse ? cfg.parse(e.target.value) : Number.parseInt(e.target.value || '0'))
                            : e.target.value;
                          const base = (trackDraft.extra ?? getDefaultExtraByType(sportType) ?? {}) as any;
                          onChange({ extra: { ...base, [cfg.key]: parsed } });
                        }}
                        fullWidth
                      />
                    );
                  })}
                </Stack>
              ))}
            </Stack>
          ) : null}
        </Stack>
      </DialogContent>
      <DialogActions>
        <Button variant="outlined" onClick={onCancel}>
          {TEXTS[lang].register.cancel}
        </Button>
        <Button variant="contained" onClick={onSubmit}>
          {TEXTS[lang].addsports.submitButton}
        </Button>
      </DialogActions>
    </Dialog>
  );
}
