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
}

#[handler]
async fn index() -> Result<impl IntoResponse> {
    Ok("this is carinae, a nix binary cache")
}

#[handler]
async fn info(args: Data<&Args>) -> Result<impl IntoResponse> {
    let store = crate::ffi::openStore(&args.store).map_err(|e| error::InternalServerError(e))?;
    Ok(Response::builder()
        .content_type("text/x-nix-cache-info")
        .body(format!(
            indoc! {"
            StoreDir: {}
            WantMassQuery: 1
            Priority: 30
        "},
            crate::ffi::storeDir(store)
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
    let store = crate::ffi::openStore(&args.store).map_err(|e| error::InternalServerError(e))?;
    let pathinfo = crate::ffi::queryPathInfoFromHashPart(
        store,
        &hash,
        &std::env::var("CARINAE_SECRET_KEY").unwrap_or(String::from("")),
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
        let store = crate::ffi::openStore(&store)?;
        crate::ffi::narFromHashPart(store, &hash, ctx, |ctx1, data| {
            ctx1.0.blocking_send(data).is_ok()
        })
    });
    Ok(Response::builder()
        .content_type("application/x-nix-nar")
        .body(Body::from_async_read(
            async_compression::tokio::bufread::ZstdEncoder::new(tokio_util::io::StreamReader::new(
                tokio_stream::wrappers::ReceiverStream::new(rx)
                    .map(|x| Result::<_, std::io::Error>::Ok(x)),
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

pub struct NarContext<'a>(tokio::sync::mpsc::Sender<&'a [u8]>);

#[cxx::bridge(namespace = "carinae")]
mod ffi {
    struct PathInfo {
        path: String,
        deriver: String, // FIXME: Option<String>
        nar_hash: String,
        references: String,
        nar_size: u64,
        sigs: Vec<String>,
        ca: String, // FIXME: Option<String>
    }
    extern "Rust" {
        type NarContext<'a>;
    }
    #[namespace = "nix"]
    unsafe extern "C++" {
        include!("carinae/include/nix.hh");
        type Store;
    }
    unsafe extern "C++" {
        fn openStore(uri: &str) -> Result<SharedPtr<Store>>;
        fn storeDir(store: SharedPtr<Store>) -> String;
        fn queryPathInfoFromHashPart(
            store: SharedPtr<Store>,
            hash: &str,
            key: &str,
        ) -> Result<PathInfo>;
        fn narFromHashPart<'a>(
            store: SharedPtr<Store>,
            hash: &str,
            ctx: Box<NarContext<'a>>,
            send: fn(&mut NarContext<'a>, &'a [u8]) -> bool,
        ) -> Result<()>;
    }
}
