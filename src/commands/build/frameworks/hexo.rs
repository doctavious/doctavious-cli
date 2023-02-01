// output defaults to public
// _config.yml
// public_dir to override
// hexo-cli
// hexo generate
// hexo --config custom.yml

use serde::{Serialize, Deserialize, de};
use crate::commands::build::frameworks::framework::{ConfigurationFileDeserialization, FrameworkInfo, FrameworkSupport, read_config_files};

#[derive(Deserialize)]
struct HexoConfig { public_dir: Option<String> }

pub struct Hexo { info: FrameworkInfo }
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
    use crate::commands::build::frameworks::framework::{FrameworkInfo, FrameworkSupport};
    use super::Hexo;

    #[test]
    fn test_hexo() {
        let hexo = Hexo {
            info: FrameworkInfo {
                name: "".to_string(),
                website: None,
                configs: Some(vec![String::from("tests/resources/framework_configs/hexo/_config.yml")]),
                project_file: None,
            }
        };

        let output = hexo.get_output_dir();
        assert_eq!(output, "build")
    }

}
