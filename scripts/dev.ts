import type { TomlTable } from "smol-toml";
import type { AppConfig } from "./config.ts";
import { getConfig } from "./config.ts";
import { getCargoTOML, getDirectoryFolders, getPackageJSON } from "./fs.ts";
import { catchError, spawnSafe } from "./process.ts";
import { getApp, getAppFeatures, getTailwindConfig } from "./prompts.ts";

const libsDir = await getDirectoryFolders("../libs");

const buildWatchPaths = async (
	app: string,
	cargoToml: TomlTable | undefined,
): Promise<string[]> => {
	const paths = ["public_script", `app/${app}`];

	if (!cargoToml) return paths;

	const appDeps = cargoToml.dependencies;
	if (typeof appDeps !== "object" || appDeps === null) return paths;

	const directLibDeps = libsDir.filter((lib) => lib in appDeps);

	const libPaths = await Promise.all(
		directLibDeps.map(async (lib) => {
			const libToml = await getCargoTOML(`libs/${lib}`);
			const libDeps = libToml?.dependencies;
			const transitiveDeps =
				typeof libDeps === "object" && libDeps !== null
					? Object.keys(libDeps).filter((dep) => libsDir.includes(dep))
					: [];
			return [lib, ...transitiveDeps].map((d) => `libs/${d}`);
		}),
	);

	return [...paths, ...new Set(libPaths.flat())];
};

export const run = async (config: AppConfig) => {
	if (config.env === "production") {
		throw new Error("This script is not supported in production mode.");
	}
	const app = await getApp(config.app);

	const cargoToml = await getCargoTOML(`app/${app}`);
	const packageJson = await getPackageJSON(app);

	if (packageJson) {
		await spawnSafe("bun", ["run", "dev"], { cwd: `app/${app}` });
	} else {
		const tailwindConfig = await getTailwindConfig(
			config.tailwindConfig || app,
		);
		process.env.TAILWIND_CONFIG = tailwindConfig;

		const watchPaths = await buildWatchPaths(app, cargoToml);
		const watchArgs = watchPaths.flatMap((p) => ["-w", p]);

		const featuresList = await getAppFeatures(
			cargoToml?.features,
			config.features,
		);
		const features =
			featuresList.length > 0 ? ` --features ${featuresList.join(",")}` : "";

		await spawnSafe("watchexec", [
			"-I",
			"-q",
			...watchArgs,
			"-r",
			`bun run build.script -l silent & cargo run -p ${app}${features}`,
		]);
	}
};

try {
	const config = await getConfig();

	await run(config);
} catch (error) {
	catchError(error);
}
