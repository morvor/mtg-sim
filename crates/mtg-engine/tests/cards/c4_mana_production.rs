//! Mana production, round 4 (cards4-costs-mana-replacements, `oracle/patterns/mana_5.rs`,
//! `kw/mana_ability_added.rs`): a list of mana, "three mana of that type", the color of the
//! mana just added, a triggered mana ability on a land's mana ability adding the chosen
//! color, one mana of a type a land produced for each other land with its name, the exiled
//! cards used to craft a permanent as a source of colors, and "the next spell they cast".

use mtg_engine::ability::AbilityKind;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::types::{Color, ColorSet};
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

fn pool(t: &TestGame, p: PlayerId) -> Vec<ManaType> {
    t.g.player(p).mana_pool.mana.iter().map(|m| m.ty).collect()
}

fn uid_of(t: &mut TestGame, src: ObjectId, prefix: &str) -> u64 {
    t.g.recompute();
    t.g.obj(src)
        .chars
        .abilities
        .iter()
        .find(|a| matches!(a.kind, AbilityKind::Activated(_)) && a.text.starts_with(prefix))
        .map(|a| a.uid)
        .unwrap_or_else(|| panic!("no activated ability starting with {prefix:?}"))
}

fn activate_uid(t: &mut TestGame, p: PlayerId, src: ObjectId, uid: u64) {
    t.g.recompute();
    t.g.turn.priority = Some(p);
    t.g.activate_ability(p, src, uid).expect("activation");
    t.g.flush_events();
}

#[test]
fn esper_terra_chapter_four_adds_two_mana_of_each_color() {
    cr!("106.1", "714.2b");
    ruling!(
        "Terra, Magical Adept // Esper Terra",
        "fourth chapter ability isn't a mana ability"
    );
    assert_supported(&["Terra, Magical Adept // Esper Terra"]);
    let mut t = TestGame::new(2);
    t.advance_to(P0, mtg_engine::turn::Step::PrecombatMain);
    let terra = t.battlefield(P0, "Terra, Magical Adept");
    assert!(mtg_engine::dfc::transform(&mut t.g, terra));
    t.g.recompute();
    let saga = t.g.current(terra);
    t.g.add_counters(Entity::Object(saga), "lore", 3, None);
    t.resolve_all();
    assert!(pool(&t, P0).is_empty());
    t.g.add_counters(Entity::Object(t.g.current(saga)), "lore", 1, None);
    // It uses the stack.
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    let mut got = pool(&t, P0);
    got.sort_by_key(|m| format!("{m:?}"));
    let mut want = vec![
        ManaType::W,
        ManaType::W,
        ManaType::U,
        ManaType::U,
        ManaType::B,
        ManaType::B,
        ManaType::R,
        ManaType::R,
        ManaType::G,
        ManaType::G,
    ];
    want.sort_by_key(|m| format!("{m:?}"));
    assert_eq!(got, want);
}

#[test]
fn incubation_druid_adds_three_mana_of_one_type_with_a_counter() {
    cr!("106.7", "605.1a");
    ruling!(
        "Incubation Druid",
        "checks the effects of all mana-producing abilities"
    );
    assert_supported(&["Incubation Druid"]);
    let mut t = TestGame::new(2);
    let druid = t.battlefield(P0, "Incubation Druid");
    t.battlefield(P0, "Island");
    let uid = uid_of(&mut t, druid, "{T}");
    activate_uid(&mut t, P0, druid, uid);
    assert_eq!(pool(&t, P0), vec![ManaType::U]);
    // With a +1/+1 counter: three mana of that type.
    let mut t = TestGame::new(2);
    let druid = t.battlefield(P0, "Incubation Druid");
    t.battlefield(P0, "Swamp");
    t.g.add_counters(Entity::Object(druid), "+1/+1", 1, None);
    let uid = uid_of(&mut t, druid, "{T}");
    activate_uid(&mut t, P0, druid, uid);
    assert_eq!(pool(&t, P0), vec![ManaType::B; 3]);
    // No land: no type, no mana (CR 106.7).
    let mut t = TestGame::new(2);
    let druid = t.battlefield(P0, "Incubation Druid");
    t.g.add_counters(Entity::Object(druid), "+1/+1", 1, None);
    let uid = uid_of(&mut t, druid, "{T}");
    activate_uid(&mut t, P0, druid, uid);
    assert!(pool(&t, P0).is_empty());
}

#[test]
fn foraging_wickermaw_becomes_the_color_of_the_mana_it_added() {
    cr!("106.1", "605.1a", "105.3");
    assert_supported(&["Foraging Wickermaw"]);
    let mut t = TestGame::new(2);
    let wm = t.battlefield(P0, "Foraging Wickermaw");
    t.battlefield(P0, "Wastes");
    let uid = uid_of(&mut t, wm, "{1}");
    activate_uid(&mut t, P0, wm, uid);
    let added = pool(&t, P0);
    assert_eq!(added.len(), 1);
    let c = added[0].color().expect("a color");
    t.g.recompute();
    assert_eq!(t.g.obj(wm).chars.colors, ColorSet::single(c));
    // Until end of turn.
    t.advance_to(P1, mtg_engine::turn::Step::Upkeep);
    assert_eq!(t.g.obj(wm).chars.colors, ColorSet::NONE);
}

#[test]
fn caged_sun_adds_one_more_mana_of_the_chosen_color_for_a_lands_ability() {
    cr!("605.1b", "607.2d");
    ruling!(
        "Caged Sun",
        "triggers from the resolution of an activated mana ability"
    );
    assert_supported(&["Caged Sun"]);
    let mut t = TestGame::new(2);
    let sun = t.battlefield(P0, "Caged Sun");
    t.g.objects[sun.0 as usize].choices.color = Some(Color::Green);
    let forest = t.battlefield(P0, "Forest");
    let island = t.battlefield(P0, "Island");
    let fu = uid_of(&mut t, forest, "{T}");
    activate_uid(&mut t, P0, forest, fu);
    // A triggered mana ability: it resolved at once, no stack.
    assert_eq!(t.stack_len(), 0);
    assert_eq!(pool(&t, P0), vec![ManaType::G, ManaType::G]);
    // Mana of another color: nothing more.
    let iu = uid_of(&mut t, island, "{T}");
    activate_uid(&mut t, P0, island, iu);
    assert_eq!(pool(&t, P0), vec![ManaType::G, ManaType::G, ManaType::U]);
    // An opponent's land: "causes you to add".
    let of = t.battlefield(P1, "Forest");
    let ou = uid_of(&mut t, of, "{T}");
    activate_uid(&mut t, P1, of, ou);
    assert_eq!(pool(&t, P1), vec![ManaType::G]);
}

#[test]
fn sasayas_essence_adds_a_mana_for_each_other_land_with_the_same_name() {
    cr!("605.1b", "106.12a");
    ruling!(
        "Sasaya, Orochi Ascendant // Sasaya's Essence",
        "If Sasaya's Essence's controller has four Forests"
    );
    assert_supported(&["Sasaya, Orochi Ascendant // Sasaya's Essence"]);
    let mut t = TestGame::new(2);
    let sasaya = t.battlefield(P0, "Sasaya, Orochi Ascendant");
    assert!(mtg_engine::flip::flip(&mut t.g, sasaya));
    t.g.recompute();
    let forests = t.lands(P0, "Forest", 4);
    t.battlefield(P0, "Island");
    let fu = uid_of(&mut t, forests[0], "{T}");
    activate_uid(&mut t, P0, forests[0], fu);
    assert_eq!(pool(&t, P0), vec![ManaType::G; 4]);
}

#[test]
fn sunbird_effigy_adds_one_mana_of_each_color_among_the_cards_used_to_craft_it() {
    cr!("702.167c", "106.1");
    ruling!(
        "Sunbird Standard // Sunbird Effigy",
        "can produce at most five mana"
    );
    assert_supported(&["Sunbird Standard // Sunbird Effigy"]);
    let mut t = TestGame::new(2);
    t.advance_to(P0, mtg_engine::turn::Step::PrecombatMain);
    let standard = t.battlefield(P0, "Sunbird Standard");
    let bolt = t.graveyard(P0, "Lightning Bolt");
    let bears = t.graveyard(P0, "Grizzly Bears");
    let elves = t.graveyard(P0, "Llanowar Elves");
    t.lands(P0, "Wastes", 5);
    let uid = uid_of(&mut t, standard, "Craft");
    t.answer_choose(
        P0,
        &[
            Entity::Object(bolt),
            Entity::Object(bears),
            Entity::Object(elves),
        ],
    );
    activate_uid(&mut t, P0, standard, uid);
    t.resolve_all();
    let effigy = t.g.current(standard);
    assert_eq!(t.g.obj(effigy).chars.name, "Sunbird Effigy");
    // Red and green: one {R} and one {G}.
    let tap = uid_of(&mut t, effigy, "{T}");
    activate_uid(&mut t, P0, effigy, tap);
    let mut got = pool(&t, P0);
    got.sort_by_key(|m| format!("{m:?}"));
    assert_eq!(got, vec![ManaType::G, ManaType::R]);
}

#[test]
fn bigger_on_the_inside_gives_the_target_players_next_spell_cascade() {
    cr!("611.2f", "702.85a");
    ruling!(
        "Bigger on the Inside",
        "won't begin to apply until the targeted player starts casting a spell this turn"
    );
    assert_supported(&["Bigger on the Inside"]);
    let mut t = TestGame::new(2);
    t.advance_to(P0, mtg_engine::turn::Step::PrecombatMain);
    let land = t.battlefield(P0, "Ornithopter");
    t.battlefield(P0, "Mountain");
    t.battlefield(P0, "Forest");
    t.lands(P0, "Wastes", 3);
    let aura = t.hand(P0, "Bigger on the Inside");
    t.cast(P0, aura).target(land).go();
    t.resolve_all();
    let land = t.g.current(land);
    // Target P0: two mana of any one color, and P0's next spell has cascade.
    let uid = uid_of(&mut t, land, "{T}: Target player");
    t.answer_targets(P0, &[Entity::Player(P0)]);
    activate_uid(&mut t, P0, land, uid);
    t.resolve();
    assert_eq!(pool(&t, P0).len(), 2);
    t.lands(P0, "Forest", 2);
    let bears = t.hand(P0, "Grizzly Bears");
    let spell = t.cast(P0, bears).go();
    t.g.recompute();
    assert!(t.g.obj(spell).chars.has_keyword(KeywordKind::Cascade));
}

#[test]
fn bigger_on_the_inside_targeting_an_opponent_waits_for_their_spell() {
    cr!("611.2f");
    let mut t = TestGame::new(2);
    t.advance_to(P0, mtg_engine::turn::Step::PrecombatMain);
    let land = t.battlefield(P0, "Ornithopter");
    t.battlefield(P0, "Mountain");
    t.battlefield(P0, "Forest");
    t.lands(P0, "Wastes", 3);
    let aura = t.hand(P0, "Bigger on the Inside");
    t.cast(P0, aura).target(land).go();
    t.resolve_all();
    let land = t.g.current(land);
    let uid = uid_of(&mut t, land, "{T}: Target player");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_uid(&mut t, P0, land, uid);
    t.resolve();
    assert_eq!(pool(&t, P1).len(), 2);
    // P0's own spell doesn't get cascade; P1's does.
    t.lands(P0, "Forest", 2);
    t.g.player_mut(P1).mana_pool.mana.clear();
    t.battlefield(P1, "Mountain");
    let mine = t.hand(P0, "Grizzly Bears");
    let mine = t.cast(P0, mine).go();
    t.g.recompute();
    assert!(!t.g.obj(mine).chars.has_keyword(KeywordKind::Cascade));
    t.resolve_all();
    let theirs = t.hand(P1, "Lightning Bolt");
    let theirs = t.cast(P1, theirs).target(P0).go();
    t.g.recompute();
    assert!(t.g.obj(theirs).chars.has_keyword(KeywordKind::Cascade));
}

#[test]
fn hall_of_gemstone_makes_lands_produce_the_upkeep_players_color() {
    cr!("106.12b", "614.1a", "607.2d");
    ruling!("Hall of Gemstone", "Any colorless mana that's produced isn't affected");
    assert_supported(&["Hall of Gemstone"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hall of Gemstone");
    let forest = t.battlefield(P1, "Forest");
    let wastes = t.battlefield(P1, "Wastes");
    let mine = t.battlefield(P0, "Island");
    // P1 chooses red (W, U, B, R, G) in their upkeep.
    t.answer(
        P1,
        DecisionKind::Option,
        mtg_engine::decision::Answer::Index(3),
    );
    t.advance_to(P1, mtg_engine::turn::Step::Upkeep);
    t.resolve_all();
    let fu = uid_of(&mut t, forest, "{T}");
    activate_uid(&mut t, P1, forest, fu);
    assert_eq!(pool(&t, P1), vec![ManaType::R]);
    let wu = uid_of(&mut t, wastes, "{T}");
    activate_uid(&mut t, P1, wastes, wu);
    assert_eq!(pool(&t, P1), vec![ManaType::R, ManaType::C]);
    // Every land: P0's Island too.
    let iu = uid_of(&mut t, mine, "{T}");
    activate_uid(&mut t, P0, mine, iu);
    assert_eq!(pool(&t, P0), vec![ManaType::R]);
}
