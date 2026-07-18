//! B.U.D./EVM cross-domain — F10 EVM ChainAdapter (RFC_F10_EVM_CHAIN_ADAPTER).
//!
//! Gerçek EVM receipt doğrulaması için in-tree kriptografik bileşenler:
//! - [`rlp`]: RLP (Recursive Length Prefix) encode/decode (Ethereum Yellow
//!   Paper Appendix B). Verifier bağlamında **strict canonical** — non-canonical
//!   encoding malleability saldırılarına kapalı.
//! - (F10.1b) `mpt`: Merkle-Patricia trie verifier (Ethereum Appendix D).
//! - (F10.2) `adapter`: `EvmChainAdapter` (StubAdapter'ın gerçek hali).
//!
//! **Kullanıcı kararı (RFC Q3 = in_tree):** alloy/ethers/trie crate'leri YOK;
//! RLP + MPT mantığı in-tree. Keccak256 primitive olarak mevcut `sha3` crate'i
//! (zaten dep) kullanılır (`sha3::Keccak256`) — bu "yeni dep" sayılmaz, minimal-
//! dep kuralı korunur.
//!
//! **Güvenlik notu:** Bu modüller konsensüs doğrulama yolundadır (deterministik,
//! network'süz). Non-canonical encoding, truncation, ve malleability RED.

pub mod rlp;
