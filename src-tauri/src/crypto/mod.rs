// higher-level age functions to be called from the frontend

mod commands;
use age::x25519::Recipient;
use age::Decryptor;
pub use commands::*;
use futures_util::{AsyncReadExt as FuturesReadExt, AsyncWriteExt as FuturesWriteExt};
use std::path::PathBuf;
use std::str::FromStr;
use tokio::fs::{metadata, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

pub async fn encrypt_file(
    public_keys: Vec<String>,
    file_path: PathBuf,
    progress: tauri::ipc::Channel<f64>,
) -> Result<PathBuf, String> {
    let file = File::open(&file_path).await.expect("failed to open file");
    let mut reader = BufReader::new(file);

    let mut encrypted_output = file_path.clone();
    encrypted_output.add_extension("age");
    let output = File::create(&encrypted_output)
        .await
        .expect("failed to get handle on output file");
    let file_writer = BufWriter::new(output).compat_write();

    let encryptor = age::Encryptor::with_recipients(
        keys_to_recipients(public_keys)
            .iter()
            .map(|recipient| recipient as _),
    )
    .expect("encryptor initialization failed");

    let mut writer = encryptor
        .wrap_async_output(file_writer)
        .await
        .expect("failed to initialize writer");

    let mut buffer = vec![0u8; 1024 * 1024 * 4]; // 4 MB buffer
    let total_byte_size = metadata(file_path).await.unwrap().len() as f64;
    let mut read_byte_size = 0 as f64;

    loop {
        let n = reader.read(&mut buffer).await.expect("failed to read file");
        if n == 0 {
            break;
        }
        writer
            .write_all(&buffer[..n])
            .await
            .expect("failed to write"); // only write the new bytes
        read_byte_size += n as f64;
        let _ = progress.send(read_byte_size / total_byte_size); // this is not a critical function
    }

    writer.close().await.expect("failed to write final chunk");
    Ok(encrypted_output)
}

pub async fn decrypt_file(private_key: String, file_path: PathBuf) -> Result<PathBuf, String> {
    let file = File::open(&file_path).await.expect("failed to open file");
    let decryptor = Decryptor::new_async_buffered(BufReader::new(file).compat())
        .await
        .expect("failed to initialize decryptor");

    let decrypted_output = file_path.with_extension("");
    let output = File::create(&decrypted_output)
        .await
        .expect("failed to get handle on output file");
    let mut file_writer = BufWriter::new(output);

    let mut decrypted_reader = decryptor
        .decrypt_async(std::iter::once(
            &age::x25519::Identity::from_str(private_key.as_str()).unwrap() as &dyn age::Identity,
        ))
        .expect("failed to decrypt contents");

    let mut buffer = [0u8; 8_192]; // 8 kb buffer

    loop {
        let n = decrypted_reader
            .read(&mut buffer)
            .await
            .expect("failed to read file");
        if n == 0 {
            break;
        }
        file_writer
            .write_all(&buffer[..n])
            .await
            .expect("failed to write"); // only write the new bytes
    }

    Ok(decrypted_output)
}

pub fn keys_to_recipients(public_keys: Vec<String>) -> Vec<Recipient> {
    return public_keys
        .iter()
        .map(|key| -> Result<Recipient, String> {
            Ok(key
                .parse::<Recipient>()
                .expect(&format!("could not parse recipient from key: {}", key)))
        })
        .filter_map(|recipient: Result<Recipient, _>| recipient.ok())
        .collect::<Vec<Recipient>>();
}

pub async fn encrypt_bytes(public_keys: Vec<String>, bytes: &[u8]) -> Vec<u8> {
    // TODO need to make error handling not be terrible here. you dont want to encrypt something to nobody
    let recipients = public_keys
        .iter()
        .map(|key| -> Result<Recipient, String> {
            Ok(key
                .parse::<Recipient>()
                .expect(&format!("could not parse recipient from key: {}", key)))
        })
        .filter_map(|recipient: Result<Recipient, _>| recipient.ok())
        .collect::<Vec<Recipient>>();

    let encryptor =
        age::Encryptor::with_recipients(recipients.iter().map(|recipient| recipient as _))
            .expect("encryptor initialization failed");

    let mut encrypted_output = vec![];
    let mut writer = encryptor
        .wrap_async_output(&mut encrypted_output)
        .await
        .expect("failed to initialize writer");

    writer
        .write_all(bytes)
        .await
        .expect("failed to write bytes");
    writer.finish().expect("failed to write final chunk");

    return encrypted_output;
}

#[tauri::command]
pub async fn encrypt_text(public_keys: Vec<String>, text: String) -> Vec<u8> {
    return encrypt_bytes(public_keys, text.as_bytes()).await;
}
