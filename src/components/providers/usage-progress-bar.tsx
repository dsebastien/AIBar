import type { RateWindow } from '@/lib/types'
import { formatPercent } from '@/lib/format'
import { Progress } from '@/components/ui/progress'
import { ResetTimeLabel } from '@/components/providers/reset-time-label'

interface UsageProgressBarProps {
    rateWindow: RateWindow
    label: string
}

export function UsageProgressBar({ rateWindow, label }: UsageProgressBarProps) {
    const { usedPercent, resetsAt, resetDescription } = rateWindow

    return (
        <div className='space-y-1'>
            <div className='flex items-center justify-between text-xs'>
                <span className='text-app-text-secondary'>{label}</span>
                <span className='font-medium'>{formatPercent(usedPercent)}</span>
            </div>
            <Progress value={usedPercent} />
            {resetsAt && (
                <div className='text-app-text-secondary text-[10px]'>
                    <ResetTimeLabel resetsAt={resetsAt} />
                </div>
            )}
            {!resetsAt && resetDescription && (
                <div className='text-app-text-secondary text-[10px]'>{resetDescription}</div>
            )}
        </div>
    )
}
