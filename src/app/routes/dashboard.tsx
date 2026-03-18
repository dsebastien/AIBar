import { useEffect } from 'react'
import { TrayPopupLayout } from '@/components/layout/tray-popup-layout'
import { ProviderGrid } from '@/components/providers/provider-grid'
import { useUsageStore } from '@/stores/usage-store'
import { useSettingsStore } from '@/stores/settings-store'
import { useRefreshTimer } from '@/hooks/use-refresh-timer'

export default function Dashboard() {
    const refreshAll = useUsageStore((s) => s.refreshAll)
    const initialize = useUsageStore((s) => s.initialize)
    const initSettings = useSettingsStore((s) => s.initialize)

    useRefreshTimer()

    useEffect(() => {
        void initialize()
        void initSettings()
    }, [initialize, initSettings])

    return (
        <TrayPopupLayout onRefresh={() => void refreshAll()}>
            <ProviderGrid />
        </TrayPopupLayout>
    )
}
