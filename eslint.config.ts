import globals from 'globals'
import pluginJs from '@eslint/js'
import tseslint from 'typescript-eslint'
import pluginReact from 'eslint-plugin-react'
import pluginReactHooks from 'eslint-plugin-react-hooks'
import eslintConfigPrettier from 'eslint-config-prettier'
import type { Linter } from 'eslint'

const config: Linter.Config[] = [
    { files: ['**/*.{js,mjs,cjs,ts,jsx,tsx}'] },
    {
        ignores: ['**/src-tauri', '**/dist/**']
    },
    {
        languageOptions: {
            globals: {
                ...globals.browser,
                ...globals.node
            }
        }
    },
    pluginJs.configs.recommended,
    ...tseslint.configs.recommended,
    {
        settings: {
            react: {
                version: 'detect'
            }
        }
    },
    pluginReact.configs.flat.recommended as Linter.Config,
    {
        plugins: {
            'react-hooks': pluginReactHooks
        },
        rules: {
            ...pluginReactHooks.configs.recommended.rules
        }
    },
    eslintConfigPrettier as Linter.Config,
    {
        rules: {
            'react/react-in-jsx-scope': 'off',
            'react/prop-types': 'off',
            'react/no-unescaped-entities': 'off'
        }
    },
    {
        files: ['**/*.cjs', '.release-it.js'],
        rules: {
            '@typescript-eslint/no-require-imports': 'off'
        }
    }
]

export default config
