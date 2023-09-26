# Uniform

- A Uniform is a BLOB ("Binary Large Object")
- A Uniform can be filled with ONLY primitive data types
- A Uniform is basically a `struct` of primitive data types
- A Uniform must be `#[repr(C)]`
- A Uniform must be `#[derive(Pod, Zeroable)]`
- **A Uniform's size must always be to the power of 2**
- E.g.:
  - if we have some type of `[f32; 3]`, the structure will have 3 * 4 bytes = 12 bytes (1x f32 == 4 bytes).
  - 2^3 bytes = 8 bytes
  - 2^4 bytes = 16 bytes
  - Thus, we need to add an extra 4 bytes (16 bytes - 12 bytes = 4 bytes) to our Uniform structure to be compliant
