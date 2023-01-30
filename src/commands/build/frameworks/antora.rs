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

            // for config in configs {
            //     // TODO: this might be better to parse into struct
            //     println!("{}", config);
            //     if let Ok(contents) = fs::read_to_string(config) {
            //         match serde_yaml::from_str::<AntoraConfig>(contents.as_str()) {
            //             Ok(c) => {
            //                 return c.output.dir
            //             }
            //             Err(e) => {
            //                 // log warning/error
            //                 println!("{}", e.to_string());
            //             }
            //         }
            //     } else {
            //         println!("could not read file {}", config);
            //     }
            // }
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
