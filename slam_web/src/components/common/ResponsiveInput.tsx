import TextField from '@mui/material/TextField';
import type { TextFieldProps } from '@mui/material/TextField';
import React from 'react';

const ResponsiveInput = React.forwardRef<HTMLDivElement, TextFieldProps>(
  function ResponsiveInput(props, ref) {
    const { sx, fullWidth, ...rest } = props;
    const finalFullWidth = fullWidth ?? true;
    return (
      <TextField
        ref={ref}
        fullWidth={finalFullWidth}
        sx={[
          {
            '& .MuiOutlinedInput-input': {
              fontSize: { xs: '0.9rem', md: '1rem' },
            },
            '& .MuiOutlinedInput-root': {
              minHeight: { xs: 32, md: 46 },
            },
          },
          ...(Array.isArray(sx) ? sx : sx ? [sx] : []),
        ]}
        {...rest}
      />
    );
  },
);

export default ResponsiveInput;
