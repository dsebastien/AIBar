import { forwardRef, type HTMLAttributes } from 'react'
import { cn } from '@/lib/utils/cn'

function getAutoColor(value: number): string {
    if (value < 50) return 'bg-app-success'
    if (value < 80) return 'bg-app-warning'
    return 'bg-app-danger'
}

function getColorClass(color: 'green' | 'yellow' | 'red' | 'auto', value: number): string {
    if (color === 'auto') return getAutoColor(value)
    if (color === 'green') return 'bg-app-success'
    if (color === 'yellow') return 'bg-app-warning'
    return 'bg-app-danger'
}

export interface ProgressProps extends HTMLAttributes<HTMLDivElement> {
    value: number
    color?: 'green' | 'yellow' | 'red' | 'auto'
}

export const Progress = forwardRef<HTMLDivElement, ProgressProps>(
    ({ className, value, color = 'auto', ...props }, ref) => {
        const clampedValue = Math.min(100, Math.max(0, value))

        return (
            <div
                ref={ref}
                className={cn(
                    'bg-app-primary relative h-2 w-full overflow-hidden rounded-full',
                    className
                )}
                {...props}
            >
                <div
                    className={cn(
                        'h-full rounded-full transition-all duration-300',
                        getColorClass(color, clampedValue)
                    )}
                    style={{ width: `${clampedValue}%` }}
                />
            </div>
        )
    }
)

Progress.displayName = 'Progress'
