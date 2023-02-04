// antora-playbook.yml
// antora antora-playbook.yml or npx antora antora-playbook.yml
// build/site
// change change via dir

// antora generate <playbook> --to-dir <dir>


use serde::{Deserialize};
use crate::commands::build::frameworks::framework::{ConfigurationFileDeserialization, FrameworkBuildArg, FrameworkBuildArgs, FrameworkBuildSettings, FrameworkInfo, FrameworkSupport, read_config_files};


#[derive(Deserialize)]
struct AntoraConfigOutputKeys { dir: Option<String> }

#[derive(Deserialize)]
struct AntoraConfig { output: Option<AntoraConfigOutputKeys> }

pub struct Antora { info: FrameworkInfo }
impl Default for Antora {
    fn default() -> Self {
        Self {
            info: FrameworkInfo {
                name: "Antora",
                website: Some("https://antora.org/"),
                configs: Some(Vec::from(["antora-playbook.yaml"])),
                project_file: None,
                build: FrameworkBuildSettings {
                    command: "antora generate",
                    command_args: Some(FrameworkBuildArgs {
                        source: None,
                        config: Some(FrameworkBuildArg::Arg(1, None)),
                        output: Some(FrameworkBuildArg::Option {
                            short: "",
                            long: "--to-dir"
                        })
                    }),
                    output_directory: "build/site",
                },
            },
        }
    }
}

impl FrameworkSupport for Antora {
    fn get_info(&self) -> &FrameworkInfo {
        &self.info
    }

    fn get_output_dir(&self) -> String {
        if let Some(configs) = &self.info.configs {
            match read_config_files::<AntoraConfig>(configs) {
                Ok(c) => {
                    if let Some(AntoraConfigOutputKeys {dir: Some(v)}) = c.output {
                        return v;
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

impl ConfigurationFileDeserialization for AntoraConfig {}


#[cfg(test)]
mod tests {
    use crate::commands::build::frameworks::framework::{FrameworkBuildSettings, FrameworkInfo, FrameworkSupport};
    use super::Antora;

    // assert!(env::set_current_dir(&root).is_ok());
    #[test]
    fn test_antora() {
        let antora = Antora {
            info: FrameworkInfo {
                name: "",
                website: None,
                configs: Some(vec!["tests/resources/framework_configs/antora/antora-playbook.yaml"]),
                project_file: None,
                build: FrameworkBuildSettings {
                    command: "",
                    command_args: None,
                    output_directory: "",
                },
            },
        };

        let output = antora.get_output_dir();
        assert_eq!(output, "./launch")
    }

}
