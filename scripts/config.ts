import { inspect } from "node:util";

import select from "@inquirer/select";
import { Command, Option } from "commander";
// oxlint-disable-next-line id-length
import * as v from "valibot";

import { getTOML } from "./fs";

const namedOption = (flags: string, description: string, name: string): Option =>
	Object.assign(new Option(flags, description), { attributeName: () => name });

export const envs = ["development", "production"] as const;
export type Env = (typeof envs)[number];

const getEnv = async (defaultEnv?: Env): Promise<Env> => {
	if (defaultEnv && envs.includes(defaultEnv)) {
		return defaultEnv;
	}

	const choices = envs.map((env) => ({ name: env, value: env }));

	return await select({
		choices,
		message: "Choose an environment",
	});
};

const DockerConfigSchema = v.object({
	image: v.optional(v.string()),
	org: v.optional(v.string()),
	tag: v.optional(v.string()),
});
export type DockerConfig = v.InferOutput<typeof DockerConfigSchema>;

const FeatureSchema = v.union([v.string(), v.array(v.string())]);
const AppConfigSchema = v.object({
	app: v.optional(v.string()),
	env: v.optional(v.picklist(envs), "development"),
	features: v.optional(FeatureSchema),
	tailwindConfig: v.optional(v.string()),
	...DockerConfigSchema.entries,
});
export type AppConfig = v.InferOutput<typeof AppConfigSchema>;

const getConfigFromToml = async (env: Env): Promise<AppConfig | undefined> => {
	const tomlConfig = await getTOML(`../config.toml`);
	let config: AppConfig | undefined = undefined;

	if (tomlConfig) {
		config =
			env === "production"
				? v.parse(AppConfigSchema, tomlConfig.docker)
				: (config = v.parse(AppConfigSchema, tomlConfig.dev));
	}

	return config;
};

export const getConfig = async (): Promise<AppConfig> => {
	const program = new Command();
	program
		.option("-a, --app <app>", "The name of the app")
		.addOption(new Option("-e, --env <env>", "The name of the environment").choices(envs))
		.option("-f, --features <features>", "The name of the features")
		.option("--tailwind-config <tailwindConfig>", "The name of the tailwind config")
		.addOption(namedOption("--docker-org <dockerOrg>", "The name of the docker org", "org"))
		.addOption(namedOption("--docker-tag <dockerTag>", "The name of the docker tag", "tag"))
		.addOption(namedOption("--docker-image <dockerImage>", "The name of the docker image", "image"))
		.parse(process.argv);

	const options = program.opts<AppConfig>();
	const env = await getEnv(options.env);

	const configFromToml = (await getConfigFromToml(env)) || {};
	return { ...configFromToml, ...options };
};

console.inspect = (...data: unknown[]): void => {
	console.log(inspect(data, { breakLength: Infinity, compact: true }));
};
