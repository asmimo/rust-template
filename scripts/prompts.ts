import checkbox from "@inquirer/checkbox";
import select from "@inquirer/select";
import type { TomlValue } from "smol-toml";
import { getDirectoryFolders } from "./fs.ts";

const appRootDir = "../app";
export const getApp = async (name?: string) => {
	const apps = await getDirectoryFolders(appRootDir);

	if (name && apps.includes(name)) {
		return name;
	}

	if (name) {
		console.log(`App '${name}' is not valid.`);
	}

	const choices = apps.map((app) => ({ name: app, value: app }));

	return await select({
		message: "Choose an app",
		choices,
	});
};

export const getAppFeatures = async (
	features?: TomlValue,
	configFeatures?: string,
) => {
	if (features && typeof features === "object" && features !== null) {
		const tomlFeatures = Object.keys(features);

		let validFeaturesList: string[] = [];
		if (configFeatures) {
			const requested = configFeatures.split(",");
			const invalid = requested.filter((f) => !tomlFeatures.includes(f));
			if (invalid.length > 0) {
				console.warn(`Unknown features ignored: ${invalid.join(", ")}`);
			}
			validFeaturesList = requested.filter((f) => tomlFeatures.includes(f));
		}

		if (validFeaturesList.length > 0) {
			return validFeaturesList;
		}

		if (tomlFeatures.length > 0) {
			return await checkbox({
				message: "Choose features",
				choices: tomlFeatures
					.filter((feature) => feature !== "default")
					.map((feature) => ({
						name: feature,
						value: feature,
						checked: validFeaturesList.includes(feature),
					})),
			});
		}
	}

	return [];
};

const tailwindConfigRootDir = "../styles";
export const getTailwindConfig = async (name?: string) => {
	const tailwindConfigs = [
		"SKIP",
		...(await getDirectoryFolders(tailwindConfigRootDir).then((f) =>
			f.filter((f) => !f.startsWith("base")),
		)),
	] as const;

	const matchedConfig =
		name &&
		tailwindConfigs.find((f) => typeof f === "string" && f.includes(name));
	if (matchedConfig) {
		return matchedConfig;
	}

	return await select<(typeof tailwindConfigs)[number]>({
		message: "Choose a tailwind config",
		choices: tailwindConfigs,
		default: "SKIP",
	});
};
