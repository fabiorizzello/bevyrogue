pub mod party_ron;
pub mod skill_timeline;
pub mod skills_ron;
pub mod units_ron;

use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;

use crate::combat::runtime::{ExtRegistries, TimelineLibrary};

use self::party_ron::PartyConfig;
use self::skill_timeline::compile_skill_book_timelines;
use self::skills_ron::{SkillBook, validate_skill_book};
use self::units_ron::UnitRoster;

/// All per-digimon unit RON sources (relative to `assets/`).
pub const DIGIMON_UNIT_PATHS: &[&str] = &[
    "data/digimon/agumon/unit.ron",
    "data/digimon/gabumon/unit.ron",
    "data/digimon/dorumon/unit.ron",
    "data/digimon/renamon/unit.ron",
    "data/digimon/patamon/unit.ron",
    "data/digimon/tentomon/unit.ron",
];

/// All per-enemy unit RON sources (relative to `assets/`).
pub const ENEMY_UNIT_PATHS: &[&str] = &[
    "data/enemies/devimon/unit.ron",
    "data/enemies/goblimon/unit.ron",
    "data/enemies/ogremon/unit.ron",
];

/// All per-digimon skill RON sources (relative to `assets/`).
pub const DIGIMON_SKILL_PATHS: &[&str] = &[
    "data/digimon/agumon/skills.ron",
    "data/digimon/gabumon/skills.ron",
    "data/digimon/dorumon/skills.ron",
    "data/digimon/renamon/skills.ron",
    "data/digimon/patamon/skills.ron",
    "data/digimon/tentomon/skills.ron",
];

/// All per-enemy skill RON sources (relative to `assets/`).
pub const ENEMY_SKILL_PATHS: &[&str] = &[
    "data/enemies/devimon/skills.ron",
    "data/enemies/goblimon/skills.ron",
    "data/enemies/ogremon/skills.ron",
];



#[derive(Resource)]
pub struct UnitRosterHandles(pub Vec<Handle<UnitRoster>>);

#[derive(Resource)]
pub struct SkillBookHandles(pub Vec<Handle<SkillBook>>);

/// Aggregated roster — assembled from per-digimon sources once all handles are loaded.
#[derive(Resource)]
pub struct UnitRosterHandle(pub Handle<UnitRoster>);

/// Aggregated skill book — assembled from per-digimon sources once all handles are loaded.
#[derive(Resource)]
pub struct SkillBookHandle(pub Handle<SkillBook>);

#[derive(Resource)]
pub struct PartyConfigHandle(pub Handle<PartyConfig>);

#[derive(Resource, Default)]
pub struct DataReady;

#[derive(Resource, Default)]
struct DataLoadState {
    roster: bool,
    party: bool,
}

/// Tracks which partial assets have reported LoadedWithDependencies.
#[derive(Resource)]
struct PartialLoadTracker {
    unit_loaded: Vec<bool>,
    skill_loaded: Vec<bool>,
    roster_assembled: bool,
    skill_book_assembled: bool,
}

pub struct DataPlugin;

impl Plugin for DataPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<UnitRoster>::new(&["ron"]))
            .add_plugins(RonAssetPlugin::<SkillBook>::new(&["ron"]))
            .add_plugins(RonAssetPlugin::<PartyConfig>::new(&["ron"]))
            .init_resource::<TimelineLibrary<String>>()
            .init_resource::<DataLoadState>()
            .add_systems(Startup, load_data)
            .add_systems(
                Update,
                (
                    (assemble_roster, assemble_skill_book),
                    (hydrate_data_ready, sync_skill_book_on_load),
                )
                    .chain(),
            );
    }
}

fn load_data(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Load all per-digimon unit files.
    let unit_handles: Vec<Handle<UnitRoster>> = DIGIMON_UNIT_PATHS
        .iter()
        .chain(ENEMY_UNIT_PATHS.iter())
        .map(|path| asset_server.load(*path))
        .collect();
    let unit_count = unit_handles.len();

    let skill_handles: Vec<Handle<SkillBook>> = DIGIMON_SKILL_PATHS
        .iter()
        .chain(ENEMY_SKILL_PATHS.iter())
        .map(|path| asset_server.load(*path))
        .collect();
    let skill_count = skill_handles.len();

    commands.insert_resource(UnitRosterHandles(unit_handles));
    commands.insert_resource(SkillBookHandles(skill_handles));
    commands.insert_resource(PartialLoadTracker {
        unit_loaded: vec![false; unit_count],
        skill_loaded: vec![false; skill_count],
        roster_assembled: false,
        skill_book_assembled: false,
    });

    let h: Handle<PartyConfig> = asset_server.load("data/party.ron");
    commands.insert_resource(PartyConfigHandle(h));
}

/// Watches for per-digimon unit RON loads and assembles the merged UnitRoster.
fn assemble_roster(
    mut commands: Commands,
    mut events: MessageReader<AssetEvent<UnitRoster>>,
    handles: Option<Res<UnitRosterHandles>>,
    mut tracker: ResMut<PartialLoadTracker>,
    mut rosters: ResMut<Assets<UnitRoster>>,
) {
    let Some(handles) = handles else {
        return;
    };
    if tracker.roster_assembled {
        return;
    }

    for event in events.read() {
        let id = match event {
            AssetEvent::LoadedWithDependencies { id } => *id,
            AssetEvent::Modified { id } => *id,
            _ => continue,
        };
        for (i, handle) in handles.0.iter().enumerate() {
            if handle.id() == id {
                tracker.unit_loaded[i] = true;
            }
        }
    }

    if !tracker.unit_loaded.iter().all(|v| *v) {
        return;
    }

    // All partial rosters loaded — merge into one.
    let mut merged = Vec::new();
    for handle in &handles.0 {
        if let Some(partial) = rosters.get(handle) {
            merged.extend(partial.0.iter().cloned());
        }
    }
    let total = merged.len();
    let aggregated_handle = rosters.add(UnitRoster(merged));
    commands.insert_resource(UnitRosterHandle(aggregated_handle));
    tracker.roster_assembled = true;
    info!("roster assembled from per-digimon sources: {} units", total);
}

/// Watches for per-digimon skill RON loads and assembles the merged SkillBook.
fn assemble_skill_book(
    mut commands: Commands,
    mut events: MessageReader<AssetEvent<SkillBook>>,
    handles: Option<Res<SkillBookHandles>>,
    mut tracker: ResMut<PartialLoadTracker>,
    mut books: ResMut<Assets<SkillBook>>,
) {
    let Some(handles) = handles else {
        return;
    };
    if tracker.skill_book_assembled {
        return;
    }

    for event in events.read() {
        let id = match event {
            AssetEvent::LoadedWithDependencies { id } => *id,
            AssetEvent::Modified { id } => *id,
            _ => continue,
        };
        for (i, handle) in handles.0.iter().enumerate() {
            if handle.id() == id {
                tracker.skill_loaded[i] = true;
            }
        }
    }

    if !tracker.skill_loaded.iter().all(|v| *v) {
        return;
    }

    // All partial skill books loaded — merge into one.
    let mut merged = Vec::new();
    for handle in &handles.0 {
        if let Some(partial) = books.get(handle) {
            merged.extend(partial.0.iter().cloned());
        }
    }
    let total = merged.len();
    let aggregated_handle = books.add(SkillBook(merged));
    commands.insert_resource(SkillBookHandle(aggregated_handle));
    tracker.skill_book_assembled = true;
    info!(
        "skill book assembled from per-digimon sources: {} skills",
        total
    );
}

fn hydrate_data_ready(
    mut commands: Commands,
    mut party_events: MessageReader<AssetEvent<PartyConfig>>,
    roster_handle: Option<Res<UnitRosterHandle>>,
    party_handle: Option<Res<PartyConfigHandle>>,
    rosters: Res<Assets<UnitRoster>>,
    parties: Res<Assets<PartyConfig>>,
    mut state: ResMut<DataLoadState>,
    data_ready: Option<Res<DataReady>>,
) {
    if data_ready.is_some() {
        return;
    }

    // Roster readiness: the aggregated handle is present and the asset exists.
    if !state.roster {
        if let Some(ref roster_handle) = roster_handle {
            if let Some(roster) = rosters.get(&roster_handle.0) {
                state.roster = true;
                info!("roster data loaded: {} units", roster.0.len());
            }
        }
    }

    // Party readiness: detect via events (loaded from the asset server).
    if !state.party {
        if let Some(ref party_handle) = party_handle {
            for event in party_events.read() {
                let id = match event {
                    AssetEvent::LoadedWithDependencies { id } => id,
                    AssetEvent::Modified { id } => id,
                    _ => continue,
                };
                if *id != party_handle.0.id() {
                    continue;
                }
                if let Some(party) = parties.get(&party_handle.0) {
                    state.party = true;
                    info!(
                        "party config loaded: allies={:?}, tamer={:?}",
                        party.ally_ids, party.tamer_id
                    );
                }
            }
        }
    }

    if state.roster && state.party {
        commands.insert_resource(DataReady);
    }
}

fn sync_skill_book_on_load(
    handle: Option<Res<SkillBookHandle>>,
    books: Res<Assets<SkillBook>>,
    regs: Res<ExtRegistries>,
    mut library: ResMut<TimelineLibrary<String>>,
    mut taxonomy: ResMut<crate::combat::runtime::signal::SignalTaxonomy>,
    tracker: Res<PartialLoadTracker>,
) {
    use crate::combat::runtime::timeline::{BeatKind, BeatPayload};

    // Only run once, after the skill book has been assembled and library is empty.
    if !tracker.skill_book_assembled || !library.timelines.is_empty() {
        return;
    }

    let Some(handle) = handle else {
        return;
    };

    if let Some(book) = books.get(&handle.0) {
        if let Err(e) = validate_skill_book(book) {
            panic!("SkillBook validation failed: {e}");
        }
        let compiled = compile_skill_book_timelines(book, &regs)
            .unwrap_or_else(|e| panic!("SkillBook timeline compilation failed: {e}"));

        for timeline in &compiled {
            for beat in &timeline.beats {
                if let Some(BeatPayload::BlueprintSignal { owner, name, .. }) = &beat.payload {
                    taxonomy.register(
                        Box::leak(owner.clone().into_boxed_str()),
                        Box::leak(name.clone().into_boxed_str()),
                    );
                }
                if let BeatKind::Loop { body, .. } = &beat.kind {
                    for inner in body {
                        if let Some(BeatPayload::BlueprintSignal { owner, name, .. }) =
                            &inner.payload
                        {
                            taxonomy.register(
                                Box::leak(owner.clone().into_boxed_str()),
                                Box::leak(name.clone().into_boxed_str()),
                            );
                        }
                    }
                }
            }
        }

        info!(
            "skill timeline library loaded: {} compiled timelines",
            compiled.len()
        );
        library.timelines = compiled;
    }
}

// ── Compile-time aggregate helpers (for tests and CLI) ──────────────────────

#[allow(dead_code)] // used by integration tests and combat_cli binary
pub fn aggregate_unit_roster() -> UnitRoster {
    let fragments: &[&str] = &[
        include_str!("../../assets/data/digimon/agumon/unit.ron"),
        include_str!("../../assets/data/digimon/gabumon/unit.ron"),
        include_str!("../../assets/data/digimon/dorumon/unit.ron"),
        include_str!("../../assets/data/digimon/renamon/unit.ron"),
        include_str!("../../assets/data/digimon/patamon/unit.ron"),
        include_str!("../../assets/data/digimon/tentomon/unit.ron"),
        include_str!("../../assets/data/enemies/devimon/unit.ron"),
        include_str!("../../assets/data/enemies/goblimon/unit.ron"),
        include_str!("../../assets/data/enemies/ogremon/unit.ron"),
    ];
    let mut merged = Vec::new();
    for fragment in fragments {
        let partial: UnitRoster =
            ron::from_str(fragment).expect("failed to parse per-digimon unit.ron");
        merged.extend(partial.0);
    }
    UnitRoster(merged)
}

#[allow(dead_code)] // used by integration tests and combat_cli binary
pub fn aggregate_skill_book() -> SkillBook {
    let fragments: &[&str] = &[
        include_str!("../../assets/data/digimon/agumon/skills.ron"),
        include_str!("../../assets/data/digimon/gabumon/skills.ron"),
        include_str!("../../assets/data/digimon/dorumon/skills.ron"),
        include_str!("../../assets/data/digimon/renamon/skills.ron"),
        include_str!("../../assets/data/digimon/patamon/skills.ron"),
        include_str!("../../assets/data/digimon/tentomon/skills.ron"),
        include_str!("../../assets/data/enemies/devimon/skills.ron"),
        include_str!("../../assets/data/enemies/goblimon/skills.ron"),
        include_str!("../../assets/data/enemies/ogremon/skills.ron"),
    ];
    let mut merged = Vec::new();
    for fragment in fragments {
        let partial: SkillBook =
            ron::from_str(fragment).expect("failed to parse per-digimon skills.ron");
        merged.extend(partial.0);
    }
    SkillBook(merged)
}

#[allow(dead_code)] // used by integration tests
pub fn aggregate_skill_book_ron_text() -> String {
    let fragments: &[&str] = &[
        include_str!("../../assets/data/digimon/agumon/skills.ron"),
        include_str!("../../assets/data/digimon/gabumon/skills.ron"),
        include_str!("../../assets/data/digimon/dorumon/skills.ron"),
        include_str!("../../assets/data/digimon/renamon/skills.ron"),
        include_str!("../../assets/data/digimon/patamon/skills.ron"),
        include_str!("../../assets/data/digimon/tentomon/skills.ron"),
        include_str!("../../assets/data/enemies/devimon/skills.ron"),
        include_str!("../../assets/data/enemies/goblimon/skills.ron"),
        include_str!("../../assets/data/enemies/ogremon/skills.ron"),
    ];

    // Merge all fragments into a single RON list by stripping outer brackets
    // and concatenating with commas.
    let mut inner_parts = Vec::new();
    for fragment in fragments {
        let trimmed = fragment.trim();
        // Each fragment is a RON list: [ ... ]. Strip the outer brackets.
        if let Some(inner) = trimmed.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            let inner = inner.trim().trim_end_matches(',').trim();
            if !inner.is_empty() {
                inner_parts.push(inner.to_string());
            }
        }
    }
    format!("[\n{}\n]", inner_parts.join(",\n"))
}
