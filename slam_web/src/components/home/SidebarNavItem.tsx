import { ListItemButton, ListItemIcon, ListItemText } from '@mui/material';
import type { MouseEventHandler, ReactNode } from 'react';

export default function SidebarNavItem({
  selected,
  onClick,
  icon,
  label,
}: {
  selected: boolean;
  onClick: MouseEventHandler<HTMLDivElement> | (() => void);
  icon: ReactNode;
  label: string;
}) {
  return (
    <ListItemButton
      selected={selected}
      onClick={onClick as MouseEventHandler<HTMLDivElement>}
      sx={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        py: 1.5,
        color: 'text.secondary',
        '&.Mui-selected': { color: 'primary.main' },
      }}
    >
      <ListItemIcon sx={{ minWidth: 0, color: 'inherit' }}>{icon}</ListItemIcon>
      <ListItemText
        primary={label}
        primaryTypographyProps={{
          align: 'center',
          variant: 'caption',
          sx: { color: 'inherit' },
        }}
      />
    </ListItemButton>
  );
}
