use clap::Parser;

#[derive(Parser, Debug)]
#[command(about = "Create a new deploy from the contents of a folder")]
pub(crate) struct Deploy {
    #[arg(long, short, help = "Specify a folder to deploy")]
    pub directory: Option<String>,

    // To customize the subdomain of your draft URL with a unique string, use the --alias

    pub prod: bool


    //--prebuilt': Boolean,

    // -a --auth <token>
    // --build_mod Run build_mod command before deploying

    // -m, --message <message> A short message to include in the deploy log

    // -o, --open Open site after deploy (default: false)

    // -s, --site <name-or-id> A site name or ID to deploy to

    // --timeout <number>  Timeout to wait for deployment to finish

    // '-y': '--yes', is autoConfirm
}
