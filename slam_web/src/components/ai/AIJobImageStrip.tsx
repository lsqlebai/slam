import { Box, CircularProgress, Portal } from '@mui/material';
import { useEffect, useRef, useState } from 'react';
import { type AIJobAsset, getAIAssetBlob } from '../../services/aiJob';
import ImagePreviewDialog from '../common/ImagePreviewDialog';

export default function AIJobImageStrip({
  assets,
  compact = false,
}: {
  assets: AIJobAsset[];
  compact?: boolean;
}) {
  const [thumbnails, setThumbnails] = useState<Record<string, string>>({});
  const [originals, setOriginals] = useState<Record<string, string>>({});
  const [preview, setPreview] = useState<string | null>(null);
  const [hovered, setHovered] = useState<string | null>(null);
  const hoverTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const hoverTarget = useRef<string | null>(null);
  const originalUrls = useRef<string[]>([]);

  useEffect(() => {
    let active = true;
    const urls: string[] = [];
    Promise.all(
      assets.map(async asset => {
        const blob = await getAIAssetBlob(asset.id, 'thumbnail');
        const url = URL.createObjectURL(blob);
        urls.push(url);
        return [asset.id, url] as const;
      }),
    )
      .then(entries => {
        if (active) setThumbnails(Object.fromEntries(entries));
      })
      .catch(() => {});
    return () => {
      active = false;
      for (const url of urls) URL.revokeObjectURL(url);
    };
  }, [assets]);

  useEffect(
    () => () => {
      for (const url of originalUrls.current) URL.revokeObjectURL(url);
      if (hoverTimer.current) clearTimeout(hoverTimer.current);
    },
    [],
  );

  const loadOriginal = async (asset: AIJobAsset) => {
    if (originals[asset.id]) return originals[asset.id];
    const blob = await getAIAssetBlob(asset.id, 'content');
    const url = URL.createObjectURL(blob);
    originalUrls.current.push(url);
    setOriginals(previous => ({ ...previous, [asset.id]: url }));
    return url;
  };

  const ordered = [...assets].sort((a, b) => a.position - b.position);
  const hoveredUrl = hovered ? originals[hovered] : undefined;

  return (
    <>
      <Box
        onClick={event => event.stopPropagation()}
        sx={{
          position: 'relative',
          display: 'flex',
          alignItems: 'center',
          maxWidth: '100%',
          minHeight: 0,
          overflow: 'hidden',
        }}
      >
        <Box
          sx={{
            display: 'flex',
            alignItems: 'center',
            gap: { xs: 0.75, sm: 1 },
            width: '100%',
            overflowX: 'auto',
            overscrollBehaviorX: 'contain',
            WebkitOverflowScrolling: 'touch',
            py: compact ? 0 : 1,
            pb: compact ? 0.5 : 1,
          }}
        >
          {ordered.map(asset => (
            <Box
              key={asset.id}
              onMouseEnter={() => {
                hoverTarget.current = asset.id;
                hoverTimer.current = setTimeout(() => {
                  loadOriginal(asset)
                    .then(() => {
                      if (hoverTarget.current === asset.id) {
                        setHovered(asset.id);
                      }
                    })
                    .catch(() => {});
                }, 300);
              }}
              onMouseLeave={() => {
                hoverTarget.current = null;
                if (hoverTimer.current) clearTimeout(hoverTimer.current);
                setHovered(null);
              }}
              onClick={() => {
                loadOriginal(asset)
                  .then(setPreview)
                  .catch(() => {});
              }}
              sx={{ position: 'relative', flex: '0 0 auto', cursor: 'zoom-in' }}
            >
              {thumbnails[asset.id] ? (
                <Box
                  component="img"
                  src={thumbnails[asset.id]}
                  alt=""
                  sx={{
                    width: compact ? { xs: 64, sm: 96 } : { xs: 88, sm: 112 },
                    height: compact ? { xs: 64, sm: 96 } : { xs: 88, sm: 112 },
                    objectFit: 'cover',
                    borderRadius: 2,
                    border: '1px solid',
                    borderColor: 'divider',
                    display: 'block',
                  }}
                />
              ) : (
                <Box
                  sx={{
                    width: compact ? { xs: 64, sm: 96 } : { xs: 88, sm: 112 },
                    height: compact ? { xs: 64, sm: 96 } : { xs: 88, sm: 112 },
                    display: 'grid',
                    placeItems: 'center',
                    bgcolor: 'grey.100',
                    borderRadius: 2,
                  }}
                >
                  <CircularProgress size={18} />
                </Box>
              )}
            </Box>
          ))}
        </Box>
      </Box>
      {hoveredUrl && (
        <Portal>
          <Box
            sx={{
              display: { xs: 'none', md: 'grid' },
              inset: 0,
              position: 'fixed',
              zIndex: 1700,
              placeItems: 'center',
              bgcolor: 'rgba(0, 0, 0, 0.72)',
              p: 3,
              boxSizing: 'border-box',
              pointerEvents: 'none',
            }}
          >
            <Box
              component="img"
              src={hoveredUrl}
              alt=""
              sx={{
                width: '100%',
                height: '100%',
                minWidth: 0,
                minHeight: 0,
                objectFit: 'contain',
                display: 'block',
              }}
            />
          </Box>
        </Portal>
      )}
      <ImagePreviewDialog
        open={Boolean(preview)}
        imageUrl={preview}
        onClose={() => setPreview(null)}
      />
    </>
  );
}
