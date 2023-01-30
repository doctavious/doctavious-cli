// _config.yml or _config.toml
// _site/
// change be changed via destination

// destination: DIR
// -d, --destination DIR

use serde::{Serialize, Deserialize, de};
use crate::commands::build::frameworks::framework::{ConfigurationFileDeserialization, FrameworkInfo, FrameworkSupport, read_config_files};


#[derive(Deserialize)]
struct JekyllConfig { destination: Option<String> }

pub struct Jekyll { info: FrameworkInfo }
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
    use crate::commands::build::frameworks::framework::{FrameworkInfo, FrameworkSupport};
    use super::Jekyll;

    #[test]
    fn test_jekyll() {
        let jekyll = Jekyll {
            info: FrameworkInfo {
                name: "".to_string(),
                website: None,
                configs: Some(vec![String::from("tests/resources/framework_configs/jekyll/_config.yml")]),
                project_file: None,
            },
        };

        let output = jekyll.get_output_dir();
        assert_eq!(output, "build")
    }

}
