import { useEffect, useState } from 'react'
import { formatResetTime } from '@/lib/format'

interface ResetTimeLabelProps {
    resetsAt: string
}

export function ResetTimeLabel({ resetsAt }: ResetTimeLabelProps) {
    const [tick, setTick] = useState(0)

    useEffect(() => {
        const interval = setInterval(() => {
            setTick((t) => t + 1)
        }, 60_000)
        return () => clearInterval(interval)
    }, [resetsAt])

    // Re-derive on every render (triggered by tick or prop change)
    void tick
    const label = formatResetTime(resetsAt)

    return <span>{label}</span>
}
