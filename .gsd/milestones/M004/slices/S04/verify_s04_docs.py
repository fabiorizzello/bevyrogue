#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]

S03_SUMMARY = ROOT / ".gsd/milestones/M004/slices/S03/S03-SUMMARY.md"
SCOPE_DOC = ROOT / ".gsd/milestones/M004/slices/S04/M004-VALIDATION-SCOPE.md"
BOUNDARY_DOC = ROOT / ".gsd/milestones/M004/slices/S04/M004-BOUNDARY-MAP.md"

REQUIRED_DOCS = [SCOPE_DOC, BOUNDARY_DOC, S03_SUMMARY]
REQUIRED_SOURCE_PATHS = [
    ROOT / "src/animation/vfx_asset.rs",
    ROOT / "src/combat/runtime/registry.rs",
    ROOT / "src/combat/blueprints/agumon/mod.rs",
    ROOT / "assets/digimon/agumon/vfx.ron",
    ROOT / "src/windowed/render.rs",
    ROOT / "tests/animation/vfx_asset_schema.rs",
    ROOT / "tests/animation/vfx_asset_eval.rs",
    ROOT / "tests/animation/vfx_asset_load.rs",
    ROOT / "tests/animation/vfx_variant_selection.rs",
    ROOT / "tests/animation/render_no_vfx_kind_guard.rs",
    ROOT / "tests/windowed_only/vfx_asset_impact_render.rs",
    ROOT / ".gsd/KNOWLEDGE.md",
    ROOT / ".gsd/milestones/M004/slices/S01/S01-SUMMARY.md",
    ROOT / ".gsd/milestones/M004/slices/S03/S03-SUMMARY.md",
    ROOT / ".gsd/milestones/M004/M004-CONTEXT.md",
]

REQUIRED_TOKENS = {
    SCOPE_DOC: [
        "## Scope table",
        "## Producer → consumer boundary map",
        "## S03 consumed contracts from earlier slices",
        "## Pending scope reserved for S05 and S06",
        "Sharp Claws",
        "HDR bloom / additive rendering",
        "K001",
        "S05",
        "S06",
        "This dependency statement is part of the validation scope even though the original `S03-SUMMARY.md` frontmatter left `requires: []`.",
        "| S03 | S01 | Typed `VfxAsset` schema, effect resolution helpers, deterministic appearance/curve evaluation, owned `assets/digimon/agumon/vfx.ron` load path |",
        "| S03 | S02 | `PlacementExt` registry axis, registered Agumon placement verbs, `validate_effects`, registry-resolved render dispatcher, hardcoded kind-dispatch removal |",
    ],
    BOUNDARY_DOC: [
        "## Boundary table",
        "## S03 consumed contracts from earlier slices",
        "## Explicit limits for validators",
        "## Reader test",
        "Sharp Claws",
        "HDR bloom/additive rendering",
        "K001",
        "S05/S06",
        "| 7 | K001 manual visual boundary |",
        "| S03 | S01 | Typed `VfxAsset` schema, `resolve_effect`, deterministic `eval_scale` / `eval_color`, and the owned `assets/digimon/agumon/vfx.ron` load path |",
        "| S03 | S02 | Registered `PlacementExt` verbs, `validate_effects`, registry-resolved render dispatcher, and removal of legacy VFX-kind dispatch |",
    ],
}

TEST_TOKENS = {
    ROOT / "tests/animation/vfx_asset_schema.rs": [
        "fn all_authored_effects_round_trip()",
        "fn placement_is_reflectable_with_typed_params_and_anchor()",
    ],
    ROOT / "tests/animation/vfx_asset_eval.rs": [
        "fn eval_scale_is_deterministic_across_repeated_calls()",
        "fn eval_color_is_deterministic_across_repeated_calls()",
    ],
    ROOT / "tests/animation/vfx_asset_load.rs": [
        "fn validate_effects_accepts_the_real_asset()",
        "fn projectile_on_expire_chains_the_impact_burst()",
        "fn baby_burner_detonate_is_fan_out_burst_chaining_flash()",
        "fn validate_effects_names_an_unregistered_verb()",
        "fn validate_effects_names_a_dangling_on_expire()",
    ],
    ROOT / "tests/animation/vfx_variant_selection.rs": [
        "fn select_variant_maps_context_to_expected_effect()",
        "fn select_variant_is_deterministic_across_repeated_calls()",
        "fn select_variant_returns_none_for_unmapped_keys()",
        "fn validate_effects_names_a_dangling_variant_target()",
    ],
    ROOT / "tests/animation/render_no_vfx_kind_guard.rs": [
        "fn render_rs_has_no_vfx_kind_dispatch()",
    ],
    ROOT / "tests/windowed_only/vfx_asset_impact_render.rs": [
        "fn built_registry_resolves_all_authored_placement_verbs()",
        "fn every_effect_resolves_and_its_verb_is_registered()",
        "fn projectile_on_expire_chains_the_impact_then_flash_fan()",
    ],
    ROOT / "src/windowed/render.rs": [
        "fn on_enter_charge_seeds_both_the_orb_and_the_ember_swirl()",
    ],
}

S03_REQUIRES_TOKENS = [
    "S01:",
    "typed VfxAsset schema",
    "resolve/eval API",
    "resolve_effect",
    "eval_scale",
    "eval_color",
    "assets/digimon/agumon/vfx.ron",
    "S02:",
    "PlacementExt",
    "registered Agumon placement verbs",
    "validate_effects",
    "registry-resolved windowed VFX data path",
]


class CheckFailure(RuntimeError):
    pass


def fail(message: str) -> None:
    raise CheckFailure(message)


def read_text(path: Path) -> str:
    if not path.exists():
        fail(f"missing path: {path.relative_to(ROOT)}")
    return path.read_text(encoding="utf-8")


def require_tokens(path: Path, tokens: list[str]) -> None:
    text = read_text(path)
    rel = path.relative_to(ROOT)
    for token in tokens:
        if token not in text:
            fail(f"missing token in {rel}: {token}")


def frontmatter_block(text: str) -> str:
    match = re.match(r"^---\n(.*?)\n---\n", text, re.DOTALL)
    if not match:
        fail("S03 summary is missing leading frontmatter block")
    return match.group(1)


def verify_s03_requires() -> None:
    text = read_text(S03_SUMMARY)
    frontmatter = frontmatter_block(text)
    if re.search(r"(?m)^requires:\s*\n\s*\[\]", frontmatter):
        fail("S03 summary still has the old empty dependency metadata: requires: []")
    requires_section = re.search(r"(?ms)^requires:\s*\n(.*?)(?:^affects:\s*\n)", frontmatter)
    if not requires_section:
        fail("S03 summary frontmatter is missing a parseable requires section before affects")
    requires_text = requires_section.group(1)
    for token in S03_REQUIRES_TOKENS:
        if token not in requires_text:
            fail(f"missing token in S03 requires metadata: {token}")


def verify_required_paths() -> None:
    for path in REQUIRED_DOCS + REQUIRED_SOURCE_PATHS:
        if not path.exists():
            fail(f"missing path: {path.relative_to(ROOT)}")


def verify_test_tokens() -> None:
    for path, tokens in TEST_TOKENS.items():
        require_tokens(path, tokens)


def main() -> int:
    try:
        verify_required_paths()
        for path, tokens in REQUIRED_TOKENS.items():
            require_tokens(path, tokens)
        verify_test_tokens()
        verify_s03_requires()
    except CheckFailure as exc:
        print(f"FAIL: {exc}")
        return 1

    print("OK: S04 validation docs, proof references, and S03 dependency metadata are consistent.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
