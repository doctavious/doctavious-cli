// conf.py -- <sourcedir>/conf.py
// sphinx package
// i dont see a way to configure this outside env var
// we could just default it ourselves
// BUILDDIR env var

use serde::{Serialize, Deserialize, de};
use std::env;
use crate::commands::build::frameworks::framework::{ConfigurationFileDeserialization, FrameworkBuildArg, FrameworkBuildArgs, FrameworkBuildSettings, FrameworkInfo, FrameworkSupport, read_config_files};


pub struct Sphinx { info: FrameworkInfo }

impl Default for Sphinx {
    fn default() -> Self {
        Self {
            info: FrameworkInfo {
                name: "Sphinx",
                website: Some("https://www.sphinx-doc.org/en/master/"),
                // this is relative to source and i dont think we need it as it doesnt help with build
                // TODO: should we remove?
                configs: Some(vec!["conf.py"]),
                project_file: None,
                build: FrameworkBuildSettings {
                    // docs docs/_build
                    command: "sphinx-build", // TODO: source has to be passed in? Default here?
                    command_args: Some(FrameworkBuildArgs {
                        source: Some(FrameworkBuildArg::Arg(1, Some("docs"))),
                        config: None,
                        output: Some(FrameworkBuildArg::Arg(2, None))
                    }),
                    // TODO: must be passed in to command which presents a problem if we dont know where the build script is
                    output_directory: "docs/_build",
                },
            },
        }
    }
}

impl FrameworkSupport for Sphinx {
    fn get_info(&self) -> &FrameworkInfo {
        &self.info
    }

    fn get_output_dir(&self) -> String {
        if let Ok(build_dir) = env::var("BUILDDIR") {
            return build_dir;
        }

        self.info.build.output_directory.to_string()
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use crate::commands::build::frameworks::framework::{FrameworkBuildSettings, FrameworkInfo, FrameworkSupport};
    use super::Sphinx;

    #[test]
    fn test_sphinx() {
        let sphinx = Sphinx {
            info: FrameworkInfo {
                name: "",
                website: None,
                configs: Some(vec!["tests/resources/framework_configs/sphinx/config.py"]),
                project_file: None,
                build: FrameworkBuildSettings {
                    command: "",
                    command_args: None,
                    output_directory: "",
                },
            },
        };

        let output = sphinx.get_output_dir();
        assert_eq!(output, "")
    }

    #[test]
    fn should_use_env_var_when_present() {
        temp_env::with_var("BUILDDIR", Some("build"), || {
            let sphinx = Sphinx {
                info: FrameworkInfo {
                    name: "",
                    website: None,
                    configs: Some(vec!["tests/resources/framework_configs/sphinx/config.py"]),
                    project_file: None,
                    build: FrameworkBuildSettings {
                        command: "",
                        command_args: None,
                        output_directory: "",
                    },
                },
            };

            let output = sphinx.get_output_dir();
            assert_eq!(output, "build")
        });
    }

}
