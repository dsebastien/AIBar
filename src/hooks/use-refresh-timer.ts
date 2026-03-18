import { useEffect } from 'react'
import { useSettingsStore } from '@/stores/settings-store'
import { useUsageStore } from '@/stores/usage-store'

const CADENCE_MS: Record<string, number | null> = {
    'manual': null,
    '1m': 60_000,
    '2m': 120_000,
    '5m': 300_000,
    '15m': 900_000
}

export function useRefreshTimer() {
    const refreshCadence = useSettingsStore((s) => s.config?.refreshCadence ?? 'manual')
    const refreshAll = useUsageStore((s) => s.refreshAll)

    useEffect(() => {
        const intervalMs = CADENCE_MS[refreshCadence] ?? null

        if (intervalMs === null) return

        const interval = setInterval(() => {
            void refreshAll()
        }, intervalMs)

        return () => clearInterval(interval)
    }, [refreshCadence, refreshAll])
}
