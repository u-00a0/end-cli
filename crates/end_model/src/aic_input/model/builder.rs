use std::collections::HashSet;

use generativity::Guard;

use super::{
    AicBuildError, AicInputs, ItemNonZeroU32Map, OutpostId, OutpostInput, Region,
    Stage2Objective,
};

#[derive(Debug)]
pub struct AicInputsBuilder<'cid, 'sid> {
    region: Region,
    supply_per_min: ItemNonZeroU32Map<'cid>,
    external_consumption_per_min: ItemNonZeroU32Map<'cid>,
    outposts: Vec<OutpostInput<'cid>>,
    outpost_keys: HashSet<crate::Key>,
    external_power_consumption_w: u32,
    stage2_objective: Stage2Objective,
    scenario_brand: generativity::Id<'sid>,
}

impl<'cid, 'sid> AicInputs<'cid, 'sid> {
    pub fn builder(
        guard: Guard<'sid>,
        external_power_consumption_w: u32,
        supply_per_min: ItemNonZeroU32Map<'cid>,
        external_consumption_per_min: ItemNonZeroU32Map<'cid>,
    ) -> AicInputsBuilder<'cid, 'sid> {
        AicInputsBuilder {
            region: Region::FourthValley,
            supply_per_min,
            external_consumption_per_min,
            outposts: Vec::new(),
            outpost_keys: HashSet::new(),
            external_power_consumption_w,
            stage2_objective: Stage2Objective::default(),
            scenario_brand: guard.into(),
        }
    }
}

impl<'cid, 'sid> AicInputsBuilder<'cid, 'sid> {
    pub fn region(mut self, region: Region) -> Self {
        self.region = region;
        self
    }

    pub fn stage2_objective(mut self, stage2_objective: Stage2Objective) -> Self {
        self.stage2_objective = stage2_objective;
        self
    }

    pub fn add_outpost(
        &mut self,
        outpost: OutpostInput<'cid>,
    ) -> Result<OutpostId<'sid>, AicBuildError> {
        if !self.outpost_keys.insert(outpost.key.clone()) {
            return Err(AicBuildError::DuplicateOutpostKey { key: outpost.key });
        }

        let id = OutpostId::from_index(self.outposts.len(), self.scenario_brand);
        self.outposts.push(outpost);
        Ok(id)
    }

    pub fn build(self) -> AicInputs<'cid, 'sid> {
        let Self {
            region,
            supply_per_min,
            external_consumption_per_min,
            outposts,
            outpost_keys: _,
            external_power_consumption_w,
            stage2_objective,
            scenario_brand,
        } = self;

        AicInputs {
            region,
            supply_per_min,
            external_consumption_per_min,
            outposts: outposts.into_boxed_slice(),
            external_power_consumption_w,
            stage2_objective,
            scenario_brand,
        }
    }
}
