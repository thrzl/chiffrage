# chiffrage

a desktop UI for [age](https://github.com/FiloSottile/age) encryption.

built with tauri and sveltekit

huge work in progress. nothing is set right now, not even database format/structure. should not be relied on in its current state.

## todo
- [x] secure storage backend in tauri*
- [ ] age backend in tauri
- [ ] complete frontend
- [ ] implement key management
- [ ] automated builds

*\*may be modified to use tauri's built-in store instead of sled*

## contributing
all contributions welcome! please open an issue mentioning what you intend to change before submitting a pull request.

there are currently no special build steps excluding tauri's (i use bun, but anything works):
```sh
bun install

# run the dev server
bun run tauri dev

# build
bun run tauri build
```

## under the hood

all data is serialized with CBOR (perhaps i'll switch to something else or simply use JSON)

### rust backend

keys are stored in an index via [sled](https://docs.rs/sled/latest/sled/) (perhaps i'll use tauri's builtin store soon?). key metadata looks like this:

```rust
pub enum KeyType {
    Public,
    Private,
}

pub struct KeyMetadata {
    pub name: String,
    pub key_type: KeyType,
    pub date_created: std::time::SystemTime,
}
```

the private key database uses XChaCha20Poly1350 to encrypt the keys. it's structured like so:
```json
{
  "salt": "<128-bit salt>",
  "hello": "dummy encrypted value",
  "secrets": {
    "key name": {
      "nonce": "...",
      "ciphertext": "..."
    }
  }
}
```

the `hello` entry is used to authenticate the user. the decrypted value is always `"hello"` and is checked during vault unlock.

all secrets (e.g. the vault key, private keys) are held in memory using the [secrecy](https://docs.rs/secrecy/latest/secrecy/) crate, which requires explicit exposing (via `SecretBox::expose_secret()`) and will zero memory when these values are dropped with [zeroize](https://docs.rs/zeroize/latest/zeroize/)

### sveltekit frontend

no cryptography or handling of secrets occurs on the frontend, and never should.

the finish frontend will likely be modeled after that of [kleopatra](https://apps.kde.org/kleopatra/).
different actions (e.g. creating a new keypair from the homepage) should happen in a new window

i plan to use as little external javascript/css as possible.
