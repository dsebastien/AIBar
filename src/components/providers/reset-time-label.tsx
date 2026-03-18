import { useEffect, useMemo, useState } from 'react'
import { formatResetTime } from '@/lib/format'

interface ResetTimeLabelProps {
    resetsAt: string
}

export function ResetTimeLabel({ resetsAt }: ResetTimeLabelProps) {
    const initialLabel = useMemo(() => formatResetTime(resetsAt), [resetsAt])
    const [label, setLabel] = useState(initialLabel)

    useEffect(() => {
        const interval = setInterval(() => {
            setLabel(formatResetTime(resetsAt))
        }, 60_000)

        return () => clearInterval(interval)
    }, [resetsAt])

    // Sync label when resetsAt changes
    useEffect(() => {
        setLabel(initialLabel)
    }, [initialLabel])

    return <span>{label}</span>
}
