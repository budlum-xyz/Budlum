# BLS/PQ HSM Policy (ADIM 2 §1.1)

**Tarih:** 2026-07-15  
**Durum:** Fail-closed policy gate + signer capability yüzeyi eklendi. Donanım-vendor native BLS/Dilithium mekanizma entegrasyonu ayrı, daha ileri bir iştir.  
**Kod:** `src/crypto/signer.rs`, `src/crypto/pkcs11.rs`, `src/main.rs`, `src/cli/commands.rs`

> Bu belge “mock HSM” değildir ve mainnet için sahte-yeşil iddia üretmez.
> Amaç: disk-backed `ValidatorKeys` yolunu mainnet’te kapalı tutmak, PKCS#11
> backend’in Ed25519 yanında BLS ve Dilithium/PQ materyali de taşıdığını runtime’da
> doğrulamak ve eksik capability’de node’u fail-closed durdurmaktır.

## Politika özeti

| Ortam | İzinli validator key yolu | BLS/PQ davranışı |
|-------|---------------------------|------------------|
| Devnet/testnet | `ValidatorKeys` dosyası veya PKCS#11 | Disk key kabul edilebilir; test amaçlıdır. |
| Mainnet validator | Sadece `validator.signer.backend = "pkcs11"` | PKCS#11 backend Ed25519 + BLS + Dilithium/PQ capability bildirmelidir. |
| Mainnet validator + disk `ValidatorKeys` | Yasak | Process fail-closed çıkar. |
| Mainnet validator + Ed25519-only HSM | Yasak | Process fail-closed çıkar. |

## Kod sınırları

### `ConsensusSigner` capability yüzeyi

`src/crypto/signer.rs` trait’i artık şunları sunar:

- `bls_public_key() -> Option<Vec<u8>>`
- `pq_public_key() -> Option<Vec<u8>>`
- `has_bls_key() -> bool`
- `has_pq_key() -> bool`

Bu sayede policy kodu, secret key istemeden backend’in BLS/PQ capability’sini
kontrol edebilir.

### PKCS#11 backend

`src/crypto/pkcs11.rs`:

- Ed25519 blok imzası PKCS#11 private key üzerinden yapılır.
- BLS ve Dilithium/PQ materyali `BUD_BLS_KEY` / `BUD_PQ_KEY` label’lı private
  PKCS#11 data object olarak aranır.
- ADIM 2 §1.1 kapsamında bu path **disk ValidatorKeys yerine PKCS#11-backed key
  inventory** sağlar; vendor-native non-extractable BLS/Dilithium sign mekanizması
  henüz iddia edilmez.

### Runtime fail-closed gate

`src/main.rs` mainnet validator başlatırken:

1. PKCS#11 backend’i zorunlu kılar.
2. Disk `ValidatorKeys` dosyasını reddeder.
3. PKCS#11 signer `has_bls_key()` ve `has_pq_key()` sağlamazsa process’i durdurur.

## Operatör yapılandırması

`config/mainnet.toml` örneği:

```toml
[validator]
backend = "pkcs11"

[validator.signer]
backend = "pkcs11"

[validator.signer.pkcs11]
module_path = "/path/to/vendor-pkcs11.so"
slot_id = 0
token_pin_env = "BUDLUM_PKCS11_TOKEN_PIN"
```

PIN değeri repo, CLI argümanı veya log’a yazılmaz; yalnızca environment/secret
manager üzerinden verilir.

## Doğrulama

```bash
cargo test --lib keypair_signer_advertises_bls_pq_capabilities_only_when_bound
cargo test --lib default_consensus_signer_rejects_missing_bls_pq_material
cargo clippy --lib --tests -- -D warnings
cargo fmt --all -- --check
```

Bu Arena sandbox’ında `cargo`/`rustc` yoksa PR CI zorunlu kanıt kabul edilir.

## Kabul kriteri

- [x] Disk-backed `ValidatorKeys` mainnet validator için reddedilir.
- [x] Ed25519-only PKCS#11 backend mainnet validator için reddedilir.
- [x] Signer capability metotları secret key sızdırmadan BLS/PQ public inventory sağlar.
- [x] BLS prevote/precommit imzalama path’i local secret yoksa signer backend’e düşebilir.
- [x] Mock HSM production koduna eklenmedi.
- [ ] Vendor-native non-extractable BLS/Dilithium PKCS#11 mekanizma entegrasyonu ayrıca yapılacak.

## Sınırlar / yapılmayanlar

- Harici audit yapılmadı.
- BLS/Dilithium için vendor-specific native PKCS#11 mechanism desteği iddia edilmez.
- B.U.D. Proof-of-Storage Faz 3 ile ilişkili değildir.
- Bu policy, `VerifyMerkle` Z-B gate’ini açmaz.
