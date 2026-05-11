# Setup (Ubuntu)

Prerequisiti per compilare `bevyrogue` con la config fast-compile già in repo
(`mold` + dynamic linking + cranelift su nightly + `-Zshare-generics=y`).

Testato su Ubuntu 22.04 / 24.04.

---

## 1. Pacchetti di sistema

```bash
sudo apt update
sudo apt install -y \
  build-essential \
  pkg-config \
  mold \
  libasound2-dev \
  libudev-dev \
  libwayland-dev \
  libxkbcommon-dev
```

Cosa serve e perché:

- **`build-essential`** — `gcc`, `g++`, `make`. Il linker usato è `gcc` (con
  `-fuse-ld=mold`), non `cc` direttamente.
- **`pkg-config`** — richiesto dai build script di diverse crate native
  (`alsa-sys`, `libudev-sys`, …).
- **`mold`** — linker veloce, usato in dev al posto di `ld`/`lld`. Configurato in
  `.cargo/config.toml` come `-C link-arg=-fuse-ld=mold`. Se su Ubuntu <22.04
  `apt` non ha `mold`, vedi §1a.
- **`libasound2-dev`** — header ALSA, richiesti da `bevy_audio` (transitivo anche
  in build headless su alcune versioni Bevy).
- **`libudev-dev`** — richiesto da `gilrs` (gamepad), linkato transitivamente.
- **`libwayland-dev` + `libxkbcommon-dev`** — servono **solo** per la build
  `windowed` (`cargo winx`), che tira dentro winit + wgpu. Per lavorare headless
  (`cargo dev`) si possono omettere, ma tenerli installati non costa nulla e
  evita sorprese al primo `cargo winx`.

### 1a. Mold su distro senza pacchetto apt

Se `apt install mold` non trova il pacchetto (Ubuntu 20.04 o derivate vecchie),
scarica il release prebuilt da
[rui314/mold](https://github.com/rui314/mold/releases) ed estrai in
`~/.local/bin` (deve essere in `PATH`):

```bash
MOLD_VERSION=2.32.0
curl -fL "https://github.com/rui314/mold/releases/download/v${MOLD_VERSION}/mold-${MOLD_VERSION}-x86_64-linux.tar.gz" \
  | tar -xz -C /tmp
mkdir -p ~/.local/bin
cp /tmp/mold-${MOLD_VERSION}-x86_64-linux/bin/mold ~/.local/bin/
mold --version
```

Verifica: `which mold` deve risolvere. `.cargo/config.toml` passa `-fuse-ld=mold`
a `gcc`, che a sua volta cerca `mold` in `PATH`.

---

## 2. Rust toolchain

Serve `rustup`. Se non è installato:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
source "$HOME/.cargo/env"
```

Il progetto pinna la toolchain a **nightly** via `rust-toolchain.toml` e
richiede il componente `rustc-codegen-cranelift-preview`. Rustup li installa
automaticamente al primo comando `cargo` eseguito dentro la repo — non c'è nulla
da fare a mano.

Verifica:

```bash
cd /path/to/bevyrogue
rustc --version          # deve riportare "nightly"
rustup component list --installed | grep cranelift
```

Atteso: `rustc-codegen-cranelift-preview-x86_64-unknown-linux-gnu` presente.

Il resto del sistema resta su stable — il pin è per-progetto, non globale.

---

## 3. Build

```bash
cargo check-dev   # verifica rapida, niente binario
cargo dev         # run headless (default per agenti / sviluppo)
cargo winx        # run con finestra — solo per verifica visiva
```

Al primo build da pulito aspettati **~50-60s** cold (con tutto configurato).
Le iterazioni incrementali successive sono nell'ordine dei secondi grazie a
mold + dynamic linking + cranelift + share-generics.

---

## 4. Sprite atlases (generated, not tracked)

Gli atlas pixel-art in `assets/digimon/*_atlas.png` + `*_atlas.json` sono
**rigenerati** dal sprite pipeline e non vengono committati (vedi `.gitignore`).
Servono solo alla UI windowed; build/test headless girano senza.

Per rigenerarli:

```bash
cd tools/sprite_pipeline
# vedi GETTING_STARTED.md per requisiti (Blender + python deps)
./scripts/pipeline.sh
```

Output finale in `assets/digimon/` (copiato dagli script). Il pipeline lavora
sui `raw_models/` `.fbx`/`.glb` tracciati nel repo.

---

## 5. Troubleshooting

**`error: linker 'mold' not found`** — `mold` non è in `PATH`. Reinstalla (§1
o §1a) e riapri la shell.

**`error: the option 'Z' is only accepted on the nightly compiler`** — stai
usando stable. Sei fuori dalla repo o `rust-toolchain.toml` è stato rimosso.
Entra nella directory del progetto e rilancia.

**`error: 'cranelift' codegen backend is not supported`** — manca il componente.
Installa esplicitamente:
```bash
rustup component add rustc-codegen-cranelift-preview --toolchain nightly
```

**`failed to run custom build command for alsa-sys`** — manca
`libasound2-dev`. Vedi §1.

**`error: failed to find native library 'udev'`** — manca `libudev-dev`.
Vedi §1.
