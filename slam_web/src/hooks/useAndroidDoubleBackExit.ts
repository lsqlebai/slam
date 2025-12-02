import { useEffect, useRef } from 'react'
import { Capacitor } from '@capacitor/core'
import { App } from '@capacitor/app'
import { useToast } from '../components/PageBase'

export default function useAndroidDoubleBackExit(message: string, intervalMs = 1500) {
  const { show } = useToast()
  const lastRef = useRef<number | null>(null)

  useEffect(() => {
    let remove: (() => void) | null = null
    if (Capacitor.isNativePlatform() && Capacitor.getPlatform() === 'android') {
      App.addListener('backButton', () => {
        const now = Date.now()
        if (lastRef.current && now - lastRef.current < intervalMs) {
          App.exitApp()
        } else {
          lastRef.current = now
          show(message)
        }
      }).then(h => {
        remove = h.remove
      })
    }
    return () => {
      if (remove) remove()
    }
  }, [message, intervalMs, show])
}
