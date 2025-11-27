import { Box } from '@mui/material';
import type { BoxProps, TypographyProps } from '@mui/material';
import SportField from './SportField';

export default function SportListTitle({
  icon,
  label,
  labelVariant = 'subtitle1',
  labelWeight = 700,
  iconColor = 'text.primary',
  labelColor = 'text.primary',
  gap = 1,
  align = 'center',
  containerSx,
}: {
  icon: JSX.Element;
  label: React.ReactNode;
  labelVariant?: TypographyProps['variant'];
  labelWeight?: number;
  iconColor?: string;
  labelColor?: string;
  gap?: number;
  align?: 'center' | 'flex-start' | 'flex-end' | 'baseline';
  containerSx?: BoxProps['sx'];
}) {
  return (
    <Box
      sx={[
        {
          py: 1,
          width: '100%',
          maxWidth: 500,
          mx: { xs: 'auto', md: 0 },
        },
        ...(Array.isArray(containerSx)
          ? containerSx
          : containerSx
            ? [containerSx]
            : []),
      ]}
    >
      <SportField
        icon={icon}
        label={label}
        labelVariant={labelVariant}
        labelWeight={labelWeight}
        iconColor={iconColor}
        labelColor={labelColor}
        colon={false}
        gap={gap}
        responsive
        align={align}
      />
    </Box>
  );
}
