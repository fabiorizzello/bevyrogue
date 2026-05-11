use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Attribute {
    Data,
    Vaccine,
    Virus,
    Free,
}

// S02 damage.rs consumes DamageTag in tests; binary integration pending S03+.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DamageTag {
    #[default]
    Physical,
    Fire,
    Ice,
    Electric,
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UnitId(pub u32);

// Produce-and-freeze for S02+ (skill references in Unit)
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct SkillId(pub String);

/// JP canonical naming per MEM019: EN 'Ultimate' = JP 'Perfect'; JP 'Ultimate' = EN 'Mega'.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EvoStage {
    BabyI,
    BabyII,
    Child,
    Adult,
    Perfect,
    Ultimate,
    SuperUltimate,
}

/// Identifies an evolutionary lineage (e.g. "agumon_line"). Wraps String — not Copy.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EvoLineId(pub String);
