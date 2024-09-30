type Module = any;
type ModuleId = number;
type ModuleFactory = ($import: ModuleImport, $exports: ModuleExports) => void;

type ModuleImport = (source: string) => Module;
type ModuleExports = Record<string, any>;

type DependencyMap = Record<string, () => Module>;
type DependencyIds = Record<string, ModuleId>;

type ModuleContext = {
  factory: ModuleFactory;
  $import: ModuleImport;
  $exports: ModuleExports;
  $dependencyMap: DependencyMap;
};

interface GlobalModuleRegistry {
  define: (
    factory: ModuleFactory,
    id: ModuleId,
    dependencyMap?: DependencyMap
  ) => void;
  update: (id: ModuleId, dependencyIds?: DependencyIds) => void;
}
