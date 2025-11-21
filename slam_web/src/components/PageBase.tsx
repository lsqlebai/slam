import { Alert, Snackbar } from '@mui/material';
import { alpha } from '@mui/material/styles';
import { createContext, useContext, useState } from 'react';

type Severity = 'success' | 'error' | 'info' | 'warning';

type ToastContextValue = {
  show: (msg: string, severity?: Severity) => void;
  showSuccess: (msg: string) => void;
  showError: (msg: string) => void;
};

const ToastContext = createContext<ToastContextValue | null>(null);

export function useToast(): ToastContextValue {
  const ctx = useContext(ToastContext);
  if (!ctx) {
    throw new Error('useToast must be used within PageBase');
  }
  return ctx;
}

export default function PageBase({ children }: { children: React.ReactNode }) {
  const [open, setOpen] = useState(false);
  const [message, setMessage] = useState('');
  const [severity, setSeverity] = useState<Severity>('info');

  const show = (msg: string, s: Severity = 'info') => {
    setMessage(msg);
    setSeverity(s);
    setOpen(true);
  };

  const handleClose = (
    _event?: React.SyntheticEvent | Event,
    reason?: string,
  ) => {
    if (reason === 'clickaway') return;
    setOpen(false);
  };

  const value: ToastContextValue = {
    show,
    showSuccess: (m: string) => show(m, 'success'),
    showError: (m: string) => show(m, 'error'),
  };

  return (
    <ToastContext.Provider value={value}>
      {children}
      <Snackbar
        key={`${severity}-${message}`}
        open={open}
        autoHideDuration={3000}
        onClose={handleClose}
        anchorOrigin={{ vertical: 'top', horizontal: 'center' }}
      >
        <Alert
          severity={severity}
          onClose={handleClose}
          variant="filled"
          sx={(theme) => ({
            width: '100%',
            borderRadius: 2,
            boxShadow: '0 10px 24px rgba(0,0,0,0.2), 0 2px 8px rgba(0,0,0,0.1)',
            bgcolor: alpha(theme.palette[severity].main, 0.92),
            backgroundImage: `linear-gradient(180deg, ${alpha(theme.palette[severity].light, 0.25)}, transparent)`,
            backdropFilter: 'saturate(160%) blur(6px)',
          })}
        >
          {message}
        </Alert>
      </Snackbar>
    </ToastContext.Provider>
  );
}
