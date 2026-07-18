import { useNavigate } from '@modern-js/runtime/router';
import {
  Box,
  Button,
  CircularProgress,
  Container,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  Stack,
  Typography,
} from '@mui/material';
import { useCallback, useEffect, useState } from 'react';
import { TEXTS } from '../../i18n';
import type { Lang } from '../../i18n';
import {
  type AIJob,
  deleteAIJob,
  listAIJobs,
  retryAIJob,
} from '../../services/aiJob';
import { useToast } from '../PageBase';
import AIJobCard from './AIJobCard';

export default function AIJobList({
  lang,
  embedded = false,
  onJobsChange,
}: {
  lang: Lang;
  embedded?: boolean;
  onJobsChange?: (jobs: AIJob[]) => void;
}) {
  const navigate = useNavigate();
  const { showError, showSuccess } = useToast();
  const [jobs, setJobs] = useState<AIJob[]>([]);
  const [loading, setLoading] = useState(true);
  const [retrying, setRetrying] = useState<string | null>(null);
  const [deleting, setDeleting] = useState<string | null>(null);
  const [pendingDelete, setPendingDelete] = useState<AIJob | null>(null);
  const text = TEXTS[lang].aiJobs;
  const hasActiveJobs = jobs.some(
    job => job.status === 'queued' || job.status === 'running',
  );

  const refresh = useCallback(
    async (silent = false) => {
      if (!silent) setLoading(true);
      try {
        const nextJobs = await listAIJobs();
        setJobs(nextJobs);
        onJobsChange?.(nextJobs);
      } finally {
        if (!silent) setLoading(false);
      }
    },
    [onJobsChange],
  );

  useEffect(() => {
    refresh().catch(() => {});
  }, [refresh]);

  useEffect(() => {
    if (!hasActiveJobs) return;
    let cancelled = false;
    let timer: number | undefined;

    const schedule = () => {
      timer = window.setTimeout(async () => {
        if (document.visibilityState === 'visible') {
          await refresh(true).catch(() => {});
        }
        if (!cancelled) schedule();
      }, 3000);
    };
    const onVisible = () => {
      if (document.visibilityState === 'visible') refresh(true).catch(() => {});
    };
    schedule();
    document.addEventListener('visibilitychange', onVisible);
    return () => {
      cancelled = true;
      if (timer !== undefined) window.clearTimeout(timer);
      document.removeEventListener('visibilitychange', onVisible);
    };
  }, [hasActiveJobs, refresh]);

  return (
    <Container
      maxWidth="md"
      sx={{
        px: embedded ? 0 : { xs: 1.5, sm: 3 },
        py: { xs: 1.5, sm: 2 },
        pb: embedded ? 2 : 'calc(env(safe-area-inset-bottom) + 48px)',
      }}
    >
      {loading ? (
        <Box sx={{ display: 'grid', placeItems: 'center', py: 8 }}>
          <CircularProgress />
        </Box>
      ) : jobs.length === 0 ? (
        <Typography color="text.secondary" align="center" sx={{ py: 8 }}>
          {text.empty}
        </Typography>
      ) : (
        <Stack spacing={2}>
          {jobs.map(job => (
            <AIJobCard
              key={job.id}
              job={job}
              lang={lang}
              retrying={retrying === job.id}
              deleting={deleting === job.id}
              onOpen={() =>
                navigate(
                  `/sport/detail?ai_job_id=${encodeURIComponent(job.id)}`,
                )
              }
              onRetry={async () => {
                setRetrying(job.id);
                try {
                  await retryAIJob(job.id);
                  await refresh(true);
                } catch (error) {
                  showError(
                    error instanceof Error ? error.message : text.retryFailed,
                  );
                } finally {
                  setRetrying(null);
                }
              }}
              onDelete={() => setPendingDelete(job)}
            />
          ))}
        </Stack>
      )}
      <Dialog
        open={pendingDelete !== null}
        onClose={() => deleting === null && setPendingDelete(null)}
        maxWidth="xs"
        fullWidth
      >
        <DialogTitle>{text.deleteConfirmTitle}</DialogTitle>
        <DialogContent>
          <Typography>
            {pendingDelete?.status === 'ready'
              ? text.deleteDraftConfirm
              : text.deleteJobConfirm}
          </Typography>
        </DialogContent>
        <DialogActions>
          <Button
            variant="outlined"
            disabled={deleting !== null}
            onClick={() => setPendingDelete(null)}
          >
            {TEXTS[lang].register.cancel}
          </Button>
          <Button
            variant="contained"
            color="error"
            disabled={deleting !== null}
            onClick={async () => {
              if (!pendingDelete) return;
              const job = pendingDelete;
              setDeleting(job.id);
              try {
                await deleteAIJob(job.id);
                await refresh(true);
                setPendingDelete(null);
                showSuccess(text.deleteSuccess);
              } catch (error) {
                showError(
                  error instanceof Error ? error.message : text.deleteFailed,
                );
              } finally {
                setDeleting(null);
              }
            }}
          >
            {deleting !== null ? (
              <CircularProgress size={18} color="inherit" />
            ) : pendingDelete?.status === 'ready' ? (
              text.deleteDraft
            ) : (
              text.deleteJob
            )}
          </Button>
        </DialogActions>
      </Dialog>
    </Container>
  );
}
