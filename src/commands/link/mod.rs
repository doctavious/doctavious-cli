use clap::Parser;

// this is used to setup the project in Doctavious
// can be auto setup as part of deploy

// this is really just linking

// reference - https://github.com/vercel/vercel/blob/cfc1c9e818ebb55d440479cf0edf18536b772b28/packages/cli/src/commands/deploy/index.ts#L274


#[derive(Parser, Debug)]
#[command(about = "")]
pub(crate) struct LinkCommand {

}
