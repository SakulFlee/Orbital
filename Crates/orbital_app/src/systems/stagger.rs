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
/// 2. Processes up to `budget - reserve` dirty lights (dirty priority),
///    then round-robin through remaining clean shadows for proactive refresh,
///    where `reserve = max(1, budget/2)` when clean casters exist (0 otherwise)
/// 3. Clears `ShadowDirtyFlag` for processed lights
/// 4. Returns the set of `light_store_index` values whose shadows need updating
///
/// If `global_dirty` is true (indicating model/changes), all shadow-casting
/// lights are included regardless of the budget.
///
/// `new_light_bootstrap`: when true (new lights were added this frame),
/// marks all shadow-casting lights dirty for immediate initialization.
pub fn sys_stagger_shadow_updates(
    ecs: &mut World,
    global_dirty: bool,
    new_light_bootstrap: bool,
) -> HashSet<u32> {
    if ecs.get_resource::<StaggerState>().is_none() {
        ecs.insert_resource(StaggerState::default());
    }

    let config = ecs
        .get_resource::<StaggeredLightConfig>()
        .map(|c| c.max_updates_per_frame)
        .unwrap_or(1)
        .max(1) as usize;

    let mut result = HashSet::new();

    // Collect all shadow-casting entities and their slot indices.
    // Falls back to sequential indexing if LightSlotIndex store is absent.
    let casters = match ecs.get_component_store::<ShadowCaster>() {
        Some(s) => s,
        None => return result,
    };
    let slot_store = ecs.get_component_store::<LightSlotIndex>();
    let has_slot_store = slot_store.is_some();

    // Build (entity_id, slot_index_or_seq) for each shadow-casting light
    let mut shadow_entities: Vec<(usize, u32)> = Vec::new();
    {
        let mut seq_counter = 0u32;
        let descs = ecs.get_component_store::<orbital_ecs_bridge::LightDescriptorEcs>();
        if has_slot_store {
            // Use LightSlotIndex components
            if let Some(ref ss) = slot_store {
                for &eid in casters.dense.as_slice() {
                    if let Some(c_idx) = casters.sparse.get(eid).copied().flatten() {
                        if !casters.components[c_idx].enabled {
                            continue;
                        }
                        if let Some(s_idx) = ss.sparse.get(eid).copied().flatten() {
                            shadow_entities.push((eid, ss.components[s_idx].0));
                        }
                    }
                }
            }
        } else {
            // Fallback: sequential counter from LightDescriptorEcs iteration order
            if let Some(ref ds) = descs {
                for &eid in ds.dense.as_slice() {
                    let has_desc = ds.sparse.get(eid).copied().flatten().is_some();
                    let has_shadow = casters.sparse.get(eid).copied().flatten()
                        .map(|ci| casters.components[ci].enabled)
                        .unwrap_or(false);
                    if has_desc && has_shadow {
                        shadow_entities.push((eid, seq_counter));
                    }
                    if has_desc {
                        seq_counter += 1;
                    }
                }
            }
        }
    }

    // Collect dirty entity IDs
    let dirty_eids: Vec<usize> = match ecs.get_component_store::<ShadowDirtyFlag>() {
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

    // --- Global dirty (model change) or new-light bootstrap → all shadows ---
    if global_dirty || new_light_bootstrap {
        for (_, slot) in &shadow_entities {
            result.insert(*slot);
        }
        if let Some(mut store) = ecs.get_component_store_mut::<ShadowDirtyFlag>() {
            for &eid in &dirty_eids {
                if let Some(idx) = store.sparse.get(eid).copied().flatten() {
                    store.components[idx].clear();
                }
            }
        }
        return result;
    }

    // --- Apply stagger budget ---
    // Reserve a floor for round-robin so an always-dirty light can't
    // permanently starve proactive shadow refresh. Dirty lights still get
    // priority (up to budget - reserve), then round-robin uses the rest.
    let mut processed: HashSet<usize> = HashSet::new();
    let non_dirty_count = shadow_entities
        .iter()
        .filter(|(e, _)| !dirty_eids.contains(e))
        .count();
    let reserve = if non_dirty_count > 0 {
        (config / 2).max(1)
    } else {
        0
    };
    let mut dirty_budget = config.saturating_sub(reserve);
    let round_robin_budget = config - dirty_budget;

    // Process dirty entities first (priority)
    for &eid in &dirty_eids {
        if dirty_budget == 0 {
            break;
        }
        let slot = if has_slot_store {
            ecs.get_component_store::<LightSlotIndex>()
                .and_then(|ss| ss.get_component(eid).copied())
                .map(|si| si.0)
        } else {
            shadow_entities.iter().find(|(e, _)| *e == eid).map(|(_, s)| *s)
        };
        if let Some(slot) = slot {
            result.insert(slot);
            processed.insert(eid);
            dirty_budget -= 1;
        }
    }

    // Round-robin through remaining shadow casters
    if round_robin_budget > 0 {
        let remaining_slots: Vec<u32> = shadow_entities
            .iter()
            .filter(|(e, _)| !processed.contains(e))
            .map(|(_, s)| *s)
            .collect();
        if !remaining_slots.is_empty() {
            let mut state = ecs.get_resource_mut::<StaggerState>().unwrap();
            for _ in 0..round_robin_budget.min(remaining_slots.len()) {
                let slot = remaining_slots[state.round_robin_pos % remaining_slots.len()];
                state.round_robin_pos = (state.round_robin_pos + 1) % remaining_slots.len();
                result.insert(slot);
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
