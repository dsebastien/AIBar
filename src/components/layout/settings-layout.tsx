import type { ReactNode } from 'react'
import { useNavigate, useLocation } from 'react-router'
import { IoArrowBack } from 'react-icons/io5'
import { Button } from '@/components/ui/button'
import { cn } from '@/lib/utils/cn'

interface SettingsLayoutProps {
    children: ReactNode
}

const NAV_LINKS = [
    { path: '/settings', label: 'General' },
    { path: '/settings/providers', label: 'Providers' },
    { path: '/settings/display', label: 'Display' },
    { path: '/settings/about', label: 'About' }
] as const

export function SettingsLayout({ children }: SettingsLayoutProps) {
    const navigate = useNavigate()
    const location = useLocation()

    return (
        <div className='flex h-full flex-col'>
            <header className='border-app-border flex items-center gap-2 border-b px-4 py-2'>
                <Button
                    variant='ghost'
                    size='sm'
                    onClick={() => void navigate('/')}
                    aria-label='Back to dashboard'
                >
                    <IoArrowBack className='h-4 w-4' />
                </Button>
                <h1 className='text-sm font-bold'>Settings</h1>
            </header>

            <div className='flex flex-1 overflow-hidden'>
                <nav className='border-app-border w-32 shrink-0 border-r py-2'>
                    {NAV_LINKS.map((link) => (
                        <button
                            key={link.path}
                            type='button'
                            className={cn(
                                'block w-full px-4 py-1.5 text-left text-xs transition-colors',
                                location.pathname === link.path
                                    ? 'bg-app-accent/20 text-app-accent font-medium'
                                    : 'text-app-text-secondary hover:text-app-text'
                            )}
                            onClick={() => void navigate(link.path)}
                        >
                            {link.label}
                        </button>
                    ))}
                </nav>

                <main className='flex-1 overflow-y-auto p-4'>{children}</main>
            </div>
        </div>
    )
}
