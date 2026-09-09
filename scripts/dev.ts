import type { TomlTable } from "smol-toml";
import type { SetRequired } from "type-fest";

import { type AppConfig, getConfig } from "./config.ts";
import { getCargoTOML, getDirectoryFolders, getPackageJSON } from "./fs.ts";
import { catchError, spawnSafe } from "./process.ts";
import { getApp, getAppFeatures, getTailwindConfig } from "./prompts.ts";

const libsDir = await getDirectoryFolders("../libs");

const buildWatchPaths = async (
	app: string,
	cargoToml: TomlTable | undefined,
): Promise<string[]> => {
	const paths = ["public_script", `app/${app}`];

	if (!cargoToml) {
		return paths;
	}

	const appDeps = cargoToml.dependencies;
	if (typeof appDeps !== "object" || appDeps === null) {
		return paths;
	}

	const directLibDeps = libsDir.filter((lib) => lib in appDeps);

	const libPaths = await Promise.all(
		directLibDeps.map(async (lib) => {
			const libToml = await getCargoTOML(`libs/${lib}`);
			const libDeps = libToml?.dependencies;
			const transitiveDeps =
				typeof libDeps === "object" && libDeps !== null
					? Object.keys(libDeps).filter((dep) => libsDir.includes(dep))
					: [];
			return [lib, ...transitiveDeps].map((dep) => `libs/${dep}`);
		}),
	);

	return [...paths, ...new Set(libPaths.flat())];
};

const runApp = async (config: SetRequired<AppConfig, "app" | "tailwindConfig">): Promise<void> => {
	const { app } = config;
	const cargoToml = await getCargoTOML(`app/${app}`);

	const watchPaths = await buildWatchPaths(app, cargoToml);
	const watchArgs = watchPaths.flatMap((path) => ["-w", path]);

	config.features = await getAppFeatures(cargoToml?.features, config.features);
	const { features: configFeatures } = config;
	const features = configFeatures.length > 0 ? ` --features ${configFeatures.join(",")}` : "";

	console.inspect(config);
	await spawnSafe("watchexec", [
		"-I",
		"-q",
		...watchArgs,
		"-r",
		`bun run build:script -l silent & cargo run -p ${app}${features}`,
	]);
};

export const run = async (config: AppConfig): Promise<void> => {
	if (config.env === "production") {
		throw new Error("This script is not supported in production mode.");
	}
	const app = await getApp(config.app);

	const packageJson = await getPackageJSON(app);

	if (packageJson) {
		await spawnSafe("bun", ["run", "dev"], { cwd: `app/${app}` });
	} else {
		const tailwindConfig = await getTailwindConfig(config.tailwindConfig || app);
		process.env.TAILWIND_CONFIG = tailwindConfig;

		await runApp({ ...config, app, tailwindConfig });
	}
};

try {
	const config = await getConfig();

	await run(config);
} catch (error) {
	catchError(error);
}
