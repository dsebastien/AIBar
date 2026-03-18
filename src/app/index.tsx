import { ErrorBoundary } from 'react-error-boundary'
import { Suspense } from 'react'

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

export function App() {
    return (
        <ErrorBoundary FallbackComponent={ErrorFallback}>
            <Suspense
                fallback={<div className='flex h-full items-center justify-center'>Loading...</div>}
            >
                <div className='flex h-full flex-col items-center justify-center'>
                    <h1 className='text-2xl font-bold'>AIBar</h1>
                    <p className='text-app-text-secondary mt-2'>AI Usage Monitor</p>
                </div>
            </Suspense>
        </ErrorBoundary>
    )
}
