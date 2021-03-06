use clap::{Parser, Subcommand};
use anyhow::{anyhow, Result};
use colored::Colorize;
use reqwest::{header, Client, Response, Url};
use mime::Mime;
use std::{collections::HashMap, str::FromStr};
use syntect::{
    easy::HighlightLines,
    parsing::SyntaxSet,
    highlighting::{ThemeSet, Style},
    util::{as_24_bit_terminal_escaped, LinesWithEndings}
};

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
#[derive(Debug, PartialEq)]
struct KvPair {
    k: String,
    v: String
}

impl FromStr for KvPair {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // use `=` to split, we will get a iterator
        let mut split = s.split('=');
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
    s.parse()
}

async fn get(client: Client, args: &Get) -> Result<()> {
    let resp = client.get(&args.url).send().await?;
    //println!("{:?}", resp.text().await?);
    Ok(print_resp(resp).await?)
}

async fn post(client: Client, args: &Post) -> Result<()> {
    let mut body = HashMap::new();
    for pair in args.body.iter() {
        body.insert(&pair.k, &pair.v);
    }

    let resp = client.post(&args.url).json(&body).send().await?;
    //println!("{:?}", resp.text().await?);
    Ok(print_resp(resp).await?)
}

/// Display the version of server and status code
fn print_status(resp: &Response) {
    let status = format!("{:?} {}", resp.version(), resp.status()).blue();
    println!("{}\n", status);
}

/// Display the HTTP header returned from server
fn print_headers(resp: &Response) {
    for (name, value) in resp.headers() {
        println!("{}: {:?}", name.to_string().green(), value);
    }
    println!();
}

/// Display the HTTP body returned from server
fn print_body(m: Option<Mime>, body: &str) {
    match m {
        Some(v) if v == mime::APPLICATION_JSON => {
            //println!("{}", jsonxf::pretty_print(body).unwrap().cyan());
            print_syntect(body);
            print!("\x1b[0m");
        }
        _ => println!("{}", body)
    }
}

fn print_syntect(s: &str) {
    // Load these once at the start of your program
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    let syntax = ps.find_syntax_by_extension("json").unwrap();
    let mut h = HighlightLines::new(syntax, &ts.themes["base16-mocha.dark"]);

    for line in LinesWithEndings::from(s) {
        let ranges: Vec<(Style, &str)> = h.highlight(line, &ps);
        let escaped = as_24_bit_terminal_escaped(&ranges[..], true);
        print!("{}", escaped);
    }
}

/// Display the whole respon
async fn print_resp(resp: Response) -> Result<()> {
    print_status(&resp);
    print_headers(&resp);
    let mime = get_content_type(&resp);
    let body = resp.text().await?;
    print_body(mime, &body);
    Ok(())
}

/// Parse `content-type` to MIME type
fn get_content_type(resp: &Response) -> Option<Mime> {
    resp.headers()
        .get(header::CONTENT_TYPE)
        .map(|v| v.to_str().unwrap().parse().unwrap())
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts = Opts::parse();
    let mut headers = header::HeaderMap::new();

    headers.insert("X-POWERED-BY", "Rust".parse()?);
    headers.insert(header::USER_AGENT, "Rust HTTPie".parse()?);
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;
    let result = match opts.command {
        SubCommands::Get(ref args) => get(client, args).await?,
        SubCommands::Post(ref args) => post(client, args).await?
    };

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_url_works() {
        assert!(parse_url("abc").is_err());
        assert!(parse_url("http://abc.xyz").is_ok());
        assert!(parse_url("https://httpbin.org/post").is_ok());
    }

    #[test]
    fn parse_kv_pair_works() {
        assert!(parse_kv_pair("a").is_err());
        assert_eq!(
            parse_kv_pair("a=1").unwrap(),
            KvPair {
                k: "a".into(),
                v: "1".into()
            }
        );
    }
}