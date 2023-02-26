mod init;
mod runtime;
mod dump;

// This is done to prevent compile time from exploding with every new command

#[tokio::main]
async fn main() {
    for arg in std::env::args() {
        match arg.as_str() {
            "--init" => {
                init::main().await;
                return;
            },
            "--dump" => {
                dump::main().await;
                return;
            },
            _ => {},
        }
    }
    runtime::main().await;
}
