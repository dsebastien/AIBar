import { forwardRef, type InputHTMLAttributes } from 'react'
import { cn } from '@/lib/utils/cn'

export interface SwitchProps extends Omit<InputHTMLAttributes<HTMLInputElement>, 'type'> {
    checked: boolean
    onCheckedChange?: (checked: boolean) => void
}

export const Switch = forwardRef<HTMLInputElement, SwitchProps>(
    ({ className, checked, onCheckedChange, ...props }, ref) => {
        return (
            <button
                type='button'
                role='switch'
                aria-checked={checked}
                className={cn(
                    'focus-visible:ring-app-accent focus-visible:ring-offset-app-background relative inline-flex h-5 w-9 shrink-0 cursor-pointer items-center rounded-full border-2 border-transparent transition-colors focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-50',
                    checked ? 'bg-app-accent' : 'bg-app-primary',
                    className
                )}
                onClick={() => onCheckedChange?.(!checked)}
            >
                <input ref={ref} type='checkbox' className='sr-only' checked={checked} {...props} />
                <span
                    className={cn(
                        'pointer-events-none block h-4 w-4 rounded-full bg-white shadow-lg ring-0 transition-transform',
                        checked ? 'translate-x-4' : 'translate-x-0'
                    )}
                />
            </button>
        )
    }
)

Switch.displayName = 'Switch'
