import { Add } from '@mui/icons-material';
import {
  Box,
  IconButton,
  Stack,
  type SxProps,
  type Theme,
  Typography,
} from '@mui/material';
import { useRef } from 'react';
import type { Lang } from '../../i18n';
import { TEXTS } from '../../i18n';

export default function ImagePickerCard({
  lang,
  onFilesSelected,
  sx,
}: {
  lang: Lang;
  onFilesSelected: (files: FileList | null) => void;
  sx?: SxProps<Theme>;
}) {
  const inputRef = useRef<HTMLInputElement | null>(null);
  return (
    <Box
      sx={[
        {
          borderRadius: 1,
          bgcolor: 'grey.100',
          border: '1px solid',
          borderColor: 'divider',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          cursor: 'pointer',
          px: { xs: 3, md: 2 },
          width: { xs: '100%', md: 300 },
          order: { xs: 2, md: 2 },
          height: { xs: 120, md: 'auto' },
          my: { md: 6 },
          alignSelf: { md: 'stretch' },
          flex: { md: '0 0 auto' },
        },
        ...(Array.isArray(sx) ? sx : sx ? [sx] : []),
      ]}
      onClick={() => inputRef.current?.click()}
    >
      <input
        ref={inputRef}
        type="file"
        accept="image/*"
        multiple
        onChange={e => onFilesSelected(e.target.files)}
        style={{ display: 'none' }}
      />
      <Stack direction="row" spacing={1} alignItems="center">
        <IconButton aria-label="add-image" color="primary">
          <Add />
        </IconButton>
        <Typography variant="body2" color="text.secondary">
          {TEXTS[lang].addsports.pickImages}
        </Typography>
      </Stack>
    </Box>
  );
}
