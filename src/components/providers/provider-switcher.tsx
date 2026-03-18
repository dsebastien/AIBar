import { useState } from 'react'
import { PROVIDERS } from '@/lib/constants'
import type { ProviderId } from '@/lib/types'
import { useSettingsStore } from '@/stores/settings-store'
import { useUsageStore } from '@/stores/usage-store'
import { cn } from '@/lib/utils/cn'
import { ProviderCard } from '@/components/providers/provider-card'

export function ProviderSwitcher() {
    const config = useSettingsStore((s) => s.config)
    const snapshots = useUsageStore((s) => s.snapshots)
    const [activeProvider, setActiveProvider] = useState<ProviderId | null>(null)

    if (!config) return null

    const enabledProviders = config.providerOrder
        .filter((id) => config.enabledProviders.includes(id))
        .map((id) => PROVIDERS.find((p) => p.id === id))
        .filter((p) => p !== undefined)

    const currentId = activeProvider ?? enabledProviders[0]?.id ?? null
    const currentProvider = enabledProviders.find((p) => p.id === currentId)

    return (
        <div className='flex flex-col'>
            <div className='border-app-border flex items-center gap-1 overflow-x-auto border-b px-3 py-2'>
                {enabledProviders.map((provider) => (
                    <button
                        key={provider.id}
                        type='button'
                        className={cn(
                            'flex h-7 w-7 shrink-0 items-center justify-center rounded-full text-xs font-bold transition-all',
                            provider.id === currentId
                                ? 'ring-app-accent ring-offset-app-background ring-2 ring-offset-1'
                                : 'opacity-60 hover:opacity-100'
                        )}
                        style={{ backgroundColor: provider.color, color: '#1a1a2e' }}
                        onClick={() => setActiveProvider(provider.id)}
                        title={provider.displayName}
                    >
                        {provider.displayName.charAt(0)}
                    </button>
                ))}
            </div>
            <div className='p-3'>
                {currentProvider && (
                    <ProviderCard
                        metadata={currentProvider}
                        snapshot={snapshots[currentProvider.id] ?? null}
                    />
                )}
            </div>
        </div>
    )
}
