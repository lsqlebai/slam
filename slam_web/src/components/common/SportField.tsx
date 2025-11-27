import { Stack, Typography } from '@mui/material';
import type { TypographyProps } from '@mui/material';
import { cloneElement } from 'react';

export default function SportField({
  icon,
  label,
  value,
  labelVariant = 'caption',
  valueVariant = 'subtitle1',
  labelColor = 'text.secondary',
  valueColor,
  iconColor,
  labelWeight,
  valueWeight,
  iconSize,
  labelSize,
  valueSize,
  gap = 0.5,
  noWrap = false,
  responsive = true,
  colon = true,
  align = 'center',
}: {
  icon: JSX.Element;
  label?: React.ReactNode;
  value?: React.ReactNode;
  labelVariant?: TypographyProps['variant'];
  valueVariant?: TypographyProps['variant'];
  labelColor?: TypographyProps['color'];
  valueColor?: TypographyProps['color'];
  iconColor?: string;
  labelWeight?: number;
  valueWeight?: number;
  iconSize?: Partial<{
    xs: number;
    sm: number;
    md: number;
    lg: number;
    xl: number;
  }>;
  labelSize?: Partial<{
    xs: number;
    sm: number;
    md: number;
    lg: number;
    xl: number;
  }>;
  valueSize?: Partial<{
    xs: number;
    sm: number;
    md: number;
    lg: number;
    xl: number;
  }>;
  gap?: number;
  noWrap?: boolean;
  responsive?: boolean;
  colon?: boolean;
  align?: 'center' | 'flex-start' | 'flex-end' | 'baseline';
}) {
  const iconSizeDefault = responsive
    ? { xs: 16, sm: 18, md: 20, lg: 22, xl: 24 }
    : undefined;
  const fontSizeBodyDefault = responsive
    ? { xs: 13, sm: 14, md: 15, lg: 16, xl: 18 }
    : undefined;
  const fontSizeLabelDefault = responsive
    ? { xs: 15, sm: 16, md: 18, lg: 20, xl: 22 }
    : undefined;

  const iconEl = cloneElement(icon, {
    sx: { fontSize: iconSize ?? iconSizeDefault, color: iconColor },
  });
  const computedLabelSizeBase =
    labelVariant === 'subtitle1' ? fontSizeLabelDefault : fontSizeBodyDefault;
  const computedValueSizeBase =
    valueVariant === 'subtitle1' ? fontSizeLabelDefault : fontSizeBodyDefault;
  const labelFontSize = labelSize ?? computedLabelSizeBase;
  const valueFontSize = valueSize ?? computedValueSizeBase;
  const computedValueWeight =
    valueWeight ?? (valueVariant === 'subtitle1' ? 700 : undefined);
  const labelLineHeight = labelVariant === 'body2' ? 1.1 : undefined;
  const valueLineHeight = valueVariant === 'body2' ? 1.1 : undefined;

  return (
    <Stack
      direction="row"
      alignItems={align}
      spacing={gap}
      sx={{ flexWrap: noWrap ? 'nowrap' : 'wrap', minWidth: 0 }}
    >
      {iconEl}
      {label !== undefined && (
        <Typography
          variant={labelVariant}
          color={labelColor}
          noWrap={noWrap}
          component="span"
          sx={{
            fontSize: labelFontSize,
            fontWeight: labelWeight,
            lineHeight: labelLineHeight,
          }}
        >
          {colon ? `${label}:` : label}
        </Typography>
      )}
      {value !== undefined && (
        <Typography
          variant={valueVariant}
          color={valueColor}
          noWrap={noWrap}
          component="span"
          sx={{
            fontWeight: computedValueWeight,
            fontSize: valueFontSize,
            lineHeight: valueLineHeight,
          }}
        >
          {value}
        </Typography>
      )}
    </Stack>
  );
}
