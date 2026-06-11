use std::path::{Path, PathBuf};

use oxc_resolver::{ResolveOptions, Resolver, TsconfigOptions, TsconfigReferences};
use rustc_hash::FxHashMap;

use super::util::{absolutize, external_package_name, is_local_specifier, normalize_existing_path};
use super::{GraphError, GraphOptions};
#[allow(unused_imports)]
use crate::profile_span;

pub(super) struct ModuleResolver {
    root: PathBuf,
    resolver: Resolver,
    external_docs_enabled: bool,
    external_sources: FxHashMap<String, PathBuf>,
}

#[derive(Debug, Clone)]
pub(super) struct ResolvedModuleRef {
    pub(super) path: PathBuf,
    pub(super) external: Option<ExternalModuleRef>,
}

#[derive(Debug, Clone)]
pub(super) struct ExternalModuleRef {
    pub(super) package: String,
    pub(super) specifier: String,
}

impl ModuleResolver {
    pub(super) fn new(root: &Path, options: &GraphOptions) -> Self {
        let mut resolve_options = ResolveOptions {
            extensions: Vec::from([
                String::from(".d.ts"),
                String::from(".d.mts"),
                String::from(".d.cts"),
                String::from(".ts"),
                String::from(".tsx"),
                String::from(".mts"),
                String::from(".cts"),
                String::from(".js"),
                String::from(".jsx"),
                String::from(".mjs"),
                String::from(".cjs"),
                String::from(".json"),
                String::from(".node"),
            ]),
            extension_alias: Vec::from([
                (
                    String::from(".js"),
                    Vec::from([
                        String::from(".ts"),
                        String::from(".tsx"),
                        String::from(".d.ts"),
                        String::from(".js"),
                    ]),
                ),
                (
                    String::from(".mjs"),
                    Vec::from([String::from(".mts"), String::from(".d.mts"), String::from(".mjs")]),
                ),
                (
                    String::from(".cjs"),
                    Vec::from([String::from(".cts"), String::from(".d.cts"), String::from(".cjs")]),
                ),
            ]),
            condition_names: Vec::from([
                String::from("types"),
                String::from("import"),
                String::from("module"),
                String::from("default"),
            ]),
            main_fields: Vec::from([
                String::from("types"),
                String::from("module"),
                String::from("main"),
            ]),
            ..ResolveOptions::default()
        };

        if let Some(tsconfig) = &options.tsconfig {
            resolve_options.tsconfig = Some(TsconfigOptions {
                config_file: absolutize(root, tsconfig),
                references: TsconfigReferences::Auto,
            });
        }

        let external_sources = options
            .external_docs
            .package_sources
            .iter()
            .map(|source| {
                (source.package.clone(), normalize_existing_path(&absolutize(root, &source.entry)))
            })
            .collect();

        Self {
            root: root.to_path_buf(),
            resolver: Resolver::new(resolve_options),
            external_docs_enabled: options.external_docs.enabled,
            external_sources,
        }
    }

    pub(super) fn resolve_specifier(
        &self,
        importer: &Path,
        specifier: &str,
    ) -> Result<Option<ResolvedModuleRef>, GraphError> {
        profile_span!("docs::resolve_specifier");
        if !is_local_specifier(specifier) && !self.external_docs_enabled {
            return Ok(None);
        }

        if let Some(path) = self.resolve_external_source_override(specifier) {
            return Ok(Some(ResolvedModuleRef {
                path,
                external: Some(ExternalModuleRef {
                    package: external_package_name(specifier),
                    specifier: specifier.to_string(),
                }),
            }));
        }

        let directory = importer.parent().unwrap_or_else(|| Path::new("."));
        match self.resolver.resolve(directory, specifier) {
            Ok(resolution) => {
                let path = normalize_existing_path(resolution.path());
                let external = (!is_local_specifier(specifier)).then(|| ExternalModuleRef {
                    package: external_package_name(specifier),
                    specifier: specifier.to_string(),
                });
                Ok(Some(ResolvedModuleRef { path, external }))
            }
            Err(error) if is_local_specifier(specifier) => Err(GraphError::Resolve {
                importer: importer.to_path_buf(),
                specifier: specifier.to_string(),
                message: error.to_string(),
            }),
            Err(_) => Ok(None),
        }
    }

    fn resolve_external_source_override(&self, specifier: &str) -> Option<PathBuf> {
        if !self.external_docs_enabled || is_local_specifier(specifier) {
            return None;
        }

        let package = external_package_name(specifier);
        self.external_sources
            .get(specifier)
            .or_else(|| {
                (specifier == package).then(|| self.external_sources.get(&package)).flatten()
            })
            .map(|path| normalize_existing_path(&absolutize(&self.root, path)))
    }
}
