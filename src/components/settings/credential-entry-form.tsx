import { useState } from 'react'
import type { ProviderId } from '@/lib/types'
import { useCredentialsStore, type CredentialStatus } from '@/stores/credentials-store'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'

interface CredentialEntryFormProps {
    providerId: ProviderId
    providerName: string
}

export function CredentialEntryForm({ providerId, providerName }: CredentialEntryFormProps) {
    const [token, setToken] = useState('')
    const [isSaving, setIsSaving] = useState(false)
    const credentials = useCredentialsStore((s) => s.credentials)
    const storeApiToken = useCredentialsStore((s) => s.storeApiToken)
    const deleteCredential = useCredentialsStore((s) => s.deleteCredential)

    const status: CredentialStatus = credentials[providerId] ?? 'missing'

    const handleSave = async () => {
        if (!token.trim()) return
        setIsSaving(true)
        try {
            await storeApiToken(providerId, token.trim())
            setToken('')
        } finally {
            setIsSaving(false)
        }
    }

    const handleDelete = async () => {
        await deleteCredential(providerId)
    }

    return (
        <div className='border-app-border space-y-2 rounded-md border p-3'>
            <div className='flex items-center justify-between'>
                <span className='text-sm font-medium'>{providerName}</span>
                <Badge status={status === 'configured' ? 'operational' : 'outage'}>
                    {status === 'configured' ? 'Configured' : 'Missing'}
                </Badge>
            </div>

            <div className='flex gap-2'>
                <input
                    type='password'
                    value={token}
                    onChange={(e) => setToken(e.target.value)}
                    placeholder='Enter API token...'
                    className='border-app-border bg-app-primary text-app-text placeholder:text-app-text-secondary focus:ring-app-accent h-8 flex-1 rounded-md border px-2 text-xs focus:ring-1 focus:outline-none'
                />
                <Button
                    size='sm'
                    onClick={() => void handleSave()}
                    disabled={isSaving || !token.trim()}
                >
                    Save
                </Button>
                {status === 'configured' && (
                    <Button size='sm' variant='destructive' onClick={() => void handleDelete()}>
                        Remove
                    </Button>
                )}
            </div>
        </div>
    )
}
