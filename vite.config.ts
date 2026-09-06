import { resolve } from "node:path";

import tailwindCss from "@tailwindcss/vite";
import { defineConfig, type Plugin } from "vite";

import { getTailwindConfig } from "./scripts/prompts.ts";

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

export default defineConfig({
	appType: "custom",
	build: {
		lib: {
			entry: [resolve(import.meta.dirname, "public_script/main.ts")],
			formats: ["es"],
		},
		minify: "oxc",
		rolldownOptions: {
			output: {
				assetFileNames: (assetInfo) => {
					const assetName = assetInfo.names[0] || "";

					if (assetName.endsWith(".css")) {
						return "output.css";
					}
					return assetName;
				},
				minify: {
					codegen: {
						removeWhitespace: true,
					},
				},
			},
		},
	},
	plugins: [injectCSSImport(), tailwindCss()],
});
