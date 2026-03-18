import type { ReactNode } from 'react'
import { useNavigate } from 'react-router'
import { IoSettingsSharp, IoRefresh } from 'react-icons/io5'
import { Button } from '@/components/ui/button'

interface TrayPopupLayoutProps {
    children: ReactNode
    onRefresh: () => void
    lastUpdated?: string
    isRefreshing?: boolean
}

export function TrayPopupLayout({
    children,
    onRefresh,
    lastUpdated,
    isRefreshing = false
}: TrayPopupLayoutProps) {
    const navigate = useNavigate()

    return (
        <div className='flex h-full flex-col'>
            <header className='border-app-border flex items-center justify-between border-b px-4 py-2'>
                <h1 className='text-sm font-bold'>AIBar</h1>
                <Button
                    variant='ghost'
                    size='sm'
                    onClick={() => void navigate('/settings')}
                    aria-label='Settings'
                >
                    <IoSettingsSharp className='h-4 w-4' />
                </Button>
            </header>

            <main className='flex-1 overflow-y-auto'>{children}</main>

            <footer className='border-app-border flex items-center justify-between border-t px-4 py-2'>
                <span className='text-app-text-secondary text-[10px]'>
                    {lastUpdated ? `Updated: ${lastUpdated}` : ''}
                </span>
                <Button
                    variant='ghost'
                    size='sm'
                    onClick={onRefresh}
                    disabled={isRefreshing}
                    aria-label='Refresh'
                >
                    <IoRefresh className={`h-4 w-4 ${isRefreshing ? 'animate-spin' : ''}`} />
                </Button>
            </footer>
        </div>
    )
}
