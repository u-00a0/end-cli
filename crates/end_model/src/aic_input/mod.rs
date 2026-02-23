mod model;

pub use model::{
    AicBuildError, AicInputs, AicInputsBuilder, ItemNonZeroU32Map, ItemU32Map, OutpostId,
    OutpostInput, Region, Stage2Objective, Stage2WeightedWeights,
};

#[cfg(test)]
mod tests;
