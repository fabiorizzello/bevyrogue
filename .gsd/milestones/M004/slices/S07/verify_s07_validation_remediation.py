#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Dict

ROOT = Path(__file__).resolve().parents[5]

ROADMAP = ROOT / ".gsd/milestones/M004/M004-ROADMAP.md"
REMEDIATION = ROOT / ".gsd/milestones/M004/slices/S07/M004-VALIDATION-REMEDIATION.md"
BOUNDARY_MAP = ROOT / ".gsd/milestones/M004/slices/S04/M004-BOUNDARY-MAP.md"
S05_ACCEPTANCE = ROOT / ".gsd/milestones/M004/slices/S05/M004-RENDERING-ACCEPTANCE.md"
S06_ASSESSMENT = ROOT / ".gsd/milestones/M004/slices/S06/S06-ASSESSMENT.md"
S06_UAT = ROOT / ".gsd/milestones/M004/slices/S06/S06-UAT.md"
SIGNOFF = ROOT / "docs/uat/M004-vfx-signoff.md"
DECISIONS = ROOT / ".gsd/DECISIONS.md"

EVIDENCE_FILES = [
    BOUNDARY_MAP,
    S05_ACCEPTANCE,
    S06_ASSESSMENT,
    S06_UAT,
    SIGNOFF,
    ROOT / "tests/animation/vfx_asset_load.rs",
    ROOT / "tests/animation/vfx_asset_eval.rs",
    ROOT / "tests/animation/render_no_vfx_kind_guard.rs",
    ROOT / "tests/windowed_only/vfx_asset_impact_render.rs",
    ROOT / "tests/windowed_only/vfx_rendering_acceptance.rs",
]

DOC_SURFACES = {
    "roadmap": ROADMAP,
    "remediation": REMEDIATION,
    "boundary_map": BOUNDARY_MAP,
    "s05_acceptance": S05_ACCEPTANCE,
    "s06_assessment": S06_ASSESSMENT,
    "s06_uat": S06_UAT,
    "signoff": SIGNOFF,
    "decisions": DECISIONS,
}

CURRENT_PROOF_TOKEN = "projectile_on_expire_chains_the_impact_then_flash_fan"
STALE_PROOF_TOKENS = [
    "projectile_on_expire_chains_the_impact_fan",
]

FORBIDDEN_AUTOMODE_CLAIMS = [
    "auto-mode ran `cargo winx`",
    "auto-mode executed `cargo winx`",
    "auto-mode launched the windowed binary",
    "auto-mode ran cargo winx",
    "auto-mode executed cargo winx",
]

REMEDIATION_TOKENS = [
    "## Requirement scope",
    "## Variant seam disposition",
    "## S06 evidence",
    "## D037 rendering rescope",
    "## UAT disposition",
    "scope-mapping issue",
    "future-consumer seam",
    "D037",
    "WAIVED",
    "S06-ASSESSMENT.md",
    "S06-UAT.md",
]

ROADMAP_TOKENS = [
    "## Boundary Map",
    "This roadmap keeps a compact validator-facing boundary summary inline.",
    "| Variant selection seam | Delivered as seam only |",
    "| HDR/Bloom overbright rendering proxy | Delivered as accepted proxy |",
    "| K001 visual-UAT boundary | Closed by waiver, not PASS |",
    CURRENT_PROOF_TOKEN,
]

SIGNOFF_WAIVER_TOKENS = [
    "**WAIVED — autonomous closeout recorded without a live `cargo winx` session",
    "- **Sharp Claws:** WAIVED",
    "- **Baby Flame:** WAIVED",
    "- **Baby Burner:** WAIVED",
    "- **Current autonomous-execution status:** WAIVED",
    "- **Final reviewer-completed status field:** `WAIVED`",
]


class CheckFailure(RuntimeError):
    pass


@dataclass(frozen=True)
class RepoDocs:
    texts: Dict[str, str]


def fail(message: str) -> None:
    raise CheckFailure(message)


def read_text(path: Path) -> str:
    if not path.exists():
        fail(f"missing required evidence file: {path.relative_to(ROOT)}")
    return path.read_text(encoding="utf-8")


def load_repo_docs() -> RepoDocs:
    texts = {name: read_text(path) for name, path in DOC_SURFACES.items()}
    return RepoDocs(texts=texts)


def require_tokens(label: str, text: str, tokens: list[str]) -> None:
    for token in tokens:
        if token not in text:
            fail(f"missing token in {label}: {token}")


def verify_evidence_files_exist() -> None:
    for path in EVIDENCE_FILES:
        if not path.exists():
            fail(f"missing required evidence file: {path.relative_to(ROOT)}")


def verify_roadmap_boundary_map(roadmap: str) -> None:
    require_tokens(".gsd/milestones/M004/M004-ROADMAP.md", roadmap, ROADMAP_TOKENS)
    if "## Boundary Map\n\nNot provided." in roadmap or re.search(
        r"(?ms)^## Boundary Map\s*\n\s*Not provided\.", roadmap
    ):
        fail("roadmap boundary map still says `Not provided.`")


def verify_remediation_doc(remediation: str) -> None:
    require_tokens(
        ".gsd/milestones/M004/slices/S07/M004-VALIDATION-REMEDIATION.md",
        remediation,
        REMEDIATION_TOKENS,
    )
    if "auto-mode did not run `cargo winx`" not in remediation:
        fail("missing honesty boundary in remediation doc: auto-mode did not run `cargo winx`")


def verify_decision_reference(decisions: str, remediation: str, s05_acceptance: str) -> None:
    if "| D037 |" not in decisions:
        fail("missing decision record in .gsd/DECISIONS.md: D037")
    if "D037" not in remediation:
        fail("missing D037 citation in remediation closeout")
    if "D037" not in s05_acceptance:
        fail("missing D037 citation in S05 rendering acceptance artifact")


def verify_proof_tokens(repo: RepoDocs) -> None:
    docs_to_scan = {
        "roadmap": repo.texts["roadmap"],
        "remediation": repo.texts["remediation"],
        "boundary_map": repo.texts["boundary_map"],
        "s05_acceptance": repo.texts["s05_acceptance"],
        "s06_assessment": repo.texts["s06_assessment"],
        "s06_uat": repo.texts["s06_uat"],
    }
    joined = "\n".join(docs_to_scan.values())
    if CURRENT_PROOF_TOKEN not in joined:
        fail(f"missing current Baby Flame proof token across closeout docs: {CURRENT_PROOF_TOKEN}")
    for token in STALE_PROOF_TOKENS:
        for name, text in docs_to_scan.items():
            if token in text:
                fail(f"stale Baby Flame proof token in {name}: {token}")


def verify_signoff_disposition(repo: RepoDocs) -> None:
    signoff = repo.texts["signoff"]
    require_tokens("docs/uat/M004-vfx-signoff.md", signoff, SIGNOFF_WAIVER_TOKENS)

    remediation = repo.texts["remediation"]
    claims_waiver_or_closure = any(word in remediation for word in ["WAIVED", "waiver", "closed with a tracked waiver"])
    if claims_waiver_or_closure and "PENDING" in signoff:
        fail(
            "external blocker: docs/uat/M004-vfx-signoff.md is still PENDING while S07 claims waiver/closure"
        )


def verify_no_forbidden_automode_claims(repo: RepoDocs) -> None:
    for name, text in repo.texts.items():
        lower = text.lower()
        for phrase in FORBIDDEN_AUTOMODE_CLAIMS:
            if phrase.lower() in lower:
                fail(f"forbidden auto-mode windowed-run claim in {name}: {phrase}")


def evaluate_texts(texts: Dict[str, str]) -> str:
    repo = RepoDocs(texts=texts)
    verify_roadmap_boundary_map(repo.texts["roadmap"])
    verify_remediation_doc(repo.texts["remediation"])
    verify_decision_reference(repo.texts["decisions"], repo.texts["remediation"], repo.texts["s05_acceptance"])
    verify_proof_tokens(repo)
    verify_signoff_disposition(repo)
    verify_no_forbidden_automode_claims(repo)
    return "OK: S07 remediation closeout, waiver disposition, roadmap boundary map, and proof-token surfaces are consistent."


def run_self_tests() -> int:
    base = {name: text for name, text in load_repo_docs().texts.items()}

    def expect_failure(name: str, mutate, needle: str) -> None:
        mutated = dict(base)
        mutate(mutated)
        try:
            evaluate_texts(mutated)
        except CheckFailure as exc:
            message = str(exc)
            if needle not in message:
                raise AssertionError(f"{name}: expected `{needle}` in `{message}`") from exc
        else:
            raise AssertionError(f"{name}: expected failure containing `{needle}`")

    expect_failure(
        "missing boundary placeholder removal",
        lambda texts: texts.__setitem__(
            "roadmap",
            texts["roadmap"].replace(
                "## Boundary Map\n\nThis roadmap keeps a compact validator-facing boundary summary inline.",
                "## Boundary Map\n\nNot provided.\n\nThis roadmap keeps a compact validator-facing boundary summary inline.",
            ),
        ),
        "roadmap boundary map still says `Not provided.`",
    )
    expect_failure(
        "missing D037 citation",
        lambda texts: texts.__setitem__("remediation", texts["remediation"].replace("D037", "D0XX")),
        "missing token in .gsd/milestones/M004/slices/S07/M004-VALIDATION-REMEDIATION.md: ## D037 rendering rescope",
    )
    expect_failure(
        "forbidden auto-mode claim",
        lambda texts: texts.__setitem__(
            "remediation",
            texts["remediation"] + "\n\nAuto note: auto-mode ran `cargo winx`.\n",
        ),
        "forbidden auto-mode windowed-run claim in remediation: auto-mode ran `cargo winx`",
    )
    print("OK: self-tests covered roadmap placeholder removal, D037 citation, and forbidden auto-mode windowed-run claim.")
    return 0


def main(argv: list[str]) -> int:
    try:
        if argv[1:] == ["--self-test"]:
            return run_self_tests()
        verify_evidence_files_exist()
        repo = load_repo_docs()
        print(evaluate_texts(repo.texts))
        return 0
    except CheckFailure as exc:
        print(f"FAIL: {exc}")
        return 1
    except AssertionError as exc:
        print(f"FAIL: self-test assertion failed: {exc}")
        return 1


if __name__ == "__main__":
    sys.exit(main(sys.argv))
