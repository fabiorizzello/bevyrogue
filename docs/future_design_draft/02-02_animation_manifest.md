# §2.2 — Animation manifest (.ron per atlas) — modello UE5 a 2 asset

> **⚠ Forma `clipmontage.ron` superseded da §2.2b.** Questo file descrive la **forma flat originale** (lista piatta di notify ai frame). La forma finale del file `clipmontage.ron` adottata per M017 è il **grafo orchestratore (Animation FSM)** descritto in [§2.2b](02-02b_animation_fsm.md). Restano invariati: schema `clip.ron`, il modello a 2 asset, il boundary gameplay/presentation, le regole di hot-reload. La lista piatta sopravvive come **degenerate case** del grafo (1 nodo all-clip) — vedi §2.2b §N migrazione. Leggi `02-02` per i razionali del 2-asset model; leggi `02-02b` per la sintassi e la semantica runtime di `clipmontage.ron`.

**Stato attuale:** `assets/digimon/<name>_atlas.json` = solo geometria del frame (start/end/count per animazione). Zero metadata di *trigger* (impact frame, particle slot, sfx cue).

**Decisione:** sostituire il json. Adottare **modello UE5 a 2 asset separati** — uno descrive l'animazione (clip + frame data), l'altro descrive il gameplay-binding (cosa succede a quale frame). I due si combinano per nome di clip.

```
assets/digimon/<name>/
  clip.ron         ← "animation asset"  (analogo a UE5 AnimSequence)
    meta { frame_size, columns, rows, total_frames }
    clips: {
      "idle":   Clip(start: 0,  end: 3,  fps: 6,  loop: true),
      "attack": Clip(start: 4,  end: 12, fps: 12, loop: false),
      "skill":  Clip(start: 51, end: 67, fps: 14, loop: false),
      "hurt":   Clip(start: 80, end: 84, fps: 12, loop: false),
      ...
    }

  clipmontage.ron  ← "montage asset" (analogo a UE5 AnimMontage: modifier timeline + trigger)
    bindings: {
      "attack": [
        // Trigger eventi (point-in-time)
        Particle { at: 4, name: "claw_slash" },
        Sfx      { at: 5, name: "claw_hit" },
        Shake    { at: 6, intensity: 0.2 },
      ],
      "skill": [
        // Timeline modifier (range)
        Hold     { at: 8, extra_frames: 2 },                    // pausa playhead per breathing
        SpeedMul { from: 12, to: 14, factor: 0.5 },             // slow-mo su impact
        Loop     { from: 3, to: 5, times: 2 },                  // ripete charge-up
        // Trigger
        Flash    { at: 8, color: "white", duration_ms: 80 },
        Particle { at: 12, name: "fireball_explode" },
        Shake    { at: 14, intensity: 0.4 },
      ],
    }
```

**Due famiglie di notify:**

| Famiglia | Variants | Effetto sul playhead |
|---|---|---|
| **Modifier** (range/point) | `Hold`, `SpeedMul`, `Loop` | Altera il timing dell'animazione (stretch, accelerate, repeat) |
| **Trigger** (point) | `Particle`, `Sfx`, `Shake`, `Flash`, `ScreenFreeze` | Fire-and-forget eventi presentation a frame X |

Modifier e trigger condividono la stessa lista per locality (chi cura il feel vede tutto insieme) ma sono enum variants distinti — il consumer (UI animator) li applica in due passi: prima calcola playback timeline con i modifier, poi schedula i trigger ai frame mappati.

**Perché separare:**
- `clip.ron` lo modifica chi fa lo spritesheet / rigging (designer di animazione)
- `clipmontage.ron` lo modifica chi cura feel/feedback visuale (FX designer)
- Riprodurre la stessa clip con montage diversi (es. "skill" caricata vs "skill" hard-cap) diventa configurazione, non duplicazione
- Mantiene il vincolo §2.1: `clip.ron` = numeri, `clipmontage.ron` = identificatori presentation (`kind` enum), zero gameplay

**Boundary stretto con `signal_bindings.ron` gameplay (§2.1)** (no overlap):

| | `signal_bindings.ron` gameplay (§2.1) | `clipmontage.ron` animation (§2.2) |
|---|---|---|
| Risponde a | **COSA** succede | **QUANDO/COME** si vede |
| Layer | Gameplay/kernel (headless) | Presentation (windowed) |
| Esempi ammessi | `apply_heated`, `build_exploit`, `open_momentum_window` | Trigger: `Particle`, `Sfx`, `Shake`, `Flash`, `ScreenFreeze`. Modifier: `Hold`, `SpeedMul`, `Loop` |
| Esempi **vietati** | `Particle`, `Sfx` (sarebbero presentation in skill) | `DamageImpact`, `ApplyStatus`, `BuildResource` (sarebbero gameplay in animation) |

**Modifier safety:** `Hold`/`SpeedMul`/`Loop` alterano solo il playback dell'animazione, non la risoluzione kernel. Il kernel emette `DamageDealt` quando il *combat tick* completa, indipendente da quanti frame l'animazione gira. Stretchare l'animazione per drama visivo → safe. Stretchare per dare più tempo al kernel di calcolare → vietato (rompe boundary).

Regola: se rimuovere il file rompe il combat headless, è gameplay → va in `signal_bindings.ron` (§2.1). Se rimuovere il file lascia il combat invariato ma "muto", è presentation → va in `clipmontage.ron` (§2.2).

**Consumer:** UI animator (windowed) emette `CombatEvent::AnimationNotify { unit, kind, frame }` quando il playhead colpisce un notify. Il combat kernel **non** dipende da questi file — è presentation metadata, allineato a `presentation_metadata_boundary.md`. Le animazioni *decorano* eventi che il kernel ha già emesso (`CombatEvent::DamageDealt`, `…StatusApplied`), non li causano.

**Frame-sync stretto (HSR-style) deferred:** se in futuro serve che il numero di danno appaia esattamente al frame d'impatto, si aggiunge `Notify::Sync(label: "impact")` e si fa bloccare il resolver su quel label. Fuori scope M017.

**Migrazione:**
1. Convertire i 6 json esistenti in `<name>/clip.ron` (lossless: stessa info).
2. Per ogni Digimon creare `<name>/clipmontage.ron` con bindings vuoti.
3. Rimuovere il json (sostituzione netta — un solo source-of-truth come da §2.1).

**Scope M017:** schema dei 2 file + Agumon con clipmontage reale come reference. Gli altri 5 hanno `clipmontage.ron` con bindings vuoti, da popolare in milestone successive.
