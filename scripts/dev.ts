import type { TomlTable } from "smol-toml";
import type { AppConfig } from "./config.ts";
import { getConfig } from "./config.ts";
import { getCargoTOML, getDirectoryFolders, getPackageJSON } from "./fs.ts";
import { catchError, runCommand } from "./process.ts";
import { getApp, getAppFeatures, getTailwindConfig } from "./prompts.ts";

const libsDir = await getDirectoryFolders("../libs");

const buildWatchPaths = async (
	app: string,
	cargoToml: TomlTable | undefined,
): Promise<string> => {
	const paths = ["-w public_script", `-w app/${app}`];

	if (!cargoToml) return paths.join(" ");

	const appDeps = cargoToml.dependencies;
	if (typeof appDeps !== "object" || appDeps === null) return paths.join(" ");

	const directLibDeps = libsDir.filter((lib) => lib in appDeps);

	const libPaths = await Promise.all(
		directLibDeps.map(async (lib) => {
			const libToml = await getCargoTOML(`libs/${lib}`);
			const libDeps = libToml?.dependencies;
			const transitiveDeps =
				typeof libDeps === "object" && libDeps !== null
					? Object.keys(libDeps).filter((dep) => libsDir.includes(dep))
					: [];
			return [lib, ...transitiveDeps].map((d) => `-w libs/${d}`);
		}),
	);

	return [...paths, ...new Set(libPaths.flat())].join(" ");
};

export const run = async (config: AppConfig) => {
	if (config.env === "production") {
		throw new Error("This script is not supported in production mode.");
	}
	const app = await getApp(config.app);

	const cargoToml = await getCargoTOML(`app/${app}`);
	const packageJson = await getPackageJSON(app);

	if (packageJson) {
		const cmd = `cd app/${app} && bun run dev`;
		await runCommand(cmd);
	} else {
		const tailwindConfig = await getTailwindConfig(
			config.tailwindConfig || app,
		);
		process.env.TAILWIND_CONFIG = tailwindConfig;

		const watchApp = await buildWatchPaths(app, cargoToml);
		const featuresList = await getAppFeatures(cargoToml?.features || {});

		const features =
			featuresList.length > 0 ? ` --features ${featuresList.join(",")}` : "";

		const cmd = `watchexec -I -q ${watchApp} -r "bun run build.script -l silent & cargo run -p ${app}${features}"`;
		console.log("Running command:", cmd);
		await runCommand(cmd);
	}
};

try {
	const config = await getConfig();

	await run(config);
} catch (error) {
	catchError(error);
}
