pub mod cli;
pub mod simulator;

pub mod prelude {
    pub use crate::{cli, simulator};
    pub use simulator::{print_result, simulate, types};
}
