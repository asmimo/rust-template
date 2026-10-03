import fs from "node:fs/promises";
import path from "node:path";

import { type TomlTable, parse } from "smol-toml";

export const getDirectoryFolders = async (
	dir: string,
	ignoreDotFiles = true,
): Promise<string[]> => {
	const resolved = path.join(import.meta.dirname, dir);

	const dirFiles = await fs.readdir(resolved).then((files: readonly string[]) => {
		const sortedFiles = files.toSorted();
		if (ignoreDotFiles) {
			return sortedFiles.filter((file) => !file.startsWith("."));
		}
		return sortedFiles;
	});

	return dirFiles;
};

export const getTOML = async (dir: string): Promise<TomlTable | undefined> => {
	const tomlPath = path.join(import.meta.dirname, dir);

	try {
		await fs.access(tomlPath);
	} catch {
		return undefined;
	}

	const tomlContent = await fs.readFile(tomlPath, "utf8");

	return parse(tomlContent);
};

export const getCargoTOML = async (dir: string): Promise<TomlTable | undefined> => {
	const toml = await getTOML(`../${dir}/Cargo.toml`);
	return toml;
};

export const getDockerfile = async (app: string): Promise<string | undefined> => {
	const dockerfilePath = path.join(import.meta.dirname, "../app", app, "Dockerfile");

	try {
		await fs.access(dockerfilePath);
	} catch {
		return undefined;
	}

	return fs.readFile(dockerfilePath, "utf8");
};
