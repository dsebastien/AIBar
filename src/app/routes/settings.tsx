import { useEffect } from 'react'
import { useLocation } from 'react-router'
import { SettingsLayout } from '@/components/layout/settings-layout'
import { ProviderToggleList } from '@/components/settings/provider-toggle-list'
import { RefreshCadenceSelector } from '@/components/settings/refresh-cadence-selector'
import { useSettingsStore } from '@/stores/settings-store'

function GeneralTab() {
    return (
        <div className='space-y-6'>
            <RefreshCadenceSelector />

            <div className='space-y-2'>
                <h3 className='text-sm font-medium'>Launch at Login</h3>
                <p className='text-app-text-secondary text-xs'>
                    Automatically start AIBar when you log in to your computer.
                </p>
            </div>
        </div>
    )
}

function ProvidersTab() {
    return <ProviderToggleList />
}

function DisplayTab() {
    return (
        <div className='space-y-4'>
            <div className='space-y-2'>
                <h3 className='text-sm font-medium'>Display Mode</h3>
                <p className='text-app-text-secondary text-xs'>
                    Choose how providers are displayed in the tray popup.
                </p>
                <div className='space-y-2'>
                    <label className='flex items-center gap-2 text-xs'>
                        <input type='radio' name='displayMode' value='individual' defaultChecked />
                        <span>Individual cards</span>
                    </label>
                    <label className='flex items-center gap-2 text-xs'>
                        <input type='radio' name='displayMode' value='merged' />
                        <span>Merged with tab switcher</span>
                    </label>
                </div>
            </div>
        </div>
    )
}

function AboutTab() {
    return (
        <div className='space-y-4'>
            <div>
                <h3 className='text-sm font-medium'>AIBar</h3>
                <p className='text-app-text-secondary text-xs'>Version 0.1.0</p>
            </div>
            <div className='space-y-1'>
                <p className='text-app-text-secondary text-xs'>
                    Cross-platform AI usage monitoring system tray application.
                </p>
            </div>
        </div>
    )
}

function SettingsContent() {
    const location = useLocation()

    switch (location.pathname) {
        case '/settings/providers':
            return <ProvidersTab />
        case '/settings/display':
            return <DisplayTab />
        case '/settings/about':
            return <AboutTab />
        default:
            return <GeneralTab />
    }
}

export default function Settings() {
    const initSettings = useSettingsStore((s) => s.initialize)

    useEffect(() => {
        void initSettings()
    }, [initSettings])

    return (
        <SettingsLayout>
            <SettingsContent />
        </SettingsLayout>
    )
}
