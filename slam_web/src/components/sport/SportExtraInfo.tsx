import { Pool, DirectionsRun, HelpOutline } from '@mui/icons-material';
import { MenuItem, Paper, Stack, TextField, Typography } from '@mui/material';
import { TEXTS } from '../../i18n';
import type { Sport, Swimming, Running, SportExtra } from '../../services/sport';
import { getSportType, SportType } from '../../services/sport';
import { getExtraConfigByType, groupByLayout, type FieldConfig, type LayoutConfig } from './ExtraConfig';

export default function SportExtraInfo({
  lang,
  sport,
  readonly,
  updateExtra,
}: {
  lang: 'zh' | 'en';
  sport: Sport;
  readonly: boolean;
  updateExtra: (patch: Partial<SportExtra>) => void;
}) {
  const sportType = getSportType(sport.type);
  const ICON_BY_TYPE: Record<SportType, JSX.Element> = {
    [SportType.Swimming]: <Pool fontSize="small" />,
    [SportType.Running]: <DirectionsRun fontSize="small" />,
    [SportType.Cycling]: <HelpOutline fontSize="small" />,
    [SportType.Unknown]: <HelpOutline fontSize="small" />,
  };

  // 直接通过 FieldConfig 的默认值做字段级回退，无需整体 model
  const extra = sport.extra as any;

  const fields: FieldConfig[] = getExtraConfigByType(lang, sportType);
  const EXTRA_LAYOUT_BY_TYPE: Record<SportType, number[]> = {
    [SportType.Swimming]: [3],
    [SportType.Running]: [2, 2, 2],
    [SportType.Cycling]: [],
    [SportType.Unknown]: [],
  };
  const layout: LayoutConfig = {
    rowFieldCounts: EXTRA_LAYOUT_BY_TYPE[sportType] ?? [],
  };
  const rows: FieldConfig[][] = groupByLayout(fields, layout);
  return (
    <Stack spacing={1} sx={{ maxWidth: 500, width: '100%' }}>
      <Stack direction="row" spacing={1} alignItems="center" sx={{ pl: 1 }}>
        {ICON_BY_TYPE[sportType]}
        <Typography variant="subtitle1">
          {TEXTS[lang].addsports.submitExtraTitle}
        </Typography>
      </Stack>
      <Paper elevation={3} sx={{ p: 2, borderRadius: 2 }}>
        {rows.length === 0 ? (
          <Typography variant="body2" color="text.secondary">
            {TEXTS[lang].addsports.noExtraData}
          </Typography>
        ) : (
          <Stack spacing={2} sx={{ pb: 2 }}>
            {rows.map((row, rIdx) => (
              <Stack key={rIdx} direction="row" spacing={2}>
                {row.map(cfg => {
                  const rawVal = extra?.[cfg.key];
                  const baseVal = rawVal ?? cfg.default;
                  const value = cfg.kind === 'number'
                    ? (baseVal === 0 ? '' : baseVal)
                    : (baseVal ?? '');
                  if (cfg.kind === 'select') {
                    return (
                      <TextField
                        key={cfg.key}
                        variant="standard"
                        select
                        label={cfg.label}
                        value={value}
                        onChange={e => updateExtra({ [cfg.key]: e.target.value } as any)}
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
                        updateExtra({ [cfg.key]: parsed } as any);
                      }}
                      InputProps={{ readOnly: readonly }}
                      fullWidth
                    />
                  );
                })}
              </Stack>
            ))}
          </Stack>
        )}
      </Paper>
    </Stack>
  );
}
