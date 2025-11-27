import { Avatar, Box, Divider, Typography } from '@mui/material';
import type { UserInfo } from '../../services/user';

export default function HomeHeader({
  title,
  greeting,
  sep,
  user,
  variant = 'desktop',
}: {
  title: string;
  greeting: string;
  sep: string;
  user: UserInfo | null;
  variant?: 'mobile' | 'desktop';
}) {
  const isMobile = variant === 'mobile';
  return (
    <>
      <Box
        sx={{
          pt: isMobile ? 'calc(env(safe-area-inset-top) + 12px)' : 2,
          px: isMobile ? 2 : { sm: 3, md: 4, lg: 5 },
          pb: 1,
          bgcolor: '#fff',
          display: isMobile
            ? { xs: 'flex', sm: 'none' }
            : { xs: 'none', sm: 'flex' },
          justifyContent: 'space-between',
          alignItems: 'center',
        }}
      >
        <Typography variant="h6" noWrap sx={{ minWidth: 0 }}>
          {title}
        </Typography>
        {user?.nickname && (
          <Box
            sx={{ display: 'flex', alignItems: 'center', gap: 1, minWidth: 0 }}
          >
            <Typography
              variant="subtitle1"
              color="text.primary"
              noWrap
              sx={{ textAlign: 'right', minWidth: 0 }}
            >
              {greeting}
              {sep}
              {user?.nickname}
            </Typography>
            <Avatar
              src={user?.avatar || undefined}
              sx={{ width: 32, height: 32 }}
            />
          </Box>
        )}
      </Box>
      <Divider
        sx={{
          mb: 1,
          display: isMobile
            ? { xs: 'block', sm: 'none' }
            : { xs: 'none', sm: 'block' },
        }}
      />
    </>
  );
}
