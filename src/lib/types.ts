export type ProviderId =
    | 'codex'
    | 'claude'
    | 'cursor'
    | 'gemini'
    | 'copilot'
    | 'augment'
    | 'amp'
    | 'kimi'
    | 'kimi_k2'
    | 'zai'
    | 'minimax'
    | 'factory'
    | 'jetbrains'
    | 'kilo'
    | 'kiro'
    | 'vertex_ai'
    | 'ollama'
    | 'synthetic'
    | 'warp'
    | 'openrouter'
    | 'antigravity'
    | 'opencode'

export type RefreshCadence =
    | 'manual'
    | 'one_minute'
    | 'two_minutes'
    | 'five_minutes'
    | 'fifteen_minutes'

export type DisplayMode = 'individual' | 'merged'

export interface RateWindow {
    usedPercent: number
    windowMinutes?: number
    resetsAt?: string
    resetDescription?: string
}

export interface ProviderCostSnapshot {
    used: number
    limit: number
    currencyCode: string
    period?: string
    resetsAt?: string
}

export interface ProviderIdentitySnapshot {
    email?: string
    team?: string
    plan?: string
}

export interface UsageSnapshot {
    primary?: RateWindow
    secondary?: RateWindow
    tertiary?: RateWindow
    providerCost?: ProviderCostSnapshot
    updatedAt: string
    identity?: ProviderIdentitySnapshot
}

export interface CreditsSnapshot {
    remaining: number
    updatedAt: string
}

export interface AppConfig {
    enabledProviders: ProviderId[]
    providerOrder: ProviderId[]
    refreshCadence: RefreshCadence
    displayMode: DisplayMode
    launchAtLogin: boolean
}

export interface ProviderMetadata {
    id: ProviderId
    displayName: string
    sessionLabel: string
    weeklyLabel: string
    supportsCredits: boolean
    defaultEnabled: boolean
    dashboardUrl?: string
    statusPageUrl?: string
    color: string
}
