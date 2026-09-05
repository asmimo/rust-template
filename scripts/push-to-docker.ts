import input from "@inquirer/input";
import type { AppConfig } from "./config.ts";
import { getConfig } from "./config.ts";
import { getCargoTOML, getDockerfile } from "./fs.ts";
import { catchError, spawnSafe } from "./process.ts";
import { getApp, getAppFeatures, getTailwindConfig } from "./prompts.ts";

export const pushToDocker = async (config: AppConfig) => {
	const app = await getApp(config.app);
	if (!app) {
		throw new Error("app not found");
	}

	const dockerOrg = await input({
		message: "Enter the docker org",
	});

	const imageName = await input({
		message: "Enter the image name",
	});

	const imageTag = await input({
		message: "Enter the image tag",
		default: app,
	});
	const fullImageName = `${dockerOrg}/${imageName}:${imageTag}`;

	const appDockerFile = await getDockerfile(app);
	if (appDockerFile) {
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
				".",
			],
			{ cwd: `app/${app}` },
		);
	} else {
		const cargoToml = await getCargoTOML(`app/${app}`);
		const featuresList = await getAppFeatures(cargoToml?.features);

		const tailwindConfig = await getTailwindConfig(
			config.tailwindConfig || app,
		);

		const buildArgs = [
			"--build-arg",
			`APP=${app}`,
			"--build-arg",
			`TAILWIND_CONFIG=${tailwindConfig}`,
			...(featuresList.length > 0
				? ["--build-arg", `FEATURES=${featuresList.join(",")}`]
				: []),
		];

		await spawnSafe("docker", [
			"buildx",
			"build",
			"--platform",
			"linux/amd64,linux/arm64",
			"--push",
			...buildArgs,
			"-t",
			fullImageName,
			".",
		]);
	}
};

try {
	const config = await getConfig();

	await pushToDocker(config);
} catch (error) {
	catchError(error);
}
