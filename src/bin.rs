use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about)]
struct L3Cli {
    #[command(subcommand)]
    command: L3Cmd,

    #[arg(short, long)]
    watch: bool,
}

#[derive(Subcommand)]
enum L3Cmd {
    Fn(FunctionArgs),
}

#[derive(Parser, Debug)]
struct FunctionArgs {
    #[clap(value_name = "FN_NAME")]
    function_name: String,
    #[clap(value_name = "SRC_DIR")]
    src_dir: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let l3 = L3Cli::parse();
    match l3.command {
        L3Cmd::Fn(cmd) => {
            fn_dev_sync(cmd.function_name, cmd.src_dir).await?;
        }
    }
    // match fs::read("l3.toml") {
    //
    // }
    let value = "".parse::<Table>().unwrap();

    // let resp = lambda.list_functions().send().await?;
    // println!("Functions:");
    // let functions = resp.functions().unwrap_or_default();
    // for function in functions {
    //     println!("  {}", function.function_name().unwrap_or_default());
    // }
    // println!("Found {} functions", functions.len());

    Ok(())
}
