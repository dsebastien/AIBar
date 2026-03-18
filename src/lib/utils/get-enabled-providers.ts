import type { AppConfig, ProviderMetadata } from '@/lib/types'
import { PROVIDERS } from '@/lib/constants'

export function getEnabledProviders(config: AppConfig): ProviderMetadata[] {
    return config.providerOrder
        .filter((id) => config.enabledProviders.includes(id))
        .map((id) => PROVIDERS.find((p) => p.id === id))
        .filter((p): p is ProviderMetadata => p !== undefined)
}
