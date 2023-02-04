// output defaults to public
// _config.yml
// public_dir to override
// hexo-cli
// hexo generate
// hexo --config custom.yml

use serde::{Serialize, Deserialize, de};
use crate::commands::build::frameworks::framework::{ConfigurationFileDeserialization, FrameworkBuildArg, FrameworkBuildArgs, FrameworkBuildSettings, FrameworkInfo, FrameworkSupport, read_config_files};

#[derive(Deserialize)]
struct HexoConfig { public_dir: Option<String> }

pub struct Hexo { info: FrameworkInfo }

impl Default for Hexo {
    fn default() -> Self {
        Self {
            info: FrameworkInfo {
                name: "Hexo",
                website: Some("https://hexo.io/"),
                configs: Some(Vec::from(["_config.yml"])),
                project_file: None,
                build: FrameworkBuildSettings {
                    command: "hexo generate",
                    command_args: Some(FrameworkBuildArgs {
                        source: None,
                        config: Some(FrameworkBuildArg::Option {
                            short: "",
                            long: "--config"
                        }),
                        output: None
                    }),
                    output_directory: "public",
                },
            }
        }
    }
}

impl FrameworkSupport for Hexo {
    fn get_info(&self) -> &FrameworkInfo {
        &self.info
    }

    fn get_output_dir(&self) -> String {
        if let Some(configs) = &self.info.configs {
            match read_config_files::<HexoConfig>(configs) {
                Ok(c) => {
                    if let Some(dir) = c.public_dir {
                        return dir;
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

impl ConfigurationFileDeserialization for HexoConfig {}

#[cfg(test)]
mod tests {
    use crate::commands::build::frameworks::framework::{FrameworkBuildSettings, FrameworkInfo, FrameworkSupport};
    use super::Hexo;

    #[test]
    fn test_hexo() {
        let hexo = Hexo {
            info: FrameworkInfo {
                name: "",
                website: None,
                configs: Some(vec!["tests/resources/framework_configs/hexo/_config.yml"]),
                project_file: None,
                build: FrameworkBuildSettings {
                    command: "",
                    command_args: None,
                    output_directory: "",
                },
            }
        };

        let output = hexo.get_output_dir();
        assert_eq!(output, "build")
    }

}
