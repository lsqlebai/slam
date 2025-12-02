import { useEffect } from 'react'
import { Capacitor } from '@capacitor/core'
import { App } from '@capacitor/app'

export default function useUnifiedBack(onBack: () => void) {
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onBack()
    }
    window.addEventListener('keydown', onKey)
    let remove: (() => void) | null = null
    if (Capacitor.isNativePlatform() && Capacitor.getPlatform() === 'android') {
      App.addListener('backButton', () => {
        onBack()
      }).then(h => {
        remove = h.remove
      })
    }
    return () => {
      window.removeEventListener('keydown', onKey)
      if (remove) remove()
    }
  }, [onBack])
}
