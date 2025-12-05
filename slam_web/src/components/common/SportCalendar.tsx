import {
  ChevronLeft,
  ChevronRight,
  LocalFireDepartment,
} from '@mui/icons-material';
import { Box, IconButton, Stack, Typography } from '@mui/material';
import type { Lang } from '../../i18n';

export default function SportCalendar({
  year,
  month,
  lang,
  activeDays,
  disablePrev,
  disableNext,
  onPrev,
  onNext,
}: {
  year: number;
  month: number;
  lang: Lang;
  activeDays: Set<number>;
  disablePrev: boolean;
  disableNext: boolean;
  onPrev: () => void;
  onNext: () => void;
}) {
  const current = new Date(year, month, 1);
  const locale = lang === 'zh' ? 'zh-CN' : 'en-US';
  const monthTitle = new Intl.DateTimeFormat(locale, {
    year: 'numeric',
    month: 'long',
  }).format(current);
  const firstDay = new Date(year, month, 1);
  const startOffset = (firstDay.getDay() + 6) % 7;
  const daysInMonth = new Date(year, month + 1, 0).getDate();
  const mondayThisWeek = new Date(current);
  mondayThisWeek.setDate(current.getDate() - ((current.getDay() + 6) % 7));
  const weekdayLabels = Array.from({ length: 7 }, (_, i) =>
    new Intl.DateTimeFormat(locale, { weekday: 'short' }).format(
      new Date(
        mondayThisWeek.getFullYear(),
        mondayThisWeek.getMonth(),
        mondayThisWeek.getDate() + i,
      ),
    ),
  );
  const blankKeys = Array.from({ length: startOffset }, (_, i) =>
    new Date(year, month, 1 - (startOffset - i)).toISOString(),
  );

  const handlePrev = () => onPrev();
  const handleNext = () => onNext();

  return (
    <Box
      sx={{
        p: 1.5,
        borderRadius: 2,
        bgcolor: '#fff',
        boxShadow: '0 2px 10px rgba(0,0,0,0.08)',
        maxWidth: 500,
        width: { xs: '100%', sm: 500, md: 500 },
        mx: { xs: 'auto', sm: 'auto', md: 'auto', lg: 0 },
        alignSelf: { xs: 'center', sm: 'center', md: 'stretch', lg: 'stretch' },
      }}
    >
      <Box
        sx={{
          mb: 1,
          display: 'grid',
          gridTemplateColumns: 'repeat(7, 1fr)',
          gap: 1,
          alignItems: 'center',
        }}
      >
        <IconButton
          size="small"
          onClick={handlePrev}
          disabled={disablePrev}
          sx={{ gridColumn: '1', justifySelf: 'center' }}
        >
          <ChevronLeft />
        </IconButton>
        <Stack
          direction="row"
          justifyContent="center"
          alignItems="center"
          spacing={0.75}
          sx={{ gridColumn: '2 / span 5' }}
        >
          <LocalFireDepartment
            sx={{
              color: 'primary.main',
              fontSize: { xs: 18, sm: 20, md: 22, lg: 24, xl: 26 },
            }}
          />
          <Typography variant="subtitle1" sx={{ fontWeight: 600 }}>
            {monthTitle}
          </Typography>
          <LocalFireDepartment
            sx={{
              color: 'primary.main',
              fontSize: { xs: 18, sm: 20, md: 22, lg: 24, xl: 26 },
            }}
          />
        </Stack>
        <IconButton
          size="small"
          onClick={handleNext}
          disabled={disableNext}
          sx={{ gridColumn: '7', justifySelf: 'center' }}
        >
          <ChevronRight />
        </IconButton>
      </Box>
      <Box
        sx={{
          display: 'grid',
          gridTemplateColumns: 'repeat(7, 1fr)',
          gap: 1,
        }}
      >
        {weekdayLabels.map(w => (
          <Typography
            key={w}
            variant="caption"
            color="text.secondary"
            sx={{ textAlign: 'center', fontWeight: 600 }}
          >
            {w}
          </Typography>
        ))}
      </Box>
      <Box
        sx={{
          display: 'grid',
          gridTemplateColumns: 'repeat(7, 1fr)',
          gap: 0.75,
          mt: 1,
        }}
      >
        {blankKeys.map(k => (
          <Box key={k} />
        ))}
        {Array.from({ length: daysInMonth }, (_, i) => {
          const day = i + 1;
          const isActive = activeDays.has(day);
          return (
            <Box
              key={`day-${day}`}
              sx={{
                height: 32,
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                borderRadius: 1,
                bgcolor: isActive ? 'primary.light' : 'transparent',
                color: 'text.primary',
                fontWeight: isActive ? 600 : 500,
              }}
            >
              <Typography variant="body2">{day}</Typography>
            </Box>
          );
        })}
      </Box>
    </Box>
  );
}
