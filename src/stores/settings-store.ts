import { invoke } from '@tauri-apps/api/core'
import { create } from 'zustand'
import type { AppConfig, ProviderId, RefreshCadence } from '@/lib/types'

interface SettingsState {
    config: AppConfig | null
    isLoading: boolean
    initialize: () => Promise<void>
    toggleProvider: (id: ProviderId, enabled: boolean) => Promise<void>
    setRefreshCadence: (cadence: RefreshCadence) => Promise<void>
    reorderProviders: (order: ProviderId[]) => Promise<void>
}

async function updateSettings(config: AppConfig): Promise<void> {
    await invoke('update_settings', { settings: config })
}

export const useSettingsStore = create<SettingsState>((set, get) => ({
    config: null,
    isLoading: true,

    initialize: async () => {
        set({ isLoading: true })
        try {
            const config = await invoke<AppConfig>('get_settings')
            set({ config, isLoading: false })
        } catch {
            set({ isLoading: false })
        }
    },

    toggleProvider: async (id: ProviderId, enabled: boolean) => {
        const { config } = get()
        if (!config) return

        const enabledProviders = enabled
            ? [...config.enabledProviders, id]
            : config.enabledProviders.filter((p) => p !== id)

        const updatedConfig: AppConfig = { ...config, enabledProviders }
        set({ config: updatedConfig })

        try {
            await updateSettings(updatedConfig)
        } catch {
            set({ config })
        }
    },

    setRefreshCadence: async (cadence: RefreshCadence) => {
        const { config } = get()
        if (!config) return

        const updatedConfig: AppConfig = { ...config, refreshCadence: cadence }
        set({ config: updatedConfig })

        try {
            await updateSettings(updatedConfig)
        } catch {
            set({ config })
        }
    },

    reorderProviders: async (order: ProviderId[]) => {
        const { config } = get()
        if (!config) return

        const updatedConfig: AppConfig = { ...config, providerOrder: order }
        set({ config: updatedConfig })

        try {
            await updateSettings(updatedConfig)
        } catch {
            set({ config })
        }
    }
}))
