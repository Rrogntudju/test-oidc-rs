use server::filters::*;
use std::env::{args, Args};
use std::error::Error;
use std::net::SocketAddr;
use std::path::PathBuf;
use warp::Filter;

fn parse_args(args: &mut Args) -> Result<(SocketAddr, PathBuf), Box<dyn Error>> {
    let addr = match args.skip(1).next() {
        Some(arg) => arg.parse::<SocketAddr>()?,
        None => return Err("IP:Port est manquant".into()),
    };

    let path = match args.next() {
        Some(arg) => arg.parse::<PathBuf>()?,
        None => return Err("Le répertoire est manquant".into()),
    };

    if !path.is_dir() {
        return Err(format!("{} n'existe pas ou n'est pas accessible", path.to_string_lossy()).into());
    }

    Ok((addr, path))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (addr, path) = parse_args(&mut args())?;
    let routes = static_file(path);

    warp::serve(routes).run(addr).await;
    Ok(())
}
