import cp from "node:child_process";
import fs from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";

import checkbox from "@inquirer/checkbox";
import select from "@inquirer/select";
import { Command } from "commander";
import { parse, type TomlValue } from "smol-toml";

export const __filename = fileURLToPath(import.meta.url);
export const __dirname = dirname(__filename);

export const exec = promisify(cp.exec);
// export const spawn = promisify(cp.spawn);

const envs = ["development", "production"] as const;
type Env = (typeof envs)[number];
const getEnv = async (env: Env = "development") => {
	if (envs.includes(env)) {
		return env;
	}

	const choices = envs.map((env) => ({
		name: env,
		value: env,
	}));

	return await select({
		message: "Choose an environment",
		choices,
	});
};

export const getDirectoryFolders = async (
	dir: string,
	ignoreDotFiles: boolean = true,
) => {
	const resolved = join(__dirname, dir);

	return fs.readdir(resolved).then((f) => {
		if (ignoreDotFiles) {
			return f.filter((f) => !f.startsWith("."));
		}
		return f;
	});
};

const appRootDir = "../app";
export const getApp = async (name?: string) => {
	const apps = await getDirectoryFolders(appRootDir).then((f) =>
		f.filter((f) => !f.startsWith(".") && !f.startsWith("admin")).sort(),
	);

	if (name && apps.includes(name)) {
		return name;
	}

	if (name) {
		console.log(`App '${name}' is not valid.`);
	}

	const choices = apps.map((app) => ({
		name: app,
		value: app,
	}));

	return await select({
		message: "Choose an app",
		choices,
	});
};

export const getAppFeatures = async (features: TomlValue) => {
	if (typeof features === "object" && features !== null) {
		const choices = Object.keys(features)
			.filter((feature) => feature !== "default")
			.map((feature) => ({
				name: feature,
				value: feature,
			}));

		if (choices.length > 0) {
			return await checkbox({
				message: "Choose features",
				choices,
			});
		}
	}

	return [];
};

const tailwindConfigRootDir = "../styles";
export const getTailwindConfig = async (name?: string) => {
	const tailwindConfigsV4 = await getDirectoryFolders(
		tailwindConfigRootDir,
	).then((f) =>
		f.filter((f) => !f.startsWith(".") && !f.startsWith("base")).sort(),
	);
	const tailwindConfigs = ["SKIP", ...tailwindConfigsV4] as const;

	const matchedConfig =
		name &&
		tailwindConfigs.find((f) => typeof f === "string" && f.includes(name));
	if (matchedConfig) {
		return matchedConfig;
	}

	if (name) {
		console.log(`Tailwind config '${name}' is not valid.`);
	}

	return await select<(typeof tailwindConfigs)[number]>({
		message: "Choose a tailwind config",
		choices: tailwindConfigs,
		default: "SKIP",
	});
};

export interface AppConfig {
	env: Env;
	app?: string;
	tailwindConfig?: string;
	features?: string;
}
export const getConfig = async (): Promise<AppConfig> => {
	const program = new Command();
	program
		.option("-e, --env <env>", "The name of the environment")
		.option("-a, --app <app>", "The name of the app")
		.option(
			"--tailwind-config <tailwindConfig>",
			"The name of the tailwind config",
		)
		.option("-f, --features <features>", "The name of the features")
		.parse(process.argv);

	const options = program.opts<AppConfig>();

	const env = await getEnv(options.env || envs[0]);

	return { ...options, env };
};

export const getPackageJSON = async (
	app: string,
): Promise<unknown | undefined> => {
	const packageJsonPath = join(__dirname, "../app", app, "package.json");

	try {
		await fs.access(packageJsonPath);
	} catch {
		return undefined;
	}
	const packageJsonContent = await fs.readFile(packageJsonPath, "utf-8");
	return JSON.parse(packageJsonContent);
};

export const getCargoTOML = async (path: string) => {
	const tomlPath = join(__dirname, "../", path, "Cargo.toml");

	try {
		await fs.access(tomlPath);
	} catch {
		return undefined;
	}

	const tomlContent = await fs.readFile(tomlPath, "utf-8");
	return parse(tomlContent);
};

export const getDockerfile = async (app: string) => {
	const dockerfilePath = join(__dirname, "../app", app, "Dockerfile");

	try {
		await fs.access(dockerfilePath);
	} catch {
		return undefined;
	}

	const dockerfileContent = await fs.readFile(dockerfilePath, "utf-8");
	return dockerfileContent;
};

export const runCommand = async (cmd: string, args: string[] = []) => {
	return new Promise<void>((resolve, reject) => {
		const child = cp.spawn(cmd, args, {
			stdio: "inherit",
			shell: true,
		});

		child.on("close", (code) => {
			if (code === 0) {
				resolve();
			} else {
				reject(new Error(`Command failed with exit code ${code}`));
			}
		});

		child.on("error", (error) => {
			reject(error);
		});
	});
};

export const catchError = (error: unknown) => {
	if (error instanceof Error) {
		if (error.name === "ExitPromptError") {
			console.log("\nGracefully shutting down from SIGINT (Ctrl-C)");
		} else {
			console.error("An error occurred:", error.message);
			if (error.stack) {
				console.error(error.stack);
			}
			process.exit(1);
		}
	} else {
		console.error("An unexpected error occurred:", error);
		process.exit(1);
	}
};
