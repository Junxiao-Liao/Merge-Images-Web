import js from '@eslint/js';
import tseslint from 'typescript-eslint';
import svelte from 'eslint-plugin-svelte';
import globals from 'globals';
import prettier from 'eslint-config-prettier';
import prettierPlugin from 'eslint-plugin-prettier';
import svelteConfig from './svelte.config.js';

const typeScriptFiles = ['**/*.{ts,tsx,cts,mts}'];

const typeCheckedConfigs = [
	...tseslint.configs.strictTypeChecked,
	...tseslint.configs.stylisticTypeChecked
].map((config) => ({
	...config,
	files: typeScriptFiles
}));

export default [
	{
		ignores: ['**/.svelte-kit/**', '**/build/**', '**/dist/**', '**/static/wasm/**']
	},
	js.configs.recommended,
	...typeCheckedConfigs,
	{
		files: typeScriptFiles,
		languageOptions: {
			parserOptions: {
				projectService: true
			}
		}
	},
	...svelte.configs['flat/recommended'],
	...svelte.configs['flat/prettier'],
	{
		languageOptions: {
			globals: {
				...globals.browser,
				...globals.node
			}
		}
	},
	{
		files: ['**/*.svelte', '**/*.svelte.ts', '**/*.svelte.js'],
		languageOptions: {
			parserOptions: {
				projectService: true,
				extraFileExtensions: ['.svelte'],
				parser: tseslint.parser,
				svelteConfig
			}
		},
		rules: {
			'svelte/block-lang': [
				'error',
				{
					script: ['ts'],
					style: ['css']
				}
			],
			// Disable for Tailwind CSS - classes are processed at build time
			'svelte/no-unused-class-name': 'off',
			// Allow implicit button type for simple buttons
			'svelte/button-has-type': 'off',
			// Disable base no-unused-vars as it incorrectly flags TS interface params
			'no-unused-vars': 'off'
		}
	},
	prettier,
	{
		plugins: { prettier: prettierPlugin },
		rules: {
			'prettier/prettier': 'error'
		}
	}
];
