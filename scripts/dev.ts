import type { TomlTable } from "smol-toml";
import {
	type AppConfig,
	catchError,
	getApp,
	getAppFeatures,
	getCargoTOML,
	getConfig,
	getDirectoryFolders,
	getPackageJSON,
	getTailwindConfig,
	runCommand,
} from "./util.js";

const libsDir = await getDirectoryFolders("../libs");

const buildWatchPaths = async (
	app: string,
	cargoToml: TomlTable | undefined,
): Promise<string> => {
	let watchApp = `-w public_script -w app/${app}`;

	if (!cargoToml) return watchApp;

	const watchPaths = new Set<string>();
	const cargoTomlDependencies = cargoToml.dependencies || {};

	// Process each lib that's a direct dependency
	for (const lib of libsDir) {
		if (
			typeof cargoTomlDependencies === "object" &&
			cargoTomlDependencies !== null &&
			lib in cargoTomlDependencies
		) {
			watchPaths.add(`-w libs/${lib}`);

			// Add lib's dependencies recursively
			const libCargoToml = await getCargoTOML(`libs/${lib}`);
			const libDeps = libCargoToml?.dependencies || {};

			for (const dep of Object.keys(libDeps)) {
				if (libsDir.includes(dep)) {
					watchPaths.add(`-w libs/${dep}`);
				}
			}
		}
	}

	for (const path of watchPaths) {
		watchApp += ` ${path}`;
	}

	return watchApp;
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
