import input from "@inquirer/input";
import type { SetRequired } from "type-fest";

import { type AppConfig, getConfig } from "./config.ts";
import { getCargoTOML, getDockerfile } from "./fs.ts";
import { catchError, spawnSafe } from "./process.ts";
import { getApp, getAppFeatures, getTailwindConfig } from "./prompts.ts";

const getImageName = async (
	defaultOrg?: string,
	defaultName?: string,
	defaultTag?: string,
): Promise<string> => {
	const dockerOrg = await input({
		default: defaultOrg,
		message: "Enter the docker org",
	});
	const imageName = await input({
		default: defaultName,
		message: "Enter the image name",
	});
	const imageTag = await input({
		default: defaultTag,
		message: "Enter the image tag",
	});
	return `${dockerOrg}/${imageName}:${imageTag}`;
};

const buildDockerArgs = async (config: SetRequired<AppConfig, "app">): Promise<string[]> => {
	const { app } = config;
	const cargoToml = await getCargoTOML(`app/${app}`);
	config.features = await getAppFeatures(cargoToml?.features, config.features);
	config.tailwindConfig = await getTailwindConfig(config.tailwindConfig || app);
	const { features, tailwindConfig } = config;

	return [
		"--build-arg",
		`APP=${app}`,
		"--build-arg",
		`TAILWIND_CONFIG=${tailwindConfig}`,
		...(features.length > 0 ? ["--build-arg", `FEATURES=${features.join(",")}`] : []),
	];
};

const runDockerBuild = async (
	fullImageName: string,
	options: { cwd?: string; buildArgs?: string[] } = {},
): Promise<void> => {
	const { buildArgs, ...restOptions } = options;
	await spawnSafe(
		"docker",
		[
			"buildx",
			"build",
			"--platform",
			"linux/amd64,linux/arm64",
			"--push",
			"-t",
			fullImageName,
			...(buildArgs ?? []),
			".",
		],
		restOptions,
	);
};

export const pushToDocker = async (config: AppConfig): Promise<void> => {
	if (config.env === "development") {
		throw new Error("This script is not supported in development mode.");
	}

	const app = await getApp(config.app);
	const appDockerFile = await getDockerfile(app);

	if (appDockerFile) {
		const fullImageName = await getImageName(config.org, config.image || app, config.tag);
		await runDockerBuild(fullImageName, {
			cwd: `app/${app}`,
		});
	} else {
		const appConfig = { ...config, app };
		const buildArgs = await buildDockerArgs(appConfig);
		const fullImageName = await getImageName(config.org, config.image || app, config.tag);

		console.inspect(appConfig);
		await runDockerBuild(fullImageName, {
			buildArgs,
		});
	}
};

try {
	const config = await getConfig();

	await pushToDocker(config);
} catch (error) {
	catchError(error);
}
