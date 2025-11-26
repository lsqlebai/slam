import Button from '@mui/material/Button';
import type { ButtonProps } from '@mui/material/Button';
import React from 'react';

const RB = React.forwardRef<HTMLButtonElement, ButtonProps>(
  function RB(props, ref) {
    const { sx, ...rest } = props;
    return (
      <Button
        ref={ref}
        {...rest}
        sx={[
          {
            py: { xs: 1, md: 1.25 },
            minHeight: { xs: 32, md: 38 },
            fontSize: { xs: '0.9rem', md: '1rem' },
          },
          ...(Array.isArray(sx) ? sx : sx ? [sx] : []),
        ]}
      />
    );
  },
);

export default RB as typeof Button;
