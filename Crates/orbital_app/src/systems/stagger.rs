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

    if global_dirty {
        let casters = match ecs.get_component_store::<ShadowCaster>() {
            Some(s) => s,
            None => return result,
        };
        let slot_store = match ecs.get_component_store::<LightSlotIndex>() {
            Some(s) => s,
            None => return result,
        };

        for &eid in casters.dense.as_slice() {
            if let Some(c_idx) = casters.sparse.get(eid).copied().flatten() {
                if !casters.components[c_idx].enabled {
                    continue;
                }
                if let Some(s_idx) = slot_store.sparse.get(eid).copied().flatten() {
                    result.insert(slot_store.components[s_idx].0);
                }
            }
        }

        if let Some(dirty_store) = ecs.get_component_store_mut::<ShadowDirtyFlag>() {
            for &eid in dirty_store.dense.as_slice() {
                if let Some(idx) = dirty_store.sparse.get(eid).copied().flatten() {
                    dirty_store.components[idx].clear();
                }
            }
        }

        return result;
    }

    // Collect dirty shadow-casting entities
    let casters = match ecs.get_component_store::<ShadowCaster>() {
        Some(s) => s,
        None => return result,
    };
    let dirty_store = match ecs.get_component_store::<ShadowDirtyFlag>() {
        Some(s) => s,
        None => return result,
    };
    let slot_store = match ecs.get_component_store::<LightSlotIndex>() {
        Some(s) => s,
        None => return result,
    };

    let mut dirty_entities: Vec<usize> = Vec::new();
    let mut all_shadow_entities: Vec<usize> = Vec::new();

    for &eid in casters.dense.as_slice() {
        if let Some(c_idx) = casters.sparse.get(eid).copied().flatten() {
            if !casters.components[c_idx].enabled {
                continue;
            }
            all_shadow_entities.push(eid);
            if let Some(d_idx) = dirty_store.sparse.get(eid).copied().flatten() {
                if dirty_store.components[d_idx].is_dirty() {
                    dirty_entities.push(eid);
                }
            }
        }
    }

    drop(casters);
    drop(dirty_store);
    drop(slot_store);

    let mut processed: HashSet<usize> = HashSet::new();
    let mut budget = config;

    // Process dirty entities first (priority)
    for &eid in &dirty_entities {
        if budget == 0 {
            break;
        }
        if let Some(si) = ecs
            .get_component_store::<LightSlotIndex>()
            .and_then(|s| s.get_component(eid))
        {
            result.insert(si.0);
        }
        processed.insert(eid);
        budget -= 1;
    }

    // If budget remains, round-robin through remaining shadow casters
    // (proactive refresh to catch stale shadow maps)
    if budget > 0 {
        let remaining: Vec<usize> = all_shadow_entities
            .iter()
            .copied()
            .filter(|e| !processed.contains(e))
            .collect();
        if !remaining.is_empty() {
            let mut state = ecs.get_resource_mut::<StaggerState>().unwrap();
            while budget > 0 && !remaining.is_empty() {
                let eid = remaining[state.round_robin_pos % remaining.len()];
                state.round_robin_pos = (state.round_robin_pos + 1) % remaining.len();
                if processed.insert(eid) {
                    if let Some(si) = ecs
                        .get_component_store::<LightSlotIndex>()
                        .and_then(|s| s.get_component(eid))
                    {
                        result.insert(si.0);
                    }
                    budget -= 1;
                } else {
                    break;
                }
            }
        }
    }

    // Clear ShadowDirtyFlag for processed entities
    if let Some(store) = ecs.get_component_store_mut::<ShadowDirtyFlag>() {
        for &eid in &processed {
            if let Some(idx) = store.sparse.get(eid).copied().flatten() {
                store.components[idx].clear();
            }
        }
    }

    result
}
