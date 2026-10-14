//! Structure-coverage tests, round 5 (item 9): triggered-ability structures shared by
//! several cards that no other test exercised (see `docs/STRUCTURE_COVERAGE.md`). Each test
//! checks a real card's ability does what its text says: the right objects and players
//! affected and not others, amounts, durations ending, conditions true and false, and the
//! targets the engine offers. Combat triggers are in `structure_triggered_9_combat.rs`.

use super::structure_activated_2::{act, shuffled, target_choices};
use super::structure_activated_6::entity_choices;
use super::structure_seed_common::*;
use super::structure_triggered_5::{etb, next_turn, obj};
use super::structure_triggered_7::{kill, tokens};
use mtg_engine::decision::{Action, Answer, SpecialAction};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::kwa::ring::ring_bearer;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{counters, Color, Supertype};
use mtg_engine::*;

/// Triggered abilities on the stack or waiting to be put there.
pub fn waiting(t: &TestGame) -> usize {
    t.stack_len() + t.g.pending_triggers.len()
}

// ---------------------------------------------------------------------------
// "When ~ enters, ..."
// ---------------------------------------------------------------------------

#[test]
fn treasure_hunter_may_return_an_artifact_card_from_your_graveyard() {
    cr!("603.6a", "603.5", "115.1");
    // "When this creature enters, you may return target artifact card from your graveyard
    // to your hand."
    supported("Treasure Hunter");
    for yes in [true, false] {
        let mut t = TestGame::new(2);
        let ring = t.graveyard(P0, "Sol Ring");
        let bears = t.graveyard(P0, "Grizzly Bears");
        let theirs = t.graveyard(P1, "Sol Ring");
        t.answer_targets(P0, &[obj(ring)]);
        t.answer_yes(P0, yes);
        etb(&mut t, P0, "Treasure Hunter");
        assert_eq!(t.in_hand(P0, "Sol Ring"), yes);
        assert_eq!(t.zone(ring) == Zone::Graveyard(P0), !yes);
        assert_eq!(t.zone(theirs), Zone::Graveyard(P1));
        let offered = &target_choices(&t, P0)[0];
        assert!(offered.contains(&obj(ring)));
        assert!(!offered.contains(&obj(bears)), "not an artifact card");
        assert!(!offered.contains(&obj(theirs)), "not in your graveyard");
    }
}

#[test]
fn accursed_centaur_sacrifices_a_creature() {
    cr!("603.6a", "701.21a");
    // "When this creature enters, sacrifice a creature."
    supported("Accursed Centaur");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let forest = t.battlefield(P0, "Forest");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.answer_choose(P0, &[obj(bears)]);
    let c = etb(&mut t, P0, "Accursed Centaur");
    assert!(!t.on_battlefield(bears));
    assert!(t.on_battlefield(c) && t.on_battlefield(theirs) && t.on_battlefield(forest));
    let offered = &entity_choices(&t, P0)[0];
    assert!(offered.contains(&obj(bears)) && offered.contains(&obj(c)));
    assert!(!offered.contains(&obj(theirs)) && !offered.contains(&obj(forest)));
    // Alone, it sacrifices itself.
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    etb(&mut t, P0, "Accursed Centaur");
    assert!(t.in_graveyard(P0, "Accursed Centaur"));
    assert!(t.on_battlefield(theirs));
}

#[test]
fn utrom_scientists_taps_up_to_one_creature_and_stuns_it() {
    cr!("603.6a", "603.3d", "122.1d");
    // "When this creature enters, tap up to one target creature and put a stun counter on
    // it."
    supported("Utrom Scientists");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let forest = t.battlefield(P1, "Forest");
    t.answer_targets(P0, &[obj(bears)]);
    let s = etb(&mut t, P0, "Utrom Scientists");
    assert!(t.obj_now(bears).tapped);
    assert_eq!(t.counters(bears, counters::STUN), 1);
    assert!(!t.obj_now(giant).tapped);
    assert_eq!(t.counters(giant, counters::STUN), 0);
    let offered = &target_choices(&t, P0)[0];
    assert!(offered.contains(&obj(bears)) && offered.contains(&obj(giant)));
    assert!(offered.contains(&obj(s)));
    assert!(!offered.contains(&obj(forest)));
    // In P1's untap step the stun counter is removed instead of untapping it.
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj_now(bears).tapped);
    assert_eq!(t.counters(bears, counters::STUN), 0);
    // Up to one: no target, nothing happens.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[]);
    etb(&mut t, P0, "Utrom Scientists");
    assert!(!t.obj_now(bears).tapped);
    assert_eq!(t.counters(bears, counters::STUN), 0);
}

#[test]
fn stalactite_dagger_creates_a_changeling_shapeshifter() {
    cr!("603.6a", "111.1", "702.73a", "604.3");
    // "When this Equipment enters, create a 1/1 colorless Shapeshifter creature token with
    // changeling."
    supported("Stalactite Dagger");
    let mut t = TestGame::new(2);
    let d = etb(&mut t, P0, "Stalactite Dagger");
    let s = tokens(&t, P0, "Shapeshifter");
    assert_eq!(s.len(), 1);
    let o = t.obj_now(s[0]);
    assert_eq!(t.pt(s[0]), (1, 1));
    assert!(o.chars.colors.is_colorless());
    assert!(o.has_keyword(KeywordKind::Changeling));
    assert!(o.chars.has_subtype("Goblin") && o.chars.has_subtype("Elf"));
    assert!(tokens(&t, P1, "Shapeshifter").is_empty());
    assert_eq!(t.obj_now(d).attached_to, None);
    // Changeling is a characteristic-defining ability (layer 4): losing all abilities
    // (layer 6) doesn't take its creature types away.
    t.battlefield(P1, "Humility");
    t.g.recompute();
    let o = t.obj_now(s[0]);
    assert!(!o.has_keyword(KeywordKind::Changeling));
    assert!(o.chars.has_subtype("Goblin") && o.chars.has_subtype("Shapeshifter"));
}

#[test]
fn stickytongue_sentinel_returns_up_to_one_other_permanent_you_control() {
    cr!("603.6a", "603.3d", "115.1");
    // "When this creature enters, return up to one other target permanent you control to
    // its owner's hand."
    supported("Stickytongue Sentinel");
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(forest)]);
    let s = etb(&mut t, P0, "Stickytongue Sentinel");
    assert!(t.in_hand(P0, "Forest"));
    assert!(t.on_battlefield(bears) && t.on_battlefield(s) && t.on_battlefield(theirs));
    let offered = &target_choices(&t, P0)[0];
    assert!(offered.contains(&obj(forest)) && offered.contains(&obj(bears)));
    assert!(!offered.contains(&obj(s)), "other");
    assert!(!offered.contains(&obj(theirs)), "you control");
    // Up to one: none chosen.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[]);
    etb(&mut t, P0, "Stickytongue Sentinel");
    assert!(t.on_battlefield(bears));
    assert_eq!(t.hand_size(P0), 0);
}

#[test]
fn unsettling_twins_manifests_dread() {
    cr!("603.6a", "701.62a");
    // "When this creature enters, manifest dread."
    supported("Unsettling Twins");
    let mut t = TestGame::new(2);
    let bears = t.library_top(P0, "Grizzly Bears");
    let giant = t.library_top(P0, "Hill Giant");
    t.answer_choose(P0, &[obj(bears)]);
    etb(&mut t, P0, "Unsettling Twins");
    let m = t.g.current(bears);
    assert!(t.on_battlefield(m) && t.obj_now(m).face_down);
    assert_eq!(t.obj_now(m).controller, P0);
    assert_eq!(t.pt(m), (2, 2));
    assert_eq!(t.zone(giant), Zone::Graveyard(P0));
    assert_eq!(t.library_size(P0), 30);
    assert_eq!(t.library_size(P1), 30);
    // The top two were the choices.
    let offered = &entity_choices(&t, P0)[0];
    assert_eq!(offered.len(), 2);
    assert!(offered.contains(&obj(bears)) && offered.contains(&obj(giant)));
}

#[test]
fn jiang_yanggu_creates_mowu() {
    cr!("603.6a", "111.4");
    // "When Jiang Yanggu enters, create Mowu, a legendary 3/3 green Dog creature token."
    supported("Jiang Yanggu, Never Alone");
    let mut t = TestGame::new(2);
    etb(&mut t, P0, "Jiang Yanggu, Never Alone");
    let m = t.named_on_battlefield("Mowu");
    assert_eq!(m.len(), 1);
    let o = t.obj_now(m[0]);
    assert!(o.is_token());
    assert_eq!(o.controller, P0);
    assert!(o.chars.has_supertype(Supertype::Legendary));
    assert!(o.chars.has_subtype("Dog"));
    assert_eq!(t.pt(m[0]), (3, 3));
    assert!(o.chars.colors.contains(Color::Green));
    assert_eq!(o.chars.colors.count(), 1);
}

#[test]
fn angelheart_protector_makes_your_creature_indestructible_until_end_of_turn() {
    cr!("603.6a", "702.12b", "514.2");
    // "When this creature enters, target creature you control gains indestructible until
    // end of turn."
    supported("Angelheart Protector");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    let a = etb(&mut t, P0, "Angelheart Protector");
    let offered = &target_choices(&t, P0)[0];
    assert!(offered.contains(&obj(bears)) && offered.contains(&obj(a)));
    assert!(!offered.contains(&obj(theirs)), "you control");
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Indestructible));
    assert!(!t.obj_now(a).has_keyword(KeywordKind::Indestructible));
    kill(&mut t, bears);
    kill(&mut t, other);
    assert!(t.on_battlefield(bears));
    assert!(!t.on_battlefield(other));
    next_turn(&mut t);
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::Indestructible));
    kill(&mut t, bears);
    assert!(!t.on_battlefield(bears));
}

#[test]
fn mogg_war_marshal_makes_a_goblin_when_it_enters_or_dies() {
    cr!("603.6a", "603.6c", "603.10a", "111.1");
    // "When this creature enters or dies, create a 1/1 red Goblin creature token."
    supported("Mogg War Marshal");
    let mut t = TestGame::new(2);
    let m = etb(&mut t, P0, "Mogg War Marshal");
    let g = tokens(&t, P0, "Goblin");
    assert_eq!(g.len(), 1);
    assert_eq!(t.pt(g[0]), (1, 1));
    assert!(t.obj_now(g[0]).chars.colors.contains(Color::Red));
    kill(&mut t, m);
    assert_eq!(tokens(&t, P0, "Goblin").len(), 2);
    assert!(tokens(&t, P1, "Goblin").is_empty());
    // A Goblin token dying doesn't trigger it (it's gone, and it's "this creature").
    kill(&mut t, g[0]);
    assert_eq!(tokens(&t, P0, "Goblin").len(), 1);
}

#[test]
fn arborea_pegasus_pumps_and_gives_flying_until_end_of_turn() {
    cr!("603.6a", "613.4c", "613.1f", "514.2");
    // "When this creature enters, target creature gets +1/+1 and gains flying until end of
    // turn."
    supported("Arborea Pegasus");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(theirs)]);
    etb(&mut t, P0, "Arborea Pegasus");
    assert_eq!(t.pt(theirs), (3, 3));
    assert!(t.obj_now(theirs).has_keyword(KeywordKind::Flying));
    assert_eq!(t.pt(mine), (2, 2));
    assert!(!t.obj_now(mine).has_keyword(KeywordKind::Flying));
    next_turn(&mut t);
    assert_eq!(t.pt(theirs), (2, 2));
    assert!(!t.obj_now(theirs).has_keyword(KeywordKind::Flying));
}

#[test]
fn batterhorn_may_destroy_target_artifact() {
    cr!("603.6a", "603.5", "701.8a");
    // "When this creature enters, you may destroy target artifact."
    supported("Batterhorn");
    for yes in [true, false] {
        let mut t = TestGame::new(2);
        let ring = t.battlefield(P1, "Sol Ring");
        let mine = t.battlefield(P0, "Ornithopter");
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.answer_targets(P0, &[obj(ring)]);
        t.answer_yes(P0, yes);
        etb(&mut t, P0, "Batterhorn");
        assert_eq!(t.on_battlefield(ring), !yes);
        assert!(t.on_battlefield(mine) && t.on_battlefield(bears));
        let offered = &target_choices(&t, P0)[0];
        assert!(offered.contains(&obj(ring)) && offered.contains(&obj(mine)));
        assert!(!offered.contains(&obj(bears)));
    }
}

#[test]
fn mire_triton_mills_two_and_gains_two_life() {
    cr!("603.6a", "701.17a", "119.3");
    // "When this creature enters, mill two cards and you gain 2 life."
    supported("Mire Triton");
    let mut t = TestGame::new(2);
    etb(&mut t, P0, "Mire Triton");
    assert_eq!(t.library_size(P0), 28);
    assert_eq!(t.graveyard_size(P0), 2);
    assert_eq!(t.life(P0), 22);
    assert_eq!((t.library_size(P1), t.life(P1)), (30, 20));
}

#[test]
fn harvester_of_misery_shrinks_other_creatures_until_end_of_turn() {
    cr!("603.6a", "613.4c", "611.2c", "514.2");
    // "When this creature enters, other creatures get -2/-2 until end of turn."
    supported("Harvester of Misery");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let h = etb(&mut t, P0, "Harvester of Misery");
    assert!(!t.on_battlefield(mine) && !t.on_battlefield(theirs));
    assert_eq!(t.pt(giant), (1, 1));
    assert_eq!(t.pt(h), (5, 4), "other creatures");
    // A creature entering afterwards isn't affected (the set was locked in).
    let late = t.battlefield(P1, "Grizzly Bears");
    assert_eq!(t.pt(late), (2, 2));
    next_turn(&mut t);
    assert_eq!(t.pt(giant), (3, 3));
}

#[test]
fn shining_armor_attaches_to_target_knight_you_control() {
    cr!("603.6a", "301.5a", "115.1");
    // "When this Equipment enters, attach it to target Knight you control. / Equipped
    // creature gets +0/+2 and has vigilance."
    supported("Shining Armor");
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P0, "Knight Errant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Knight Errant");
    t.answer_targets(P0, &[obj(knight)]);
    let a = etb(&mut t, P0, "Shining Armor");
    assert_eq!(t.obj_now(a).attached_to, Some(obj(knight)));
    assert_eq!(t.pt(knight), (2, 4));
    assert!(t.obj_now(knight).has_keyword(KeywordKind::Vigilance));
    assert_eq!(t.pt(theirs), (2, 2));
    let offered = &target_choices(&t, P0)[0];
    assert!(offered.contains(&obj(knight)));
    assert!(!offered.contains(&obj(bears)), "not a Knight");
    assert!(!offered.contains(&obj(theirs)), "you control");
}

#[test]
fn thundermare_taps_all_other_creatures() {
    cr!("603.6a", "701.26a");
    // "When this creature enters, tap all other creatures."
    supported("Thundermare");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let land = t.battlefield(P1, "Forest");
    let m = etb(&mut t, P0, "Thundermare");
    assert!(t.obj_now(mine).tapped && t.obj_now(theirs).tapped);
    assert!(!t.obj_now(m).tapped, "other");
    assert!(!t.obj_now(land).tapped, "creatures");
}

#[test]
fn bishop_of_wings_gains_4_life_when_an_angel_you_control_enters() {
    cr!("603.6a", "119.3");
    // "Whenever an Angel you control enters, you gain 4 life."
    supported("Bishop of Wings");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bishop of Wings");
    etb(&mut t, P0, "Starlit Angel");
    assert_eq!(t.life(P0), 24);
    etb(&mut t, P1, "Starlit Angel");
    etb(&mut t, P0, "Grizzly Bears");
    assert_eq!(t.life(P0), 24, "not an opponent's Angel or a non-Angel");
    assert_eq!(t.life(P1), 20);
    etb(&mut t, P0, "Starlit Angel");
    assert_eq!(t.life(P0), 28);
}

#[test]
fn ajanis_anguish_deals_x_damage_to_any_target() {
    cr!("603.6a", "107.3m", "115.4");
    // "When this enchantment enters, it deals X damage to any target."
    supported("Ajani's Anguish");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 4);
    let a = t.hand(P0, "Ajani's Anguish");
    t.answer_targets(P0, &[obj(giant)]);
    t.cast(P0, a).x(3).go();
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
    assert_eq!(t.life(P1), 20);
    let offered = &target_choices(&t, P0)[0];
    assert!(offered.contains(&Entity::Player(P1)) && offered.contains(&Entity::Player(P0)));
    assert!(offered.contains(&obj(giant)));
    // X = 2 at a player.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 3);
    let a = t.hand(P0, "Ajani's Anguish");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, a).x(2).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.obj_now(giant).damage, 0);
}

#[test]
fn kyodai_makes_another_permanent_indestructible_while_you_control_it() {
    cr!("603.6a", "702.12b", "611.2b");
    // "When Kyodai enters, another target permanent gains indestructible for as long as
    // you control Kyodai."
    supported("Kyodai, Soul of Kamigawa");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let land = t.battlefield(P1, "Forest");
    t.answer_targets(P0, &[obj(giant)]);
    let k = etb(&mut t, P0, "Kyodai, Soul of Kamigawa");
    let offered = &target_choices(&t, P0)[0];
    assert!(offered.contains(&obj(giant)) && offered.contains(&obj(bears)));
    assert!(offered.contains(&obj(land)), "any permanent");
    assert!(!offered.contains(&obj(k)), "another");
    assert!(t.obj_now(giant).has_keyword(KeywordKind::Indestructible));
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::Indestructible));
    kill(&mut t, giant);
    assert!(t.on_battlefield(giant));
    // It outlasts the turn...
    next_turn(&mut t);
    assert!(t.obj_now(giant).has_keyword(KeywordKind::Indestructible));
    // ...but not Kyodai.
    kill(&mut t, k);
    assert!(!t.obj_now(giant).has_keyword(KeywordKind::Indestructible));
    kill(&mut t, giant);
    assert!(!t.on_battlefield(giant));
}

#[test]
fn affa_guard_hound_gives_plus_0_3_until_end_of_turn() {
    cr!("603.6a", "613.4c", "514.2");
    // "When this creature enters, target creature gets +0/+3 until end of turn."
    supported("Affa Guard Hound");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(theirs)]);
    let h = etb(&mut t, P0, "Affa Guard Hound");
    assert_eq!(t.pt(theirs), (2, 5));
    assert_eq!(t.pt(mine), (2, 2));
    assert_eq!(t.pt(h), (2, 2));
    let offered = &target_choices(&t, P0)[0];
    assert!(offered.contains(&obj(mine)) && offered.contains(&obj(h)));
    next_turn(&mut t);
    assert_eq!(t.pt(theirs), (2, 2));
}

#[test]
fn pilgrim_of_the_ages_may_search_for_a_basic_plains() {
    cr!("603.6a", "603.5", "701.23a");
    // "When this creature enters, you may search your library for a basic Plains card,
    // reveal it, put it into your hand, then shuffle."
    supported("Pilgrim of the Ages");
    let mut t = TestGame::new(2);
    let plains = t.library_top(P0, "Plains");
    let island = t.library_top(P0, "Island");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(plains)]);
    etb(&mut t, P0, "Pilgrim of the Ages");
    assert_eq!(t.zone(plains), Zone::Hand(P0));
    assert_eq!(t.zone(island), Zone::Library(P0));
    assert!(shuffled(&t, P0));
    let offered = entity_choices(&t, P0).concat();
    assert!(offered.contains(&obj(plains)));
    assert!(!offered.contains(&obj(island)), "not a Plains");
    // Declining: no search, no shuffle.
    let mut t = TestGame::new(2);
    t.library_top(P0, "Plains");
    t.answer_yes(P0, false);
    etb(&mut t, P0, "Pilgrim of the Ages");
    assert_eq!(t.hand_size(P0), 0);
    assert!(!shuffled(&t, P0));
}

// ---------------------------------------------------------------------------
// Dies triggers
// ---------------------------------------------------------------------------

#[test]
fn took_reaper_dying_has_the_ring_tempt_you() {
    cr!("603.6c", "603.10a", "701.54a");
    // "When this creature dies, the Ring tempts you."
    supported("Took Reaper");
    let mut t = TestGame::new(2);
    let reaper = t.battlefield(P0, "Took Reaper");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.answer_choose(P0, &[obj(giant)]);
    kill(&mut t, reaper);
    assert_eq!(ring_bearer(&t.g, P0), Some(giant));
    assert_eq!(t.g.player(P0).ring_level, 1);
    assert_eq!(ring_bearer(&t.g, P1), None);
    assert_eq!(t.g.player(P1).ring_level, 0);
    let offered = &entity_choices(&t, P0)[0];
    assert!(offered.contains(&obj(bears)) && offered.contains(&obj(giant)));
    assert!(!offered.contains(&obj(theirs)));
    // Another creature dying doesn't tempt you.
    kill(&mut t, bears);
    assert_eq!(t.g.player(P0).ring_level, 1);
}

#[test]
fn hurloon_shaman_dying_makes_each_player_sacrifice_a_land() {
    cr!("603.6c", "603.10a", "701.21a");
    // "When this creature dies, each player sacrifices a land of their choice."
    supported("Hurloon Shaman");
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Hurloon Shaman");
    let forest = t.battlefield(P0, "Forest");
    let mountain = t.battlefield(P0, "Mountain");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let island = t.battlefield(P1, "Island");
    let swamp = t.battlefield(P1, "Swamp");
    t.answer_choose(P0, &[obj(mountain)]);
    t.answer_choose(P1, &[obj(island)]);
    kill(&mut t, h);
    assert!(t.on_battlefield(forest) && !t.on_battlefield(mountain));
    assert!(!t.on_battlefield(island) && t.on_battlefield(swamp));
    assert!(t.on_battlefield(bears), "a land");
    // The other choice.
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Hurloon Shaman");
    t.battlefield(P0, "Forest");
    let island = t.battlefield(P1, "Island");
    let swamp = t.battlefield(P1, "Swamp");
    t.answer_choose(P1, &[obj(swamp)]);
    kill(&mut t, h);
    assert!(t.on_battlefield(island) && !t.on_battlefield(swamp));
    assert_eq!(
        t.g.battlefield
            .iter()
            .filter(|id| t.obj_now(**id).controller == P0)
            .count(),
        0
    );
}

#[test]
fn al_bhed_salvagers_drains_when_it_or_your_creature_or_artifact_dies() {
    cr!("603.6c", "603.10a", "119.3");
    // "Whenever this creature or another creature or artifact you control dies, target
    // opponent loses 1 life and you gain 1 life."
    supported("Al Bhed Salvagers");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Al Bhed Salvagers");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let ring = t.battlefield(P0, "Sol Ring");
    let forest = t.battlefield(P0, "Forest");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let their_ring = t.battlefield(P1, "Sol Ring");
    kill(&mut t, bears);
    assert_eq!((t.life(P0), t.life(P1)), (21, 19));
    assert_eq!(target_choices(&t, P0)[0], vec![Entity::Player(P1)]);
    kill(&mut t, ring);
    assert_eq!((t.life(P0), t.life(P1)), (22, 18));
    kill(&mut t, forest);
    kill(&mut t, theirs);
    kill(&mut t, their_ring);
    assert_eq!(
        (t.life(P0), t.life(P1)),
        (22, 18),
        "not a land or an opponent's"
    );
    kill(&mut t, s);
    assert_eq!((t.life(P0), t.life(P1)), (23, 17), "itself");
}

// ---------------------------------------------------------------------------
// Beginning-of-step triggers
// ---------------------------------------------------------------------------

#[test]
fn sandwurm_convergence_creates_a_wurm_at_your_end_step() {
    cr!("603.2", "111.1");
    // "At the beginning of your end step, create a 5/5 green Wurm creature token."
    supported("Sandwurm Convergence");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sandwurm Convergence");
    t.advance_to(P0, Step::End);
    assert_eq!(waiting(&t), 1);
    t.resolve_all();
    let w = tokens(&t, P0, "Wurm");
    assert_eq!(w.len(), 1);
    assert_eq!(t.pt(w[0]), (5, 5));
    assert!(t.obj_now(w[0]).chars.colors.contains(Color::Green));
    // Not at the opponent's end step.
    t.advance_to(P1, Step::End);
    assert_eq!(waiting(&t), 0);
    assert_eq!(tokens(&t, P0, "Wurm").len(), 1);
    assert!(tokens(&t, P1, "Wurm").is_empty());
}

#[test]
fn grafted_skullcap_discards_your_hand_at_your_end_step() {
    cr!("603.2", "701.9a");
    // "At the beginning of your end step, discard your hand."
    supported("Grafted Skullcap");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grafted Skullcap");
    t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Forest");
    t.hand(P1, "Grizzly Bears");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0);
    assert!(t.in_graveyard(P0, "Grizzly Bears") && t.in_graveyard(P0, "Forest"));
    assert_eq!(t.hand_size(P1), 1);
    // The opponent's end step: P0's new card stays.
    t.hand(P0, "Forest");
    t.advance_to(P1, Step::End);
    assert_eq!(waiting(&t), 0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn rushwood_elemental_may_put_a_counter_on_itself_each_upkeep() {
    cr!("503.1a", "603.5", "122.1");
    // "At the beginning of your upkeep, you may put a +1/+1 counter on this creature."
    supported("Rushwood Elemental");
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Rushwood Elemental");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(waiting(&t), 0, "not the opponent's upkeep");
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(waiting(&t), 1);
    t.resolve_all();
    assert_eq!(t.counters(r, counters::PLUS1), 1);
    assert_eq!(t.pt(r), (5, 5));
    assert_eq!(t.counters(bears, counters::PLUS1), 0);
    // Declining.
    t.advance_to(P1, Step::Upkeep);
    t.answer_yes(P0, false);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(r, counters::PLUS1), 1);
}

// ---------------------------------------------------------------------------
// Sagas
// ---------------------------------------------------------------------------

#[test]
fn the_night_of_the_doctor_chapter_i_destroys_all_creatures() {
    cr!("714.2b", "714.3a", "701.8a");
    // "I — Destroy all creatures."
    supported("The Night of the Doctor");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    let land = t.battlefield(P1, "Forest");
    let ring = t.battlefield(P1, "Sol Ring");
    let saga = etb(&mut t, P0, "The Night of the Doctor");
    assert_eq!(t.counters(saga, counters::LORE), 1);
    assert!(!t.on_battlefield(mine) && !t.on_battlefield(theirs));
    assert!(t.on_battlefield(land) && t.on_battlefield(ring) && t.on_battlefield(saga));
    // Chapter II (no legendary creature card to return) doesn't destroy creatures.
    let late = t.battlefield(P1, "Grizzly Bears");
    next_turn(&mut t);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    assert!(t.on_battlefield(late));
    assert!(
        t.in_graveyard(P0, "The Night of the Doctor"),
        "sacrificed after II"
    );
}

#[test]
fn origin_of_spider_man_chapter_iii_gives_double_strike_until_end_of_turn() {
    cr!("714.2b", "611.2a", "514.2");
    // "III — Target creature you control gains double strike until end of turn."
    supported("Origin of Spider-Man");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    etb(&mut t, P0, "Origin of Spider-Man");
    let spider = tokens(&t, P0, "Spider");
    assert_eq!(spider.len(), 1);
    // Chapter II.
    t.answer_targets(P0, &[obj(bears)]);
    next_turn(&mut t);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::DoubleStrike));
    // Chapter III.
    t.answer_targets(P0, &[obj(spider[0])]);
    next_turn(&mut t);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    let choices = target_choices(&t, P0);
    let offered = choices.last().unwrap();
    assert!(offered.contains(&obj(bears)) && offered.contains(&obj(spider[0])));
    assert!(!offered.contains(&obj(theirs)), "you control");
    assert!(t.obj_now(spider[0]).has_keyword(KeywordKind::DoubleStrike));
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::DoubleStrike));
    assert!(!t.obj_now(theirs).has_keyword(KeywordKind::DoubleStrike));
    next_turn(&mut t);
    assert!(!t.obj_now(spider[0]).has_keyword(KeywordKind::DoubleStrike));
}

// ---------------------------------------------------------------------------
// Mutate, turned face up, cast triggers
// ---------------------------------------------------------------------------

const MUTATE: CastMethod = CastMethod::Keyword(KeywordKind::Mutate);

#[test]
fn vulpikeet_gets_a_counter_when_it_mutates() {
    cr!("702.140d", "730.2a");
    // "Mutate {2}{W} / Flying / Whenever this creature mutates, put a +1/+1 counter on it."
    supported("Vulpikeet");
    for on_top in [true, false] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let other = t.battlefield(P0, "Grizzly Bears");
        t.lands(P0, "Plains", 3);
        let v = t.hand(P0, "Vulpikeet");
        t.cast(P0, v).method(MUTATE).target(bears).go();
        t.answer(
            P0,
            DecisionKind::Option,
            Answer::Index(if on_top { 0 } else { 1 }),
        );
        t.resolve_all();
        assert_eq!(t.counters(bears, counters::PLUS1), 1);
        assert_eq!(t.pt(bears), if on_top { (3, 4) } else { (3, 3) });
        assert!(t.obj_now(bears).has_keyword(KeywordKind::Flying));
        assert_eq!(t.counters(other, counters::PLUS1), 0);
    }
    // Cast normally, it doesn't mutate.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 4);
    let v = t.hand(P0, "Vulpikeet");
    t.cast(P0, v).go();
    t.resolve_all();
    assert_eq!(t.counters(v, counters::PLUS1), 0);
    assert_eq!(t.pt(v), (2, 3));
}

#[test]
fn ainok_survivalist_turned_face_up_destroys_an_opponents_artifact_or_enchantment() {
    cr!("702.37e", "708.8", "603.2");
    // "Megamorph {1}{G} / When this creature is turned face up, destroy target artifact or
    // enchantment an opponent controls."
    supported("Ainok Survivalist");
    let mut t = TestGame::new(2);
    let anthem = t.battlefield(P1, "Glorious Anthem");
    let ring = t.battlefield(P1, "Sol Ring");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let mine = t.battlefield(P0, "Sol Ring");
    t.lands(P0, "Forest", 5);
    let a = t.hand(P0, "Ainok Survivalist");
    t.cast(P0, a)
        .method(CastMethod::FaceDown(KeywordKind::Morph))
        .go();
    t.resolve_all();
    let id = t.g.current(a);
    assert!(t.obj_now(id).face_down);
    assert!(
        target_choices(&t, P0).is_empty(),
        "entering face down doesn't trigger it"
    );
    t.answer_targets(P0, &[obj(anthem)]);
    t.g.turn.priority = Some(P0);
    t.g.perform_action(P0, Action::Special(SpecialAction::TurnFaceUp { obj: id }))
        .unwrap();
    t.resolve_all();
    assert!(!t.obj_now(id).face_down);
    assert!(!t.on_battlefield(anthem));
    assert!(t.on_battlefield(ring) && t.on_battlefield(mine) && t.on_battlefield(bears));
    let offered = &target_choices(&t, P0)[0];
    assert!(offered.contains(&obj(anthem)) && offered.contains(&obj(ring)));
    assert!(!offered.contains(&obj(mine)), "an opponent controls");
    assert!(
        !offered.contains(&obj(bears)),
        "not an artifact or enchantment"
    );
}

#[test]
fn lightning_rig_crew_untaps_when_you_cast_a_pirate_spell() {
    cr!("601.2i", "603.2");
    // "Whenever you cast a Pirate spell, untap this creature."
    supported("Lightning-Rig Crew");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Lightning-Rig Crew");
    act(&mut t, P0, c, "1 damage", &[]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(c).tapped);
    assert_eq!(t.life(P1), 19);
    t.lands(P0, "Mountain", 5);
    let hero = t.hand(P0, "Goblin Hero");
    t.cast(P0, hero).go();
    t.resolve_all();
    assert!(t.obj_now(c).tapped, "not a Pirate");
    let swab = t.hand(P0, "Swab Goblin");
    t.cast(P0, swab).go();
    t.settle();
    assert!(t.obj_now(c).tapped, "not until the trigger resolves");
    t.resolve_all();
    assert!(!t.obj_now(c).tapped);
    // An opponent's Pirate spell doesn't untap it.
    act(&mut t, P0, c, "1 damage", &[]).unwrap();
    t.resolve_all();
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 2);
    let theirs = t.hand(P1, "Swab Goblin");
    t.cast(P1, theirs).go();
    t.resolve_all();
    assert!(t.obj_now(c).tapped);
}

#[test]
fn cactarantula_may_draw_when_an_opponent_targets_it() {
    cr!("603.2", "603.5", "115.1");
    // "Whenever this creature becomes the target of a spell or ability an opponent
    // controls, you may draw a card."
    supported("Cactarantula");
    for yes in [true, false] {
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, "Cactarantula");
        t.set_step(P1, Step::PrecombatMain);
        t.lands(P1, "Mountain", 1);
        let b = t.hand(P1, "Lightning Bolt");
        t.answer_yes(P0, yes);
        t.cast(P1, b).target(c).go();
        t.resolve_all();
        assert_eq!(t.hand_size(P0), yes as usize);
        assert_eq!(t.hand_size(P1), 0);
        assert_eq!(t.obj_now(c).damage, 3);
    }
    // An opponent's ability counts too.
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Cactarantula");
    let pyro = t.battlefield(P1, "Prodigal Pyromancer");
    t.set_step(P1, Step::PrecombatMain);
    t.answer_yes(P0, true);
    act(&mut t, P1, pyro, "1 damage", &[obj(c)]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    // Your own spell doesn't.
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Cactarantula");
    t.lands(P0, "Mountain", 1);
    let b = t.hand(P0, "Lightning Bolt");
    t.answer_yes(P0, true);
    t.cast(P0, b).target(c).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0);
}
