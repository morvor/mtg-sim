//! Abilities referring to choices made as a permanent entered (CR 607.2d), and ETB
//! replacements conditioned on what happened this turn.

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

fn name_card(t: &mut TestGame, p: PlayerId, name: &str) {
    t.answer(p, DecisionKind::Name, Answer::Text(name.to_string()));
}

// ---------------------------------------------------------------------------
// "the chosen name"
// ---------------------------------------------------------------------------

#[test]
fn pithing_needle_stops_non_mana_abilities_of_named_sources() {
    cr!("614.1c", "607.2d", "602.5");
    assert_supported("Pithing Needle");
    let mut t = TestGame::new(2);
    let sorc = t.battlefield(P1, "Prodigal Sorcerer");
    name_card(&mut t, P0, "Prodigal Sorcerer");
    let needle = t.enter(P0, "Pithing Needle");
    assert_eq!(
        t.obj_now(needle).choices.card_name.as_deref(),
        Some("Prodigal Sorcerer")
    );
    assert!(t.activate(P1, sorc, 0, &[Entity::Player(P0)]).is_err());
    assert_eq!(t.life(P0), 20);
}

#[test]
fn pithing_needle_allows_mana_abilities() {
    cr!("607.2d", "605.1a");
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P1, "Llanowar Elves");
    name_card(&mut t, P0, "Llanowar Elves");
    t.enter(P0, "Pithing Needle");
    assert!(t.activate(P1, elves, 0, &[]).is_ok());
}

#[test]
fn phyrexian_revoker_stops_mana_abilities_too() {
    cr!("607.2d", "605.1a");
    assert_supported("Phyrexian Revoker");
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P1, "Llanowar Elves");
    name_card(&mut t, P0, "Llanowar Elves");
    t.enter(P0, "Phyrexian Revoker");
    assert!(t.activate(P1, elves, 0, &[]).is_err());
}

#[test]
fn meddling_mage_stops_casting_the_named_spell() {
    cr!("607.2d", "601.2");
    assert_supported("Meddling Mage");
    let mut t = TestGame::new(2);
    name_card(&mut t, P0, "Lightning Bolt");
    t.enter(P0, "Meddling Mage");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 2);
    let bolt = t.hand(P1, "Lightning Bolt");
    let shock = t.hand(P1, "Shock");
    assert!(t.cast(P1, bolt).target(P0).try_go().is_err());
    assert!(t.cast(P1, shock).target(P0).try_go().is_ok());
    // Its own controller can't cast it either.
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 1);
    let bolt0 = t.hand(P0, "Lightning Bolt");
    assert!(t.cast(P0, bolt0).target(P1).try_go().is_err());
}

// ---------------------------------------------------------------------------
// "the chosen color" in triggers, "the chosen player"
// ---------------------------------------------------------------------------

#[test]
fn cast_trigger_for_spells_of_the_chosen_color() {
    cr!("607.2d", "603.2");
    assert_supported("Diamond Mare");
    let mut t = TestGame::new(2);
    // Red is index 3 in white, blue, black, red, green.
    t.answer(P0, DecisionKind::Option, Answer::Index(3));
    t.enter(P0, "Diamond Mare");
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Mountain", 1);
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 20, "green spell: no trigger");
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 21, "red spell: gain 1 life");
}

#[test]
fn protection_from_the_chosen_player() {
    cr!("702.16k", "607.2d");
    assert_supported("True-Name Nemesis");
    let mut t = TestGame::new(3);
    let players = [Entity::Player(P1)];
    t.answer_choose(P0, &players);
    let tnn = t.enter(P0, "True-Name Nemesis");
    assert_eq!(t.obj_now(tnn).choices.player, Some(P1));
    let goblin1 = t.battlefield(P1, "Raging Goblin");
    let goblin2 = t.battlefield(P2, "Raging Goblin");
    let now = t.g.current(tnn);
    // Protection from everything the chosen player controls, and only from that player.
    assert!(t.g.protected_from(now, goblin1));
    assert!(!t.g.protected_from(now, goblin2));
}

// ---------------------------------------------------------------------------
// Conditional counters: raid, revolt, "an opponent lost life this turn", ...
// ---------------------------------------------------------------------------

#[test]
fn raid_counter_only_after_attacking() {
    cr!("614.1c", "508.1");
    assert_supported("Swaggering Corsair");
    let mut t = TestGame::new(2);
    let a = t.enter(P0, "Swaggering Corsair");
    assert_eq!(t.counters(a, "+1/+1"), 0);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    let b = t.enter(P0, "Swaggering Corsair");
    assert_eq!(t.counters(b, "+1/+1"), 1);
    // The opponent didn't attack this turn.
    let c = t.enter(P1, "Swaggering Corsair");
    assert_eq!(t.counters(c, "+1/+1"), 0);
}

#[test]
fn revolt_counters_after_a_permanent_left() {
    cr!("614.1c");
    assert_supported("Greenwheel Liberator");
    let mut t = TestGame::new(2);
    let a = t.enter(P0, "Greenwheel Liberator");
    assert_eq!(t.counters(a, "+1/+1"), 0);
    // An opponent's permanent leaving doesn't count.
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(theirs).go();
    t.resolve();
    let b = t.enter(P0, "Greenwheel Liberator");
    assert_eq!(t.counters(b, "+1/+1"), 0);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let bolt2 = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt2).target(mine).go();
    t.resolve();
    let c = t.enter(P0, "Greenwheel Liberator");
    assert_eq!(t.counters(c, "+1/+1"), 2);
}

#[test]
fn counter_if_an_opponent_lost_life() {
    cr!("614.1c", "119.3");
    assert_supported("Frilled Sparkshooter");
    let mut t = TestGame::new(2);
    let a = t.enter(P0, "Frilled Sparkshooter");
    assert_eq!(t.counters(a, "+1/+1"), 0);
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve();
    let b = t.enter(P0, "Frilled Sparkshooter");
    assert_eq!(t.counters(b, "+1/+1"), 1);
}

#[test]
fn counters_if_two_spells_were_cast() {
    cr!("614.1c", "601.2i");
    assert_supported("Effortless Master");
    // As the first spell cast this turn: no counters.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Island", 3);
    let m = t.hand(P0, "Effortless Master");
    t.cast(P0, m).go();
    t.resolve();
    assert_eq!(t.counters(m, "+1/+1"), 0);
    // After another spell: it's the second spell cast this turn.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Island", 3);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve();
    let m = t.hand(P0, "Effortless Master");
    t.cast(P0, m).go();
    t.resolve();
    assert_eq!(t.counters(m, "+1/+1"), 2);
}

#[test]
fn counters_if_not_cast() {
    cr!("614.1c");
    assert_supported("Freestrider Commando");
    let mut t = TestGame::new(2);
    let a = t.enter(P0, "Freestrider Commando");
    assert_eq!(t.counters(a, "+1/+1"), 2);
    t.lands(P0, "Forest", 3);
    let b = t.hand(P0, "Freestrider Commando");
    t.cast(P0, b).go();
    t.resolve();
    assert_eq!(t.counters(b, "+1/+1"), 0);
}

// ---------------------------------------------------------------------------
// "This effect doesn't remove this Aura" (CR 702.16n)
// ---------------------------------------------------------------------------

#[test]
fn protection_aura_doesnt_remove_itself() {
    cr!("702.16c", "702.16n", "704.5m");
    assert_supported("White Ward");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let pacifism = t.battlefield(P1, "Pacifism");
    assert!(t.g.attach(pacifism, Entity::Object(bears)));
    t.lands(P0, "Plains", 1);
    let ward = t.hand(P0, "White Ward");
    t.cast(P0, ward).target(bears).go();
    t.resolve();
    // Protection from white removes Pacifism, but not White Ward itself.
    assert!(t.on_battlefield(ward));
    assert_eq!(
        t.obj_now(ward).attached_to,
        Some(Entity::Object(t.g.current(bears)))
    );
    assert!(!t.on_battlefield(pacifism));
    assert!(t.in_graveyard(P1, "Pacifism"));
}

#[test]
fn chosen_color_protection_aura_and_sacrifice_ability() {
    cr!("702.16n", "607.2d", "611.2c");
    assert_supported("Floating Shield");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.lands(P0, "Plains", 3);
    let shield = t.hand(P0, "Floating Shield");
    // Choose white (index 0): the white Aura stays on the creature.
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.cast(P0, shield).target(bears).go();
    t.resolve();
    assert!(t.on_battlefield(shield));
    let knight = t.battlefield(P1, "Elite Vanguard");
    let bears_now = t.g.current(bears);
    assert!(t.g.protected_from(bears_now, knight));
    // "Sacrifice this Aura: Target creature gains protection from the chosen color
    // until end of turn."
    let shield_now = t.g.current(shield);
    t.activate(P0, shield_now, 0, &[Entity::Object(elves)])
        .unwrap();
    t.resolve();
    assert!(!t.on_battlefield(shield));
    assert!(t.g.protected_from(t.g.current(elves), knight));
    assert!(!t.g.protected_from(t.g.current(bears), knight));
}

// ---------------------------------------------------------------------------
// "is the chosen type" (basic land types)
// ---------------------------------------------------------------------------

#[test]
fn land_is_the_chosen_basic_land_type() {
    cr!("305.6", "607.2d", "614.12a");
    assert_supported("Multiversal Passage");
    let mut t = TestGame::new(2);
    // Plains, Island, Swamp, Mountain, Forest: choose Swamp; then pay 2 life.
    t.answer(P0, DecisionKind::Option, Answer::Index(2));
    t.answer_yes(P0, true);
    let p = t.hand(P0, "Multiversal Passage");
    t.play_land(P0, p).unwrap();
    let now = t.g.current(p);
    assert!(!t.obj_now(p).tapped);
    assert_eq!(t.life(P0), 18);
    assert!(t.g.obj(now).chars.has_subtype("Swamp"));
    // The intrinsic "{T}: Add {B}" ability.
    t.activate(P0, now, 0, &[]).unwrap();
    assert_eq!(
        t.g.players[0]
            .mana_pool
            .count(mtg_engine::mana::ManaType::B),
        1
    );
}

#[test]
fn enchanted_land_is_the_chosen_type() {
    cr!("305.7", "607.2d");
    assert_supported("Convincing Mirage");
    let mut t = TestGame::new(2);
    let land = t.battlefield(P1, "Mountain");
    t.lands(P0, "Island", 2);
    let m = t.hand(P0, "Convincing Mirage");
    // Choose Island (index 1).
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.cast(P0, m).target(land).go();
    t.resolve();
    let now = t.g.current(land);
    assert!(t.g.obj(now).chars.has_subtype("Island"));
    assert!(!t.g.obj(now).chars.has_subtype("Mountain"));
    t.activate(P1, now, 0, &[]).unwrap();
    assert_eq!(
        t.g.players[1]
            .mana_pool
            .count(mtg_engine::mana::ManaType::U),
        1
    );
}
