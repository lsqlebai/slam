import { Refresh } from '@mui/icons-material';
import {
  Box,
  Button,
  Card,
  CardContent,
  Chip,
  CircularProgress,
  Typography,
} from '@mui/material';
import type { Lang } from '../../i18n';
import { TEXTS } from '../../i18n';
import type { AIJob } from '../../services/aiJob';
import AIJobImageStrip from './AIJobImageStrip';

export default function AIJobCard({
  job,
  lang,
  retrying,
  onOpen,
  onRetry,
}: {
  job: AIJob;
  lang: Lang;
  retrying: boolean;
  onOpen: () => void;
  onRetry: () => void;
}) {
  const text = TEXTS[lang].aiJobs;
  const waitingRetry = job.status === 'queued' && Boolean(job.next_attempt_at);
  const statusLabel = waitingRetry
    ? text.waitingRetry.replace('{attempt}', String(job.attempts + 1))
    : text.status[job.status];
  const ready = job.status === 'ready';
  const showAttempts =
    job.status === 'failed' || waitingRetry || job.attempts > 1;

  return (
    <Card
      sx={{
        borderRadius: 3,
        transition: 'transform 0.2s ease, box-shadow 0.2s ease',
        '@media (hover: hover)': {
          '&:hover': { transform: 'translateY(-2px)', boxShadow: 5 },
        },
      }}
    >
      <CardContent
        sx={{
          display: 'grid',
          gridTemplateAreas:
            job.assets.length > 0
              ? {
                  xs: '"info" "media" "action"',
                  sm: '"media info" "media action"',
                }
              : '"info" "action"',
          gridTemplateColumns: {
            xs: 'minmax(0, 1fr)',
            sm:
              job.assets.length > 0
                ? 'minmax(220px, 38%) minmax(0, 1fr)'
                : 'minmax(0, 1fr)',
          },
          gridTemplateRows: { sm: 'minmax(0, 1fr) auto' },
          alignItems: 'stretch',
          columnGap: { xs: 0, sm: 3 },
          rowGap: { xs: 1.5, sm: 1 },
          p: { xs: 2, sm: 3 },
          '&:last-child': { pb: { xs: 2, sm: 3 } },
        }}
      >
        <Box
          sx={{
            gridArea: 'info',
            display: 'flex',
            flexDirection: 'column',
            minWidth: 0,
          }}
        >
          <Box
            sx={{
              display: 'flex',
              alignItems: 'center',
              flexWrap: 'wrap',
              gap: 1,
              mb: 1.5,
            }}
          >
            {(job.status === 'queued' || job.status === 'running') && (
              <CircularProgress size={16} />
            )}
            <Chip
              size="small"
              label={statusLabel}
              color={
                job.status === 'ready'
                  ? 'success'
                  : job.status === 'failed'
                    ? 'error'
                    : 'default'
              }
            />
            <Typography variant="caption" color="text.secondary">
              {new Date(job.created_at * 1000).toLocaleString()}
            </Typography>
          </Box>
          {job.result && (
            <Typography variant="body1" sx={{ mb: 1 }}>
              {job.result.type} · {job.result.distance_meter} m ·{' '}
              {job.result.duration_second} s
            </Typography>
          )}
          {job.error_message && (
            <Typography variant="body2" color="error" sx={{ mb: 1 }}>
              {job.error_message}
            </Typography>
          )}
          {showAttempts && (
            <Typography variant="caption" color="text.secondary" sx={{ mb: 1 }}>
              {text.attempts}: {job.attempts}
            </Typography>
          )}
        </Box>
        {job.assets.length > 0 && (
          <Box
            sx={{
              gridArea: 'media',
              minWidth: 0,
              minHeight: { xs: 112, sm: 150 },
              overflow: 'hidden',
              border: '1px solid',
              borderColor: 'divider',
              borderRadius: 2,
              bgcolor: 'grey.50',
              p: 1,
              display: 'flex',
              alignItems: 'center',
            }}
          >
            <AIJobImageStrip assets={job.assets} compact />
          </Box>
        )}
        <Box
          sx={{
            gridArea: 'action',
            display: 'flex',
            alignItems: 'flex-end',
            justifyContent: { xs: 'stretch', sm: 'flex-end' },
          }}
        >
          {job.status === 'failed' && (
            <Button
              variant="outlined"
              startIcon={
                retrying ? <CircularProgress size={16} /> : <Refresh />
              }
              disabled={retrying}
              sx={{ width: { xs: '100%', sm: 176 } }}
              onClick={onRetry}
            >
              {text.retry}
            </Button>
          )}
          {ready && (
            <Button
              variant="contained"
              sx={{ width: { xs: '100%', sm: 176 } }}
              onClick={onOpen}
            >
              {text.openReady}
            </Button>
          )}
        </Box>
      </CardContent>
    </Card>
  );
}
