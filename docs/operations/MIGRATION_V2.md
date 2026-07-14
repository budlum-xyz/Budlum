# ConsensusStateV2 Migration İskeleti (ADIM 2 §1.4)

**Tarih:** 2026-07-15  
**Durum:** Minimum migration hook + offline gate mevcut; canlı state transform yok.  
**Kod:** `src/chain/snapshot.rs`, `src/main.rs`, `src/cli/commands.rs`

> Amaç: mainnet öncesi state şema geçişlerinin “sessizce kabul” yerine
> fail-closed, backup-zorunlu ve audit edilebilir bir kapıdan geçmesini sağlamak.

## Sürüm penceresi

`src/chain/snapshot.rs` içinde iki sabit vardır:

| Sabit | Değer | Anlam |
|-------|-------|-------|
| `MIN_SUPPORTED_STATE_SNAPSHOT_SCHEMA_VERSION` | `2` | Bu binary’nin kabul ettiği en eski V2 snapshot şeması. |
| `CURRENT_STATE_SNAPSHOT_SCHEMA_VERSION` | `3` | Bu binary’nin ürettiği güncel durable snapshot şeması. |

Desteklenmeyen şemalar fail-closed reddedilir:

- `schema_version < 2` → legacy snapshot reddedilir; ara release ile restore gerekir.
- `schema_version > 3` → future snapshot reddedilir; downgrade/yanlış binary riski.

## Kod iskeleti

`StateSnapshotV2::migration_report()` şu raporu üretir:

- `original_schema_version`
- `target_schema_version`
- `migrated` (`schema_version < CURRENT` ise `true`)
- `requires_backup` (`true`; migration öncesi backup zorunlu)
- `notes` (schema-2 default-field kabulü veya zaten güncel notu)

`StateSnapshotV2::from_bytes()` artık doğrudan gömülü sayı kontrolü yapmaz;
aynı kontrolü bu migration hook üzerinden çağırır. Böylece gelecek şema
geçişleri tek noktadan genişletilebilir.

## Offline CLI kapısı

`src/cli/commands.rs` içinde:

```bash
budlum-core --migrate-v2 ./data/node.db --backup-dir ./data/backups
```

akışı tanımlıdır. `src/main.rs` bu modda:

1. hedef sled DB’yi açar,
2. migration öncesi atomik ve doğrulanmış backup üretir,
3. desteklenen schema penceresini raporlar,
4. canlı node başlatmadan çıkar.

Bu ADIM 2 iskeleti **veri transformasyonu yapmaz**; sadece preflight + backup +
fail-closed policy sağlar. Gerçek çok-adımlı migration gerekirse bu hook’a
explicit `v2 -> v3 -> v4` transform fonksiyonları eklenecek.

## Test kapsamı

`src/chain/snapshot.rs::tests::test_snapshot_v2_migration_hook_rejects_unsupported_versions`
şunları doğrular:

- `schema_version = 1` reddedilir,
- `schema_version = 99` reddedilir,
- `schema_version = 2` desteklenir ve `migrated = true` raporu üretir,
- güncel schema desteklenir ve `migrated = false` raporu üretir.

## Doğrulama komutları

```bash
cargo test --lib snapshot_v2_migration_hook -- --nocapture
cargo test --lib persistence -- --nocapture
cargo test --lib --verbose
```

Bu sandbox oturumunda `cargo`/`rustc` binary’si bulunmadığı için komutlar yerelde
çalıştırılamadı; PR CI sonucu zorunlu kanıt olarak izlenecektir.

## Kabul kriteri

- [x] Desteklenen schema penceresi sabitlerle merkezi tanımlı.
- [x] Unsupported legacy/future snapshot fail-closed.
- [x] Offline migration CLI preflight + zorunlu backup mevcut.
- [x] Migration raporu audit edilebilir veri döndürüyor.
- [x] Test iskeleti schema-2/current/legacy/future yollarını kapsıyor.
- [ ] CI `Budlum Core` yeşil.

## Bilinçli sınırlar

- Canlı zincirde otomatik state rewrite yok.
- Unknown future schema için “best effort” deserialize yok.
- Backup alınmadan migration kapısı başarı raporu vermez.
