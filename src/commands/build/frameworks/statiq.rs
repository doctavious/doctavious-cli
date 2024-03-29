// https://www.statiq.dev/guide/configuration/settings
// maybe check that its a dotnet project with Statiq.Docs package
// look for .csproj with PackageReference of Statiq.Docs / Statiq.Web
// or Program.cs has Statiq.Docs or Bootstrapper
// dotnet run

// -o|--output

// output

// https://www.statiq.dev/guide/configuration/settings#configuration-files


use serde::{Serialize, Deserialize, de};
use swc_ecma_ast::Program;
use crate::commands::build::framework::{ConfigurationFileDeserialization, FrameworkBuildArg, FrameworkBuildArgs, FrameworkBuildSettings, FrameworkDetectionItem, FrameworkDetector, FrameworkInfo, FrameworkMatchingStrategy, FrameworkSupport, read_config_files};
use crate::commands::build::language::Language;

// #[derive(Deserialize)]
// struct AntoraConfigOutputKeys { dir: String }

#[derive(Deserialize)]
struct StatiqConfig { }

pub struct Statiq { info: FrameworkInfo }

impl Statiq {
    fn new(configs: Option<Vec<&'static str>>) -> Self {
        Self {
            info: FrameworkInfo {
                name: "Statiq",
                website: Some("https://www.statiq.dev/"),
                configs,
                language: Language::DotNet,
                detection: FrameworkDetector {
                    matching_strategy: FrameworkMatchingStrategy::All,
                    detectors: vec![
                        FrameworkDetectionItem::Config { content: None }
                    ]
                },
                build: FrameworkBuildSettings {
                    command: "",
                    command_args: None,
                    output_directory: "",
                }
            }
        }
    }
}

impl Default for Statiq {
    fn default() -> Self {
        Statiq::new(None)
    }
}

impl FrameworkSupport for Statiq {
    fn get_info(&self) -> &FrameworkInfo {
        &self.info
    }

    fn get_output_dir(&self) -> String {
        if let Some(configs) = &self.info.configs {
            match read_config_files::<StatiqConfig>(configs) {
                Ok(c) => {
                    return String::default();
                }
                Err(e) => {
                    // log warning/error
                    println!("{}", e.to_string());
                }
            }
        }

        self.info.build.output_directory.to_string()
    }
}

impl ConfigurationFileDeserialization for StatiqConfig {}


#[cfg(test)]
mod tests {
    use crate::commands::build::framework::{FrameworkBuildSettings, FrameworkInfo, FrameworkSupport};
    use super::Statiq;

    #[test]
    fn test_statiq() {
        let statiq = Statiq::new(Some(vec!["tests/resources/framework_configs/statiq/statiq.json"]));

        let output = statiq.get_output_dir();
        assert_eq!(output, "output")
    }

}
