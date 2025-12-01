import { Pool } from '@mui/icons-material';
import { MenuItem, Paper, Stack, TextField, Typography } from '@mui/material';
import { TEXTS } from '../../i18n';
import type { Sport, Swimming } from '../../services/sport';

export default function SportExtraInfo({
  lang,
  sport,
  readonly,
  updateExtra,
}: {
  lang: 'zh' | 'en';
  sport: Sport;
  readonly: boolean;
  updateExtra: (patch: Partial<Swimming>) => void;
}) {
  const extra = sport.extra ?? {
    main_stroke: 'unknown',
    stroke_avg: 0,
    swolf_avg: 0,
  };
  return (
    <Stack spacing={1} sx={{ maxWidth: 500, width: '100%' }}>
      <Stack direction="row" spacing={1} alignItems="center" sx={{ pl: 1 }}>
        <Pool fontSize="small" />
        <Typography variant="subtitle1">
          {TEXTS[lang].addsports.submitExtraTitle}
        </Typography>
      </Stack>
      <Paper elevation={3} sx={{ p: 2, borderRadius: 2 }}>
        <Stack spacing={2} sx={{ pb: 2 }}>
          <Stack direction="row" spacing={2}>
            <TextField
              variant="standard"
              select
              label={TEXTS[lang].addsports.submitStrokeLabel}
              value={extra.main_stroke}
              onChange={e => updateExtra({ main_stroke: e.target.value })}
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
              value={extra.stroke_avg === 0 ? '' : extra.stroke_avg}
              onChange={e =>
                updateExtra({
                  stroke_avg: Number.parseInt(e.target.value || '0'),
                })
              }
              InputProps={{ readOnly: readonly }}
              fullWidth
            />
            <TextField
              variant="standard"
              label={TEXTS[lang].addsports.submitSwolfAvgLabel}
              type="number"
              value={extra.swolf_avg === 0 ? '' : extra.swolf_avg}
              onChange={e =>
                updateExtra({
                  swolf_avg: Number.parseInt(e.target.value || '0'),
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
