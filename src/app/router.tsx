import { lazy } from 'react'
import { createBrowserRouter, RouterProvider } from 'react-router'

const Dashboard = lazy(() => import('@/app/routes/dashboard'))
const Settings = lazy(() => import('@/app/routes/settings'))

const router = createBrowserRouter([
    {
        path: '/',
        Component: Dashboard
    },
    {
        path: '/settings',
        Component: Settings
    },
    {
        path: '/settings/:tab',
        Component: Settings
    }
])

export function AppRouter() {
    return <RouterProvider router={router} />
}
