pub mod party_ron;
pub mod skills_ron;
pub mod units_ron;

use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;

use self::party_ron::PartyConfig;
use self::skills_ron::{validate_skill_book, SkillBook};
use self::units_ron::UnitRoster;

#[derive(Resource)]
pub struct UnitRosterHandle(pub Handle<UnitRoster>);

#[allow(dead_code)]
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

pub struct DataPlugin;

impl Plugin for DataPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<UnitRoster>::new(&["ron"]))
            .add_plugins(RonAssetPlugin::<SkillBook>::new(&["ron"]))
            .add_plugins(RonAssetPlugin::<PartyConfig>::new(&["ron"]))
            .init_resource::<DataLoadState>()
            .add_systems(Startup, load_data)
            .add_systems(Update, (hydrate_data_ready, validate_skill_book_on_load));
    }
}

fn load_data(mut commands: Commands, asset_server: Res<AssetServer>) {
    let h: Handle<UnitRoster> = asset_server.load("data/units.ron");
    commands.insert_resource(UnitRosterHandle(h));
    let h: Handle<SkillBook> = asset_server.load("data/skills.ron");
    commands.insert_resource(SkillBookHandle(h));
    let h: Handle<PartyConfig> = asset_server.load("data/party.ron");
    commands.insert_resource(PartyConfigHandle(h));
}

fn hydrate_data_ready(
    mut commands: Commands,
    mut roster_events: MessageReader<AssetEvent<UnitRoster>>,
    mut party_events: MessageReader<AssetEvent<PartyConfig>>,
    roster_handle: Option<Res<UnitRosterHandle>>,
    party_handle: Option<Res<PartyConfigHandle>>,
    rosters: Res<Assets<UnitRoster>>,
    parties: Res<Assets<PartyConfig>>,
    mut state: ResMut<DataLoadState>,
    data_ready: Option<Res<DataReady>>,
) {
    let Some(roster_handle) = roster_handle else {
        return;
    };
    let Some(party_handle) = party_handle else {
        return;
    };

    for event in roster_events.read() {
        let id = match event {
            AssetEvent::LoadedWithDependencies { id } => id,
            AssetEvent::Modified { id } => id,
            _ => continue,
        };
        if *id != roster_handle.0.id() {
            continue;
        }
        if let Some(roster) = rosters.get(&roster_handle.0) {
            state.roster = true;
            info!("roster data loaded: {} units", roster.0.len());
        }
    }

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

    if state.roster && state.party && data_ready.is_none() {
        commands.insert_resource(DataReady);
    }
}

fn validate_skill_book_on_load(
    mut events: MessageReader<AssetEvent<SkillBook>>,
    handle: Option<Res<SkillBookHandle>>,
    books: Res<Assets<SkillBook>>,
) {
    let Some(handle) = handle else { return };
    for event in events.read() {
        let id = match event {
            AssetEvent::LoadedWithDependencies { id } => id,
            AssetEvent::Modified { id } => id,
            _ => continue,
        };
        if *id != handle.0.id() {
            continue;
        }
        if let Some(book) = books.get(&handle.0) {
            if let Err(e) = validate_skill_book(book) {
                panic!("SkillBook validation failed: {e}");
            }
        }
    }
}
