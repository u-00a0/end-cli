mod builder;
mod types;

pub use builder::AicInputsBuilder;
pub use types::{
    AicBuildError, AicInputs, ItemNonZeroU32Map, ItemU32Map, OutpostId, OutpostInput,
    Region, Stage2Objective, Stage2WeightedWeights,
};

impl<'cid, 'sid> AicInputs<'cid, 'sid> {
    pub fn external_power_consumption_w(&self) -> u32 {
        self.external_power_consumption_w
    }

    pub fn stage2_objective(&self) -> Stage2Objective {
        self.stage2_objective
    }

    pub fn region(&self) -> Region {
        self.region
    }

    pub fn supply_per_min(&self) -> &ItemNonZeroU32Map<'cid> {
        &self.supply_per_min
    }

    pub fn external_consumption_per_min(&self) -> &ItemNonZeroU32Map<'cid> {
        &self.external_consumption_per_min
    }

    pub fn outposts(&self) -> &[OutpostInput<'cid>] {
        &self.outposts
    }

    pub fn outpost(&self, id: OutpostId<'sid>) -> Option<&OutpostInput<'cid>> {
        self.outposts.get(id.index())
    }

    pub fn outposts_with_id(
        &self,
    ) -> impl Iterator<Item = (OutpostId<'sid>, &OutpostInput<'cid>)> + '_ {
        let brand = self.scenario_brand;
        self.outposts
            .iter()
            .enumerate()
            .map(move |(index, outpost)| (OutpostId::from_index(index, brand), outpost))
    }
}
