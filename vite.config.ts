import { defineConfig } from "vite-plus";

export default defineConfig(() => ({
	fmt: {
		sortImports: true,
		useTabs: true,
	},
	lint: {
		categories: {
			correctness: "error",
			nursery: "off",
			pedantic: "warn",
			perf: "warn",
			restriction: "warn",
			style: "warn",
			suspicious: "warn",
		},
		options: {
			typeAware: true,
			typeCheck: true,
		},
		overrides: [
			{
				files: ["vite.config.ts"],
				rules: {
					"max-lines-per-function": "off",
					"no-magic-numbers": "off",
				},
			},
			{
				files: ["scripts/**/*"],
				rules: {
					"no-console": "off",
				},
			},
			{
				files: ["scripts/process.ts"],
				rules: {
					"unicorn/no-process-exit": "off",
				},
			},
			{
				files: ["taboola_*.js"],
				rules: {
					"unicorn/filename-case": ["off"],
				},
			},
		],
		rules: {
			"max-statements": ["warn", 12],
			"no-magic-numbers": [
				"warn",
				{
					ignore: [0, 1, -1],
				},
			],
			"no-ternary": "off",
			"no-undefined": "off",
			"one-var": "off",
			"oxc/no-async-await": "off",
			"oxc/no-optional-chaining": "off",
			"oxc/no-rest-spread-properties": "off",
			"sort-imports": [
				"warn",
				{
					ignoreDeclarationSort: true,
				},
			],
		},
	},
}));
