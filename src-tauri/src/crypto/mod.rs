// higher-level age functions to be called from the frontend

pub mod commands;
use age::x25519;
use age::Decryptor;
use age::{Identity, Recipient};
pub use commands::*;
use futures_util::{AsyncReadExt as FuturesReadExt, AsyncWriteExt as FuturesWriteExt};
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

/// can be any type that implements `age::Recipient`. `Send + Sync` for async compat
pub type WildcardRecipient = dyn Recipient + Send + Sync;
pub type WildcardIdentity = dyn Identity + Send + Sync;

/// every time a new chunk is encrypted, the callback will be run with the amount of bytes that were encrypted
pub async fn encrypt_file<F>(
    recipients: &Vec<Box<WildcardRecipient>>,
    file_path: &PathBuf,
    armor: bool,
    mut callback: F,
) -> Result<PathBuf, String>
where
    // im the greatest rust programmer ever
    F: FnMut(usize) + Send,
{
    let file = File::open(file_path).await.expect("failed to open file");
    let mut reader = BufReader::new(file);

    let mut encrypted_output = file_path.clone();
    encrypted_output.add_extension("age");
    let output = File::create(&encrypted_output)
        .await
        .expect("failed to get handle on output file");
    let format = if armor {
        age::armor::Format::AsciiArmor
    } else {
        age::armor::Format::Binary
    };
    let file_writer =
        age::armor::ArmoredWriter::wrap_async_output(BufWriter::new(output).compat_write(), format);

    let encryptor = age::Encryptor::with_recipients(
        recipients
            .iter()
            .map(|recipient| &**recipient as &dyn Recipient), // bro wtf
    )
    .expect("encryptor initialization failed");

    let mut writer = encryptor
        .wrap_async_output(file_writer)
        .await
        .expect("failed to initialize writer");

    let mut buffer = vec![0u8; 1024 * 1024 * 16]; // 16 MB buffer

    loop {
        let n = reader.read(&mut buffer).await.expect("failed to read file");
        if n == 0 {
            break;
        }
        writer
            .write_all(&buffer[..n])
            .await
            .expect("failed to write"); // only write the new bytes
        callback(n); // this is not a critical function
    }

    writer.close().await.expect("failed to write final chunk");
    Ok(encrypted_output)
}

pub async fn decrypt_file<F>(
    identity: &Box<WildcardIdentity>,
    file_path: &PathBuf,
    armor: bool,
    mut callback: F,
) -> Result<PathBuf, String>
where
    F: FnMut(usize) + Send,
{
    let file = File::open(file_path).await.expect("failed to open file");
    let reader: Box<dyn futures_io::AsyncBufRead + Unpin + Send + Sync> = if armor {
        Box::new(age::armor::ArmoredReader::from_async_reader(
            BufReader::new(file).compat(),
        ))
    } else {
        Box::new(BufReader::new(file).compat())
    };

    let decryptor: Decryptor<Box<dyn futures_io::AsyncBufRead + Unpin + Send + Sync>> =
        Decryptor::new_async_buffered(reader)
            .await
            .expect("failed to initialize decryptor");

    let decrypted_output = file_path.with_extension("");
    let output = File::create(&decrypted_output)
        .await
        .expect("failed to get handle on output file");
    let mut file_writer = BufWriter::new(output);

    let mut decrypted_reader = {
        let result = decryptor.decrypt_async(std::iter::once(&**identity as &dyn age::Identity));
        if let Ok(decryptor_reader) = result {
            decryptor_reader
        } else {
            return Err(format!(
                "decryption failed: {}",
                result.err().unwrap().to_string()
            ));
        }
    };

    let target_size = 1024 * 1024 * 4; // only send at most every 4MB
    let mut accumulator: usize = 0;
    let mut buffer = vec![0u8; 1024 * 1024 * 16]; // 16 MB buffer

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
        accumulator += n;
        if accumulator >= target_size {
            callback(accumulator);
            accumulator = 0;
        }
    }
    callback(accumulator); // ensure that it's sent at some point
    Ok(decrypted_output)
}

pub fn keys_to_x25519_recipients(
    public_keys: &Vec<String>,
) -> Result<Vec<x25519::Recipient>, String> {
    let keys_iter: Result<Vec<x25519::Recipient>, _> = public_keys
        .iter()
        .map(|key| key.parse::<x25519::Recipient>())
        .collect();
    if let Err(e) = keys_iter {
        return Err(format!("failed to parse key(s): {e}"));
    } else {
        return Ok(keys_iter.unwrap());
    }
}
