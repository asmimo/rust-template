import select from "@inquirer/select";
import { Command, Option } from "commander";
// oxlint-disable-next-line id-length
import * as v from "valibot";

import { getTOML } from "./fs";

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

const FeatureSchema = v.union([v.string(), v.array(v.string())]);
const AppConfigSchema = v.object({
	app: v.optional(v.string()),
	env: v.optional(v.picklist(envs), "development"),
	features: v.optional(FeatureSchema),
	tailwindConfig: v.optional(v.string()),
});
export type AppConfig = v.InferOutput<typeof AppConfigSchema>;

const DockerConfigSchema = v.object({
	image: v.string(),
	org: v.string(),
	tag: v.string(),
});
export type DockerConfig = v.InferOutput<typeof DockerConfigSchema>;

const DevConfigSchema = v.object({
	app: v.object({
		...AppConfigSchema.entries,
		env: v.literal("development"),
	}),
	docker: v.optional(DockerConfigSchema),
});

const ProdConfigSchema = v.object({
	app: v.object({
		...AppConfigSchema.entries,
		env: v.literal("production"),
	}),
	docker: DockerConfigSchema,
});
const ConfigSchema = v.union([DevConfigSchema, ProdConfigSchema]);

export type Config = v.InferOutput<typeof ConfigSchema>;

const getConfigFromToml = async (env: Env): Promise<Config | undefined> => {
	const tomlConfig = await getTOML(`../config.toml`);
	let config: Config | undefined = undefined;

	if (tomlConfig) {
		if (env === "production") {
			let appConfig: AppConfig | undefined = undefined;
			let dockerConfig: DockerConfig | undefined = undefined;
			if (typeof tomlConfig.docker === "object" && tomlConfig.docker !== null) {
				if ("app" in tomlConfig.docker) {
					appConfig = v.parse(AppConfigSchema, tomlConfig.docker.app);
				} else {
					return;
				}
				dockerConfig = v.parse(DockerConfigSchema, tomlConfig.docker);
			} else {
				return;
			}

			config = { app: { ...appConfig, env: "production" }, docker: dockerConfig };
		} else {
			const appConfig = v.parse(AppConfigSchema, tomlConfig.dev);
			config = { app: { ...appConfig, env: "development" } };
		}
	}

	return config;
};

export const getConfig = async (): Promise<Config> => {
	const program = new Command();
	program
		.option("-a, --app <app>", "The name of the app")
		.addOption(new Option("-e, --env <env>", "The name of the environment").choices(envs))
		.option("-f, --features <features>", "The name of the features")
		.option("--tailwind-config <tailwindConfig>", "The name of the tailwind config")
		.parse(process.argv);

	const options = program.opts<AppConfig>();
	const env = await getEnv(options.env);
	const configFromToml = await getConfigFromToml(env);

	if (configFromToml) {
		configFromToml.app = { ...configFromToml.app, ...options };

		return configFromToml;
	}

	if (env === "production") {
		return {
			app: { ...options, env: "production" },
			docker: {
				image: "",
				org: "",
				tag: "",
			},
		};
	}

	return { app: { ...options, env: "development" } };
};
