import { invoke } from '@tauri-apps/api/core'
import { create } from 'zustand'
import type { ProviderId } from '@/lib/types'

export type CredentialStatus = 'stored' | 'missing' | 'checking'

interface CredentialsState {
    credentials: Record<string, CredentialStatus>
    storeApiToken: (id: ProviderId, token: string) => Promise<void>
    deleteCredential: (id: ProviderId) => Promise<void>
    checkCredentials: () => Promise<void>
}

export const useCredentialsStore = create<CredentialsState>((set, get) => ({
    credentials: {},

    storeApiToken: async (id: ProviderId, token: string) => {
        try {
            await invoke('store_api_token', { providerId: id, token })
            set({
                credentials: { ...get().credentials, [id]: 'stored' as CredentialStatus }
            })
        } catch {
            // Store failed
        }
    },

    deleteCredential: async (id: ProviderId) => {
        try {
            await invoke('delete_credential', { providerId: id })
            set({
                credentials: { ...get().credentials, [id]: 'missing' as CredentialStatus }
            })
        } catch {
            // Delete failed
        }
    },

    checkCredentials: async () => {
        try {
            const statuses = await invoke<Record<string, boolean>>('check_credentials')
            const credentials: Record<string, CredentialStatus> = {}
            for (const [key, hasCredential] of Object.entries(statuses)) {
                credentials[key] = hasCredential ? 'stored' : 'missing'
            }
            set({ credentials })
        } catch {
            // Check failed
        }
    }
}))
