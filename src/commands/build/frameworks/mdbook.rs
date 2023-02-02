// book.toml
// ./book -> default
// change be changed via build.build-dir

use serde::{Serialize, Deserialize, de};
use crate::commands::build::frameworks::framework::{ConfigurationFileDeserialization, FrameworkInfo, FrameworkSupport, read_config_files};

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct MDBookBuildOptions { build_dir: Option<String> }

#[derive(Deserialize)]
struct MDBookConfig { build: Option<MDBookBuildOptions> }

pub struct MDBook { info: FrameworkInfo }
impl FrameworkSupport for MDBook {
    fn get_info(&self) -> &FrameworkInfo {
        &self.info
    }

    fn get_output_dir(&self) -> String {
        if let Some(configs) = &self.info.configs {
            match read_config_files::<MDBookConfig>(configs) {
                Ok(c) => {
                    if let Some(MDBookBuildOptions {build_dir: Some(v)}) = c.build {
                        return v;
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

impl ConfigurationFileDeserialization for MDBookConfig {}

#[cfg(test)]
mod tests {
    use crate::commands::build::frameworks::framework::{FrameworkInfo, FrameworkSupport};
    use super::MDBook;

    #[test]
    fn test_mdbook() {
        let book = MDBook {
            info: FrameworkInfo {
                name: "".to_string(),
                website: None,
                configs: Some(vec![String::from("tests/resources/framework_configs/mdbook/book.toml")]),
                project_file: None,
            }
        };

        let output = book.get_output_dir();
        assert_eq!(output, "build")
    }

}
