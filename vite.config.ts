import { resolve } from "node:path";

import tailwindCss from "@tailwindcss/vite";
import { minify } from "terser";
import { defineConfig, type Plugin } from "vite";
import { getTailwindConfig } from "./scripts/util";

// import * as esbuild from "esbuild";

const tailwindConfig = await getTailwindConfig(process.env.TAILWIND_CONFIG);

const injectCSSImport = (): Plugin => {
	return {
		name: "inject-css-import",
		transform(code, id) {
			if (id.includes("public_script/main.ts") && tailwindConfig !== "SKIP") {
				return `import "../styles/${tailwindConfig}";\n${code}`;
			}
			return code;
		},
	};
};

const minifyBundle = (): Plugin => {
	return {
		name: "minify-bundle",
		async generateBundle(_, bundle) {
			for (const asset of Object.values(bundle)) {
				if (asset.type === "chunk") {
					// asset.code = (
					//   await esbuild.transform(asset.code, { minify: true })
					// ).code;

					const minifiedCode = await minify(asset.code, {
						sourceMap: false,
						mangle: true,
						compress: true,
					});

					if (minifiedCode.code) {
						asset.code = minifiedCode.code;
					}
				}
			}
		},
	};
};

export default defineConfig({
	appType: "custom",
	build: {
		// copyPublicDir: false,
		// emptyOutDir: false,
		// outDir: "public",
		lib: {
			entry: [resolve(__dirname, "public_script/main.ts")],
			formats: ["es"],
		},
		rollupOptions: {
			output: {
				assetFileNames: (assetInfo) => {
					if (assetInfo.name?.endsWith(".css")) {
						return "output.css";
					}
					return assetInfo.name || "";
				},
			},
		},
		// minify: false,
	},
	plugins: [injectCSSImport(), minifyBundle(), tailwindCss()],
});
