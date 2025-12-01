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
    <Dialog open={open} onClose={onClose} maxWidth="md" fullWidth>
      <DialogContent sx={{ p: 0, position: 'relative' }}>
        <IconButton
          aria-label="close-preview"
          onClick={onClose}
          sx={{
            position: 'absolute',
            top: 8,
            left: 8,
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
            sx={{ width: '100%', height: 'auto', display: 'block' }}
          />
        )}
      </DialogContent>
    </Dialog>
  );
}
