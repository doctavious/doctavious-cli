// config.toml/yaml/json
// multiple can be used
// also has a config directory
// has options that would need to be merged. how to handle?
// hugo command
// hugo -d, --destination

// /public
// can be changed via publishDir

use serde::{Serialize, Deserialize, de};
use crate::commands::build::frameworks::framework::{ConfigurationFileDeserialization, FrameworkInfo, FrameworkSupport, read_config_files};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct HugoConfig { publish_dir: Option<String> }

pub struct Hugo { info: FrameworkInfo }

impl Default for Hugo {
    fn default() -> Self {
        Self {
            info: FrameworkInfo {
                name: "Hexo",
                website: Some("https://gohugo.io/"),
                configs: Some(Vec::from(["config.toml", "config.yaml", "config.json"])),
                project_file: None,
            }
        }
    }
}


impl FrameworkSupport for Hugo {
    fn get_info(&self) -> &FrameworkInfo {
        &self.info
    }

    fn get_output_dir(&self) -> String {
        if let Some(configs) = &self.info.configs {
            match read_config_files::<HugoConfig>(configs) {
                Ok(c) => {
                    if let Some(dir) = c.publish_dir {
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

impl ConfigurationFileDeserialization for HugoConfig {}

#[cfg(test)]
mod tests {
    use crate::commands::build::frameworks::framework::{FrameworkInfo, FrameworkSupport};
    use super::Hugo;

    #[test]
    fn test_hugo() {
        let hugo = Hugo {
            info: FrameworkInfo {
                name: "",
                website: None,
                configs: Some(vec!["tests/resources/framework_configs/hugo/config.toml"]),
                project_file: None,
            }
        };

        let output = hugo.get_output_dir();
        assert_eq!(output, "build")
    }

}
