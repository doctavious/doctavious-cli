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
use crate::commands::build::framework_detection::detect_framework;
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

// The build command can be used to build a projects locally or in your own CI environment
#[derive(Parser, Debug)]
#[command(about = "Build on your local machine")]
pub(crate) struct BuildCommand {
    // Dry run: show instructions without running them (default: false)
    // should this just find framework and show command it will run?
    #[arg(long, short, help = "Dry run: show instructions without running them")]
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

    // TODO
    // Delete output directory from potential previous build
    // also delete .doctavious/output -- assuming we are putting things here
    // Create fresh new output directory
    // write builds.json file  (includes errors)
    // Store the build result to generate the final `config.json` after all builds have completed


    // vercel has concept of builders (@vercel/static) which is different than frameworks


    let detected_framework = detect_framework(get_frameworks());
    if let Some(detected_framework) = detected_framework {
        let info = detected_framework.get_info();
        println!("build command {}", &info.build.command);

        let build_command = info.build.command;
        // let (program, args) = if build_command.contains(" ") {
        //     let split_command: Vec<&str> = build_command.splitn(2, " ").collect();
        //     (split_command[0], Some(split_command[1]))
        // } else {
        //     (build_command, None)
        // };
        //
        // let mut build_process_command = Command::new(program);
        // if let Some(args) = args {
        //     build_process_command.arg(args);
        // }
        //
        // let build_process = build_process_command.spawn();

        // if (process.platform === 'win32') {
        //     await spawnAsync('cmd.exe', ['/C', command], opts);
        // } else {
        //     await spawnAsync('sh', ['-c', command], opts);
        // }

        // let build_process = Command::new("sh").args(["-c", build_command]).spawn();
        let build_process = if cfg!(target_os = "windows") {
            Command::new("cmd").args(["/C", build_command]).spawn()
        } else {
            Command::new("sh").args(["-c", build_command]).spawn()
        };

        match build_process {
            Ok(mut process) => {
                let result = process.wait();
                match result {
                    Ok(r) => {

                    }
                    Err(e) => {
                        println!("child error {e}");
                    }
                }
            }
            Err(e) => {
                println!("{:?}", e);
            }
        }

    } else {

    }

    // for framework in get_frameworks() {
    //     // check if framework is supported / detected
    //     println!("{}", serde_json::to_string(framework.get_info())?);
    //     let info = framework.get_info();
    //     if info.detected() {
    //
    //     }
    //
    // }

    Ok(())
}

// https://github.com/vercel/vercel/blob/61de63d2859c740d49ba1d28288fb7242886420c/packages/cli/src/commands/build.ts#L659
// https://github.com/vercel/vercel/blob/61de63d2859c740d49ba1d28288fb7242886420c/packages/static-build/src/index.ts#L1
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
