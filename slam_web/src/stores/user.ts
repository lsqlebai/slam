import { create } from 'zustand';
import { info, type UserInfo } from '../services/user';

type UserState = {
  user: UserInfo | null;
  setUser: (u: UserInfo) => void;
  refresh: () => Promise<boolean>;
  updateAvatarLocal: (avatar: string) => void;
};

export const useUserStore = create<UserState>((set, get) => ({
  user: null,
  setUser: u => set({ user: u }),
  refresh: async () => {
    try {
      const u = await info();
      set({ user: u });
      return true;
    } catch {
      return false;
    }
  },
  updateAvatarLocal: avatar => {
    const u = get().user;
    if (u) set({ user: { ...u, avatar } });
  },
}));
