use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("StorybookBuilder: {0}")]
    StorybookBuilder(String),
}
