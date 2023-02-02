// conf.py
// sphinx package
// i dont see a way to configure this outside env var
// we could just default it ourselves
// BUILDDIR env var

use serde::{Serialize, Deserialize, de};
use std::env;
use crate::commands::build::frameworks::framework::{ConfigurationFileDeserialization, FrameworkInfo, FrameworkSupport, read_config_files};


pub struct Sphinx { info: FrameworkInfo }
impl FrameworkSupport for Sphinx {
    fn get_info(&self) -> &FrameworkInfo {
        &self.info
    }

    fn get_output_dir(&self) -> String {
        if let Ok(build_dir) = env::var("BUILDDIR") {
            return build_dir;
        }

        // TODO: return actual default
        String::default()
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use crate::commands::build::frameworks::framework::{FrameworkInfo, FrameworkSupport};
    use super::Sphinx;

    #[test]
    fn test_jekyll() {
        let sphinx = Sphinx {
            info: FrameworkInfo {
                name: "".to_string(),
                website: None,
                configs: Some(vec![String::from("tests/resources/framework_configs/sphinx/config.py")]),
                project_file: None,
            },
        };

        let output = sphinx.get_output_dir();
        assert_eq!(output, "")
    }

    #[test]
    fn should_use_env_var_when_present() {
        env::set_var("BUILDDIR", "build");
        let sphinx = Sphinx {
            info: FrameworkInfo {
                name: "".to_string(),
                website: None,
                configs: Some(vec![String::from("tests/resources/framework_configs/sphinx/config.py")]),
                project_file: None,
            },
        };

        let output = sphinx.get_output_dir();
        assert_eq!(output, "build")
    }

}
