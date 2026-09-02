import input from "@inquirer/input";
import {
	type AppConfig,
	catchError,
	getApp,
	getAppFeatures,
	getCargoTOML,
	getConfig,
	getDockerfile,
	getTailwindConfig,
	runCommand,
} from "./util.ts";

const VITE_ENV_KEYS = ["VITE_STRIPE_PUBLIC_KEY", "VITE_RETURN_URL"];

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
		const buildArgs = VITE_ENV_KEYS.map((key) =>
			process.env[key] ? ` --build-arg ${key}="${process.env[key]}"` : "",
		).join("");

		console.log("Build args:", buildArgs);

		const cmd = `cd app/${app} && docker build --platform=linux/amd64${buildArgs} -t ${fullImageName} .`;
		runCommand(cmd);
	} else {
		let buildArgs = "";
		const tailwindConfig = await getTailwindConfig(
			config.tailwindConfig || app,
		);
		buildArgs += ` --build-arg TAILWIND_CONFIG="${tailwindConfig}"`;

		const cargoToml = await getCargoTOML(`app/${app}`);
		const featuresList = await getAppFeatures(cargoToml?.features || {});

		if (featuresList.length > 0) {
			buildArgs += ` --build-arg FEATURES="${featuresList.join(",")}"`;
		}

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
