// mkdocs.yml
// site --> default
// change be changed via site_dir

use serde::{Serialize, Deserialize, de};
use crate::commands::build::frameworks::framework::{ConfigurationFileDeserialization, FrameworkInfo, FrameworkSupport, read_config_files};


#[derive(Deserialize)]
struct MKDocsConfig { site_dir: Option<String> }

pub struct MKDocs { info: FrameworkInfo }
impl FrameworkSupport for MKDocs {
    fn get_info(&self) -> &FrameworkInfo {
        &self.info
    }

    fn get_output_dir(&self) -> String {
        if let Some(configs) = &self.info.configs {
            match read_config_files::<MKDocsConfig>(configs) {
                Ok(c) => {
                    if let Some(dir) = c.site_dir {
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

impl ConfigurationFileDeserialization for MKDocsConfig {}

#[cfg(test)]
mod tests {
    use crate::commands::build::frameworks::framework::{FrameworkInfo, FrameworkSupport};
    use super::MKDocs;

    #[test]
    fn test_hugo() {
        let mkdocs = MKDocs {
            info: FrameworkInfo {
                name: "".to_string(),
                website: None,
                configs: Some(vec![String::from("tests/resources/framework_configs/mkdocs/mkdocs.yml")]),
                project_file: None,
            }
        };

        let output = mkdocs.get_output_dir();
        assert_eq!(output, "build")
    }

}
