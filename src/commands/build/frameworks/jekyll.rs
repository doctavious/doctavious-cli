// _config.yml or _config.toml
// _site/
// change be changed via destination

// destination: DIR
// jekyll build -d, --destination DIR

use serde::{Serialize, Deserialize, de};
use crate::commands::build::frameworks::framework::{ConfigurationFileDeserialization, FrameworkBuildArg, FrameworkBuildArgs, FrameworkBuildOption, FrameworkBuildSettings, FrameworkInfo, FrameworkSupport, read_config_files};


#[derive(Deserialize)]
struct JekyllConfig { destination: Option<String> }

pub struct Jekyll { info: FrameworkInfo }

impl Default for Jekyll {
    fn default() -> Self {
        Self {
            info: FrameworkInfo {
                name: "Jekyll",
                website: Some("https://jekyllrb.com/"),
                configs: Some(Vec::from(["_config.yml", "_config.toml"])),
                project_file: None,
                build: FrameworkBuildSettings {
                    // bundle exec jekyll build
                    command: "jekyll build",
                    command_args: Some(FrameworkBuildArgs {
                        config: Some(FrameworkBuildArg::Option(FrameworkBuildOption {
                            short: "",
                            long: "--config",
                        })),
                        output: Some(FrameworkBuildArg::Option(FrameworkBuildOption {
                            short: "-d",
                            long: "--destination"
                        }))
                    }),
                    output_directory: "_site",
                },
            }
        }
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

        //self.info.build.outputdir
        String::default()
    }
}

impl ConfigurationFileDeserialization for JekyllConfig {}


#[cfg(test)]
mod tests {
    use crate::commands::build::frameworks::framework::{FrameworkBuildSettings, FrameworkInfo, FrameworkSupport};
    use super::Jekyll;

    #[test]
    fn test_jekyll() {
        let jekyll = Jekyll {
            info: FrameworkInfo {
                name: "",
                website: None,
                configs: Some(vec!["tests/resources/framework_configs/jekyll/_config.yml"]),
                project_file: None,
                build: FrameworkBuildSettings {
                    command: "",
                    command_args: None,
                    output_directory: "",
                },
            },
        };

        let output = jekyll.get_output_dir();
        assert_eq!(output, "build")
    }

}
