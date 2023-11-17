use smaf::{Smaf, SmafChunk};

#[derive(Default)]
pub struct Audio {}

impl Audio {
    pub fn new() -> Self {
        Self {}
    }

    pub fn load_smaf(&mut self, data: &[u8]) -> anyhow::Result<()> {
        let smaf = Smaf::new(data)?;

        for chunk in &smaf.chunks {
            match chunk {
                SmafChunk::ContentsInfo(_) => {}
                SmafChunk::OptionalData(_) => {}
                SmafChunk::ScoreTrack(x, _) => {
                    tracing::info!("Loaded ScoreTrack({})", x)
                }
                SmafChunk::PCMAudioTrack(x, _) => {
                    tracing::info!("Loaded PcmTrack({})", x)
                }
            }
        }

        Ok(())
    }
}
