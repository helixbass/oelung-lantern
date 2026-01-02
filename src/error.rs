use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("AnimatedGradientBuilder: {0}")]
    AnimatedGradientBuilder(String),
    #[error("StorybookBuilder: {0}")]
    StorybookBuilder(String),
}
