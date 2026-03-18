import { ErrorBoundary } from 'react-error-boundary'
import { Suspense } from 'react'
import { TooltipProvider } from '@/components/ui/tooltip'
import { AppRouter } from '@/app/router'

function ErrorFallback({ error }: { error: Error }) {
    return (
        <div className='flex h-full items-center justify-center p-4'>
            <div className='text-center'>
                <h1 className='text-app-danger mb-2 text-lg font-bold'>Something went wrong</h1>
                <pre className='text-app-text-secondary text-sm'>{error.message}</pre>
            </div>
        </div>
    )
}

export function AppProvider() {
    return (
        <ErrorBoundary FallbackComponent={ErrorFallback}>
            <TooltipProvider>
                <Suspense
                    fallback={
                        <div className='flex h-full items-center justify-center'>Loading...</div>
                    }
                >
                    <AppRouter />
                </Suspense>
            </TooltipProvider>
        </ErrorBoundary>
    )
}
