#[cfg(test)]
mod tests {
    use crate::chain::blockchain::Blockchain;
    use crate::consensus::pow::PoWEngine;
    use crate::core::address::Address;
    use crate::core::transaction::{Transaction, TransactionType};
    use crate::storage::db::Storage;
    use std::sync::Arc;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_chaos_v2_disaster_recovery_full_state() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let db_path = temp_dir.path().join("dr_test.db");
        let db_path_str = db_path.to_str().unwrap();

        let alice = Address::from([0xAA; 32]);
        let cid = crate::storage::content_id::ContentId([0x42; 32]);

        // 1. Initial Setup and State Creation
        {
            let storage = Storage::new(db_path_str).expect("failed to open storage");
            let consensus = Arc::new(PoWEngine::new(0));
            let mut bc = Blockchain::new(consensus, Some(storage), 1337, None);

            // Fund Alice
            bc.state.add_balance(&alice, 1_000_000);

            // Register a BNS name
            let bns_data = bincode::serialize(&("ayaz.bud".to_string(), 100u64)).unwrap();
            let mut bns_tx = Transaction::new(alice, Address::zero(), 10000, bns_data);
            bns_tx.tx_type = TransactionType::BnsRegister;
            bc.add_transaction(bns_tx).unwrap();

            // Mint an NFT (SocialFi)
            let nft_data = bincode::serialize(&(cid, Some("ayaz.bud".to_string()))).unwrap();
            let mut nft_tx = Transaction::new(alice, Address::zero(), 0, nft_data);
            nft_tx.tx_type = TransactionType::NftMint;
            bc.add_transaction(nft_tx).unwrap();

            // Produce a block to persist state
            bc.produce_block(Address::zero());
            
            assert!(bc.state.bns_registry.resolve("ayaz.bud", 0).is_some());
            assert_eq!(bc.state.nft_registry.nfts.len(), 1);

            // FORCE HALT: bc and storage are dropped here
            info!("SIMULATED CRASH: Node process killed.");
        }

        // 2. Recovery from Disk
        {
            info!("RECOVERY: Starting node from disk...");
            let storage = Storage::new(db_path_str).expect("failed to open storage");
            let consensus = Arc::new(PoWEngine::new(0));
            
            // Reconstruct blockchain from existing storage
            let bc = Blockchain::new(consensus, Some(storage), 1337, None);

            // 3. Verify Integrity of "Universal Consensus Layer"
            
            // Verify BNS survived
            let resolved = bc.state.bns_registry.resolve("ayaz.bud", 0);
            assert_eq!(resolved, Some(alice), "BNS record must survive crash");

            // Verify NFT survived
            assert_eq!(bc.state.nft_registry.nfts.len(), 1, "NFT records must survive crash");
            let nft = bc.state.nft_registry.get_nft(0).unwrap();
            assert_eq!(nft.content_id, cid);
            assert_eq!(nft.owner, alice);

            // Verify Balances
            let balance = bc.state.get_balance(&alice);
            assert!(balance > 0, "Alice's balance must survive crash");

            info!("SUCCESS: Disaster Recovery verified. Budlum is immortal.");
        }
    }

    #[tokio::test]
    async fn test_chaos_v2_nft_burn_pruning_after_restart() {
        let temp_dir = tempdir().expect("failed to create temp dir");
        let db_path = temp_dir.path().join("pruning_test.db");
        let db_path_str = db_path.to_str().unwrap();

        let alice = Address::from([0xAA; 32]);
        let cid = crate::storage::content_id::ContentId([0xEE; 32]);

        // 1. Create NFT
        {
            let storage = Storage::new(db_path_str).unwrap();
            let consensus = Arc::new(PoWEngine::new(0));
            let mut bc = Blockchain::new(consensus, Some(storage), 1337, None);
            bc.state.add_balance(&alice, 1000);

            let nft_data = bincode::serialize(&(cid, None::<String>)).unwrap();
            let mut nft_tx = Transaction::new(alice, Address::zero(), 0, nft_data);
            nft_tx.tx_type = TransactionType::NftMint;
            bc.add_transaction(nft_tx).unwrap();
            bc.produce_block(Address::zero());
        }

        // 2. Burn NFT and Simulate Pruning Signal
        {
            let storage = Storage::new(db_path_str).unwrap();
            let consensus = Arc::new(PoWEngine::new(0));
            let mut bc = Blockchain::new(consensus, Some(storage), 1337, None);

            let burn_data = bincode::serialize(&0u64).unwrap(); // nft_id 0
            let mut burn_tx = Transaction::new(alice, Address::zero(), 0, burn_data);
            burn_tx.tx_type = TransactionType::NftBurn;
            
            // The executor emits a tracing signal here
            bc.add_transaction(burn_tx).unwrap();
            bc.produce_block(Address::zero());
            
            assert_eq!(bc.state.nft_registry.nfts.len(), 0, "NFT must be burned in state");
        }

        // 3. Verify State consistency after another restart
        {
            let storage = Storage::new(db_path_str).unwrap();
            let consensus = Arc::new(PoWEngine::new(0));
            let bc = Blockchain::new(consensus, Some(storage), 1337, None);
            
            assert_eq!(bc.state.nft_registry.nfts.len(), 0, "NFT burn must be persistent");
        }
    }
}

use tracing::info;
