use crate::runtime::FnSourceParser;
use crate::{
    FnDependencies, FnEntrypoint, FnParseError, FnParseManifest, FnParseResult, FnParseSpec,
    FnSource, ModuleImport,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio::task::JoinHandle;

enum ParseFnMessage {
    ParsedSourceFile { source_file: FnSource },
}

enum SourceParsingState {
    Parsed(FnSource),
    Parsing,
}

// todo use tokio::select macro to handle errors in spawned tasks
pub async fn parse_fn_inner(
    parse_spec: &FnParseSpec,
    source_parser: Arc<Box<dyn FnSourceParser>>,
) -> FnParseResult<FnParseManifest> {
    let mut sources: HashMap<PathBuf, SourceParsingState> = HashMap::new();
    let mut uses_deps = false;
    let mut parsing: usize = 1;
    let mut join_handles: Vec<JoinHandle<_>> = Vec::new();
    let (tx, mut rx) = unbounded_channel::<ParseFnMessage>();
    let (handlers, entrypoint) = {
        let (source_file, handlers) = source_parser
            .parse_fn_entrypoint(&parse_spec.project_dir, parse_spec.entrypoint.clone())?;
        let path = source_file.path.clone();
        tx.send(ParseFnMessage::ParsedSourceFile { source_file })
            .unwrap();
        (handlers, path)
    };
    source_parser
        .collect_runtime_sources(&parse_spec.project_dir)
        .into_iter()
        .for_each(|source| {
            _ = sources.insert(source.path.clone(), SourceParsingState::Parsed(source))
        });
    while let Some(msg) = rx.recv().await {
        match msg {
            ParseFnMessage::ParsedSourceFile { source_file } => {
                for import in &source_file.imports {
                    match import {
                        ModuleImport::PackageDependency { .. } => uses_deps = true,
                        ModuleImport::RelativeSource(relative_source) => {
                            debug_assert!(
                                relative_source.is_relative(),
                                "{} not relative",
                                relative_source.to_string_lossy()
                            );
                            if !sources.contains_key(relative_source) {
                                parsing += 1;
                                sources.insert(
                                    relative_source.to_path_buf(),
                                    SourceParsingState::Parsing,
                                );
                                join_handles.push(tokio::spawn(parse_source_file(
                                    tx.clone(),
                                    source_parser.clone(),
                                    parse_spec.project_dir.clone(),
                                    relative_source.to_path_buf(),
                                )));
                            }
                        }
                        ModuleImport::Unknown(specifier) => {
                            return Err(FnParseError::UnresolvedImport {
                                from: source_file.path,
                                import: specifier.clone(),
                            })
                        }
                    }
                }
                parsing -= 1;
                sources.insert(
                    source_file.path.clone(),
                    SourceParsingState::Parsed(source_file),
                );
                if parsing == 0 {
                    break;
                }
            }
        }
    }
    Ok(FnParseManifest {
        dependencies: if uses_deps {
            FnDependencies::Required
        } else {
            FnDependencies::Unused
        },
        entrypoint: FnEntrypoint {
            handlers,
            path: entrypoint,
        },
        sources: sources
            .into_values()
            .map(|source_parsing_state| match source_parsing_state {
                SourceParsingState::Parsed(source_file) => source_file,
                SourceParsingState::Parsing => panic!(),
            })
            .collect(),
    })
}

async fn parse_source_file(
    tx: UnboundedSender<ParseFnMessage>,
    source_parser: Arc<Box<dyn FnSourceParser>>,
    project_dir: Arc<PathBuf>,
    source_path: PathBuf,
) {
    debug_assert!(
        project_dir.is_absolute(),
        "{} not absolute",
        project_dir.to_string_lossy()
    );
    debug_assert!(
        project_dir.is_dir(),
        "{} not directory",
        project_dir.to_string_lossy()
    );
    debug_assert!(
        source_path.is_relative(),
        "{} not relative",
        source_path.to_string_lossy()
    );
    let source_file = source_parser.parse_for_imports(&project_dir, source_path);
    _ = tx.send(ParseFnMessage::ParsedSourceFile {
        source_file: source_file.unwrap(),
    });
}
