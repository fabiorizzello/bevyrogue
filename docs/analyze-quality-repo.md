You are a senior game engine architect. Analyze this turn-based combat engine codebase and evaluate:

  1. **Structural health**
     - Files > 500 lines: list them, note what they contain
     - Identify God classes / God modules
     - Coupling score: how many cross-domain imports exist?

  2. **LOC composition**
     - Estimate % pure logic vs data definitions (stats, tables, configs)
     - Estimate % boilerplate (getters, DTOs, serialization)
     - Real logic LOC = total - data - boilerplate

  3. **Test quality**
     - Tests assert behavior or implementation?
     - If I rename an internal method, how many tests break? (estimate from patterns)
     - Coverage on critical paths: skill resolution, turn order, buff stacking

  4. **Architecture signals**
     - Does skill system use composition or inheritance?
     - Are effects/triggers data-driven or hardcoded?
     - Is turn engine decoupled from presentation/UI layer?

  5. **Early-stage risk**
     - What is the highest-complexity module?
     - Where is technical debt most concentrated?
     - What would break first at 10x content scale?

  For each criterion give: current state, risk level (low/medium/high), and one concrete action if risk is medium+.

  Codebase stats: ~30k LOC source, ~30k LOC tests, turn-based combat engine (HSR-style), early stage.
