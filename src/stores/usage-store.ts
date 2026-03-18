import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { create } from 'zustand'
import type { ProviderId, UsageSnapshot } from '@/lib/types'

let unlistenUsageUpdated: UnlistenFn | null = null

interface UsageState {
    snapshots: Record<string, UsageSnapshot | null>
    refreshing: Record<string, boolean>
    lastRefreshAt: Record<string, number>
    initialize: () => Promise<void>
    refreshProvider: (id: ProviderId) => Promise<void>
    refreshAll: () => Promise<void>
}

export const useUsageStore = create<UsageState>((set, get) => ({
    snapshots: {},
    refreshing: {},
    lastRefreshAt: {},

    initialize: async () => {
        try {
            const allUsage = await invoke<Record<string, UsageSnapshot | null>>('get_all_usage')
            set({ snapshots: allUsage })
        } catch {
            // Backend may not be ready yet
        }

        // Clean up previous listener before registering a new one
        if (unlistenUsageUpdated) {
            unlistenUsageUpdated()
            unlistenUsageUpdated = null
        }

        unlistenUsageUpdated = await listen<Record<string, UsageSnapshot | null>>(
            'usage-updated',
            (event) => {
                set((state) => ({
                    snapshots: { ...state.snapshots, ...event.payload }
                }))
            }
        )
    },

    refreshProvider: async (id: ProviderId) => {
        const { refreshing } = get()
        if (refreshing[id]) return

        set({ refreshing: { ...get().refreshing, [id]: true } })

        try {
            const snapshot = await invoke<UsageSnapshot | null>('refresh_provider', {
                providerId: id
            })
            set({
                snapshots: { ...get().snapshots, [id]: snapshot },
                lastRefreshAt: { ...get().lastRefreshAt, [id]: Date.now() }
            })
        } catch {
            // Refresh failed silently
        } finally {
            set({ refreshing: { ...get().refreshing, [id]: false } })
        }
    },

    refreshAll: async () => {
        try {
            const allUsage = await invoke<Record<string, UsageSnapshot | null>>('get_all_usage')
            const now = Date.now()
            const lastRefreshAt: Record<string, number> = {}
            for (const key of Object.keys(allUsage)) {
                lastRefreshAt[key] = now
            }
            set({ snapshots: allUsage, lastRefreshAt })
        } catch {
            // Refresh failed silently
        }
    }
}))
