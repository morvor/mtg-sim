//! Cast and play trigger conditions (CR 601.2i, 603.2, 305.1): spell alternatives ("a
//! noncreature or Dragon spell"), "other than your first spell each turn", "the first
//! noncreature spell of a turn", "that targets an opponent or a creature an opponent
//! controls", "with the same name as a card in their graveyard", "during your main phase",
//! "another spell that has flash", "the chosen player", "enchanted player casts …",
//! "when you cast ~ from anywhere other than exile", "an opponent plays a nonbasic land".

use crate::basic_effects_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

/// The number of permanents with that subtype (tokens are named after their subtypes,
/// CR 111.4).
fn with_subtype(t: &TestGame, s: &str) -> usize {
    t.g.battlefield
        .iter()
        .filter(|o| t.g.obj(**o).chars.subtypes.iter().any(|x| x == s))
        .count()
}

#[test]
fn firespitter_whelp_triggers_on_noncreature_or_dragon_spells() {
    cr!("601.2i", "603.2");
    assert_supported("Firespitter Whelp");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Firespitter Whelp");
    t.lands(P0, "Mountain", 5);
    t.lands(P0, "Forest", 2);
    // A noncreature spell.
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 20 - 3 - 1);
    // A creature spell that isn't a Dragon.
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    // A Dragon creature spell.
    let whelp = t.hand(P0, "Firespitter Whelp");
    t.cast(P0, whelp).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
}

#[test]
fn arcbound_tracker_counts_spells_after_the_first() {
    cr!("601.2i");
    assert_supported("Arcbound Tracker");
    let mut t = TestGame::new(2);
    let tracker = t.battlefield(P0, "Arcbound Tracker");
    t.g.objects[tracker.0 as usize]
        .counters
        .insert("+1/+1".into(), 2);
    t.lands(P0, "Mountain", 4);
    let before = t.counters(tracker, "+1/+1");
    for _ in 0..2 {
        let shock = t.hand(P0, "Shock");
        t.cast(P0, shock).target(Entity::Player(P1)).go();
        t.resolve_all();
    }
    // Only the second spell this turn is "other than your first spell".
    assert_eq!(t.counters(tracker, "+1/+1"), before + 1, "{}", t.dump_log());
}

#[test]
fn ichneumon_druid_punishes_the_second_instant() {
    cr!("601.2i");
    assert_supported("Ichneumon Druid");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ichneumon Druid");
    t.lands(P1, "Mountain", 3);
    let a = t.hand(P1, "Shock");
    t.cast(P1, a).target(Entity::Player(P0)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    // A sorcery isn't an instant; a second instant is.
    let b = t.hand(P1, "Shock");
    t.cast(P1, b).target(Entity::Player(P0)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn nullstone_gargoyle_counters_only_the_first_noncreature_spell_of_a_turn() {
    cr!("601.2i", "701.6a");
    assert_supported("Nullstone Gargoyle");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Nullstone Gargoyle");
    t.lands(P0, "Mountain", 3);
    let bears = t.hand(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    t.cast(P0, bears).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    let first = t.hand(P0, "Shock");
    t.cast(P0, first).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 20, "the first noncreature spell is countered");
    let second = t.hand(P0, "Shock");
    t.cast(P0, second).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn danitha_spear_of_agony_counts_spells_targeting_opponents_or_their_creatures() {
    cr!("115.9b", "601.2i");
    assert_supported("Danitha, Spear of Agony");
    let mut t = TestGame::new(2);
    let danitha = t.battlefield(P0, "Danitha, Spear of Agony");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 3);
    let a = t.hand(P0, "Shock");
    t.cast(P0, a).target(Entity::Object(mine)).go();
    t.resolve_all();
    assert_eq!(t.counters(danitha, "+1/+1"), 0);
    let b = t.hand(P0, "Shock");
    t.cast(P0, b).target(Entity::Object(theirs)).go();
    t.resolve_all();
    assert_eq!(t.counters(danitha, "+1/+1"), 1);
    let c = t.hand(P0, "Shock");
    t.cast(P0, c).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.counters(danitha, "+1/+1"), 2);
}

#[test]
fn reparations_draws_when_an_opponent_targets_you() {
    cr!("115.9b");
    assert_supported("Reparations");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Reparations");
    t.lands(P1, "Mountain", 1);
    let shock = t.hand(P1, "Shock");
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.cast(P1, shock).target(Entity::Player(P0)).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn dragonlord_kolaghan_punishes_a_spell_named_like_a_graveyard_card() {
    cr!("201.2", "601.2i");
    assert_supported("Dragonlord Kolaghan");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dragonlord Kolaghan");
    t.lands(P1, "Forest", 4);
    t.set_step(P1, Step::PrecombatMain);
    let a = t.hand(P1, "Grizzly Bears");
    t.cast(P1, a).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    t.graveyard(P1, "Grizzly Bears");
    let b = t.hand(P1, "Grizzly Bears");
    t.cast(P1, b).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 10);
}

#[test]
fn dovins_acuity_returns_only_for_instants_in_your_main_phase() {
    cr!("505.1", "601.2i");
    assert_supported("Dovin's Acuity");
    let mut t = TestGame::new(2);
    let acuity = t.battlefield(P0, "Dovin's Acuity");
    t.lands(P0, "Mountain", 2);
    t.set_step(P0, Step::BeginningOfCombat);
    let a = t.hand(P0, "Shock");
    t.answer_yes(P0, true);
    t.cast(P0, a).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert!(t.on_battlefield(acuity), "not during a main phase");
    t.set_step(P0, Step::PostcombatMain);
    let b = t.hand(P0, "Shock");
    t.answer_yes(P0, true);
    t.cast(P0, b).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Dovin's Acuity"));
}

#[test]
fn slitherwisp_triggers_on_another_spell_with_flash() {
    cr!("702.8a", "601.2i");
    assert_supported("Slitherwisp");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Slitherwisp");
    t.lands(P0, "Island", 6);
    let hand = t.hand_size(P0);
    let bears = t.hand(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    t.cast(P0, bears).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    let wisp = t.hand(P0, "Ambush Viper");
    t.lands(P0, "Forest", 2);
    t.cast(P0, wisp).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn sewer_nemesis_mills_only_the_chosen_player() {
    cr!("607.2d", "601.2i");
    assert_supported("Sewer Nemesis");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    // Cards in both graveyards, so it survives whichever player is chosen.
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P1, "Grizzly Bears");
    let nemesis = t.hand(P0, "Sewer Nemesis");
    t.cast(P0, nemesis).go();
    t.resolve_all();
    t.lands(P0, "Mountain", 1);
    t.lands(P1, "Mountain", 1);
    let mut milled = vec![];
    for p in [P0, P1] {
        let libs = (t.library_size(P0), t.library_size(P1));
        let shock = t.hand(p, "Shock");
        t.cast(p, shock).target(Entity::Player(if p == P0 { P1 } else { P0 })).go();
        t.resolve_all();
        milled.push((
            p,
            libs.0 - t.library_size(P0),
            libs.1 - t.library_size(P1),
        ));
    }
    // Exactly one of the two players is the chosen one, and only their spell makes them
    // mill.
    let triggered: Vec<_> = milled.iter().filter(|(_, a, b)| a + b > 0).collect();
    assert_eq!(triggered.len(), 1, "{milled:?}");
    let (p, a, b) = triggered[0];
    assert_eq!(if *p == P0 { (*a, *b) } else { (*b, *a) }, (1, 0));
}

#[test]
fn rory_williams_triggers_on_being_cast_from_hand() {
    cr!("113.6", "601.2i");
    assert_supported("Rory Williams");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Island", 1);
    let rory = t.hand(P0, "Rory Williams");
    t.cast(P0, rory).go();
    t.resolve_all();
    assert!(t.in_exile("Rory Williams"));
    let exiled = t.g.exile.iter().copied().find(|o| t.g.obj(*o).chars.name == "Rory Williams");
    assert_eq!(t.counters(exiled.unwrap(), "time"), 3);
    let clues = t
        .g
        .battlefield
        .iter()
        .filter(|o| t.g.obj(**o).chars.subtypes.iter().any(|s| s == "Clue"))
        .count();
    assert_eq!(clues, 1);
}

#[test]
fn hidden_herd_wakes_up_when_an_opponent_plays_a_nonbasic_land() {
    cr!("305.1", "603.4");
    assert_supported("Hidden Herd");
    let mut t = TestGame::new(2);
    let herd = t.battlefield(P0, "Hidden Herd");
    // You playing a nonbasic land doesn't count.
    t.set_step(P0, Step::PrecombatMain);
    let mine = t.hand(P0, "Ghost Quarter");
    t.play_land(P0, mine).unwrap();
    t.resolve_all();
    assert!(!t.obj_now(herd).is(CardType::Creature));
    // An opponent playing a basic land doesn't either.
    t.set_step(P1, Step::PrecombatMain);
    let forest = t.hand(P1, "Forest");
    t.play_land(P1, forest).unwrap();
    t.resolve_all();
    assert!(!t.obj_now(herd).is(CardType::Creature));
    t.g.players[1].lands_played_this_turn = 0;
    let land = t.hand(P1, "Ghost Quarter");
    t.play_land(P1, land).unwrap();
    t.resolve_all();
    assert!(t.obj_now(herd).is(CardType::Creature));
    assert_eq!(t.pt(herd), (3, 3));
}

#[test]
fn hidden_herd_checks_that_it_is_still_an_enchantment() {
    cr!("603.4");
    let mut t = TestGame::new(2);
    let herd = t.battlefield(P0, "Hidden Herd");
    // Already a creature (not an enchantment): the intervening "if" fails, so it doesn't
    // trigger and stays as it is.
    t.set_step(P1, Step::PrecombatMain);
    let land = t.hand(P1, "Ghost Quarter");
    t.play_land(P1, land).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(herd), (3, 3));
    let stack_before = t.stack_len();
    t.g.players[1].lands_played_this_turn = 0;
    let land = t.hand(P1, "Ghost Quarter");
    t.play_land(P1, land).unwrap();
    t.settle();
    assert_eq!(t.stack_len(), stack_before, "no trigger: it isn't an enchantment");
}

#[test]
fn saproling_infestation_triggers_on_kicked_spells() {
    cr!("702.33d", "601.2i");
    assert_supported("Saproling Infestation");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Saproling Infestation");
    t.lands(P1, "Mountain", 6);
    let a = t.hand(P1, "Burst Lightning");
    t.cast(P1, a).target(Entity::Player(P0)).go();
    t.resolve_all();
    assert_eq!(with_subtype(&t, "Saproling"), 0, "not kicked");
    let b = t.hand(P1, "Burst Lightning");
    t.cast(P1, b).target(Entity::Player(P0)).kicked(true).go();
    t.resolve_all();
    assert_eq!(with_subtype(&t, "Saproling"), 1);
}

#[test]
fn curse_of_shaken_faith_hits_spells_after_the_first() {
    cr!("303.4", "601.2i");
    assert_supported("Curse of Shaken Faith");
    let mut t = TestGame::new(2);
    let curse = t.battlefield(P0, "Curse of Shaken Faith");
    assert!(t.g.attach(curse, Entity::Player(P1)));
    t.lands(P1, "Mountain", 2);
    for _ in 0..2 {
        let s = t.hand(P1, "Shock");
        t.cast(P1, s).target(Entity::Player(P0)).go();
        t.resolve_all();
    }
    assert_eq!(t.life(P1), 18);
}

#[test]
fn the_lost_and_the_damned_counts_lands_not_from_your_hand() {
    cr!("400.7", "603.6a");
    assert_supported("The Lost and the Damned");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Lost and the Damned");
    let forest = t.hand(P0, "Forest");
    t.play_land(P0, forest).unwrap();
    t.resolve_all();
    assert_eq!(with_subtype(&t, "Spawn"), 0, "played from hand");
    let land = t.graveyard(P0, "Forest");
    t.g.move_object(
        land,
        mtg_engine::object::Zone::Battlefield,
        mtg_engine::events::MoveCause::Effect,
        Some(P0),
    );
    t.resolve_all();
    assert_eq!(with_subtype(&t, "Spawn"), 1);
}

#[test]
fn full_throttle_untaps_attackers_at_each_combat() {
    cr!("603.7b", "500.8");
    assert_supported("Full Throttle");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 6);
    let ft = t.hand(P0, "Full Throttle");
    t.cast(P0, ft).go();
    t.resolve_all();
    t.advance_to(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        mtg_engine::decision::Answer::Attackers(vec![(bears, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::EndOfCombat);
    assert!(t.obj_now(bears).tapped);
    // The next combat phase begins: the creature that attacked untaps.
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert!(!t.obj_now(bears).tapped);
}
