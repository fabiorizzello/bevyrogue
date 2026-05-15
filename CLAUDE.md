# bevyrogue — Agent Onboarding

## Build & Test

```bash
cargo check                   # default = headless
cargo test                    # full integration suite (tests/)
cargo run                     # headless run
cargo run --features windowed # con UI egui
```

Toolchain: vedi `rust-toolchain.toml`. Dev profile usa `cranelift` (vedi `Cargo.toml`).


## Convenzioni

- **Headless first:** ogni system deve girare senza `windowed`. Gating: `#[cfg(feature = "windowed")]` solo per egui/winit.
- **Tests:** integration in `tests/`. Naming **funzionale** (es. `follow_up_triggers.rs`, non `s10_…`). Non aggiungere unit test inline in `src/` salvo `#[cfg(test)] mod tests` brevi.
- **Determinismo:** tests devono essere deterministici (no wall-clock, no RNG senza seed).

## Don't

- Non aggiungere dipendenze winit/wgpu/egui fuori da `windowed` feature gate.
- Non riempire root con `.md` — vanno in `docs/` o `.gsd/`.
