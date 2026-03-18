import { formatDistanceToNowStrict } from 'date-fns'

export function formatPercent(value: number): string {
    return `${Math.round(value)}%`
}

export function formatResetTime(isoDate: string): string {
    try {
        const resetDate = new Date(isoDate)
        const now = new Date()

        if (resetDate <= now) {
            return 'resets soon'
        }

        const distance = formatDistanceToNowStrict(resetDate, { addSuffix: false })
        return `resets in ${distance}`
    } catch {
        return 'reset time unknown'
    }
}

export function formatCost(usd: number): string {
    return `$${usd.toFixed(2)}`
}

export function formatTokenCount(count: number): string {
    if (count >= 1_000_000) {
        return `${(count / 1_000_000).toFixed(1)}M tokens`
    }
    if (count >= 1_000) {
        return `${(count / 1_000).toFixed(1)}K tokens`
    }
    return `${count} tokens`
}
