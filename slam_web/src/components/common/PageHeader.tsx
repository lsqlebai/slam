import { Helmet } from '@modern-js/runtime/head';
import { useNavigate } from '@modern-js/runtime/router';
import { ArrowBack } from '@mui/icons-material';
import { Box, Divider, IconButton, Typography } from '@mui/material';

export default function PageHeader({
  headTitle,
  title,
}: {
  headTitle: string;
  title: string;
}) {
  const navigate = useNavigate();
  return (
    <>
      <Helmet>
        <title>{headTitle}</title>
      </Helmet>
      <Box
        sx={{
          position: 'sticky',
          top: 0,
          zIndex: 1300,
          bgcolor: '#fff',
          pt: 'calc(env(safe-area-inset-top) + 12px)',
          pl: 1,
          pr: 2,
          py: 1,
          minHeight: 56,
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
        }}
      >
        <IconButton
          aria-label="back"
          onClick={() => navigate(-1)}
          size="small"
          sx={{ color: 'text.secondary', '&:hover': { color: 'text.primary' } }}
        >
          <ArrowBack fontSize="medium" />
        </IconButton>
        <Typography variant="h6" sx={{ textAlign: 'right', fontWeight: 600 }}>
          {title}
        </Typography>
      </Box>
      <Divider sx={{ mb: 1, boxShadow: '0 2px 8px rgba(0,0,0,0.08)' }} />
    </>
  );
}
