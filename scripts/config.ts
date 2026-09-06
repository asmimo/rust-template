import select from "@inquirer/select";
import { Command } from "commander";

export const envs = ["development", "production"] as const;
export type Env = (typeof envs)[number];

const getEnv = async (env: Env = "development") => {
	if (envs.includes(env)) {
		return env;
	}

	const choices = envs.map((env) => ({ name: env, value: env }));

	return await select({
		message: "Choose an environment",
		choices,
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
