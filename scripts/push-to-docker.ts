import input from "@inquirer/input";
import type { SetRequired } from "type-fest";

import { type AppConfig, getConfig } from "./config.ts";
import { getCargoTOML, getDockerfile } from "./fs.ts";
import { catchError, spawnSafe } from "./process.ts";
import { getApp, getAppFeatures, getTailwindConfig } from "./prompts.ts";

const getImageName = async (defaultApp: string): Promise<string> => {
	const dockerOrg = await input({
		message: "Enter the docker org",
	});
	const imageName = await input({
		message: "Enter the image name",
	});
	const imageTag = await input({
		default: defaultApp,
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

export const pushToDocker = async (config: AppConfig): Promise<void> => {
	const app = await getApp(config.app);
	if (!app) {
		throw new Error("app not found");
	}

	const fullImageName = await getImageName(app);
	const appDockerFile = await getDockerfile(app);

	if (appDockerFile) {
		await runDockerBuild(fullImageName, {
			cwd: `app/${app}`,
		});
	} else {
		const buildArgs = await buildDockerArgs({ ...config, app });
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
