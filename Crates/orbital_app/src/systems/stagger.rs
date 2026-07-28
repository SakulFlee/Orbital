use std::collections::HashSet;

use orbital_ecs::World;
use orbital_ecs_bridge::{
    LightSlotIndex, ShadowDirtyFlag, StaggeredLightConfig, StaggerState,
};
use orbital_resources::ShadowCaster;

/// Build the set of light store indices whose shadow maps should be
/// rendered this frame, respecting the per-frame update budget.
///
/// On each frame this:
/// 1. Finds all shadow-casting lights marked `ShadowDirtyFlag`
/// 2. Processes up to `max_updates_per_frame` dirty lights (dirty-first,
///    then round-robin through remaining clean shadows for proactive refresh)
/// 3. Clears `ShadowDirtyFlag` for processed lights
/// 4. Returns the set of `light_store_index` values whose shadows need updating
///
/// If `global_dirty` is true (indicating model/changes), all shadow-casting
/// lights are included regardless of the budget.
pub fn sys_stagger_shadow_updates(
    ecs: &mut World,
    global_dirty: bool,
) -> HashSet<u32> {
    // Ensure StaggerState exists
    if ecs.get_resource::<StaggerState>().is_none() {
        ecs.insert_resource(StaggerState::default());
    }

    let config = ecs
        .get_resource::<StaggeredLightConfig>()
        .map(|c| c.max_updates_per_frame)
        .unwrap_or(1)
        .max(1) as usize;

    let mut result = HashSet::new();

    // --- Phase 1: Collect all read-only data ---

    let dirty_eids: Vec<usize>;
    let all_shadow_with_slots: Vec<(usize, u32)>;

    {
        // Determine which entity IDs have ShadowDirtyFlag set
        let dirty_store = ecs.get_component_store::<ShadowDirtyFlag>();
        dirty_eids = match dirty_store {
            Some(ref ds) => ds
                .dense
                .iter()
                .copied()
                .filter(|&eid| {
                    ds.sparse
                        .get(eid)
                        .copied()
                        .flatten()
                        .map(|idx| ds.components[idx].is_dirty())
                        .unwrap_or(false)
                })
                .collect(),
            None => Vec::new(),
        };
    }

    {
        // Collect all shadow-casting entities with their slot indices
        let casters = ecs.get_component_store::<ShadowCaster>();
        let slot_store = ecs.get_component_store::<LightSlotIndex>();
        let mut shadow_list = Vec::new();
        if let Some(ref cs) = casters {
            if let Some(ref ss) = slot_store {
                for &eid in cs.dense.as_slice() {
                    if let Some(c_idx) = cs.sparse.get(eid).copied().flatten() {
                        if !cs.components[c_idx].enabled {
                            continue;
                        }
                        if let Some(s_idx) = ss.sparse.get(eid).copied().flatten() {
                            shadow_list.push((eid, ss.components[s_idx].0));
                        }
                    }
                }
            }
        }
        all_shadow_with_slots = shadow_list;
    }

    // --- Phase 2: Handle global dirty case ---

    if global_dirty {
        for (_, slot) in &all_shadow_with_slots {
            result.insert(*slot);
        }
        // Clear all shadow dirty flags
        if let Some(mut store) = ecs.get_component_store_mut::<ShadowDirtyFlag>() {
            for &eid in &dirty_eids {
                if let Some(idx) = store.sparse.get(eid).copied().flatten() {
                    store.components[idx].clear();
                }
            }
        }
        return result;
    }

    // --- Phase 3: Apply stagger budget ---

    let mut processed: HashSet<usize> = HashSet::new();
    let mut budget = config;

    // Process dirty entities first (priority)
    for &eid in &dirty_eids {
        if budget == 0 {
            break;
        }
        for (e, slot) in &all_shadow_with_slots {
            if *e == eid {
                result.insert(*slot);
                processed.insert(eid);
                break;
            }
        }
        budget -= 1;
    }

    // If budget remains, round-robin through remaining shadow casters
    // (proactive refresh to catch stale shadow maps)
    if budget > 0 {
        let remaining: Vec<&(usize, u32)> = all_shadow_with_slots
            .iter()
            .filter(|(e, _)| !processed.contains(e))
            .collect();
        if !remaining.is_empty() {
            let mut state = ecs.get_resource_mut::<StaggerState>().unwrap();
            let mut attempts = 0;
            while budget > 0 && attempts < remaining.len() {
                let (eid, slot) = remaining[state.round_robin_pos % remaining.len()];
                state.round_robin_pos = (state.round_robin_pos + 1) % remaining.len();
                if processed.insert(*eid) {
                    result.insert(*slot);
                    budget -= 1;
                }
                attempts += 1;
            }
        }
    }

    // Clear ShadowDirtyFlag for processed entities
    if let Some(mut store) = ecs.get_component_store_mut::<ShadowDirtyFlag>() {
        for &eid in &processed {
            if let Some(idx) = store.sparse.get(eid).copied().flatten() {
                store.components[idx].clear();
            }
        }
    }

    result
}
