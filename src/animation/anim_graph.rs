use std::collections::{BTreeMap, BTreeSet};

use bevy::prelude::{Asset, TypePath};
use serde::{Deserialize, Serialize};

/// Closed, typed animation graph schema for data-driven skill sequencing.
///
/// The graph stays generic on game/domain identifiers (`String` newtypes) while
/// keeping the command, predicate, playback, and target-shape vocabularies
/// closed so unknown RON values fail during deserialization.
#[derive(Asset, TypePath, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnimGraph {
    pub id: AnimGraphId,
    pub clip: ClipId,
    pub entry: NodeId,
    pub nodes: BTreeMap<NodeId, AnimNode>,
    pub transitions: Vec<AnimEdge>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AnimGraphId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ClipId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NodeId(pub String);

/// Closed role vocabulary injected by the kernel at cast time.
///
/// Graphs and players stay pure by talking only in terms of these typed roles,
/// never world entities, coordinates, or ad-hoc string labels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum AnimGraphRole {
    Caster,
    PrimaryTarget,
    AdjacentLeftTarget,
    AdjacentRightTarget,
}

/// Read-only, typed input surface for graph/player evaluation.
///
/// This stays intentionally small for M002: the contract proves the runtime
/// consumes explicit role input without granting world access or a mutable
/// graph context object.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnimGraphInput {
    #[serde(default)]
    pub roles: BTreeSet<AnimGraphRole>,
}

impl AnimGraphInput {
    pub fn new<I>(roles: I) -> Self
    where
        I: IntoIterator<Item = AnimGraphRole>,
    {
        Self {
            roles: roles.into_iter().collect(),
        }
    }

    pub fn contains(&self, role: AnimGraphRole) -> bool {
        self.roles.contains(&role)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ParamKey(pub String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StatusId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SkillIdRef(pub String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ParticleId(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Priority(pub u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrameRange(pub u32, pub u32);

impl FrameRange {
    pub fn start(self) -> u32 {
        self.0
    }

    pub fn end(self) -> u32 {
        self.1
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnimNode {
    pub frames: FrameRange,
    #[serde(default)]
    pub on_enter: Vec<Command>,
    #[serde(default)]
    pub cues: Vec<FrameCue>,
    #[serde(default)]
    pub modifier: Option<PlaybackModifier>,
    #[serde(default)]
    pub reverse: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrameCue {
    pub at: u32,
    pub command: FrameCueCommand,
}

/// Closed enum: either a presentation-layer command or a kernel release signal.
/// Keeping it closed means S02+ extensions are explicit, not ad-hoc.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FrameCueCommand {
    Presentation(Command),
    ReleaseKernel(ReleaseKernelCue),
}

/// Unit signal emitted at a specific frame to hand control back to the combat kernel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseKernelCue;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnimEdge {
    pub from: NodeId,
    pub to: TransitionTarget,
    pub when: Predicate,
    #[serde(default)]
    pub priority: Option<Priority>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransitionTarget {
    Node(NodeId),
    Exit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaybackModifier {
    Hold { extra_frames: u32 },
    SpeedMul { pct: u16 },
    Loop { count: u16 },
}

/// The command vocabulary intentionally follows the draft's approved verbs.
/// IDs, particles, and param keys stay data-driven strings/newtypes, but the
/// operation set is closed so later slices can extend it deliberately rather
/// than via ad-hoc `Custom(String)` escape hatches.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Command {
    EmitDamage {
        hits: ParamRef,
        mul: ParamRef,
        #[serde(default)]
        status: Option<StatusId>,
        #[serde(default)]
        chance_pct: Option<ParamRef>,
        #[serde(default)]
        duration: Option<ParamRef>,
        target: TargetShape,
    },
    EmitStatus {
        id: StatusId,
        duration: ParamRef,
        #[serde(default)]
        chance_pct: Option<ParamRef>,
        target: TargetShape,
    },
    SpawnParticle {
        name: ParticleId,
        origin: VfxLocus,
        motion: VfxMotion,
    },
    Shake {
        intensity: ParamRef,
        duration_ms: ParamRef,
    },
    StartQte {
        kind: QteKind,
        window: ParamRef,
        headless_default: QteOutcome,
    },
    EmitHeal {
        amount: ParamRef,
        amount_kind: HealAmountKind,
        target: TargetShape,
    },
    EmitCleanse {
        count: ParamRef,
        selector: CleanseSelector,
        target: TargetShape,
    },
    AdvanceTurn {
        pct: ParamRef,
        target: TargetShape,
    },
    DelayTurn {
        pct: ParamRef,
        target: TargetShape,
    },
    ApplyBuff {
        id: StatusId,
        duration: ParamRef,
        kind: BuffKind,
        target: TargetShape,
    },
    EmitSpGrant {
        amount: ParamRef,
        target: TargetShape,
    },
    Reposition {
        anchor: PositionAnchor,
        target: TargetShape,
    },
    BlockReaction {
        kind: BlockReactionKind,
        target_ref: BlockReactionTarget,
        damage_mult: ParamRef,
        duration: ParamRef,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParamRef {
    Static(ParamKey),
    Snapshot(ParamKey),
    BlueprintState(ParamKey),
    Literal(i32),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Predicate {
    TimeInNode,
    KernelEvent(KernelEventFilter),
    UserInput(UserInputFilter),
    Unlock(NodeId),
    And(Box<Predicate>, Box<Predicate>),
    Or(Box<Predicate>, Box<Predicate>),
    Not(Box<Predicate>),
    Always,
    /// Fires when the AnimGraph runtime sees a `ReleaseKernelCue` at the current frame.
    KernelCue,
}

/// Event kinds are kept closed even though payload identifiers stay generic.
/// This intentionally chooses a smaller typed filter surface than the design
/// draft's future full combat event catalog so extension remains explicit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KernelEventFilter {
    IncomingDamage,
    DamageDealt,
    UnitDied,
    UltimateUsed,
    Healed,
    SpGranted,
    BlockReactionTriggered,
    CasterIncapacitated,
    StatusApplied { status: StatusId },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserInputFilter {
    QteSuccess,
    QteFail,
    Confirm,
    Cancel,
    BranchChoice { choice: NodeId },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QteKind {
    Mash,
    TimedPress,
    Sequence,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QteOutcome {
    Success,
    Fail,
    Timeout,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealAmountKind {
    HpPctMax,
    HpPctMissing,
    Flat,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CleanseSelector {
    Fifo,
    Lifo,
    Random,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuffKind {
    Buff,
    Debuff,
    Dr,
    Aura,
    Mark,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PositionAnchor {
    Primary,
    Self_,
    AdjLeft,
    AdjRight,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockReactionKind {
    Guard,
    Parry,
    Shield,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockReactionTarget {
    Self_,
    Primary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VfxLocus {
    CasterCenter,
    TargetCenter,
    PrimaryTargetCenter,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VfxMotion {
    Static,
    FollowTarget,
    ArcToTarget,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetShape {
    Primary,
    Self_,
    AdjLeft,
    AdjRight,
    SingleAlly {
        slot: Option<u8>,
    },
    AdjLowest {
        metric: AdjMetric,
        side: Side,
    },
    LowestHpPctAlive {
        side: Side,
    },
    NextAliveAdj {
        side: Side,
        scan: ScanDirection,
    },
    RandomEnemyAlive {
        seed: SeedSource,
    },
    Blast(TargetAnchor),
    AoE {
        side: Side,
        exclude_dead: bool,
    },
    Bounce {
        hits: u8,
        selector: Box<TargetShape>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetAnchor {
    Primary,
    Self_,
    AdjLeft,
    AdjRight,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdjMetric {
    HpPctMin,
    HpMin,
    RawHpMin,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    EnemyTeam,
    AllyTeam,
    BothTeams,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScanDirection {
    ClockWise,
    CounterClockWise,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SeedSource {
    TurnRng,
    CombatRng,
}
