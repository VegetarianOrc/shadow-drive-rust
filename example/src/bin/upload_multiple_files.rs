use byte_unit::{Byte, ByteUnit};
use futures::{future::join_all, TryStreamExt};
use shadow_drive_rust::{models::ShdwFile, Client};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey,
    signer::{keypair::read_keypair_file, Signer},
};
use std::str::FromStr;
use tokio::fs::File;
use tokio_stream::StreamExt;
use tracing::Level;

const KEYPAIR_PATH: &str = "keypair.json";

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("off,shadow_drive_rust=debug")
        .init();

    //load keypair from file
    let keypair = read_keypair_file(KEYPAIR_PATH).expect("failed to load keypair at path");
    let pubkey = keypair.pubkey();
    let (storage_account_key, _) =
        shadow_drive_rust::derived_addresses::storage_account(&pubkey, 4);

    //create shdw drive client
    let solana_rpc = RpcClient::new("https://ssc-dao.genesysgo.net");
    let shdw_drive_client = Client::new(keypair, solana_rpc);

    //ensure storage account
    if let Err(_) = shdw_drive_client
        .get_storage_account(&storage_account_key)
        .await
    {
        println!("Error finding storage account, assuming it's not created yet");
        shdw_drive_client
            .create_storage_account(
                "shadow-drive-rust-test-2",
                Byte::from_str("1MB").expect("failed to parse byte string"),
            )
            .await
            .expect("failed to create storage account");
    }

    let dir = tokio::fs::read_dir("multiple_uploads")
        .await
        .expect("failed to read multiple uploads dir");

    let files = tokio_stream::wrappers::ReadDirStream::new(dir)
        .filter(Result::is_ok)
        .and_then(|entry| async move {
            Ok(ShdwFile {
                name: entry
                    .file_name()
                    .into_string()
                    .expect("failed to convert os string to regular string"),
                file: File::open(entry.path()).await.expect("failed to open file"),
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .await
        .expect("failed to create shdw files for dir");

    let upload_results = shdw_drive_client
        .upload_multiple_files(&storage_account_key, files)
        .await
        .expect("failed to upload files");

    println!("upload results: {:#?}", upload_results);
}