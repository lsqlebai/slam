import { Box, Typography } from '@mui/material';
import { TEXTS } from '../../i18n';
import type { Lang } from '../../i18n';

export default function Stats({ lang }: { lang: Lang }) {
  return (
    <Box sx={{ px: 2, py: 2 }}>
      <Typography variant="h5">{TEXTS[lang].home.stats}</Typography>
    </Box>
  );
}
;