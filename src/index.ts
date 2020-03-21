import fs from "fs-extra";
import path from "path";
import execa from "execa";
import toml from "@iarna/toml";
import {
	glob,
	createLambda,
	debug,
	download,
	FileRef,
	FileFsRef,
	runShellScript,
	BuildOptions,
	PrepareCacheOptions,
	DownloadedFiles,
	Lambda
} from "@now/build-utils"; // eslint-disable-line import/no-extraneous-dependencies
import { installRustAndFriends } from './install-rust';

interface PackageManifest {
	targets: { kind: string; name: string }[];
}

interface CargoConfig {
	env: Record<string, any>;
	cwd: string;
}

interface CargoToml extends toml.JsonMap {
	package: toml.JsonMap;
	dependencies: toml.JsonMap;
}

const codegenFlags = [
	"-C",
	"target-cpu=ivybridge",
	"-C",
	"target-feature=-aes,-avx,+fxsr,-popcnt,+sse,+sse2,-sse3,-sse4.1,-sse4.2,-ssse3,-xsave,-xsaveopt"
];

export const version = 3;
const builderDebug = process.env.NOW_BUILDER_DEBUG ? true : false;

async function inferCargoBinaries(config: CargoConfig) {
	try {
		const { stdout: manifestStr } = await execa(
			"cargo",
			["read-manifest"],
			config
		);

		const { targets } = JSON.parse(manifestStr) as PackageManifest;

		return targets
			.filter(({ kind }) => kind.includes("bin"))
			.map(({ name }) => name);
	} catch (err) {
		console.error("failed to run `cargo read-manifest`");
		throw err;
	}
}

async function parseTOMLStream(stream: NodeJS.ReadableStream) {
	return toml.parse.stream(stream);
}

async function buildWholeProject(
	{ entrypoint, config }: BuildOptions,
	downloadedFiles: DownloadedFiles,
	extraFiles: DownloadedFiles,
	rustEnv: Record<string, string>
) {
	const entrypointDirname = path.dirname(downloadedFiles[entrypoint].fsPath);
	debug("Running `cargo build`...");
	try {
		await execa(
			"cargo",
			["build", "--verbose"].concat(config.debug ? [] : ["--release"]),
			{
				env: rustEnv,
				cwd: entrypointDirname,
				stdio: "inherit"
			}
		);
	} catch (err) {
		console.error("failed to `cargo build`");
		throw err;
	}

	const targetPath = path.join(
		entrypointDirname,
		"target",
		config.debug ? "debug" : "release"
	);
	const binaries = await inferCargoBinaries({
		env: rustEnv,
		cwd: entrypointDirname
	});

	const lambdas: Record<string, Lambda> = {};
	const lambdaPath = path.dirname(entrypoint);
	await Promise.all(
		binaries.map(async binary => {
			const fsPath = path.join(targetPath, binary);
			const lambda = await createLambda({
				files: {
					...extraFiles,
					bootstrap: new FileFsRef({ mode: 0o755, fsPath })
				},
				handler: "bootstrap",
				runtime: "provided"
			});

			lambdas[path.join(lambdaPath, binary)] = lambda;
		})
	);

	return lambdas;
}

async function gatherExtraFiles(
	globMatcher: string | string[] | undefined,
	entrypoint: string
) {
	if (!globMatcher) return {};

	debug("Gathering extra files for the fs...");

	const entryDir = path.dirname(entrypoint);

	if (Array.isArray(globMatcher)) {
		const allMatches = await Promise.all(
			globMatcher.map(pattern => glob(pattern, entryDir))
		);

		return allMatches.reduce((acc, matches) => ({ ...acc, ...matches }), {});
	}

	return glob(globMatcher, entryDir);
}

async function runUserScripts(entrypoint: string) {
	const entryDir = path.dirname(entrypoint);
	const buildScriptPath = path.join(entryDir, "build.sh");
	const buildScriptExists = await fs.pathExists(buildScriptPath);

	if (buildScriptExists) {
		debug("Running `build.sh`...");
		await runShellScript(buildScriptPath);
	}
}

async function cargoLocateProject(config: CargoConfig) {
	try {
		const { stdout: projectDescriptionStr } = await execa(
			"cargo",
			["locate-project"],
			config
		);
		const projectDescription = JSON.parse(projectDescriptionStr);
		if (projectDescription != null && projectDescription.root != null) {
			return projectDescription.root;
		}
	} catch (e) {
		if (!/could not find/g.test(e.stderr)) {
			console.error("Couldn't run `cargo locate-project`");
			throw e;
		}
	}

	return null;
}

async function buildSingleFile(
	{ entrypoint }: BuildOptions,
	downloadedFiles: DownloadedFiles,
	extraFiles: DownloadedFiles,
	rustEnv: Record<string, string>
) {
	debug("Building single file");
	const entrypointPath = downloadedFiles[entrypoint].fsPath;
	const entrypointDirname = path.dirname(entrypointPath);

	// Find a Cargo.toml file or TODO: create one
	const cargoTomlFile = await cargoLocateProject({
		env: rustEnv,
		cwd: entrypointDirname
	});

	// TODO: we're assuming there's a Cargo.toml file. We need to create one
	// otherwise
	let cargoToml: CargoToml;
	try {
		cargoToml = (await parseTOMLStream(
			fs.createReadStream(cargoTomlFile)
		)) as CargoToml;
	} catch (err) {
		console.error("Failed to parse TOML from entrypoint:", entrypoint);
		throw err;
	}

	const binName = path
		.basename(entrypointPath)
		.replace(path.extname(entrypointPath), "");
	const { package: pkg, dependencies } = cargoToml;
	// default to latest now_lambda
	dependencies.now_lambda = "*";
	const tomlToWrite = toml.stringify({
		package: pkg,
		dependencies,
		bin: [
			{
				name: binName,
				path: entrypointPath
			}
		]
	});
	debug("Writing following toml to file:", tomlToWrite);

	// Overwrite the Cargo.toml file with one that includes the `now_lambda`
	// dependency and our binary. `dependencies` is a map so we don't run the
	// risk of having 2 `now_lambda`s in there.
	await fs.writeFile(cargoTomlFile, tomlToWrite);

	debug("Running `cargo build`...");
	try {
		await execa(
			"cargo",
			["build", "--bin", binName].concat(
				builderDebug ? ["--verbose"] : ["--quiet", "--release"]
			),
			{
				env: rustEnv,
				cwd: entrypointDirname,
				stdio: "inherit"
			}
		);
	} catch (err) {
		console.error("failed to `cargo build`");
		throw err;
	}

	const bin = path.join(
		path.dirname(cargoTomlFile),
		"target",
		builderDebug ? "debug" : "release",
		binName
	);

	const lambda = await createLambda({
		files: {
			...extraFiles,
			bootstrap: new FileFsRef({ mode: 0o755, fsPath: bin })
		},
		handler: "bootstrap",
		runtime: "provided"
	});

	if (version === 3) {
		return { output: lambda };
	}

	return {
		[entrypoint]: lambda
	};
}

export async function build(opts: BuildOptions) {
	await installRustAndFriends();

	const { files, entrypoint, workPath, config, meta = {} } = opts;
	debug("Downloading files");
	const downloadedFiles = await download(files, workPath, meta);
	const entryPath = downloadedFiles[entrypoint].fsPath;

	const { PATH, HOME } = process.env;
	const rustEnv: Record<string, string> = {
		...process.env,
		PATH: `${path.join(HOME!, ".cargo/bin")}:${PATH}`,
		RUSTFLAGS: [process.env.RUSTFLAGS, ...codegenFlags]
			.filter(Boolean)
			.join(" ")
	};

	await runUserScripts(entryPath);
	const extraFiles = await gatherExtraFiles(config.includeFiles, entryPath);

	if (path.extname(entrypoint) === ".toml" && version !== 3) {
		return buildWholeProject(opts, downloadedFiles, extraFiles, rustEnv);
	}
	return buildSingleFile(opts, downloadedFiles, extraFiles, rustEnv);
}

export async function prepareCache({
	cachePath,
	entrypoint,
	workPath
}: PrepareCacheOptions) {
	debug("Preparing cache...");

	let targetFolderDir: string;

	if (path.extname(entrypoint) === ".toml") {
		targetFolderDir = path.dirname(path.join(workPath, entrypoint));
	} else {
		const { PATH, HOME } = process.env;
		const rustEnv: Record<string, string> = {
			...process.env,
			PATH: `${path.join(HOME!, ".cargo/bin")}:${PATH}`,
			RUSTFLAGS: [process.env.RUSTFLAGS, ...codegenFlags]
				.filter(Boolean)
				.join(" ")
		};
		const entrypointDirname = path.dirname(path.join(workPath, entrypoint));
		const cargoTomlFile = await cargoLocateProject({
			env: rustEnv,
			cwd: entrypointDirname
		});

		if (cargoTomlFile != null) {
			targetFolderDir = path.dirname(cargoTomlFile);
		} else {
			// `Cargo.toml` doesn't exist, in `build` we put it in the same
			// path as the entrypoint.
			targetFolderDir = path.dirname(path.join(workPath, entrypoint));
		}
	}

	const cacheEntrypointDirname = path.join(
		cachePath,
		path.relative(workPath, targetFolderDir)
	);

	// Remove the target folder to avoid 'directory already exists' errors
	fs.removeSync(path.join(cacheEntrypointDirname, "target"));
	fs.mkdirpSync(cacheEntrypointDirname);
	// Move the target folder to the cache location
	fs.renameSync(
		path.join(targetFolderDir, "target"),
		path.join(cacheEntrypointDirname, "target")
	);

	const cacheFiles = await glob("**/**", cachePath);

	// eslint-disable-next-line no-restricted-syntax
	for (const f of Object.keys(cacheFiles)) {
		const accept =
			/(?:^|\/)target\/release\/\.fingerprint\//.test(f) ||
			/(?:^|\/)target\/release\/build\//.test(f) ||
			/(?:^|\/)target\/release\/deps\//.test(f) ||
			/(?:^|\/)target\/debug\/\.fingerprint\//.test(f) ||
			/(?:^|\/)target\/debug\/build\//.test(f) ||
			/(?:^|\/)target\/debug\/deps\//.test(f);
		if (!accept) {
			delete cacheFiles[f];
		}
	}

	return cacheFiles;
}

function findCargoToml(
	files: BuildOptions["files"],
	entrypoint: BuildOptions["entrypoint"]
) {
	let currentPath = path.dirname(entrypoint);
	let cargoTomlPath;

	// eslint-disable-next-line no-constant-condition
	while (true) {
		cargoTomlPath = path.join(currentPath, "Cargo.toml");
		if (files[cargoTomlPath]) break;
		const newPath = path.dirname(currentPath);
		if (currentPath === newPath) break;
		currentPath = newPath;
	}

	return cargoTomlPath;
}

export const getDefaultCache = ({ files, entrypoint }: BuildOptions) => {
	const cargoTomlPath = findCargoToml(files, entrypoint);
	if (!cargoTomlPath) return undefined;
	const defaultCacheRef = new FileRef({
		digest:
			"sha:204e0c840c43473bbd130d7bc704fe5588b4eab43cda9bc940f10b2a0ae14b16"
	});
	return { '.': defaultCacheRef };
};

export { shouldServe } from "@now/build-utils";
