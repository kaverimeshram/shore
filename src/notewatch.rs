//! View-only note watcher: scans lightwalletd compact blocks with a UFVK (never a spending
//! key) and reports the notes that landed, with txid and mined height read from the wallet db.
use crate::blockcache::MemBlockCache;
use anyhow::{anyhow, Context, Result};
use rand::{rngs::SysRng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use std::path::Path;
use zcash_client_backend::{
    data_api::{AccountBirthday, AccountPurpose, WalletRead, WalletWrite},
    proto::service::{compact_tx_streamer_client::CompactTxStreamerClient, BlockId, ChainSpec},
};
use zcash_client_sqlite::{util::SystemClock, wallet::init::init_wallet_db, WalletDb};
use zcash_keys::keys::UnifiedFullViewingKey;
use zcash_protocol::consensus::Network;

type Db = WalletDb<rusqlite::Connection, Network, SystemClock, ChaCha20Rng>;
type Lwd = CompactTxStreamerClient<tonic::transport::Channel>;

const MAX_DECODE: usize = 64 * 1024 * 1024;

pub fn endpoint(n: Network) -> &'static str {
    match n {
        Network::MainNetwork => "https://zec.rocks:443",
        Network::TestNetwork => "https://testnet.zec.rocks:443",
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Landed {
    pub pool: String,
    pub value_zat: u64,
    pub txid: String,
    pub mined_height: Option<u32>,
}

pub struct NoteWatch {
    pub(crate) network: Network,
    pub(crate) db: Db,
    path: std::path::PathBuf,
}

pub(crate) async fn client(n: Network) -> Result<Lwd> {
    let ch = tonic::transport::Channel::from_shared(endpoint(n).to_string())?
        .tls_config(tonic::transport::ClientTlsConfig::new().with_webpki_roots())?
        .connect()
        .await
        .context("connect lightwalletd")?;
    Ok(CompactTxStreamerClient::new(ch).max_decoding_message_size(MAX_DECODE).max_encoding_message_size(MAX_DECODE))
}

pub async fn tip(n: Network) -> Result<u32> {
    Ok(client(n).await?.get_latest_block(ChainSpec {}).await?.into_inner().height as u32)
}

/// Height at which `txid_hex` was mined, read through lightwalletd `GetTransaction`.
pub async fn mined_height(n: Network, txid_hex: &str) -> Result<u64> {
    use zcash_client_backend::proto::service::TxFilter;
    let mut wire = hex::decode(txid_hex).context("txid is not hex")?;
    wire.reverse();
    let tx = client(n).await?.get_transaction(TxFilter { block: None, index: 0, hash: wire }).await?.into_inner();
    if tx.height == 0 || tx.height == u64::MAX {
        anyhow::bail!("{txid_hex} is not mined (mempool or unknown)");
    }
    Ok(tx.height)
}

impl NoteWatch {
    /// Opens (creating on first use) the watcher db at `path`. `birthday` is the first height to
    /// scan; a fresh db starts there so only notes at or after it are seen.
    pub async fn open(path: &Path, network: Network, ufvk: &str, birthday: u32) -> Result<Self> {
        let parsed = UnifiedFullViewingKey::decode(&network, ufvk).map_err(|e| anyhow!("ufvk: {e}"))?;
        let fresh = !path.exists();
        let rng = ChaCha20Rng::try_from_rng(&mut SysRng).map_err(|e| anyhow!("os rng: {e:?}"))?;
        let mut db = WalletDb::for_path(path, network, SystemClock, rng)?;
        // No seed is passed: this db is view-only by construction.
        init_wallet_db(&mut db, None).map_err(|e| anyhow!("init_wallet_db: {e:?}"))?;
        if fresh {
            let ts = client(network)
                .await?
                .get_tree_state(BlockId { height: birthday.saturating_sub(1) as u64, hash: vec![] })
                .await?
                .into_inner();
            let b = AccountBirthday::from_treestate(ts, None).map_err(|e| anyhow!("birthday: {e:?}"))?;
            db.import_account_ufvk("airlock", &parsed, &b, AccountPurpose::ViewOnly, None)
                .map_err(|e| anyhow!("import_account_ufvk: {e}"))?;
        }
        Ok(Self { network, db, path: path.to_path_buf() })
    }

    pub async fn sync(&mut self) -> Result<u32> {
        let mut c = client(self.network).await?;
        zcash_client_backend::sync::run(&mut c, &self.network, &MemBlockCache::default(), &mut self.db, 1000)
            .await
            .map_err(|e| anyhow!("sync: {e}"))?;
        let s = self.db.get_wallet_summary(zcash_client_backend::data_api::wallet::ConfirmationsPolicy::MIN).map_err(|e| anyhow!("{e}"))?;
        Ok(s.map_or(0, |s| u32::from(s.fully_scanned_height())))
    }

    /// Notes the wallet has received, from the db tables the sync filled.
    pub fn landed(&self) -> Result<Vec<Landed>> {
        let conn = rusqlite::Connection::open_with_flags(&self.path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name LIKE '%\\_received\\_notes' ESCAPE '\\'")?
            .query_map([], |r| r.get(0))?
            .collect::<std::result::Result<_, _>>()?;
        let mut out = Vec::new();
        for t in tables {
            let pool = t.trim_end_matches("_received_notes").to_string();
            let q = format!("SELECT n.value, hex(t.txid), t.mined_height FROM {t} n JOIN transactions t ON t.id_tx = n.transaction_id");
            let rows = conn.prepare(&q)?.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, Option<u32>>(2)?)))?.collect::<std::result::Result<Vec<_>, _>>()?;
            for (v, txid_hex, h) in rows {
                // sqlite stores txids in wire order; the displayed txid is byte-reversed
                let mut b = hex::decode(&txid_hex)?;
                b.reverse();
                out.push(Landed { pool: pool.clone(), value_zat: v as u64, txid: hex::encode(b), mined_height: h });
            }
        }
        Ok(out)
    }
}
