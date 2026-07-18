import { Close } from '@mui/icons-material';
import { Box, Dialog, DialogContent, IconButton } from '@mui/material';

export default function ImagePreviewDialog({
  open,
  imageUrl,
  onClose,
}: {
  open: boolean;
  imageUrl: string | null;
  onClose: () => void;
}) {
  return (
    <Dialog
      open={open}
      onClose={onClose}
      maxWidth="md"
      fullWidth
      slotProps={{
        paper: {
          sx: {
            m: { xs: 0, sm: 4 },
            width: { xs: '100%', sm: 'calc(100% - 64px)' },
            maxHeight: { xs: '100dvh', sm: 'calc(100% - 64px)' },
            borderRadius: { xs: 0, sm: 1 },
            bgcolor: 'grey.950',
          },
        },
      }}
    >
      <DialogContent
        sx={{
          p: 0,
          position: 'relative',
          minHeight: { xs: '100dvh', sm: 0 },
          display: 'grid',
          placeItems: 'center',
        }}
      >
        <IconButton
          aria-label="close-preview"
          onClick={onClose}
          sx={{
            position: 'absolute',
            top: 'calc(env(safe-area-inset-top) + 8px)',
            left: 'calc(env(safe-area-inset-left) + 8px)',
            backgroundColor: 'rgba(0,0,0,0.35)',
            color: '#fff',
            '&:hover': { backgroundColor: 'rgba(0,0,0,0.5)' },
            zIndex: 1,
          }}
        >
          <Close />
        </IconButton>
        {imageUrl && (
          <Box
            component="img"
            src={imageUrl}
            alt=""
            sx={{
              width: '100%',
              height: { xs: '100dvh', sm: 'auto' },
              maxHeight: { sm: 'calc(100dvh - 64px)' },
              objectFit: 'contain',
              display: 'block',
            }}
          />
        )}
      </DialogContent>
    </Dialog>
  );
}
