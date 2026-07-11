# Known Limitations

## IntoSystem Macro — Multi-Resource + Multi-Component Gap

**Date:** 2025-07-11

### Problem

The `IntoSystem` macro system in `orbital_ecs` does not support system functions that need **multiple resources AND multiple component writes** simultaneously.

### Affected Pattern

```rust
// THIS DOES NOT WORK with the current macros:
fn sys_camera_controller(
    dt: Res<DeltaTime>,        // resource read
    input: Res<InputSnapshot>, // resource read
    pos: &mut Position,        // component write
    rot: &mut Rotation,        // component write
) { ... }
```

### What Works Today

| Pattern | Macro | Status |
|---------|-------|--------|
| `Res<A>` only | `param_resource.rs` | ✅ |
| `ResMut<A>` only | `param_resource.rs` | ✅ |
| `Res<A> + Res<B>` | `impl_2_res_read_res_read` | ✅ |
| `ResMut<A> + Res<B>` | `impl_2_res_write_res_read` | ✅ |
| `&mut A` only | `param_function.rs` | ✅ |
| `&mut A + &mut B` | `impl_2_ww` | ✅ |
| `Res<A> + &mut B` | `impl_2_write_comp_res_read` | ✅ |
| `Res<A> + Res<B> + &mut C` | New 3-arg impl | ✅ |
| `Res<A> + Res<B> + &mut C + &mut D` | New 4-arg impl | ⚠️ Defined but compiler doesn't find it |
| `Res<A> + Res<B> + &mut C + &mut D + ...` | — | ❌ Not defined |

### Root Cause

The 4-arg impl (`Res<A> + Res<B> + &mut C + &mut D`) was added but the compiler fails to resolve `IntoSystem` for function pointer types (`fn(Res<A>, Res<B>, &mut C, &mut D)`). The impl exists in `param_resource.rs` but doesn't match during type inference.

This may be related to:
1. Function pointer vs closure type inference differences
2. Higher-ranked lifetime bounds (`for<'a, 'b>`) not being inferred correctly for function pointers
3. The impl being placed after other impls causing resolution issues

### Workaround

For systems needing multiple resources + multiple component writes, use one of:

**Option A: Split into multiple systems**
```rust
fn sys_move(rot: &mut Rotation, dt: Res<DeltaTime>) { ... }
fn sys_apply(pos: &mut Position, rot: &Rotation) { ... }
```

**Option B: Use direct `&mut World` access (not scheduled)**
```rust
fn my_system(ecs: &mut World) {
    let dt = ecs.get_resource::<DeltaTime>().unwrap();
    let input = ecs.get_resource::<InputSnapshot>().unwrap();
    // Direct component access via store iteration
}
```

**Option C: Closure in `Module::setup()` (not scheduled)**
```rust
fn setup(&self, ecs: &mut World, ...) -> Vec<Box<dyn System>> {
    // Direct world access for complex initialization
    // Return only simple scheduled systems
}
```

### How to Fix

1. Debug why the 4-arg `impl` doesn't match function pointer types
2. Consider converting all macros to a single variadic macro or procedural macro
3. Alternatively, add a `SystemBuilder` API for complex system signatures

### Impact

The roll_camera example's camera controller (WASD + mouse) cannot be a scheduled system with the current macros. The roll system works fine since it only needs `Res<A> + &mut B`.
