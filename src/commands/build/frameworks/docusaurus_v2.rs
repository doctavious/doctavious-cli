// docusaurus.config.js
// npm run build / docusaurus build
// build directory
// Both build/serve commands take an output dir option, and there's even a --build option on the serve command. We don't plan to add output dir to the config sorry


// docusaurus v1
// docusaurus-start
// website/siteConfig.js
// publish directory -> website/build/<projectName>
// where projectName is the value you defined in your siteConfig.js

// vercel just sees if there is a single file (directory) and uses it
// Code
// If there is only one file in it that is a dir we'll use it as dist dir
// if (content.length === 1 && content[0].isDirectory()) {
// return join(base, content[0].name);
// }


// docusaurus v2
// docusaurus build --out-dir
// docusaurus.config.js - doesnt contain output
// defaults to build


// TODO: support monorepo

use serde::{Deserialize};

use crate::commands::build::framework::{
    ConfigurationFileDeserialization,
    FrameworkBuildArg,
    FrameworkBuildArgs,
    FrameworkBuildSettings,
    FrameworkInfo,
    FrameworkSupport,
    read_config_files
};

// TODO: given there is no option to override does it make sense to still enforce Deserialize
// and ConfigurationFileDeserialization?
// I suppose we can determine if gatsby-plugin-output is in the plugins and grab it from there
#[derive(Deserialize)]
struct DocusaurusV2Config { output: String }

pub struct DocusaurusV2 { info: FrameworkInfo }

impl Default for DocusaurusV2 {
    fn default() -> Self {
        Self {
            info: FrameworkInfo {
                name: "Docusaurus 2",
                website: Some("https://docusaurus.io/"),
                configs: Some(Vec::from(["docusaurus.config.js"])),
                project_file: None,
                build: FrameworkBuildSettings {
                    command: "docusaurus build",
                    command_args: Some(FrameworkBuildArgs {
                        source: None,
                        config: Some(FrameworkBuildArg::Option {
                            short: "",
                            long: "--config",
                        }),
                        output: Some(FrameworkBuildArg::Option {
                            short: "",
                            long: "--out-dir",
                        })
                    }),
                    output_directory: "build",
                },
            }
        }
    }
}

impl FrameworkSupport for DocusaurusV2 {
    fn get_info(&self) -> &FrameworkInfo {
        &self.info
    }

    fn get_output_dir(&self) -> String {
        // doesnt support overriding via configuration file
        // TODO: look at package.json scripts build

        // if (content.length === 1 && content[0].isDirectory()) {
        // return join(base, content[0].name);
        // }

        self.info.build.output_directory.to_string()
    }
}

impl ConfigurationFileDeserialization for DocusaurusV2Config {}

#[cfg(test)]
mod tests {
    use crate::commands::build::framework::{FrameworkBuildSettings, FrameworkInfo, FrameworkSupport};
    use super::DocusaurusV2;

    #[test]
    fn test_docusaurus() {
        let docusaurus = DocusaurusV2 {
            info: FrameworkInfo {
                name: "",
                website: None,
                configs: Some(vec!["tests/resources/framework_configs/docusaurus2/docusaurus.config.js"]),
                project_file: None,
                build: FrameworkBuildSettings {
                    command: "",
                    command_args: None,
                    output_directory: "",
                },
            }
        };

        let output = docusaurus.get_output_dir();
        assert_eq!(output, "build")
    }
}
