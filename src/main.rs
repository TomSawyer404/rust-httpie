use clap::{Parser, Subcommand};
use anyhow::{anyhow, Result};
use reqwest::{header, Client, Response, Url};
use std::{collections::HashMap, str::FromStr};

// A naive httpie implementation with Rust, can you imagine how easy it is?
#[derive(Parser, Debug)]
#[clap(author = "MrBanana <tomsawyer404@outlook.com>", version = "1.0" )]
struct Opts {
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
    #[clap(parse(try_from_str = parse_url))]
    url: String,
}

fn parse_url(s: &str) -> Result<String> {
    let _url: Url = s.parse()?;
    Ok(s.into())
}

/// Feed post with an url and optional `key=value` pairs. We will post the data
/// as JSON, and retrieve the response for you
#[derive(Parser, Debug)]
struct Post {
    #[clap(parse(try_from_str = parse_url))]
    url: String,

    #[clap(parse(try_from_str = parse_kv_pair))]
    body: Vec<KvPair>
}

/// The `key=value` pair in command line can parse into `KvPair` struct using `parse_kv_pair`
#[derive(Debug)]
struct KvPair {
    k: String,
    v: String
}

impl FromStr for KvPair {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // use `=` to split, we will get a iterator
        let mut split = s.split("=");
        let err = || anyhow!(format!("Failed to parse {}", s));
        Ok(Self{
            // Get first result from iterator as `key`, the iterator return `Some(T)/None`
            // We transform `Ok(T)/Err(E)`, then use `?` to deal with error
            k: (split.next().ok_or_else(err)?).to_string(),

            // Get second result from iterator as `value`
            v: (split.next().ok_or_else(err)?).to_string()
        })
    }
}

fn parse_kv_pair(s: &str) -> Result<KvPair> {
    Ok(s.parse()?)
}

async fn get(client: Client, args: &Get) -> Result<()> {
    let resp = client.get(&args.url).send().await?;
    println!("{:?}", resp.text().await?);
    Ok(())
}

async fn post(client: Client, args: &Post) -> Result<()> {
    let mut body = HashMap::new();
    for pair in args.body.iter() {
        body.insert(&pair.k, &pair.v);
    }

    let resp = client.post(&args.url).json(&body).send().await?;
    println!("{:?}", resp.text().await?);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts = Opts::parse();
    let client = Client::new();
    let result = match opts.command {
        SubCommands::Get(ref args) => get(client, args).await?,
        SubCommands::Post(ref args) => post(client, args).await?
    };

    Ok(())
}