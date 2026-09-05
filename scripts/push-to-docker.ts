import input from "@inquirer/input";
import type { AppConfig } from "./config.ts";
import { getConfig } from "./config.ts";
import { getCargoTOML, getDockerfile } from "./fs.ts";
import { catchError, runCommand } from "./process.ts";
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
		const cmd = `cd app/${app} && docker buildx build --platform=linux/amd64,linux/arm64 -t ${fullImageName} .`;
		console.log("Running command:", cmd);
		runCommand(cmd);
	} else {
		let buildArgs = "";

		const cargoToml = await getCargoTOML(`app/${app}`);
		const featuresList = await getAppFeatures(cargoToml?.features);
		if (featuresList.length > 0) {
			buildArgs += ` --build-arg FEATURES="${featuresList.join(",")}"`;
		}

		const tailwindConfig = await getTailwindConfig(
			config.tailwindConfig || app,
		);
		buildArgs += ` --build-arg TAILWIND_CONFIG="${tailwindConfig}"`;

		const cmd = `docker buildx build --platform=linux/amd64,linux/arm64 --push --build-arg APP="${app}"${buildArgs} -t ${fullImageName} .`;
		console.log("Running command:", cmd);
		await runCommand(cmd);
	}
};

try {
	const config = await getConfig();

	await pushToDocker(config);
} catch (error) {
	catchError(error);
}
