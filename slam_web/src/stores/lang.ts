import { create } from 'zustand';
import { createJSONStorage, persist } from 'zustand/middleware';
import type { Lang } from '../i18n';

type LangState = {
  lang: Lang;
  setLang: (l: Lang) => void;
};

export const useLangStore = create(
  persist<LangState>(
    set => ({
      lang: 'zh',
      setLang: l => set({ lang: l }),
    }),
    {
      name: 'lang',
      storage: createJSONStorage(() => localStorage),
      version: 1,
    },
  ),
);
