// _config.yml or _config.toml
// _site/
// change be changed via destination

// destination: DIR
// jekyll build -d, --destination DIR

use serde::{Deserialize};
use crate::commands::build::framework::{ConfigurationFileDeserialization, FrameworkBuildArg, FrameworkBuildArgs, FrameworkBuildSettings, FrameworkDetectionItem, FrameworkDetector, FrameworkInfo, FrameworkMatchingStrategy, FrameworkSupport, read_config_files};
use crate::commands::build::language::Language;

#[derive(Deserialize)]
struct JekyllConfig { destination: Option<String> }

pub struct Jekyll { info: FrameworkInfo }

impl Jekyll {
    fn new(configs: Option<Vec<&'static str>>) -> Self {
        Self {
            info: FrameworkInfo {
                name: "Jekyll",
                website: Some("https://jekyllrb.com/"),
                configs,
                language: Language::Ruby,
                detection: FrameworkDetector {
                    matching_strategy: FrameworkMatchingStrategy::Every,
                    detectors: vec![
                        FrameworkDetectionItem::Package {dependency: "jekyll"}
                    ]
                },
                build: FrameworkBuildSettings {
                    // bundle exec jekyll build
                    command: "jekyll build",
                    command_args: Some(FrameworkBuildArgs {
                        source: None,
                        config: Some(FrameworkBuildArg::Option {
                            short: "",
                            long: "--config",
                        }),
                        output: Some(FrameworkBuildArg::Option {
                            short: "-d",
                            long: "--destination"
                        })
                    }),
                    output_directory: "_site",
                },
            }
        }
    }
}

impl Default for Jekyll {
    fn default() -> Self {
        Jekyll::new(
            Some(Vec::from(["_config.yml", "_config.toml"]))
        )
    }
}


impl FrameworkSupport for Jekyll {
    fn get_info(&self) -> &FrameworkInfo {
        &self.info
    }

    fn get_output_dir(&self) -> String {
        if let Some(configs) = &self.info.configs {
            match read_config_files::<JekyllConfig>(configs) {
                Ok(c) => {
                    if let Some(destination) = c.destination {
                        return destination;
                    }
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

impl ConfigurationFileDeserialization for JekyllConfig {}


#[cfg(test)]
mod tests {
    use crate::commands::build::framework::{FrameworkSupport};
    use super::Jekyll;

    #[test]
    fn test_jekyll() {
        let jekyll = Jekyll::new(
            Some(vec!["tests/resources/framework_configs/jekyll/_config.yml"])
        );

        let output = jekyll.get_output_dir();
        assert_eq!(output, "build")
    }

}
