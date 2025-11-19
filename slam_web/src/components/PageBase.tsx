import { Alert, Snackbar } from '@mui/material';
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

  const value: ToastContextValue = {
    show,
    showSuccess: (m: string) => show(m, 'success'),
    showError: (m: string) => show(m, 'error'),
  };

  return (
    <ToastContext.Provider value={value}>
      {children}
      <Snackbar
        open={open}
        autoHideDuration={3000}
        onClose={() => setOpen(false)}
        anchorOrigin={{ vertical: 'top', horizontal: 'center' }}
      >
        <Alert
          severity={severity}
          onClose={() => setOpen(false)}
          sx={{ width: '100%' }}
        >
          {message}
        </Alert>
      </Snackbar>
    </ToastContext.Provider>
  );
}
