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

export async function getCargoMetadata(): Promise<CargoMetadataRoot> {
  const { stdout: cargoMetaData } = await execa('cargo', [
    'metadata',
    '--format-version',
    '1',
  ]);

  return JSON.parse(cargoMetaData) as CargoMetadataRoot;
}
