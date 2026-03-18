import { useUsageStore } from '@/stores/usage-store'

export function useRefreshTimer() {
    const refreshAll = useUsageStore((s) => s.refreshAll)

    return { refreshAll }
}
