import type { UserConfig } from '@commitlint/types'

// eslint-disable-next-line @typescript-eslint/no-require-imports
const scopes = require('./conventional-commit-scopes.cjs')

const config: UserConfig = {
    extends: ['@commitlint/config-conventional'],
    rules: {
        'header-max-length': [1, 'always', 100],
        'scope-enum': [2, 'always', scopes],
        'scope-case': [2, 'always', 'lowercase']
    }
}

export default config
