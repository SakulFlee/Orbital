# WebAssembly (WASM) Support

This engine uses Rayon for parallel system execution. Rayon
can work in WASM environments via `wasm-bindgen-rayon`, which
uses Web Workers + `SharedArrayBuffer`.

## Server Requirements

Your web server **MUST** send these HTTP response headers when
serving the `.wasm` and `.js` files:

```
Cross-Origin-Opener-Policy: same-origin
Cross-Origin-Embedder-Policy: require-corp
```

### What these do

- **COOP**: Isolates the document from cross-origin windows,
  preventing data leaks via `window` references.
- **COEP**: Requires all loaded resources to explicitly opt
  into sharing via CORS or CORP headers.

These are required because `SharedArrayBuffer` (used for
thread communication) was restricted after Spectre/Meltdown.

### How to configure

**Nginx** (add to the WASM location block):
```nginx
add_header Cross-Origin-Opener-Policy "same-origin" always;
add_header Cross-Origin-Embedder-Policy "require-corp" always;
```

**Cloudflare Workers / Pages**: Set headers via a `_headers`
file or the dashboard rules.

## Feature Flag

Enable the `wasm-threads` feature to activate WASM threading:

```toml
[dependencies]
orbital_ecs = { path = "../Crates/ecs", features = ["wasm-threads"] }
```

Without this flag, compilation on WASM will fail with a
compile-time error pointing to this document.

## Cross-origin resources

COEP `require-corp` affects third-party resources (fonts,
images, scripts from other origins). You may need to add
`crossorigin="anonymous"` to `<img>`, `<link>`, etc. or
ensure the external server sends appropriate CORS headers.
