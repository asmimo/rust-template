import input from "@inquirer/input";
import type { SetRequired } from "type-fest";

import { type AppConfig, type Config, getConfig } from "./config.ts";
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
	const featuresList = await getAppFeatures(cargoToml?.features, config.features);
	const tailwindConfig = await getTailwindConfig(config.tailwindConfig || app);

	return [
		"--build-arg",
		`APP=${app}`,
		"--build-arg",
		`TAILWIND_CONFIG=${tailwindConfig}`,
		...(featuresList.length > 0 ? ["--build-arg", `FEATURES=${featuresList.join(",")}`] : []),
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

export const pushToDocker = async (config: Config): Promise<void> => {
	if (config.app.env === "development") {
		throw new Error("This script is not supported in development mode.");
	}

	const app = await getApp(config.app.app);

	const fullImageName = await getImageName(
		config.docker?.org,
		config.docker?.image,
		config.docker?.tag,
	);
	const appDockerFile = await getDockerfile(app);

	if (appDockerFile) {
		await runDockerBuild(fullImageName, {
			cwd: `app/${app}`,
		});
	} else {
		const buildArgs = await buildDockerArgs({ ...config.app, app });
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
