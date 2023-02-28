mod framework;
mod frameworks;
mod js_module;
mod language;
mod package_manager;
mod framework_detection;
mod projects;

use std::process::Command;
use std::path::PathBuf;
use clap::Parser;
use crate::commands::build::frameworks::get_frameworks;
use crate::DOCTAVIOUS_DIR;
use crate::doctavious_error::Result;
use crate::output::Output;
use crate::settings::{SETTINGS, SETTINGS_FILE};

// https://vercel.com/docs/project-configuration#project-configuration/install-command
// https://github.com/vercel/vercel/search?p=3&q=InstallCommand
// https://github.com/vercel/vercel/blob/6b23950b650011f612b62c1b2c79982cdee76bf9/packages/build-utils/src/types.ts
// https://github.com/vercel/vercel/blob/6b23950b650011f612b62c1b2c79982cdee76bf9/packages/frameworks/src/frameworks.ts
// https://github.com/vercel/vercel/tree/6b23950b650011f612b62c1b2c79982cdee76bf9/packages/fs-detectors

#[derive(Parser, Debug)]
#[command(about = "Build on your local machine")]
pub(crate) struct BuildCommand {
    // Dry run: show instructions without running them (default: false)
    // should this just find framework and show command it will run?
    pub dry: bool,
    // context Specify a build_mod context or branch (contexts: "production", "deploy-preview", "branch-deploy", "dev") (default: "production")

    // option can be used to provide a working directory (that can be different from the current directory) when running CLI commands.
    // --cwd
    // pub cwd: String

    // The --debug option, shorthand -d, can be used to provide a more verbose output when running Vercel CLI commands.

    // publish
}

pub(crate) fn handle_build_command(command: BuildCommand, output: Option<Output>) -> Result<()> {
    // check if doctavious.toml is present
    if SETTINGS_FILE.exists() {
        // run build command from settings if it exists otherwise return failure?
        // probably wouldnt hurt to try to infer if build isnt present
        if let Some(build_settings) = &SETTINGS.build_settings {
            if build_settings.command.trim().is_empty() {
                // return Err
            }

            // execute and return - i think spawn and wait is preferred which should stream output
            // output executes command as a child process waiting for it to finish and collecting all of its output
            // spawn executes command as a child process returning a handle to it
            let output = Command::new("").output();
            match output {
                Ok(o) => {
                    // status
                    // stderror
                    // stdout
                }
                Err(e) => {

                }
            }
        }
    }

    for framework in get_frameworks() {
        // check if framework is supported / detected
        println!("{}", serde_json::to_string(framework.get_info())?);
        let info = framework.get_info();
        if info.detected() {

        }

    }

    Ok(())
}

// supporting dir probably makes mono-repos with separate docs easier for end-user
pub(crate) fn build(
    dir: Option<PathBuf> // maybe we always assume current working directory or we change how to get settings
) -> Result<()> {
    // check if doctavious.toml is present
    if SETTINGS_FILE.exists() {
        // run build command from settings if it exists otherwise return failure?
        // probably wouldnt hurt to try to infer if build isnt present
        if let Some(build_settings) = &SETTINGS.build_settings {
            if build_settings.command.trim().is_empty() {
                // return Err
            }

            // execute and return - i think spawn and wait is preferred which should stream output
            // output executes command as a child process waiting for it to finish and collecting all of its output
            // spawn executes command as a child process returning a handle to it
            let output = Command::new("").output();
            match output {
                Ok(o) => {
                    // status
                    // stderror
                    // stdout
                }
                Err(e) => {

                }
            }
        }
    }

    for framework in get_frameworks() {
        // check if framework is supported / detected
        println!("{}", serde_json::to_string(framework.get_info())?);
    }

    // infer build
    // if not log and then try to infer build system
    // log each attempt along with where its looking
    // warn if no build system is found and return error

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::commands::build::frameworks::get_frameworks;
    use crate::doctavious_error::Result as DoctaviousResult;

    #[test]
    fn should_iterate() -> DoctaviousResult<()> {
        for framework in get_frameworks() {
            println!("{}", serde_json::to_string(framework.get_info())?);
        }

        Ok(())
    }

}
