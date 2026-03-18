import type { ProviderMetadata } from './types'

export const PROVIDERS: ProviderMetadata[] = [
    {
        id: 'claude',
        displayName: 'Claude',
        sessionLabel: 'Session',
        weeklyLabel: 'Weekly',
        supportsCredits: false,
        defaultEnabled: true,
        dashboardUrl: 'https://claude.ai',
        statusPageUrl: 'https://status.anthropic.com',
        color: '#d4a574'
    },
    {
        id: 'codex',
        displayName: 'Codex',
        sessionLabel: 'Session',
        weeklyLabel: 'Weekly',
        supportsCredits: true,
        defaultEnabled: true,
        dashboardUrl: 'https://chatgpt.com',
        color: '#10a37f'
    },
    {
        id: 'cursor',
        displayName: 'Cursor',
        sessionLabel: 'Premium',
        weeklyLabel: 'Usage',
        supportsCredits: false,
        defaultEnabled: true,
        dashboardUrl: 'https://cursor.com/settings',
        color: '#00d4aa'
    },
    {
        id: 'gemini',
        displayName: 'Gemini',
        sessionLabel: 'Session',
        weeklyLabel: 'Daily',
        supportsCredits: false,
        defaultEnabled: true,
        dashboardUrl: 'https://gemini.google.com',
        color: '#4285f4'
    },
    {
        id: 'copilot',
        displayName: 'Copilot',
        sessionLabel: 'Premium',
        weeklyLabel: 'Chat',
        supportsCredits: false,
        defaultEnabled: false,
        dashboardUrl: 'https://github.com/settings/copilot',
        statusPageUrl: 'https://www.githubstatus.com',
        color: '#a855f7'
    },
    {
        id: 'augment',
        displayName: 'Augment',
        sessionLabel: 'Session',
        weeklyLabel: 'Weekly',
        supportsCredits: false,
        defaultEnabled: false,
        dashboardUrl: 'https://augmentcode.com',
        color: '#6366f1'
    },
    {
        id: 'amp',
        displayName: 'Amp',
        sessionLabel: 'Session',
        weeklyLabel: 'Weekly',
        supportsCredits: false,
        defaultEnabled: false,
        dashboardUrl: 'https://ampcode.com',
        color: '#f59e0b'
    },
    {
        id: 'kimi',
        displayName: 'Kimi',
        sessionLabel: '5-Hour',
        weeklyLabel: 'Weekly',
        supportsCredits: false,
        defaultEnabled: false,
        color: '#8b5cf6'
    },
    {
        id: 'kimi_k2',
        displayName: 'Kimi K2',
        sessionLabel: 'Session',
        weeklyLabel: 'Weekly',
        supportsCredits: false,
        defaultEnabled: false,
        color: '#7c3aed'
    },
    {
        id: 'zai',
        displayName: 'z.ai',
        sessionLabel: 'Session',
        weeklyLabel: 'Weekly',
        supportsCredits: false,
        defaultEnabled: false,
        dashboardUrl: 'https://z.ai',
        color: '#ec4899'
    },
    {
        id: 'minimax',
        displayName: 'MiniMax',
        sessionLabel: 'Session',
        weeklyLabel: 'Weekly',
        supportsCredits: false,
        defaultEnabled: false,
        color: '#14b8a6'
    },
    {
        id: 'factory',
        displayName: 'Factory',
        sessionLabel: 'Session',
        weeklyLabel: 'Weekly',
        supportsCredits: false,
        defaultEnabled: false,
        color: '#f97316'
    },
    {
        id: 'jetbrains',
        displayName: 'JetBrains',
        sessionLabel: 'Session',
        weeklyLabel: 'Weekly',
        supportsCredits: false,
        defaultEnabled: false,
        color: '#ff318c'
    },
    {
        id: 'kilo',
        displayName: 'Kilo',
        sessionLabel: 'Session',
        weeklyLabel: 'Weekly',
        supportsCredits: false,
        defaultEnabled: false,
        color: '#06b6d4'
    },
    {
        id: 'kiro',
        displayName: 'Kiro',
        sessionLabel: 'Session',
        weeklyLabel: 'Weekly',
        supportsCredits: false,
        defaultEnabled: false,
        color: '#f43f5e'
    },
    {
        id: 'vertex_ai',
        displayName: 'Vertex AI',
        sessionLabel: 'Session',
        weeklyLabel: 'Weekly',
        supportsCredits: false,
        defaultEnabled: false,
        color: '#4285f4'
    },
    {
        id: 'ollama',
        displayName: 'Ollama',
        sessionLabel: 'Session',
        weeklyLabel: 'Weekly',
        supportsCredits: false,
        defaultEnabled: false,
        dashboardUrl: 'http://localhost:11434',
        color: '#ffffff'
    },
    {
        id: 'synthetic',
        displayName: 'Synthetic',
        sessionLabel: 'Session',
        weeklyLabel: 'Weekly',
        supportsCredits: false,
        defaultEnabled: false,
        color: '#a3e635'
    },
    {
        id: 'warp',
        displayName: 'Warp',
        sessionLabel: 'Session',
        weeklyLabel: 'Weekly',
        supportsCredits: false,
        defaultEnabled: false,
        color: '#00d4ff'
    },
    {
        id: 'openrouter',
        displayName: 'OpenRouter',
        sessionLabel: 'Credits',
        weeklyLabel: 'Usage',
        supportsCredits: true,
        defaultEnabled: false,
        dashboardUrl: 'https://openrouter.ai',
        color: '#6d28d9'
    },
    {
        id: 'antigravity',
        displayName: 'Antigravity',
        sessionLabel: 'Session',
        weeklyLabel: 'Weekly',
        supportsCredits: false,
        defaultEnabled: false,
        color: '#84cc16'
    },
    {
        id: 'opencode',
        displayName: 'OpenCode',
        sessionLabel: 'Session',
        weeklyLabel: 'Weekly',
        supportsCredits: false,
        defaultEnabled: false,
        color: '#0ea5e9'
    }
]

export const DEFAULT_REFRESH_CADENCE = 'five_minutes'
export const DEFAULT_DISPLAY_MODE = 'individual'
