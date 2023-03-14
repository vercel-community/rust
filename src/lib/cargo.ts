import fs from 'node:fs';
import path from 'node:path';
import toml from '@iarna/toml';
import execa from 'execa';

export interface CargoMetadataRoot {
  packages: CargoPackage[];
  workspace_members: string[];
  resolve: CargoResolve;
  target_directory: string;
  version: number;
  workspace_root: string;
  metadata: CargoMetadata;
}

interface CargoPackage {
  name: string;
  version: string;
  id: string;
  license: string;
  license_file: string;
  description: string;
  source: any;
  dependencies: CargoDependency[];
  targets: CargoTarget[];
  features: CargoFeatures;
  manifest_path: string;
  metadata: CargoDocsMetadata;
  publish: string[];
  authors: string[];
  categories: string[];
  default_run: any;
  rust_version: string;
  keywords: string[];
  readme: string;
  repository: string;
  homepage: string;
  documentation: string;
  edition: string;
  links: any;
}

interface CargoDependency {
  name: string;
  source: string;
  req: string;
  kind: any;
  rename: any;
  optional: boolean;
  uses_default_features: boolean;
  features: any[];
  target: string;
  path: string;
  registry: any;
}

interface CargoTarget {
  kind: string[];
  crate_types: string[];
  name: string;
  src_path: string;
  edition: string;
  'required-features': string[];
  doc: boolean;
  doctest: boolean;
  test: boolean;
}

interface CargoFeatures {
  default: string[];
  feat1: any[];
  feat2: any[];
}

interface CargoDocsMetadata {
  docs: CargoDocs;
}

interface CargoDocs {
  rs: Rs;
}

interface Rs {
  'all-features': boolean;
}

interface CargoResolve {
  nodes: Node[];
  root: string;
}

interface Node {
  id: string;
  dependencies: string[];
  deps: Dep[];
  features: string[];
}

interface Dep {
  name: string;
  pkg: string;
  dep_kinds: DepKind[];
}

interface DepKind {
  kind: any;
  target: string;
}

interface CargoMetadata {
  docs: Docs2;
}

interface Docs2 {
  rs: Rs2;
}

interface Rs2 {
  'all-features': boolean;
}

export async function getCargoMetadata(
  options: execa.Options,
): Promise<CargoMetadataRoot> {
  const { stdout: cargoMetaData } = await execa(
    'cargo',
    ['metadata', '--format-version', '1'],

    options,
  );

  return JSON.parse(cargoMetaData) as CargoMetadataRoot;
}

interface CargoConfig {
  env: Record<string, any>;
  cwd: string;
}

interface CargoBuildTarget {
  name?: string;
  path?: string;
}

interface CargoToml {
  bin?: CargoBuildTarget[];
}

interface CargoWorkspace {
  toml: CargoToml;
  root: string;
}

export async function findCargoWorkspace(
  config: CargoConfig,
): Promise<CargoWorkspace> {
  const { stdout: projectDescriptionStr } = await execa(
    'cargo',
    ['locate-project'],
    config,
  );
  const projectDescription = JSON.parse(projectDescriptionStr) as {
    root: string;
  };
  return {
    toml: await toml.parse.stream(
      fs.createReadStream(projectDescription.root),
    ),
    root: projectDescription.root,
  };
}

export function findBinaryName(
  workspace: CargoWorkspace,
  entryPath: string,
): string {
  const { bin } = workspace.toml;
  if (bin) {
    const relativePath = path.relative(path.dirname(workspace.root), entryPath);
    const entry = bin.find((binEntry) => binEntry.path === relativePath);
    if (entry?.name) {
      return entry.name;
    }
  }
  return path.basename(entryPath, '.rs').replace('[', '_').replace(']', '_');
}
