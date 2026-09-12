use orbital_ecs::Entity;

use crate::components::ResolvedLayout;

/// Result of a hit test.
#[derive(Debug, Clone)]
pub struct HitResult {
    /// The entity that was hit.
    pub entity: Entity,
    /// The resolved layout of the hit element.
    pub layout: ResolvedLayout,
    /// The z-index of the hit element.
    pub z_index: i32,
}

/// Performs hit testing on UI elements.
///
/// Returns the topmost element at the given screen position,
/// or None if no element was hit.
pub fn hit_test(
    screen_x: f32,
    screen_y: f32,
    elements: &[(Entity, ResolvedLayout, i32)], // (entity, layout, z_index)
) -> Option<HitResult> {
    // Sort by z-index (highest first) for topmost element
    let mut sorted = elements.to_vec();
    sorted.sort_by(|a, b| b.2.cmp(&a.2));

    for (entity, layout, z_index) in sorted {
        if layout.contains(screen_x, screen_y) {
            return Some(HitResult {
                entity,
                layout,
                z_index,
            });
        }
    }

    None
}

/// Performs hit testing and returns all elements at the given position.
///
/// Returns elements sorted by z-index (highest first).
pub fn hit_test_all(
    screen_x: f32,
    screen_y: f32,
    elements: &[(Entity, ResolvedLayout, i32)],
) -> Vec<HitResult> {
    let mut results: Vec<HitResult> = elements
        .iter()
        .filter(|(_, layout, _)| layout.contains(screen_x, screen_y))
        .map(|(entity, layout, z_index)| HitResult {
            entity: *entity,
            layout: *layout,
            z_index: *z_index,
        })
        .collect();

    results.sort_by(|a, b| b.z_index.cmp(&a.z_index));
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entity(index: usize) -> Entity {
        Entity::new(index, 0)
    }

    #[test]
    fn hit_test_basic() {
        let elements = vec![
            (
                make_entity(0),
                ResolvedLayout {
                    x: 0.0,
                    y: 0.0,
                    width: 100.0,
                    height: 50.0,
                },
                0,
            ),
        ];

        let result = hit_test(50.0, 25.0, &elements);
        assert!(result.is_some());
        assert_eq!(result.unwrap().entity, make_entity(0));
    }

    #[test]
    fn hit_test_miss() {
        let elements = vec![
            (
                make_entity(0),
                ResolvedLayout {
                    x: 0.0,
                    y: 0.0,
                    width: 100.0,
                    height: 50.0,
                },
                0,
            ),
        ];

        let result = hit_test(150.0, 25.0, &elements);
        assert!(result.is_none());
    }

    #[test]
    fn hit_test_z_order() {
        let elements = vec![
            (
                make_entity(0),
                ResolvedLayout {
                    x: 0.0,
                    y: 0.0,
                    width: 100.0,
                    height: 50.0,
                },
                0,
            ),
            (
                make_entity(1),
                ResolvedLayout {
                    x: 10.0,
                    y: 10.0,
                    width: 80.0,
                    height: 30.0,
                },
                1,
            ),
        ];

        let result = hit_test(50.0, 25.0, &elements);
        assert!(result.is_some());
        // Should return the higher z-index element
        assert_eq!(result.unwrap().entity, make_entity(1));
    }

    #[test]
    fn hit_test_all_results() {
        let elements = vec![
            (
                make_entity(0),
                ResolvedLayout {
                    x: 0.0,
                    y: 0.0,
                    width: 100.0,
                    height: 50.0,
                },
                0,
            ),
            (
                make_entity(1),
                ResolvedLayout {
                    x: 10.0,
                    y: 10.0,
                    width: 80.0,
                    height: 30.0,
                },
                1,
            ),
        ];

        let results = hit_test_all(50.0, 25.0, &elements);
        assert_eq!(results.len(), 2);
        // Should be sorted by z-index (highest first)
        assert_eq!(results[0].entity, make_entity(1));
        assert_eq!(results[1].entity, make_entity(0));
    }
}
