# Architectural Patterns

This document describes the architectural style and core design patterns in use.

## Overall Style: Headless-First ECS Monolith

`bevyrogue` is built on the [Entity Component System (ECS)](https://en.wikipedia.org/wiki/Entity_component_system) pattern provided by the Bevy engine. It follows a "Headless-First" design philosophy, where all gameplay logic is strictly decoupled from the rendering stack.

### Key Benefits
- **Deterministic Simulation**: Enables perfectly reproducible combat outcomes across different machines.
- **Automated Testing**: Combat scenarios can be run as unit/integration tests without a GPU.
- **CLI Tooling**: A specialized CLI harness (`combat_cli`) can drive the game for balancing and development.

## Core Data Flow: Combat Resolution

Combat follows a structured pipeline:
1. **Declaration**: An `ActionIntent` (Basic Attack, Skill, Ultimate) is emitted.
2. **Resolution**: The intent is checked for legality and translated into a `ResolvedAction`.
3. **Application**: The `intent_applier` exclusive system executes the action, producing state mutations (HP changes, status effects) and `CombatEvent` messages.
4. **Passive Reaction**: `SignalBus` and `PassiveListeners` catch events to trigger follow-up actions or blueprint-specific logic.

## Domain Boundaries

The combat system is divided into two major layers:

### 1. Combat Kernel (`src/combat/kernel`)
The low-level vocabulary of combat. It defines primitives like:
- **Tactical Cycle**: The progression of turns and phases.
- **Strain & Flow**: Dynamic resources used to fuel advanced mechanics.
- **Fatigue**: A limiting factor on repetitive actions.
- **Combat Tags**: Lightweight markers used for conditional triggers.

### 2. Combat Runtime (`src/combat/runtime`)
The engine that executes gameplay abilities.
- **Intents**: The closed set of valid commands.
- **Timeline Engine**: A finite state machine (FSM) that drives multi-beat skills.
- **Signal Bus**: A central dispatch for inter-module communication.
- **Registries**: Extension points for custom behaviors (hooks, selectors, predicates).

## Content Architecture: Blueprints

Individual Digimon and enemies are implemented as "Blueprints" (`src/combat/blueprints`).
- **Declarative Definitions**: Data is defined in RON files.
- **Code Hooks**: Specific logic (e.g., Agumon's fire core mechanics) is implemented via `CombatKernelHook` and registered at boot time. This keeps core engine code free of content-specific "match ladders."

## Modularity via Plugins

The project uses Bevy's `Plugin` system to encapsulate functionality:
- `CombatPlugin`: Mounts the full combat runtime and registries.
- `DataPlugin`: Handles asynchronous loading and merging of RON assets.
