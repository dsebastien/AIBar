import { PROVIDERS } from '@/lib/constants'
import { useSettingsStore } from '@/stores/settings-store'
import { Switch } from '@/components/ui/switch'

export function ProviderToggleList() {
    const config = useSettingsStore((s) => s.config)
    const toggleProvider = useSettingsStore((s) => s.toggleProvider)

    if (!config) return null

    return (
        <div className='space-y-2'>
            <h2 className='mb-3 text-sm font-medium'>Providers</h2>
            {PROVIDERS.map((provider) => {
                const enabled = config.enabledProviders.includes(provider.id)
                return (
                    <div
                        key={provider.id}
                        className='border-app-border flex items-center justify-between rounded-md border px-3 py-2'
                    >
                        <div className='flex items-center gap-2'>
                            <div
                                className='text-app-background flex h-5 w-5 items-center justify-center rounded-full text-[10px] font-bold'
                                style={{ backgroundColor: provider.color }}
                            >
                                {provider.displayName.charAt(0)}
                            </div>
                            <span className='text-sm'>{provider.displayName}</span>
                        </div>
                        <Switch
                            checked={enabled}
                            onCheckedChange={(checked) => void toggleProvider(provider.id, checked)}
                        />
                    </div>
                )
            })}
        </div>
    )
}
