use clap::{Parser, Subcommand};

// A naive httpie implementation with Rust, can you imagine how easy it is?
#[derive(Parser, Debug)]
#[clap(author = "MrBanana <tomsawyer404@outlook.com>", version = "1.0" )]
struct Cli {
    #[clap(subcommand)]
    command: SubCommands,
}

// SubCommands map different HTTP methods, we only support GET/POST method
#[derive(Subcommand, Debug)]
enum SubCommands {
    Get(Get),
    Post(Post),
}

/// Feed get with an url and we will retrieve the response for you
#[derive(Parser, Debug)]
struct Get {
    url: String,
}

/// Feed post with an url and optional `key=value` pairs. We will post the data
/// as JSON, and retrieve the response for you
#[derive(Parser, Debug)]
struct Post {
    url: String,
    body: Vec<String>
}

fn main() {
    let cli = Cli::parse();

    println!("{:?}", cli);
    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level app
    //match &cli.command {
    //    Commands::Add { name } => {
    //        println!("'myapp add' was used, name is: {:?}", name)
    //    }
    //}
}