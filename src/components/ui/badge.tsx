import { cva, type VariantProps } from 'class-variance-authority'
import type { HTMLAttributes } from 'react'
import { cn } from '@/lib/utils/cn'

const badgeVariants = cva('inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium', {
    variants: {
        status: {
            operational: 'bg-app-success/20 text-app-success',
            degraded: 'bg-app-warning/20 text-app-warning',
            outage: 'bg-app-danger/20 text-app-danger'
        }
    },
    defaultVariants: {
        status: 'operational'
    }
})

export interface BadgeProps
    extends HTMLAttributes<HTMLSpanElement>,
        VariantProps<typeof badgeVariants> {}

export function Badge({ className, status, ...props }: BadgeProps) {
    return <span className={cn(badgeVariants({ status, className }))} {...props} />
}
