// antora-playbook.yml
// antora antora-playbook.yml or npx antora antora-playbook.yml
// build/site
// change change via dir

// antora generate <playbook> --to-dir <dir>


use serde::{Serialize, Deserialize, de};
use crate::commands::build::frameworks::framework::{ConfigurationFileDeserialization, FrameworkInfo, FrameworkSupport, read_config_files};


#[derive(Deserialize)]
struct AntoraConfigOutputKeys { dir: Option<String> }

#[derive(Deserialize)]
struct AntoraConfig { output: Option<AntoraConfigOutputKeys> }

pub struct Antora { info: FrameworkInfo }
impl FrameworkSupport for Antora {
    fn get_info(&self) -> &FrameworkInfo {
        &self.info
    }

    fn get_output_dir(&self) -> String {
        if let Some(configs) = &self.info.configs {
            match read_config_files::<AntoraConfig>(configs) {
                Ok(c) => {
                    if let Some(AntoraConfigOutputKeys {dir: Some(v)}) = c.output {
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

impl ConfigurationFileDeserialization for AntoraConfig {}


#[cfg(test)]
mod tests {
    use crate::commands::build::frameworks::framework::{FrameworkInfo, FrameworkSupport};
    use super::Antora;

    // assert!(env::set_current_dir(&root).is_ok());
    #[test]
    fn test_antora() {
        let antora = Antora {
            info: FrameworkInfo {
                name: "".to_string(),
                website: None,
                configs: Some(vec![String::from("tests/resources/framework_configs/antora/antora-playbook.yaml")]),
                project_file: None,
            },
        };

        let output = antora.get_output_dir();
        assert_eq!(output, "./launch")
    }

}
