use serde::{Serialize, Deserialize, de};
use crate::commands::build::frameworks::framework::{ConfigurationFileDeserialization, FrameworkInfo, FrameworkSupport, read_config_files};


// #[derive(Deserialize)]
// struct AntoraConfigOutputKeys { dir: String }

#[derive(Deserialize)]
struct StatiqConfig { }

pub struct Statiq { info: FrameworkInfo }
impl FrameworkSupport for Statiq {
    fn get_info(&self) -> &FrameworkInfo {
        &self.info
    }

    fn get_output_dir(&self) -> String {
        if let Some(configs) = &self.info.configs {
            match read_config_files::<StatiqConfig>(configs) {
                Ok(c) => {
                    return String::default();
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

impl ConfigurationFileDeserialization for StatiqConfig {}


#[cfg(test)]
mod tests {
    use crate::commands::build::frameworks::framework::{FrameworkInfo, FrameworkSupport};

    // #[test]
    // fn test_static() {
    //     let antora = Antora {
    //         info: FrameworkInfo {
    //             name: "".to_string(),
    //             website: None,
    //             configs: Some(vec![String::from("antora-playbook.yaml")]),
    //             project_file: None,
    //         },
    //     };
    //
    //     let output = antora.get_output_dir();
    //     println!("{}", output);
    // }

}
