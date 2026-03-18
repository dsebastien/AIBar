import type { RefreshCadence } from '@/lib/types'
import { useSettingsStore } from '@/stores/settings-store'
import { Select, type SelectOption } from '@/components/ui/select'

const CADENCE_OPTIONS: SelectOption[] = [
    { value: 'manual', label: 'Manual' },
    { value: 'one_minute', label: 'Every 1 minute' },
    { value: 'two_minutes', label: 'Every 2 minutes' },
    { value: 'five_minutes', label: 'Every 5 minutes' },
    { value: 'fifteen_minutes', label: 'Every 15 minutes' }
]

export function RefreshCadenceSelector() {
    const config = useSettingsStore((s) => s.config)
    const setRefreshCadence = useSettingsStore((s) => s.setRefreshCadence)

    if (!config) return null

    return (
        <div className='space-y-2'>
            <label className='text-sm font-medium' htmlFor='refresh-cadence'>
                Refresh Cadence
            </label>
            <Select
                id='refresh-cadence'
                options={CADENCE_OPTIONS}
                value={config.refreshCadence}
                onChange={(e) => void setRefreshCadence(e.target.value as RefreshCadence)}
            />
        </div>
    )
}
