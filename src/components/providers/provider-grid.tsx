import { PROVIDERS } from '@/lib/constants'
import { useSettingsStore } from '@/stores/settings-store'
import { useUsageStore } from '@/stores/usage-store'
import { ProviderCard } from '@/components/providers/provider-card'

export function ProviderGrid() {
    const config = useSettingsStore((s) => s.config)
    const snapshots = useUsageStore((s) => s.snapshots)

    if (!config) {
        return null
    }

    const orderedProviders = config.providerOrder
        .filter((id) => config.enabledProviders.includes(id))
        .map((id) => PROVIDERS.find((p) => p.id === id))
        .filter((p) => p !== undefined)

    return (
        <div className='grid grid-cols-1 gap-3 p-3'>
            {orderedProviders.map((provider) => (
                <ProviderCard
                    key={provider.id}
                    metadata={provider}
                    snapshot={snapshots[provider.id] ?? null}
                />
            ))}
            {orderedProviders.length === 0 && (
                <div className='py-8 text-center'>
                    <p className='text-app-text-secondary text-sm'>No providers enabled</p>
                    <p className='text-app-text-secondary mt-1 text-xs'>
                        Enable providers in Settings
                    </p>
                </div>
            )}
        </div>
    )
}
