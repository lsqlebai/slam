import { Info } from '@mui/icons-material';
import { MenuItem, Paper, Stack, TextField, Typography } from '@mui/material';
import { TEXTS } from '../../i18n';
import type { Sport } from '../../services/sport';
import {
  fromHMS,
  fromInputDateTime,
  toDisplayDateTime,
  toHMS,
  toInputDateTime,
} from '../../utils/time';

export default function SportBasicInfo({
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
  return (
    <Stack spacing={1} sx={{ mt: 0, maxWidth: 500, width: '100%' }}>
      <Stack direction="row" spacing={1} alignItems="center" sx={{ pl: 1 }}>
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
              sx={{
                '& .MuiInputBase-input.Mui-disabled': {
                  WebkitTextFillColor: 'inherit',
                  color: 'text.primary',
                },
                '& .MuiInputLabel-root.Mui-disabled': {
                  color: 'text.secondary',
                },
              }}
              fullWidth
            >
              {[
                { label: TEXTS[lang].addsports.optUnknown, value: 'Unknown' },
                { label: TEXTS[lang].addsports.optSwimming, value: 'Swimming' },
                { label: TEXTS[lang].addsports.optRunning, value: 'Running' },
                { label: TEXTS[lang].addsports.optCycling, value: 'Cycling' },
              ].map(opt => (
                <MenuItem key={opt.value} value={opt.value}>
                  {opt.label}
                </MenuItem>
              ))}
            </TextField>
            <TextField
              variant="standard"
              label={TEXTS[lang].addsports.submitStartTimeLabel}
              type={readonly ? 'text' : 'datetime-local'}
              value={
                readonly
                  ? toDisplayDateTime(sport.start_time)
                  : toInputDateTime(sport.start_time)
              }
              slotProps={readonly ? undefined : { htmlInput: { step: 1 } }}
              onChange={e =>
                update({ start_time: fromInputDateTime(e.target.value) })
              }
              InputProps={{ readOnly: readonly }}
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
                update({ calories: Number.parseInt(e.target.value || '0') })
              }
              InputProps={{ readOnly: readonly }}
              fullWidth
            />
            <TextField
              variant="standard"
              label={TEXTS[lang].addsports.submitDistanceLabel}
              type="number"
              value={sport.distance_meter === 0 ? '' : sport.distance_meter}
              onChange={e =>
                update({
                  distance_meter: Number.parseInt(e.target.value || '0'),
                })
              }
              InputProps={{ readOnly: readonly }}
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
              InputProps={{ readOnly: readonly }}
              fullWidth
            />
            <TextField
              variant="standard"
              label={TEXTS[lang].addsports.submitPaceLabel}
              value={sport.pace_average}
              onChange={e => update({ pace_average: e.target.value })}
              InputProps={{ readOnly: readonly }}
              fullWidth
            />
          </Stack>
          <Stack direction="row" spacing={2}>
            <TextField
              variant="standard"
              label={TEXTS[lang].addsports.submitHRAvgLabel}
              type="number"
              value={sport.heart_rate_avg === 0 ? '' : sport.heart_rate_avg}
              onChange={e =>
                update({
                  heart_rate_avg: Number.parseInt(e.target.value || '0'),
                })
              }
              InputProps={{ readOnly: readonly }}
              fullWidth
            />
            <TextField
              variant="standard"
              label={TEXTS[lang].addsports.submitHRMaxLabel}
              type="number"
              value={sport.heart_rate_max === 0 ? '' : sport.heart_rate_max}
              onChange={e =>
                update({
                  heart_rate_max: Number.parseInt(e.target.value || '0'),
                })
              }
              InputProps={{ readOnly: readonly }}
              fullWidth
            />
          </Stack>
        </Stack>
      </Paper>
    </Stack>
  );
}
