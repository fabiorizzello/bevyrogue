# Project

## What This Is

**bevyrogue** is a headless-first roguelite, turn-based monster-taming RPG in Rust + Bevy 0.18 with an optional `windowed` egui UI. The current center of gravity is a single authoritative combat engine that future UI/CLI/run layers read rather than reimplement.

## Core Value

One playable run where combat, party-building, and future interfaces all consume the same combat authority.

## Project Shape

- **Complexity:** complex
- **Why:** Brownfield codebase with many completed milestones, explicit contracts/requirements, multiple runtime surfaces, and architecture-sensitive sequencing.

## Current State

M020 is complete. The combat event bus is now normalized around canonical events such as `UltimateUsed { unit_id }` and `UnitDied { status_remaining, heated_remaining }`. Legacy blueprint shims were removed, tests are green, and M016-M020 form the validated combat baseline. The active execution lane is M002/S09 planning, while the broader product direction points toward M021 (`trait Skill` + `SkillCtx`) as the next architectural generalization seam.

## Architecture / Key Patterns

Headless-first Bevy, a typed combat kernel, RON as data/presentation config rather than gameplay logic, and per-Digimon Rust blueprints that emit generic kernel intents through a shared combat authority. Canonical event-bus, validation snapshots, and legality contracts are treated as product surfaces, not incidental test scaffolding.

## Capability Contract

See `.gsd/REQUIREMENTS.md` for the explicit capability contract, requirement status, and coverage mapping.

## Milestone Sequence

- [ ] M001: Animation asset pipeline foundation — establish typed animation assets, validation, and generic loading seams.
- [ ] M002: First on-screen combat — prove Agumon-only windowed combat against the authoritative kernel.
- [ ] M021: Skill abstraction seam — generalize skill resolution through `trait Skill` + `SkillCtx` so future kits and effects stop hardcoding special cases.
