import { openUrl } from '@tauri-apps/plugin-opener'
import type { ProviderMetadata, UsageSnapshot } from '@/lib/types'
import { Card, CardContent, CardHeader } from '@/components/ui/card'
import { UsageProgressBar } from '@/components/providers/usage-progress-bar'

interface ProviderCardProps {
    metadata: ProviderMetadata
    snapshot: UsageSnapshot | null
}

export function ProviderCard({ metadata, snapshot }: ProviderCardProps) {
    const handleClick = () => {
        if (metadata.dashboardUrl) {
            void openUrl(metadata.dashboardUrl)
        }
    }

    return (
        <Card
            className={
                metadata.dashboardUrl
                    ? 'hover:border-app-accent/50 cursor-pointer transition-colors'
                    : ''
            }
            onClick={handleClick}
        >
            <CardHeader className='pb-2'>
                <div className='flex items-center gap-2'>
                    <div
                        className='text-app-background flex h-6 w-6 items-center justify-center rounded-full text-xs font-bold'
                        style={{ backgroundColor: metadata.color }}
                    >
                        {metadata.displayName.charAt(0)}
                    </div>
                    <span className='text-sm font-medium'>{metadata.displayName}</span>
                </div>
            </CardHeader>
            <CardContent>
                {snapshot ? (
                    <div className='space-y-3'>
                        {snapshot.primary && (
                            <UsageProgressBar
                                rateWindow={snapshot.primary}
                                label={metadata.sessionLabel}
                            />
                        )}
                        {snapshot.secondary && (
                            <UsageProgressBar
                                rateWindow={snapshot.secondary}
                                label={metadata.weeklyLabel}
                            />
                        )}
                    </div>
                ) : (
                    <p className='text-app-text-secondary text-xs'>No data available</p>
                )}
            </CardContent>
        </Card>
    )
}
