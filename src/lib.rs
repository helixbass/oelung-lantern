pub mod mpsc;
mod receive;
pub mod spinner;

pub use proc_macros::generate_sender;

pub use receive::ReceiveEvent;
pub use spinner::{BallsSpinner, BarSpinner, BounceSpinner, PhaseSpinner, SnakeSpinner};
