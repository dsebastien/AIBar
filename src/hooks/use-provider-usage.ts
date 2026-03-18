import type { ProviderId, UsageSnapshot } from '@/lib/types'
import { useUsageStore } from '@/stores/usage-store'

export function useProviderUsage(providerId: ProviderId): UsageSnapshot | null {
    return useUsageStore((s) => s.snapshots[providerId] ?? null)
}
