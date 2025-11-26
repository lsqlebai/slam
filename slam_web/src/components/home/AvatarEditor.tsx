import { ZoomIn, ZoomOut } from '@mui/icons-material';
import {
  Box,
  Button,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  IconButton,
  Slider,
} from '@mui/material';
import { useRef, useState } from 'react';

export default function AvatarEditor({
  open,
  imageUrl,
  cropSize = 128,
  editorSize = 300,
  onCancel,
  onConfirm,
}: {
  open: boolean;
  imageUrl: string | null;
  cropSize?: number;
  editorSize?: number;
  onCancel: () => void;
  onConfirm: (base64: string) => void | Promise<void>;
}) {
  const [scale, setScale] = useState(1);
  const [offset, setOffset] = useState({ x: 0, y: 0 });
  const [dragging, setDragging] = useState(false);
  const dragStartRef = useRef<{ x: number; y: number } | null>(null);
  const imgRef = useRef<HTMLImageElement | null>(null);

  return (
    <Dialog open={open} onClose={onCancel} fullWidth maxWidth="xs">
      <DialogTitle sx={{ textAlign: 'center' }}>编辑头像</DialogTitle>
      <DialogContent>
        <Box
          sx={{
            position: 'relative',
            width: editorSize,
            height: editorSize,
            bgcolor: 'grey.300',
            overflow: 'hidden',
            mx: 'auto',
            touchAction: 'none',
          }}
          onMouseDown={e => {
            setDragging(true);
            dragStartRef.current = {
              x: e.clientX - offset.x,
              y: e.clientY - offset.y,
            };
          }}
          onMouseMove={e => {
            if (!dragging || !dragStartRef.current) return;
            const nx = e.clientX - dragStartRef.current.x;
            const ny = e.clientY - dragStartRef.current.y;
            setOffset({ x: nx, y: ny });
          }}
          onMouseUp={() => {
            setDragging(false);
            dragStartRef.current = null;
          }}
          onMouseLeave={() => {
            setDragging(false);
            dragStartRef.current = null;
          }}
          onTouchStart={e => {
            if (e.touches.length === 1) {
              const t = e.touches[0];
              setDragging(true);
              dragStartRef.current = {
                x: t.clientX - offset.x,
                y: t.clientY - offset.y,
              };
            }
          }}
          onTouchMove={e => {
            if (e.touches.length === 1 && dragging && dragStartRef.current) {
              const t = e.touches[0];
              const nx = t.clientX - dragStartRef.current.x;
              const ny = t.clientY - dragStartRef.current.y;
              setOffset({ x: nx, y: ny });
            }
          }}
          onTouchEnd={e => {
            if (e.touches.length === 0) {
              setDragging(false);
              dragStartRef.current = null;
            }
          }}
        >
          {imageUrl && (
            <Box
              component="img"
              ref={imgRef}
              src={imageUrl}
              sx={{
                position: 'absolute',
                transformOrigin: 'top left',
                left: offset.x,
                top: offset.y,
                transform: `scale(${scale})`,
                userSelect: 'none',
              }}
              draggable={false}
            />
          )}
          <Box
            sx={{
              position: 'absolute',
              left: '50%',
              top: '50%',
              width: cropSize,
              height: cropSize,
              borderRadius: '50%',
              border: '2px dashed',
              borderColor: 'divider',
              transform: 'translate(-50%, -50%)',
              boxShadow: '0 0 0 9999px rgba(0,0,0,0.35)',
              backgroundColor: 'transparent',
              pointerEvents: 'none',
            }}
          />
        </Box>
      </DialogContent>
      <DialogActions sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
        <Box
          sx={{ display: 'flex', alignItems: 'center', gap: 1, flex: 1, px: 1 }}
        >
          <IconButton
            onClick={() => setScale(s => Math.max(0.5, +(s - 0.1).toFixed(2)))}
          >
            <ZoomOut />
          </IconButton>
          <Slider
            min={0.5}
            max={3}
            step={0.01}
            value={scale}
            onChange={(_, v) => setScale(v as number)}
            sx={{ flex: 1 }}
          />
          <IconButton
            onClick={() => setScale(s => Math.min(3, +(s + 0.1).toFixed(2)))}
          >
            <ZoomIn />
          </IconButton>
        </Box>
        <Button onClick={onCancel}>取消</Button>
        <Button
          variant="contained"
          onClick={async () => {
            if (!imgRef.current) return;
            const canvas = document.createElement('canvas');
            canvas.width = cropSize;
            canvas.height = cropSize;
            const ctx = canvas.getContext('2d');
            if (!ctx) return;
            const cx = editorSize / 2;
            const cy = editorSize / 2;
            const px0 = cx - cropSize / 2;
            const py0 = cy - cropSize / 2;
            const sx = (px0 - offset.x) / scale;
            const sy = (py0 - offset.y) / scale;
            const sw = cropSize / scale;
            const sh = cropSize / scale;
            ctx.drawImage(
              imgRef.current,
              sx,
              sy,
              sw,
              sh,
              0,
              0,
              cropSize,
              cropSize,
            );
            const b64 = canvas.toDataURL('image/jpeg', 0.9);
            await onConfirm(b64);
          }}
        >
          确定
        </Button>
      </DialogActions>
    </Dialog>
  );
}
