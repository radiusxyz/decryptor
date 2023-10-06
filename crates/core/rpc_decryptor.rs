use jsonrpsee::server::Server;
use jsonrpsee::RpcModule;

use primitives::sync::{Arc, Mutex};
use primitives::types::EncryptedTransaction;
use runtime::rpc::RpcParameter;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};

extern crate dotenv;
use dotenv::dotenv;
use std::env;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    dotenv().ok();

    tracing_subscriber::fmt().init();
    let flag = Arc::new(AtomicBool::new(true));

    let thread_count = env::var("WORKER_THREAD_COUNT")
        .expect("WORKER_THREAD_COUNT must be set")
        .parse()?;

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(thread_count)
        .build()?;

    let server_flag = flag.clone();
    runtime.spawn(async move {
        run_server(server_flag.clone()).await.unwrap();
    });

    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        match input.trim() {
            "`" => {
                flag.store(false, Ordering::SeqCst);
                break;
            }
            _ => continue,
        }
    }
    Ok(())
}

async fn run_server(flag: Arc<AtomicBool>) -> Result<()> {
    let host = env::var("HOST").expect("HOST must be set");
    let port = env::var("PORT").expect("PORT must be set");

    let server = Server::builder()
        .build(format!("{}:{}", host, port))
        .await?;

    let state = Mutex::new(HashMap::<String, usize>::new());

    let mut module = RpcModule::new(state);

    module.register_async_method(
        EncryptedTransaction::method_name(),
        |parameters, _state| async move {
            let param = parameters.parse::<EncryptedTransaction>().unwrap();
            param.handler().await
        },
    )?;

    let addr = server.local_addr()?;
    tracing::info!("ws://{}", addr);
    let handle = server.start(module);

    tokio::spawn(async move {
        while flag.load(Ordering::SeqCst) {}
        handle.stop().unwrap();
    });
    Ok(())
}
