# chiffrage

a desktop UI for [age](https://github.com/FiloSottile/age) encryption.

built with tauri and sveltekit

<details><summary>screenshots</summary>
<p>

<img src="/screenshots/homepage.png"/>

<img src="/screenshots/encrypt.png"/>

</p>
</details> 

## note on mac usage

i don't have a developer license, so it'll complain that the app is damaged or something when you try to run it

to get around this, you need to run this once:
```sh
xattr -c /Applications/chiffrage.app
```

## todo
- [x] secure storage backend in tauri
- [x] age backend in tauri
- [x] complete frontend
- [x] implement key management
    - [x] keypair generation
    - [ ] metadata editor + notes
    - [x] key export
    - [x] key import
- [ ] encrypt folders
- [x] encrypt multiple files
- [ ] password encryption
- [x] automated builds

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

all data is serialized with CBOR

### rust backend

keys are stored in a database. key metadata looks like this:

```rust
pub enum KeyType {
    Public,
    Private,
}

pub struct KeyMetadata {
    pub id: String,
    pub name: String,
    pub key_type: KeyType,
    pub date_created: SystemTime,
    pub contents: KeyPair,
}

pub struct KeyPair {
    pub public: String,
    pub private: Option<EncryptedSecret>,
}

pub struct EncryptedSecret {
    nonce: Vec<u8>,
    ciphertext: Vec<u8>,
}
```

the private key database uses XChaCha20Poly1350 to encrypt the keys. it's structured like so:
```json
{
  "salt": "<128-bit salt>",
  "hello": "dummy encrypted value",
  "secrets": {
    "private_key": {
      "contents": {
        "private": {
          "nonce": "...",
          "ciphertext": "..."
        },
        "public": "..."
      },
      "key_type": "Private",
      "date_created": "..."
    },
    "public_key": {
      "contents": {
        "private": null,
        "public": "..."
      },
      "key_type": "Public",
      "date_created": "..."
    }
  }
}
```

key type in typescript (this is in the rust too, but json/ts are more readable):

```ts
export type Key = {
  id: string;
  name: string;
  key_type: "public" | "private";
  date_created: { secs_since_epoch: number }; // pretty sure there's also millis_since_epoch
  contents: {
    public: String;
    private: {
      nonce: number[];
      ciphertext: number[];
    } | null;
  };
};
```

the `hello` entry is used to authenticate the user. the decrypted value is always `"hello"` and is checked during vault unlock.

all secrets (e.g. the vault key, private keys) are held in memory using the [secrecy](https://docs.rs/secrecy/latest/secrecy/) crate, which requires explicit exposing (via `SecretBox::expose_secret()`) and will zero memory when these values are dropped with [zeroize](https://docs.rs/zeroize/latest/zeroize/)

### sveltekit frontend

no cryptography or handling of secrets occurs on the frontend, and never should.

the finish frontend will likely be modeled after that of [kleopatra](https://apps.kde.org/kleopatra/).
different actions (e.g. creating a new keypair from the homepage) should happen in a new window

i plan to use as little external javascript/css as possible.
