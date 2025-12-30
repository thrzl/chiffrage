# security policy

## cryptographic implementation

chiffrage implements the [age encryption format](https://age-encryption.org/v1) with support for:

- **x25519** - standard elliptic curve diffie-hellman key exchange
- **ml-kem768-x25519 (x-wing)** - post-quantum hybrid encryption using [draft-connolly-cfrg-xwing-kem-06](https://datatracker.ietf.org/doc/html/draft-connolly-cfrg-xwing-kem-06)
- **scrypt** - password-based encryption

### key storage

private keys are stored encrypted in a local vault using:

- **xchacha20-poly1305** for authenticated encryption
- **argon2** for vault key derivation from passwords
- **secrecy** and **zeroize** crates to zero the memory of secrets and prevent accidental exposure

vault structure is serialized with cbor and stored at the platform-specific app data directory.

### important security notes

**post-quantum security**: when encrypting to both x25519 and ml-kem768-x25519 keys in a single operation, pq keys are downgraded to x25519 for compatibility. to maintain post-quantum security, encrypt only to ml-kem768-x25519 recipients.

**no authentication by default**: age provides encryption but not authentication. anyone with access to recipient public keys can create valid encrypted files. for authenticated encryption, use separate signing tools.

## build verification

releases are built directly from source on github actions, with:

- **build provenance**: cryptographically verifies that the binary was built from the commit listed
- **[sigstore](https://www.sigstore.dev/) signing**: cryptographically verifies that the binary was built on github actions and was not tampered with

## threat model

chiffrage protects against:

- eavesdropping on encrypted files
- unauthorized access to stored private keys (with strong vault password)
- future quantum computer attacks (when using ml-kem768-x25519)

chiffrage does **not** protect against:

- malware with access to vault key in memory
- keyloggers capturing vault password
- physical access to unlocked vault
- side-channel attacks on the host system

## reporting vulnerabilities

**do not** open public github issues for security vulnerabilities.

instead, please [privately report a vulnerability](https://github.com/thrzl/chiffrage/security).

include:

- description of the vulnerability
- steps to reproduce
- potential impact
- suggested fix (if any)

i'll respond promptly and work with you on a fix before public disclosure.

## security best practices

### for users

- use strong, unique passwords for your vault
- lock your vault when not in use
- keep your private keys in the vault only
- backup your vault file separately
- verify recipients before encrypting sensitive data

### for contributors

- never commit secrets or test keys
- use `bun audit` and `cargo audit` before submitting prs
- follow rust security guidelines
- keep dependencies updated
- use `secrecy::SecretBox` for all sensitive data

## dependencies

security-critical dependencies are monitored via dependabot. key crypto libraries:

- `rage` - file encryption format implementation
- `chacha20poly1305` - authenticated encryption
- `argon2` - password hashing
- `libcrux-ml-kem` - post-quantum kem
- `x25519-dalek` - elliptic curve operations

## audit status

chiffrage has not undergone a formal security audit. the underlying `age` and `rage` implementations are more mature and battle-tested.
