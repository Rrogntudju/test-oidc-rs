use server::filters::*;
use std::env::{args, Args};
use std::error::Error;
use std::net::SocketAddr;
use std::path::PathBuf;
use warp::Filter;

fn parse_args(args: &mut Args) -> Result<(SocketAddr, PathBuf, Option<PathBuf>), Box<dyn Error>> {
    let addr = match args.skip(1).next() {
        Some(arg) => arg.parse::<SocketAddr>()?,
        None => return Err("IP:Port est manquant".into()),
    };

    let path_static = match args.next() {
        Some(arg) => arg.parse::<PathBuf>()?,
        None => return Err("Le chemin du répertoire static est manquant".into()),
    };

    if !path_static.is_dir() {
        return Err(format!("{} n'existe pas ou n'est pas accessible", path_static.to_string_lossy()).into());
    }

    let path_tls = match args.next() {
        Some(arg) => {
            let p = arg.parse::<PathBuf>()?;
            if !p.is_dir() {
                return Err(format!("{} n'existe pas ou n'est pas accessible", p.to_string_lossy()).into());
            }
            Some(p)
        }
        None => None,
    };

    Ok((addr, path_static, path_tls))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (addr, path_static, path_tls) = parse_args(&mut args())?;
    let routes = static_file(path_static).or(userinfos()).or(auth());

    let server = warp::serve(routes);
    if let Some(p) = path_tls {
        server
            .tls()
            .cert_path(p.join("server.pem"))
            .key_path(p.join("server-key.pem"))
            .run(addr)
            .await;
    } else {
        server.run(addr).await;
    }

    Ok(())
}
