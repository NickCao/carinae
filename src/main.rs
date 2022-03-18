use argh::FromArgs;
use futures::stream::StreamExt;
use indoc::indoc;
use poem::EndpointExt;
use poem::{
    error, get, handler, listener::TcpListener, web::Data, web::Path, Body, IntoResponse, Response,
    Result, Route, Server,
};

#[derive(FromArgs, Clone)]
/// serve any nix store as a binary cache
struct Args {
    /// address to listen on (default: 127.0.0.1:3000)
    #[argh(option, short = 'l', default = "String::from(\"127.0.0.1:3000\")")]
    listen: String,

    /// store to serve (default: daemon)
    #[argh(option, default = "String::from(\"daemon\")")]
    store: String,

    /// key to sign paths
    #[argh(option)]
    key: Option<String>,
}

#[handler]
async fn index() -> Result<impl IntoResponse> {
    Ok("this is carinae, a nix binary cache")
}

#[handler]
async fn info(args: Data<&Args>) -> Result<impl IntoResponse> {
    let store =
        crate::ffi::nixOpenStore(args.store.clone()).map_err(|e| error::InternalServerError(e))?;
    Ok(Response::builder()
        .content_type("text/x-nix-cache-info")
        .body(format!(
            indoc! {"
            StoreDir: {}
            WantMassQuery: 1
            Priority: 30
        "},
            crate::ffi::nixStoreDir(store)
        )))
}

fn format_pair(key: &str, value: &str) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        Some(format!("{}: {}", key, value))
    }
}

#[handler]
async fn narinfo(Path(hash): Path<String>, args: Data<&Args>) -> Result<impl IntoResponse> {
    let store =
        crate::ffi::nixOpenStore(args.store.clone()).map_err(|e| error::InternalServerError(e))?;
    let pathinfo = crate::ffi::nixPathInfoFromHashPart(
        store,
        hash.clone(),
        args.key.as_ref().unwrap_or(&String::from("")).clone(),
    )
    .map_err(|e| error::NotFound(e))?;
    Ok(Response::builder().content_type("text/x-nix-narinfo").body(
        [
            format_pair("StorePath", &pathinfo.path),
            format_pair("URL", &format!("nar/{}.nar.zst", &hash)),
            format_pair("Compression", "zstd"),
            format_pair("NarHash", &pathinfo.nar_hash),
            format_pair("NarSize", &format!("{}", pathinfo.nar_size)),
            format_pair("References", &pathinfo.references),
            format_pair("Deriver", &pathinfo.deriver),
            format_pair("CA", &pathinfo.ca),
        ]
        .into_iter()
        .chain(pathinfo.sigs.iter().map(|x| format_pair("Sig", x)))
        .chain([Some(String::from(""))].into_iter())
        .flatten()
        .collect::<Vec<String>>()
        .join("\n"),
    ))
}

#[handler]
async fn nar(Path(hash): Path<String>, args: Data<&Args>) -> Result<impl IntoResponse> {
    let (tx, rx) = tokio::sync::mpsc::channel(2048);
    let ctx = Box::new(NarContext(tx));
    let store = args.store.clone();
    tokio::task::spawn_blocking(move || {
        let store = crate::ffi::nixOpenStore(store)?;
        crate::ffi::nixNarFromHashPart(store, hash, ctx, |ctx1, data| {
            ctx1.0.blocking_send(data).is_ok()
        })
    });
    Ok(Response::builder()
        .content_type("application/x-nix-nar")
        .body(Body::from_async_read(
            async_compression::tokio::bufread::ZstdEncoder::new(tokio_util::io::StreamReader::new(
                tokio_stream::wrappers::ReceiverStream::new(rx)
                    .map(|x| Result::<_, std::io::Error>::Ok(std::collections::VecDeque::from(x))),
            )),
        )))
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let args: Args = argh::from_env();
    let app = Route::new()
        .at("/", get(index))
        .at("/nix-cache-info", get(info))
        .at("/:hash<[0-9a-z]+>.narinfo", get(narinfo))
        .at("/nar/:hash<[0-9a-z]+>.nar.zst", get(nar))
        .with(poem::middleware::AddData::new(args.clone()));
    // TODO: log
    // TODO: realisation
    // TODO: ls
    // TODO: debug info
    Server::new(TcpListener::bind(args.listen)).run(app).await
}

pub struct NarContext(tokio::sync::mpsc::Sender<Vec<u8>>);

#[cxx::bridge(namespace = "carinae")]
mod ffi {
    struct NixPathInfo {
        path: String,
        deriver: String, // FIXME: Option<String>
        nar_hash: String,
        references: String,
        nar_size: u64,
        sigs: Vec<String>,
        ca: String, // FIXME: Option<String>
    }
    extern "Rust" {
        type NarContext;
    }
    #[namespace = "nix"]
    unsafe extern "C++" {
        include!("carinae/include/nix.hh");
        type Store;
    }
    unsafe extern "C++" {
        fn nixOpenStore(uri: String) -> Result<SharedPtr<Store>>;
        fn nixStoreDir(store: SharedPtr<Store>) -> String;
        fn nixPathInfoFromHashPart(
            store: SharedPtr<Store>,
            hash: String,
            key: String,
        ) -> Result<NixPathInfo>;
        fn nixNarFromHashPart(
            store: SharedPtr<Store>,
            hash: String,
            ctx: Box<NarContext>,
            send: fn(&mut NarContext, Vec<u8>) -> bool,
        ) -> Result<()>;
    }
}
