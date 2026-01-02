// higher-level age functions to be called from the frontend

pub mod commands;
use age::Decryptor;
use age::{Identity, Recipient};
use age_xwing::{HybridIdentity, HybridRecipient};
pub use commands::*;
use futures_util::{AsyncReadExt as FuturesReadExt, AsyncWriteExt as FuturesWriteExt};
use secrecy::SecretString;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

/// an enum representing the x25519, hybrid, and scrypt age recipient types. directly implements `age::Recipient`.
pub enum WildcardRecipient {
    X25519(age::x25519::Recipient),
    Hybrid(HybridRecipient),
    Scrypt(age::scrypt::Recipient),
}

impl WildcardRecipient {
    pub fn to_string(&self) -> Result<String, String> {
        Ok(match self {
            Self::Hybrid(recipient) => recipient.to_string(),
            Self::X25519(recipient) => recipient.to_string(),
            Self::Scrypt(_) => return Err("cannot convert scrypt identity to string".to_string()),
        })
    }
}

impl Recipient for WildcardRecipient {
    fn wrap_file_key(
        &self,
        file_key: &age_core::format::FileKey,
    ) -> Result<
        (
            Vec<age_core::format::Stanza>,
            std::collections::HashSet<String>,
        ),
        age::EncryptError,
    > {
        match self {
            Self::X25519(recipient) => recipient.wrap_file_key(file_key),
            Self::Hybrid(recipient) => recipient.wrap_file_key(file_key),
            Self::Scrypt(recipient) => recipient.wrap_file_key(file_key),
        }
    }
}

/// an enum representing the x25519, hybrid, and scrypt age identity types. directly implements `age::Identity`.
pub enum WildcardIdentity {
    X25519(age::x25519::Identity),
    Hybrid(HybridIdentity),
    Scrypt(age::scrypt::Identity),
}

impl WildcardIdentity {
    pub fn to_public(&self) -> Result<WildcardRecipient, String> {
        Ok(match self {
            Self::Hybrid(identity) => WildcardRecipient::Hybrid(identity.to_public()),
            Self::X25519(identity) => WildcardRecipient::X25519(identity.to_public()),
            Self::Scrypt(_) => return Err("cannot convert scrypt identity to public".to_string()),
        })
    }

    pub fn to_string(&self) -> Result<SecretString, String> {
        Ok(match self {
            Self::Hybrid(identity) => identity.to_string(),
            Self::X25519(identity) => SecretString::from(identity.to_string()),
            Self::Scrypt(_) => return Err("cannot convert scrypt identity to string".to_string()),
        })
    }
}

impl Identity for WildcardIdentity {
    fn unwrap_stanza(
        &self,
        stanza: &age_core::format::Stanza,
    ) -> Option<Result<age_core::format::FileKey, age::DecryptError>> {
        match self {
            Self::X25519(identity) => identity.unwrap_stanza(stanza),
            Self::Hybrid(identity) => identity.unwrap_stanza(stanza),
            Self::Scrypt(identity) => identity.unwrap_stanza(stanza),
        }
    }
}

const MEGABYTE: usize = 1024 * 1024;
/// every time a new chunk is encrypted, the callback will be run with the amount of bytes that were encrypted
pub async fn encrypt_file<F>(
    recipients: &Vec<WildcardRecipient>,
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
        recipients.iter().map(|recipient| recipient as _), // bro wtf
    )
    .expect("encryptor initialization failed");

    let mut writer = encryptor
        .wrap_async_output(file_writer)
        .await
        .map_err(|e| e.to_string())?;

    let mut buffer = vec![0u8; MEGABYTE * 16]; // 16 MB buffer

    loop {
        let n = reader.read(&mut buffer).await.map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        writer
            .write_all(&buffer[..n])
            .await
            .map_err(|e| e.to_string())?; // only write the new bytes
        callback(n); // this is not a critical function
    }

    writer.close().await.map_err(|e| e.to_string())?;
    Ok(encrypted_output)
}

pub async fn decrypt_armored_text(
    identity: &WildcardIdentity,
    text: String,
) -> Result<String, String> {
    let decryptor = Decryptor::new_async_buffered(age::armor::ArmoredReader::from_async_reader(
        &text.as_bytes()[..],
    ))
    .await
    .map_err(|e| e.to_string())?;
    let mut reader = match identity {
        WildcardIdentity::Hybrid(hybrid_identity) => decryptor.decrypt_async(
            vec![
                identity as _,
                &WildcardIdentity::X25519(hybrid_identity.to_x25519()) as _,
            ]
            .into_iter(),
        ),
        _ => decryptor.decrypt_async(std::iter::once(identity as _)),
    }
    .map_err(|e| e.to_string())?;
    let mut decrypted = vec![];
    reader
        .read_to_end(&mut decrypted)
        .await
        .map_err(|e| e.to_string())?;
    Ok(String::from_utf8(decrypted).map_err(|e| e.to_string())?)
}

pub async fn encrypt_armored_text(
    recipients: &Vec<WildcardRecipient>,
    text: String,
) -> Result<String, String> {
    let mut encrypted = vec![];
    let encryptor = age::Encryptor::with_recipients(
        recipients.iter().map(|recipient| recipient as _), // bro wtf
    )
    .expect("encryptor initialization failed");
    let mut writer = age::armor::ArmoredWriter::wrap_async_output(
        &mut encrypted,
        age::armor::Format::AsciiArmor,
    );
    let mut writer = encryptor
        .wrap_async_output(&mut writer)
        .await
        .map_err(|e| e.to_string())?;
    writer
        .write_all(text.as_bytes())
        .await
        .map_err(|e| e.to_string())?;
    writer.close().await.map_err(|e| e.to_string())?;
    Ok(String::from_utf8(encrypted).map_err(|e| e.to_string())?)
}

pub async fn decrypt_file<F>(
    identity: &WildcardIdentity,
    file_path: &PathBuf,
    armor: bool,
    mut callback: F,
) -> Result<PathBuf, String>
where
    F: FnMut(usize) + Send,
{
    let mut file = File::open(file_path).await.expect("failed to open file");
    let file_size = file.metadata().await.map_err(|e| e.to_string())?.len() as usize;
    if armor && file_size > MEGABYTE * 100 {
        return Err("armored files over 100 MB are not supported".to_string());
    }
    let mut contents = Vec::with_capacity(MEGABYTE * if armor { 100 } else { 0 }); // don't allocate anything if not necessary
    let reader: Box<dyn futures_io::AsyncBufRead + Unpin + Send + Sync> = if armor {
        file.read_to_end(&mut contents)
            .await
            .map_err(|e| e.to_string())?;
        Box::new(age::armor::ArmoredReader::from_async_reader(&contents[..]))
    } else {
        drop(contents);
        Box::new(BufReader::new(file).compat())
    };

    let decryptor: Decryptor<Box<dyn futures_io::AsyncBufRead + Unpin + Send + Sync>> =
        Decryptor::new_async_buffered(reader)
            .await
            .map_err(|e| e.to_string())?;

    let decrypted_output = file_path.with_extension("");
    let output = File::create(&decrypted_output)
        .await
        .map_err(|e| e.to_string())?;
    let mut file_writer = BufWriter::new(output);

    let mut decrypted_reader = {
        let result = match identity {
            WildcardIdentity::Hybrid(hybrid_identity) => decryptor.decrypt_async(
                vec![
                    identity as _,
                    &WildcardIdentity::X25519(hybrid_identity.to_x25519()) as _,
                ]
                .into_iter(),
            ),
            _ => decryptor.decrypt_async(std::iter::once(identity as _)),
        };
        if let Ok(decryptor_reader) = result {
            decryptor_reader
        } else {
            return Err(format!(
                "decryption failed: {}",
                result.err().unwrap().to_string()
            ));
        }
    };

    let target_size = MEGABYTE * 4; // only send at most every 4MB
    let mut accumulator: usize = 0;
    let mut buffer = vec![0u8; MEGABYTE * 16]; // 16 MB buffer

    loop {
        let n = decrypted_reader
            .read(&mut buffer)
            .await
            .map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        file_writer
            .write_all(&buffer[..n])
            .await
            .map_err(|e| e.to_string())?; // only write the new bytes
        accumulator += n;
        if accumulator >= target_size {
            callback(accumulator);
            accumulator = 0;
        }
    }
    callback(accumulator); // ensure that it's sent at some point
    Ok(decrypted_output)
}
