// docfx.json
// "docfx <docfx_project>/docfx.json"
// _site
// docfx build [-o:<output_path>] [-t:<template folder>]

use serde::{Deserialize};
use crate::commands::build::frameworks::framework::{ConfigurationFileDeserialization, FrameworkBuildArg, FrameworkBuildArgs, FrameworkBuildSettings, FrameworkInfo, FrameworkSupport, read_config_files};

#[derive(Deserialize)]
struct DocFxConfigBuild { dest: String }

#[derive(Deserialize)]
struct DocFxConfig { build: DocFxConfigBuild }

pub struct DocFx { info: FrameworkInfo }

impl Default for DocFx {
    fn default() -> Self {
        Self {
            info: FrameworkInfo {
                name: "DocFX",
                website: Some("https://dotnet.github.io/docfx/"),
                configs: Some(Vec::from(["docfx.json"])),
                project_file: None,
                build: FrameworkBuildSettings {
                    command: "docfx build",
                    command_args: Some(FrameworkBuildArgs {
                        source: None,
                        config: None,
                        output: Some(FrameworkBuildArg::Option {
                            short: "-o",
                            long: ""
                        })
                    }),
                    output_directory: "_site",
                },
            }
        }
    }
}

impl FrameworkSupport for DocFx {
    fn get_info(&self) -> &FrameworkInfo {
        &self.info
    }

    fn get_output_dir(&self) -> String {
        if let Some(configs) = &self.info.configs {
            match read_config_files::<DocFxConfig>(configs) {
                Ok(c) => {
                    return c.build.dest
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

impl ConfigurationFileDeserialization for DocFxConfig {}

#[cfg(test)]
mod tests {
    use crate::commands::build::frameworks::framework::{FrameworkBuildSettings, FrameworkInfo, FrameworkSupport};
    use crate::commands::build::frameworks::docfx::DocFx;

    #[test]
    fn test_docfx() {
        let docfx = DocFx {
            info: FrameworkInfo {
                name: "",
                website: None,
                configs: Some(vec!["tests/resources/framework_configs/docfx/docfx.json"]),
                project_file: None,
                build: FrameworkBuildSettings {
                    command: "",
                    command_args: None,
                    output_directory: "",
                },
            }
        };

        let output = docfx.get_output_dir();
        assert_eq!(output, "dist")
    }

}
