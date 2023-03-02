use std::fs;
use regex::{Error, Regex, RegexBuilder};
use serde_json::Value;
use crate::commands::build::projects::csproj::CSProj;
use crate::commands::build::framework::{FrameworkDetectionItem, FrameworkInfo, FrameworkMatchingStrategy};
use crate::commands::build::projects::project_file::{Proj, ProjectFile};
use crate::doctavious_error::Result as DoctaviousResult;

// Return matched Framework
// which should have framework info
// as well as project
pub(crate) struct MatchedFramework<'a> {
    pub framework_info: &'a FrameworkInfo,
    pub project: Option<ProjectFile>
}

pub(crate) struct MatchResult {
    pub project: Option<ProjectFile>
    // dependency -- could also do a dependency/version struct tuple and have an array of them
    // detected_version: String
}


pub(crate) fn detect_framework(frameworks: &[FrameworkInfo]) -> Option<&FrameworkInfo> {

    for framework in frameworks {
        let m = matches(framework);
        if m.is_some() {
            return Some(framework);
        }
    }

    None
}

fn matches(framework: &FrameworkInfo) -> Option<MatchResult> {
    let mut results: Vec<Option<MatchResult>> = vec![];
    // for detector in &framework.detection.detectors {
    //     let result = match detector {
    //         FrameworkDetectionItem::Config { content } => {
    //             if let Some(configs) = &framework.configs {
    //                 for config in configs {
    //                     // TODO: extract to check_config method
    //                     if let Ok(file_content) = fs::read_to_string(config) {
    //                         if let Some(content) = content {
    //                             // TODO: switch to regex
    //                             if file_content.contains(content) {
    //                                 return Some(MatchResult { project: None });
    //                             }
    //                             continue;
    //                         }
    //                         return Some(MatchResult { project: None });
    //                     }
    //                 }
    //             }
    //             None
    //         }
    //         FrameworkDetectionItem::Dependency { name: dependency } => {
    //             for p in framework.language.project_files() {
    //                 for path in p.get_project_paths() {
    //                     if !path.exists() {
    //                         // TODO: log
    //                         continue;
    //                     }
    //
    //                     if path.is_dir() {
    //                         // TODO: log
    //                         continue;
    //                     }
    //
    //                     let file_content = fs::read_to_string(path);
    //                     match file_content {
    //                         Ok(c) => {
    //                             let found = has_dependency(p, c, dependency);
    //                             match found {
    //                                 Ok(f) => {
    //                                     if f {
    //                                         return Some(MatchResult { project: Some(*p) });
    //                                     } else {
    //                                         // TODO: log -- dependency not found
    //                                     }
    //                                 }
    //                                 Err(_) => {
    //                                     // TODO: log -- error checking file for dependency
    //                                 }
    //                             }
    //                         }
    //                         Err(e) => {
    //                             // TODO: log -- error reading file
    //                             continue;
    //                         }
    //                     }
    //                 }
    //             }
    //
    //             None
    //         }
    //     };
    //
    //     match &framework.detection.matching_strategy {
    //         FrameworkMatchingStrategy::All => {
    //             results.push(result);
    //         }
    //         FrameworkMatchingStrategy::Any => {
    //             if let Some(result) = result {
    //                 results.push(Some(result));
    //                 break;
    //             }
    //         }
    //     }
    // }

    match &framework.detection.matching_strategy {
        FrameworkMatchingStrategy::All => {
            let a = framework.detection.detectors
                .iter().map(|item| check(&framework, item));
            results.extend(a);
        }
        FrameworkMatchingStrategy::Any => {
            for item in &framework.detection.detectors {
                let result = check(&framework, item);
                if result.is_some() {
                    results.push(result);
                    break;
                }
            }
        }
    }


    // use std::convert::identity might be more idiomatic here
    if results.iter().all(|r| r.is_some()) {
        return None;
    }

    // get first

    None
}

fn check(framework: &FrameworkInfo, item: &FrameworkDetectionItem) -> Option<MatchResult> {
    match item {
        FrameworkDetectionItem::Config { content } => {
            if let Some(configs) = &framework.configs {
                for config in configs {
                    if let Ok(file_content) = fs::read_to_string(config) {
                        if let Some(content) = content {
                            let regex = RegexBuilder::new(content)
                                .multi_line(true)
                                .build();
                            match regex {
                                Ok(regex) => {
                                    if regex.is_match(file_content.as_str()) {
                                        return Some(MatchResult { project: None });
                                    }
                                }
                                Err(e) => {
                                    // TODO: log
                                }
                            }
                        }
                        return Some(MatchResult { project: None });
                    }
                }
            }
            None
        }
        FrameworkDetectionItem::Dependency { name: dependency } => {
            for p in framework.language.project_files() {
                for path in p.get_project_paths() {
                    if !path.exists() {
                        // TODO: log
                        continue;
                    }

                    if path.is_dir() {
                        // TODO: log
                        continue;
                    }

                    let file_content = fs::read_to_string(path);
                    match file_content {
                        Ok(c) => {
                            let found = has_dependency(p, c, dependency);
                            match found {
                                Ok(f) => {
                                    if f {
                                        return Some(MatchResult { project: Some(*p) });
                                    } else {
                                        // TODO: log -- dependency not found
                                    }
                                }
                                Err(_) => {
                                    // TODO: log -- error checking file for dependency
                                }
                            }
                        }
                        Err(e) => {
                            // TODO: log -- error reading file
                            continue;
                        }
                    }
                }
            }

            None
        }
        _ => { None }
    }
}

fn has_dependency(project_type: &ProjectFile, content: String, dependency: &str) -> DoctaviousResult<bool> {
    let found = match project_type {
        ProjectFile::CargoToml => {
            let root: toml::Value = toml::from_str(content.as_str())?;
            // TODO: do we want to check dev-packages
            root.get("dependencies")
                .and_then(|o| o.get(dependency))
                .is_some()
        }
        ProjectFile::CSProj => {
            let build_proj: CSProj = serde_xml_rs::from_str(content.as_str())?;
            build_proj.has_package_reference(dependency)
        }
        ProjectFile::GemFile => {
            content.contains(&format!("gem '{}'", dependency))
        }
        ProjectFile::GoMod => {
            content.contains(&format!("{}", dependency))
        }
        ProjectFile::PackageJson => {
            let root: Value = serde_json::from_str(content.as_str())?;
            // TODO: do we want to check devDependencies
            root.get("dependencies")
                .and_then(|o| o.get(dependency))
                .is_some()
        }
        ProjectFile::PipFile => {
            let root: toml::Value = toml::from_str(content.as_str())?;
            // TODO: do we want to check dev-packages
            root.get("packages")
                .and_then(|o| o.get(dependency))
                .is_some()
        }
        ProjectFile::PyProject => {
            let root: toml::Value = toml::from_str(content.as_str())?;
            // might be to do these individual lookup
            root.get("tool.poetry.dependencies")
                .and_then(|o| o.get(dependency))
                .is_some()
        }
        ProjectFile::RequirementsTxt => {
            content.contains(&format!("{}==", dependency))
        }
    };

    Ok(found)
}
