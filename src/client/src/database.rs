use config::Config;
use eyre::Result;

pub trait Database {
    fn new(config: &Config) -> Result<Self>
    where
        Self: Sized;
    fn save_checkpoint(&self, checkpoint: Vec<u8>) -> Result<()>;
    fn load_checkpoint(&self) -> Result<Vec<u8>>;
}

pub struct ConfigDB {
    checkpoint: Vec<u8>,
}

impl Database for ConfigDB {
    fn new(config: &Config) -> Result<Self> {
        Ok(Self {
            checkpoint: config
                .checkpoint
                .clone()
                .unwrap_or(config.default_checkpoint.clone()),
        })
    }

    fn load_checkpoint(&self) -> Result<Vec<u8>> {
        Ok(self.checkpoint.clone())
    }

    fn save_checkpoint(&self, _checkpoint: Vec<u8>) -> Result<()> {
        Ok(())
    }
}
