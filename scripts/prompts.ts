import checkbox from "@inquirer/checkbox";
import select from "@inquirer/select";
import type { TomlValue } from "smol-toml";

import { getDirectoryFolders } from "./fs.ts";

const appRootDir = "../app";
export const getApp = async (name?: string): Promise<string> => {
	const apps = await getDirectoryFolders(appRootDir);

	if (name !== undefined && apps.includes(name)) {
		return name;
	}

	if (name !== undefined) {
		console.log(`App '${name}' is not valid.`);
	}

	const choices = apps.map((app) => ({ name: app, value: app }));

	const app = await select({
		choices,
		message: "Choose an app",
	});

	return app;
};

const validateFeatures = (
	features: Readonly<string[]>,
	validFeatures: Readonly<string[]>,
): string[] => {
	const invalid = features.filter((feature) => !validFeatures.includes(feature));
	if (invalid.length > 0) {
		console.warn(`Unknown features ignored: ${invalid.join(", ")}`);
	}
	return features.filter((feature) => validFeatures.includes(feature));
};

export const getAppFeatures = async (
	// oxlint-disable-next-line @typescript/prefer-readonly-parameter-types
	features?: TomlValue,
	configFeatures?: string | readonly string[],
): Promise<string[]> => {
	if (features !== undefined && typeof features === "object" && features !== null) {
		const tomlFeatures = Object.keys(features).filter((feature) => feature !== "default");

		let validFeaturesList: string[] = [];
		if (configFeatures !== undefined) {
			const requested =
				typeof configFeatures === "string"
					? configFeatures.split(",").map((feature) => feature.trim())
					: configFeatures;
			validFeaturesList = validateFeatures(requested, tomlFeatures);
		}

		if (validFeaturesList.length > 0) {
			return validFeaturesList;
		} else if (tomlFeatures.length > 0) {
			const selectedFeatures = await checkbox({
				choices: tomlFeatures.map((feature) => ({
					checked: validFeaturesList.includes(feature),
					name: feature,
					value: feature,
				})),
				message: "Choose features",
			});

			return selectedFeatures;
		}
	}

	return [];
};
