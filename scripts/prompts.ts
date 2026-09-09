import checkbox from "@inquirer/checkbox";
import select from "@inquirer/select";
import type { TomlValue } from "smol-toml";

import { getDirectoryFolders } from "./fs.ts";

const appRootDir = "../app";
export const getApp = async (name?: string): Promise<string> => {
	const apps = await getDirectoryFolders(appRootDir);

	if (name && apps.includes(name)) {
		return name;
	}

	if (name) {
		console.log(`App '${name}' is not valid.`);
	}

	const choices = apps.map((app) => ({ name: app, value: app }));

	return await select({
		choices,
		message: "Choose an app",
	});
};

const validateFeatures = (features: string[], validFeatures: string[]): string[] => {
	const invalid = features.filter((feature) => !validFeatures.includes(feature));
	if (invalid.length > 0) {
		console.warn(`Unknown features ignored: ${invalid.join(", ")}`);
	}
	return features.filter((feature) => validFeatures.includes(feature));
};

export const getAppFeatures = async (
	features?: TomlValue,
	configFeatures?: string | string[],
): Promise<string[]> => {
	if (features && typeof features === "object" && features !== null) {
		const tomlFeatures = Object.keys(features).filter((feature) => feature !== "default");

		let validFeaturesList: string[] = [];
		if (configFeatures) {
			const requested: string[] =
				typeof configFeatures === "string"
					? configFeatures.split(",").map((feature) => feature.trim())
					: configFeatures;
			validFeaturesList = validateFeatures(requested, tomlFeatures);
		}

		if (validFeaturesList.length > 0) {
			return validFeaturesList;
		} else if (tomlFeatures.length > 0) {
			return await checkbox({
				choices: tomlFeatures.map((feature) => ({
					checked: validFeaturesList.includes(feature),
					name: feature,
					value: feature,
				})),
				message: "Choose features",
			});
		}
	}

	return [];
};

const tailwindConfigRootDir = "../styles";
export const getTailwindConfig = async (name?: string): Promise<string> => {
	const tailwindConfigs = [
		"SKIP",
		...(await getDirectoryFolders(tailwindConfigRootDir).then((files) =>
			files.filter((file) => !file.startsWith("base")),
		)),
	] as const;

	const matchedConfig =
		name && tailwindConfigs.find((config) => typeof config === "string" && config.includes(name));
	if (matchedConfig) {
		return matchedConfig;
	}

	return await select({
		choices: tailwindConfigs,
		default: "SKIP",
		message: "Choose a tailwind config",
	});
};
