import { glob } from "node:fs/promises";
import path from "node:path";
import { parseArgs } from "node:util";

import tailwindCss from "@tailwindcss/vite";
import type { ReadonlyDeep } from "type-fest";
import { type ConfigEnv, type Plugin, defineConfig } from "vite-plus";
import type { PreRenderedAsset } from "vite/rolldown";

const dashIndex = process.argv.indexOf("--");
const args = process.argv.slice(dashIndex + 1);

const getTailwindCssPath = async (): Promise<string | undefined> => {
	const { values } = parseArgs({
		allowPositionals: true,
		args,
		options: {
			app: {
				type: "string",
			},
		},
		strict: false,
	});

	const app = values.app ?? process.env.APP;
	if (app === undefined || typeof app !== "string") {
		return undefined;
	}

	const appPath = path.resolve(import.meta.dirname, "../", app);
	const cssFilesIter = glob("**/*/main.css", {
		cwd: appPath,
	});
	const cssFiles = await Array.fromAsync(cssFilesIter);
	const cssFilesResolved = cssFiles.map((file) => path.resolve(appPath, file));
	return cssFilesResolved.at(0);
};

const injectCSSImport = async (): Promise<Plugin> => {
	const realEntry = path.resolve(import.meta.dirname, "src/main.ts");

	const tailwindCssPath = await getTailwindCssPath();
	if (tailwindCssPath === undefined || tailwindCssPath.length === 0) {
		return { name: "inject-css-import" };
	}

	return {
		name: "inject-css-import",
		transform(code, id): string {
			if (id === realEntry) {
				return `import "${tailwindCssPath}";\n${code}`;
			}
			return code;
		},
	};
};

export default defineConfig(({ mode }: Readonly<ConfigEnv>) => ({
	appType: "custom",
	build: {
		emptyOutDir: mode === "development",
		lib: {
			entry: [path.resolve(import.meta.dirname, "src/main.ts")],
			formats: ["es"],
		},
		manifest: true,
		minify: "oxc",
		outDir: "../../dist",
		rolldownOptions: {
			output: {
				assetFileNames: (assetInfo: ReadonlyDeep<PreRenderedAsset>): string => {
					const assetName = assetInfo.names[0] ?? "";

					if (assetName.endsWith(".css")) {
						return "output.[hash].css";
					}
					return "[name].[hash].[extname]";
				},
				chunkFileNames: "[name].[hash].js",
				entryFileNames: "[name].[hash].js",
				minify: {
					codegen: {
						removeWhitespace: true,
					},
				},
			},
		},
	},
	plugins: [injectCSSImport(), tailwindCss()],
}));
