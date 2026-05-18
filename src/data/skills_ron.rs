mod types;
mod validation;

pub use types::{
    BounceSelector, CustomSignalPayload, DamageCurve, Effect, LegacyEffect, LegalityReasonCode,
    RepeatPolicy, SelfTargetRule, SkillBook, SkillCustomSignal, SkillDef, SkillImplementation,
    SkillTargeting, TargetHpRule, TargetLife, TargetShape, TargetSide,
};
pub use validation::{SkillBookValidationCategory, SkillBookValidationError, validate_skill_book};

#[cfg(test)]
mod tests;
