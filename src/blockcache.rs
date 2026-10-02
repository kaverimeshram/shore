use std::collections::BTreeMap;
use std::sync::Mutex;

use async_trait::async_trait;
use zcash_client_backend::data_api::chain::{error, BlockCache, BlockSource};
use zcash_client_backend::data_api::scanning::ScanRange;
use zcash_client_backend::proto::compact_formats::CompactBlock;
use zcash_protocol::consensus::BlockHeight;

#[derive(Default)]
pub struct MemBlockCache(Mutex<BTreeMap<u32, CompactBlock>>);

impl BlockSource for MemBlockCache {
    type Error = std::convert::Infallible;

    fn with_blocks<F, WalletErrT>(
        &self,
        from_height: Option<BlockHeight>,
        limit: Option<usize>,
        mut with_block: F,
    ) -> Result<(), error::Error<WalletErrT, Self::Error>>
    where
        F: FnMut(CompactBlock) -> Result<(), error::Error<WalletErrT, Self::Error>>,
    {
        let from = from_height.map_or(0, u32::from);
        let blocks: Vec<CompactBlock> = self
            .0
            .lock()
            .unwrap()
            .range(from..)
            .take(limit.unwrap_or(usize::MAX))
            .map(|(_, b)| b.clone())
            .collect();
        blocks.into_iter().try_for_each(&mut with_block)
    }
}

#[async_trait]
impl BlockCache for MemBlockCache {
    fn get_tip_height(&self, range: Option<&ScanRange>) -> Result<Option<BlockHeight>, Self::Error> {
        let m = self.0.lock().unwrap();
        let tip = match range {
            Some(r) => m
                .range(u32::from(r.block_range().start)..u32::from(r.block_range().end))
                .next_back(),
            None => m.iter().next_back(),
        };
        Ok(tip.map(|(h, _)| BlockHeight::from_u32(*h)))
    }

    async fn read(&self, range: &ScanRange) -> Result<Vec<CompactBlock>, Self::Error> {
        let r = range.block_range();
        Ok(self
            .0
            .lock()
            .unwrap()
            .range(u32::from(r.start)..u32::from(r.end))
            .map(|(_, b)| b.clone())
            .collect())
    }

    async fn insert(&self, blocks: Vec<CompactBlock>) -> Result<(), Self::Error> {
        let mut m = self.0.lock().unwrap();
        for b in blocks {
            m.insert(b.height as u32, b);
        }
        Ok(())
    }

    async fn delete(&self, range: ScanRange) -> Result<(), Self::Error> {
        let r = range.block_range();
        self.0
            .lock()
            .unwrap()
            .retain(|h, _| *h < u32::from(r.start) || *h >= u32::from(r.end));
        Ok(())
    }
}
