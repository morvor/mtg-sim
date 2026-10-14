//! Structure-coverage tests, round 2 (item 3): triggered-ability structures shared by many
//! cards that no other test exercised (see `docs/STRUCTURE_COVERAGE.md`). Each test checks
//! a real card's ability does what its text says: the right objects and players affected
//! and not others, amounts, durations ending, conditions true and false, and the targets
//! the engine offers.

use super::structure_activated_2::target_choices;
use super::structure_seed_common::*;
use mtg_engine::decision::{Action, Answer, SpecialAction};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{CardType, Color};
use mtg_engine::*;

/// Puts `name` onto the battlefield through a real zone change and resolves its triggers.
fn etb(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = t.enter(p, name);
    t.resolve_all();
    id
}

/// Moves on to the opponent's upkeep, past the cleanup step that ends "until end of turn"
/// effects.
fn next_turn(t: &mut TestGame) {
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
}

/// Runs a combat on P0's turn: `attackers` attack P1, who blocks with `blocks`.
fn fight(t: &mut TestGame, attackers: &[ObjectId], blocks: &[(ObjectId, ObjectId)]) {
    let a: Vec<(ObjectId, Entity)> = attackers
        .iter()
        .map(|a| (*a, Entity::Player(P1)))
        .collect();
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&a, blocks);
}

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

// ---------------------------------------------------------------------------
// "When ~ enters, ..." — tokens
// ---------------------------------------------------------------------------

#[test]
fn glacier_godmaw_creates_a_lander_token() {
    cr!("603.6a", "111.10u");
    // "When this creature enters, create a Lander token."
    supported("Glacier Godmaw");
    let mut t = TestGame::new(2);
    etb(&mut t, P0, "Glacier Godmaw");
    let landers = subtype_ids(&t, P0, "Lander");
    assert_eq!(landers.len(), 1);
    let l = t.obj_now(landers[0]);
    assert!(l.is_token() && l.is(CardType::Artifact) && !l.is_creature());
    assert!(l.chars.colors.is_colorless());
    assert_eq!(count_subtype(&t, P1, "Lander"), 0);
    // "{2}, {T}, Sacrifice this token: Search your library for a basic land card, put it
    // onto the battlefield tapped, then shuffle."
    let lander = landers[0];
    t.lands(P0, "Wastes", 2);
    let forest = t.library_top(P0, "Forest");
    t.answer_choose(P0, &[obj(forest)]);
    t.activate(P0, lander, 0, &[]).unwrap();
    assert!(!t.on_battlefield(lander));
    t.resolve_all();
    assert!(t.on_battlefield(forest));
    assert!(t.obj_now(forest).tapped);
}

#[test]
fn koilos_roc_creates_a_tapped_powerstone() {
    cr!("603.6a", "111.10h");
    // "When this creature enters, create a tapped Powerstone token."
    supported("Koilos Roc");
    let mut t = TestGame::new(2);
    etb(&mut t, P0, "Koilos Roc");
    let ps = subtype_ids(&t, P0, "Powerstone");
    assert_eq!(ps.len(), 1);
    let p = t.obj_now(ps[0]);
    assert!(p.is_token() && p.is(CardType::Artifact));
    assert!(p.tapped, "created tapped");
    assert_eq!(count_subtype(&t, P1, "Powerstone"), 0);
}

#[test]
fn squires_devotion_creates_a_lifelink_vampire() {
    cr!("603.6a", "111.1", "303.4a");
    ruling!(
        "Squire's Devotion",
        "You need a creature for Squire's Devotion to target as you cast it."
    );
    // "Enchanted creature gets +1/+1 and has lifelink. / When this Aura enters, create a
    // 1/1 white Vampire creature token with lifelink."
    supported("Squire's Devotion");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 3);
    let aura = t.hand(P0, "Squire's Devotion");
    t.cast(P0, aura).target(bears).go();
    t.resolve_all();
    let vamps = subtype_ids(&t, P0, "Vampire");
    assert_eq!(vamps.len(), 1);
    let v = t.obj_now(vamps[0]);
    assert!(v.is_token() && v.is_creature());
    assert!(v.chars.colors.contains(Color::White));
    assert!(v.has_keyword(KeywordKind::Lifelink));
    assert_eq!(t.pt(vamps[0]), (1, 1), "the Aura isn't on the token");
    assert_eq!(t.pt(bears), (3, 3));
    assert_eq!(count_subtype(&t, P1, "Vampire"), 0);
    // Without a creature to enchant, it can't be cast at all.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 3);
    let aura = t.hand(P0, "Squire's Devotion");
    assert!(t.cast(P0, aura).try_go().is_err());
    assert!(t.in_hand(P0, "Squire's Devotion"));
    assert_eq!(count_subtype(&t, P0, "Vampire"), 0);
}

// ---------------------------------------------------------------------------
// "When ~ enters, ..." — players
// ---------------------------------------------------------------------------

#[test]
fn howling_banshee_makes_each_player_lose_3() {
    cr!("603.6a", "119.3");
    // "When this creature enters, each player loses 3 life."
    supported("Howling Banshee");
    let mut t = TestGame::new(3);
    etb(&mut t, P1, "Howling Banshee");
    assert_eq!((t.life(P0), t.life(P1), t.life(P2)), (17, 17, 17));
}

#[test]
fn keldon_champion_deals_3_to_a_player_or_planeswalker() {
    cr!("603.6a", "115.1", "120.3a", "120.3c");
    // "When this creature enters, it deals 3 damage to target player or planeswalker."
    supported("Keldon Champion");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let jace = t.battlefield(P1, "Jace Beleren");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let champ = etb(&mut t, P0, "Keldon Champion");
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P0), 20);
    let offered = &target_choices(&t, P0)[0];
    assert!(offered.contains(&Entity::Player(P0)) && offered.contains(&Entity::Player(P1)));
    assert!(offered.contains(&obj(jace)));
    assert!(!offered.contains(&obj(bears)), "not a creature");
    assert!(!offered.contains(&obj(champ)));
    // At a planeswalker.
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P1, "Jace Beleren");
    t.answer_targets(P0, &[obj(jace)]);
    etb(&mut t, P0, "Keldon Champion");
    assert_eq!(t.life(P1), 20);
    assert!(!t.on_battlefield(jace), "3 loyalty removed from a 3-loyalty Jace");
}

#[test]
fn oath_of_liliana_makes_each_opponent_sacrifice_a_creature() {
    cr!("603.6a", "701.21a", "704.5j");
    ruling!(
        "Oath of Liliana",
        "you can still cast a second Oath of Liliana to take advantage of its enters-the-battlefield ability"
    );
    // "When Oath of Liliana enters, each opponent sacrifices a creature of their choice."
    supported("Oath of Liliana");
    let mut t = TestGame::new(3);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let c = t.battlefield(P2, "Hill Giant");
    t.answer_choose(P1, &[obj(b)]);
    etb(&mut t, P0, "Oath of Liliana");
    assert!(t.on_battlefield(mine));
    assert!(t.on_battlefield(a), "P1 chose the Giant");
    assert!(!t.on_battlefield(b));
    assert!(!t.on_battlefield(c));
    assert!(t.in_graveyard(P1, "Hill Giant") && t.in_graveyard(P2, "Hill Giant"));
    // A second Oath still triggers, though the legend rule then puts one of the two into
    // the graveyard.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Oath of Liliana");
    let bears = t.battlefield(P1, "Grizzly Bears");
    etb(&mut t, P0, "Oath of Liliana");
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.named_on_battlefield("Oath of Liliana").len(), 1);
    assert!(t.in_graveyard(P0, "Oath of Liliana"));
}

#[test]
fn glistening_sphere_proliferates() {
    cr!("603.6a", "701.34a");
    // "This artifact enters tapped. / When this artifact enters, proliferate."
    supported("Glistening Sphere");
    let mut t = TestGame::new(2);
    let mine = with_counters(&mut t, P0, "Grizzly Bears", "+1/+1", 1);
    let theirs = with_counters(&mut t, P1, "Grizzly Bears", "-1/-1", 1);
    let skip = with_counters(&mut t, P1, "Hill Giant", "+1/+1", 1);
    t.g.players[P1.idx()]
        .counters
        .insert(types::counters::POISON.into(), 2);
    t.answer_choose(
        P0,
        &[obj(mine), obj(theirs), Entity::Player(P1)],
    );
    let sphere = etb(&mut t, P0, "Glistening Sphere");
    assert!(t.obj_now(sphere).tapped);
    assert_eq!(t.counters(mine, "+1/+1"), 2);
    assert!(!t.on_battlefield(theirs), "a 2/2 with two -1/-1 counters dies");
    assert_eq!(t.counters(skip, "+1/+1"), 1, "not chosen");
    assert_eq!(t.g.player(P1).counter(types::counters::POISON), 3);
}

fn with_counters(t: &mut TestGame, p: PlayerId, name: &str, kind: &str, n: u32) -> ObjectId {
    let id = t.battlefield(p, name);
    t.g.add_counters(Entity::Object(id), kind, n, None);
    t.g.recompute();
    id
}

// ---------------------------------------------------------------------------
// "When ~ enters, ..." — targets
// ---------------------------------------------------------------------------

#[test]
fn stingscourger_bounces_a_creature_an_opponent_controls() {
    cr!("603.6a", "115.1");
    // "When this creature enters, return target creature an opponent controls to its
    // owner's hand."
    supported("Stingscourger");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    let land = t.battlefield(P1, "Forest");
    t.answer_targets(P0, &[obj(theirs)]);
    let s = etb(&mut t, P0, "Stingscourger");
    assert_eq!(t.zone(theirs), Zone::Hand(P1));
    assert!(t.on_battlefield(mine) && t.on_battlefield(land));
    let offered = &target_choices(&t, P0)[0];
    assert!(offered.contains(&obj(theirs)));
    assert!(!offered.contains(&obj(mine)) && !offered.contains(&obj(s)));
    assert!(!offered.contains(&obj(land)));
}

#[test]
fn bespoke_bo_bounces_up_to_one_other_nonland_permanent() {
    cr!("603.6a", "115.1", "601.2c");
    // "When this Equipment enters, return up to one other target nonland permanent to its
    // owner's hand."
    supported("Bespoke Bō");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let sphere = t.battlefield(P1, "Wooden Sphere");
    let land = t.battlefield(P1, "Forest");
    t.answer_targets(P0, &[obj(sphere)]);
    let bo = etb(&mut t, P0, "Bespoke Bō");
    assert_eq!(t.zone(sphere), Zone::Hand(P1));
    assert!(t.on_battlefield(mine) && t.on_battlefield(land) && t.on_battlefield(bo));
    let offered = &target_choices(&t, P0)[0];
    assert!(offered.contains(&obj(mine)) && offered.contains(&obj(sphere)));
    assert!(!offered.contains(&obj(land)), "a land");
    assert!(!offered.contains(&obj(bo)), "other");
    // "Up to one": choosing nothing is fine.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[]);
    etb(&mut t, P0, "Bespoke Bō");
    assert!(t.on_battlefield(mine));
    assert_eq!(t.hand_size(P0), 0);
}

#[test]
fn farbog_boneflinger_gives_target_creature_minus_2_until_end_of_turn() {
    cr!("603.6a", "611.2a", "613.4c");
    ruling!(
        "Farbog Boneflinger",
        "If it’s the only creature on the battlefield, you must choose it as the target"
    );
    // "When this creature enters, target creature gets -2/-2 until end of turn."
    supported("Farbog Boneflinger");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let other = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(giant)]);
    etb(&mut t, P0, "Farbog Boneflinger");
    assert_eq!(t.pt(giant), (1, 1));
    assert_eq!(t.pt(other), (3, 3));
    next_turn(&mut t);
    assert_eq!(t.pt(giant), (3, 3));
    // It can target its own controller's creatures, and kills a 2/2.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    etb(&mut t, P0, "Farbog Boneflinger");
    assert!(!t.on_battlefield(bears));
    // The ability is mandatory: alone on the battlefield, it must target itself.
    let mut t = TestGame::new(2);
    let f = etb(&mut t, P0, "Farbog Boneflinger");
    assert!(!t.on_battlefield(f));
    assert!(t.in_graveyard(P0, "Farbog Boneflinger"));
}

#[test]
fn gale_swooper_grants_flying_until_end_of_turn() {
    cr!("603.6a", "611.2a", "613.1f");
    // "When this creature enters, target creature gains flying until end of turn."
    supported("Gale Swooper");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    etb(&mut t, P0, "Gale Swooper");
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Flying));
    assert!(!t.obj_now(other).has_keyword(KeywordKind::Flying));
    next_turn(&mut t);
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::Flying));
}

#[test]
fn topplegeist_taps_a_creature_an_opponent_controls() {
    cr!("603.6a", "701.26a");
    // "When this creature enters, tap target creature an opponent controls."
    supported("Topplegeist");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let other = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(theirs)]);
    etb(&mut t, P0, "Topplegeist");
    assert!(t.obj_now(theirs).tapped);
    assert!(!t.obj_now(other).tapped && !t.obj_now(mine).tapped);
    let offered = &target_choices(&t, P0)[0];
    assert!(!offered.contains(&obj(mine)));
    assert!(offered.contains(&obj(other)));
}

#[test]
fn weftblade_enhancer_counters_up_to_two_creatures() {
    cr!("603.6a", "601.2c", "122.6");
    // "When this creature enters, put a +1/+1 counter on each of up to two target
    // creatures."
    supported("Weftblade Enhancer");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    let c = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[obj(a), obj(b)]);
    let w = etb(&mut t, P0, "Weftblade Enhancer");
    assert_eq!(t.counters(a, "+1/+1"), 1);
    assert_eq!(t.counters(b, "+1/+1"), 1);
    assert_eq!(t.counters(c, "+1/+1"), 0);
    assert_eq!(t.counters(w, "+1/+1"), 0);
    assert_eq!(t.pt(a), (3, 3));
    // One target gets a single counter.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(a)]);
    etb(&mut t, P0, "Weftblade Enhancer");
    assert_eq!(t.counters(a, "+1/+1"), 1);
}

#[test]
fn ascendant_dustspeaker_counters_another_creature_you_control() {
    cr!("603.6a", "115.1", "122.6");
    // "When this creature enters, put a +1/+1 counter on another target creature you
    // control."
    supported("Ascendant Dustspeaker");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(mine)]);
    let d = etb(&mut t, P0, "Ascendant Dustspeaker");
    assert_eq!(t.counters(mine, "+1/+1"), 1);
    assert_eq!(t.counters(d, "+1/+1"), 0);
    let offered = &target_choices(&t, P0)[0];
    assert!(offered.contains(&obj(mine)));
    assert!(!offered.contains(&obj(theirs)), "an opponent's creature");
    assert!(!offered.contains(&obj(d)), "another");
}

#[test]
fn arashin_sunshield_exiles_up_to_two_cards_from_a_graveyard() {
    cr!("603.6a", "115.1", "406.2");
    // "When this creature enters, exile up to two target cards from a single graveyard."
    supported("Arashin Sunshield");
    let mut t = TestGame::new(2);
    let a = t.graveyard(P1, "Grizzly Bears");
    let b = t.graveyard(P1, "Lightning Bolt");
    let c = t.graveyard(P1, "Forest");
    let mine = t.graveyard(P0, "Hill Giant");
    t.answer_targets(P0, &[obj(a), obj(b)]);
    etb(&mut t, P0, "Arashin Sunshield");
    assert_eq!(t.zone(a), Zone::Exile);
    assert_eq!(t.zone(b), Zone::Exile);
    assert_eq!(t.zone(c), Zone::Graveyard(P1));
    assert_eq!(t.zone(mine), Zone::Graveyard(P0));
    let offered = &target_choices(&t, P0)[0];
    assert!(offered.contains(&obj(mine)) && offered.contains(&obj(c)));
}

// ---------------------------------------------------------------------------
// "When ~ enters, ..." — mass effects
// ---------------------------------------------------------------------------

#[test]
fn plague_mare_shrinks_opponents_creatures_until_end_of_turn() {
    cr!("603.6a", "611.2a", "611.2c", "613.4c");
    ruling!(
        "Plague Mare",
        "affects only creatures your opponents control at the time it resolves"
    );
    // "When this creature enters, creatures your opponents control get -1/-1 until end of
    // turn."
    supported("Plague Mare");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let elf = t.battlefield(P1, "Llanowar Elves");
    let giant = t.battlefield(P1, "Hill Giant");
    let mare = etb(&mut t, P0, "Plague Mare");
    assert!(!t.on_battlefield(elf), "a 1/1 dies");
    assert_eq!(t.pt(giant), (2, 2));
    assert_eq!(t.pt(mine), (2, 2));
    assert_eq!(t.pt(mare), (2, 2));
    // Locked in: a creature that enters later isn't affected.
    let late = t.battlefield(P1, "Hill Giant");
    assert_eq!(t.pt(late), (3, 3));
    next_turn(&mut t);
    assert_eq!(t.pt(giant), (3, 3));
}

// ---------------------------------------------------------------------------
// Dies triggers
// ---------------------------------------------------------------------------

#[test]
fn grim_physician_shrinks_an_opponents_creature_when_it_dies() {
    cr!("603.6c", "603.10a", "611.2a");
    // "When this creature dies, target creature an opponent controls gets -1/-1 until end
    // of turn."
    supported("Grim Physician");
    let mut t = TestGame::new(2);
    let gp = t.battlefield(P0, "Grim Physician");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(giant)]);
    t.g.destroy(gp, None);
    t.resolve_all();
    assert_eq!(t.pt(giant), (2, 2));
    assert_eq!(t.pt(mine), (2, 2));
    let offered = &target_choices(&t, P0)[0];
    assert_eq!(offered, &vec![obj(giant)]);
    next_turn(&mut t);
    assert_eq!(t.pt(giant), (3, 3));
}

#[test]
fn malakir_cullblade_grows_when_an_opponents_creature_dies() {
    cr!("603.6c", "603.10a", "122.6");
    // "Whenever a creature an opponent controls dies, put a +1/+1 counter on this
    // creature."
    supported("Malakir Cullblade");
    let mut t = TestGame::new(2);
    let mc = t.battlefield(P0, "Malakir Cullblade");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.g.destroy(mine, None);
    t.resolve_all();
    assert_eq!(t.counters(mc, "+1/+1"), 0);
    t.g.destroy(theirs, None);
    t.resolve_all();
    assert_eq!(t.counters(mc, "+1/+1"), 1);
    assert_eq!(t.pt(mc), (2, 2));
}

#[test]
fn deathbringer_thoctar_may_grow_when_another_creature_dies() {
    cr!("603.6c", "603.10a", "122.6");
    ruling!(
        "Deathbringer Thoctar",
        "Its ability will still trigger, but it will do nothing when it resolves."
    );
    // "Whenever another creature dies, you may put a +1/+1 counter on this creature."
    supported("Deathbringer Thoctar");
    let mut t = TestGame::new(2);
    let th = t.battlefield(P0, "Deathbringer Thoctar");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let third = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.g.destroy(mine, None);
    t.resolve_all();
    assert_eq!(t.counters(th, "+1/+1"), 1);
    t.answer_yes(P0, true);
    t.g.destroy(theirs, None);
    t.resolve_all();
    assert_eq!(t.counters(th, "+1/+1"), 2);
    t.answer_yes(P0, false);
    t.g.destroy(third, None);
    t.resolve_all();
    assert_eq!(t.counters(th, "+1/+1"), 2, "declined");
    // Dealt lethal damage at the same time as another creature: it still triggers, but
    // does nothing.
    let mut t = TestGame::new(2);
    let th = t.battlefield(P0, "Deathbringer Thoctar");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.g.deal_damage_batch(vec![(bears, obj(th), 3), (th, obj(bears), 2)], false);
    t.resolve_all();
    assert!(!t.on_battlefield(th) && !t.on_battlefield(bears));
    assert!(t.dump_log().contains("Resolving Deathbringer Thoctar ability"));
    assert!(t.in_graveyard(P0, "Deathbringer Thoctar"));
}

#[test]
fn grave_venerations_drains_when_your_creature_dies() {
    cr!("603.6c", "603.10a", "119.3");
    // "Whenever a creature you control dies, each opponent loses 1 life and you gain 1
    // life."
    supported("Grave Venerations");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Grave Venerations");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.g.destroy(theirs, None);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1), t.life(P2)), (20, 20, 20));
    t.g.destroy(mine, None);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1), t.life(P2)), (21, 19, 19));
}

// ---------------------------------------------------------------------------
// Other permanents entering
// ---------------------------------------------------------------------------

#[test]
fn mausoleum_wanderer_grows_when_another_spirit_you_control_enters() {
    cr!("603.6a", "611.2a");
    // "Whenever another Spirit you control enters, this creature gets +1/+1 until end of
    // turn."
    supported("Mausoleum Wanderer");
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Mausoleum Wanderer");
    etb(&mut t, P1, "Gallows Warden");
    etb(&mut t, P0, "Grizzly Bears");
    assert_eq!(t.pt(w), (1, 1), "an opponent's Spirit and a non-Spirit");
    etb(&mut t, P0, "Battleground Geist");
    // +1/+1 from the trigger, +1/+0 from Battleground Geist.
    assert_eq!(t.pt(w), (3, 2));
    next_turn(&mut t);
    assert_eq!(t.pt(w), (2, 1));
}

#[test]
fn soul_stair_expedition_may_add_a_quest_counter_on_landfall() {
    cr!("603.6a", "122.6");
    // "Landfall — Whenever a land you control enters, you may put a quest counter on this
    // enchantment."
    supported("Soul Stair Expedition");
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Soul Stair Expedition");
    t.answer_yes(P0, true);
    etb(&mut t, P0, "Swamp");
    assert_eq!(t.counters(e, "quest"), 1);
    etb(&mut t, P1, "Swamp");
    assert_eq!(t.counters(e, "quest"), 1, "an opponent's land");
    etb(&mut t, P0, "Grizzly Bears");
    assert_eq!(t.counters(e, "quest"), 1, "not a land");
    t.answer_yes(P0, false);
    etb(&mut t, P0, "Swamp");
    assert_eq!(t.counters(e, "quest"), 1, "declined");
}

// ---------------------------------------------------------------------------
// Cast triggers
// ---------------------------------------------------------------------------

#[test]
fn kemuri_onna_may_return_to_hand_on_a_spirit_or_arcane_spell() {
    cr!("603.2", "601.2i");
    // "Whenever you cast a Spirit or Arcane spell, you may return this creature to its
    // owner's hand."
    supported("Kemuri-Onna");
    // (spell, caster, answer, returned)
    let cases = [
        ("Lightning Bolt", P0, true, false),
        ("Mausoleum Wanderer", P0, true, true),
        ("Glacial Ray", P0, true, true),
        ("Glacial Ray", P1, true, false),
        ("Mausoleum Wanderer", P0, false, false),
    ];
    for (spell, caster, yes, returned) in cases {
        let mut t = TestGame::new(2);
        let k = t.battlefield(P0, "Kemuri-Onna");
        t.lands(caster, "Mountain", 2);
        t.lands(caster, "Island", 1);
        let c = t.hand(caster, spell);
        t.answer_yes(P0, yes);
        let mut b = t.cast(caster, c);
        if spell != "Mausoleum Wanderer" {
            b = b.target(P1);
        }
        b.go();
        t.resolve_all();
        let want = if returned { Zone::Hand(P0) } else { Zone::Battlefield };
        assert_eq!(t.zone(k), want, "{spell} cast by {caster:?}, yes: {yes}");
    }
}

#[test]
fn malachite_talisman_may_pay_3_to_untap_on_a_green_spell() {
    cr!("603.2", "118.12", "701.26b");
    // "Whenever a player casts a green spell, you may pay {3}. If you do, untap target
    // permanent."
    supported("Malachite Talisman");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Malachite Talisman");
    let wastes = t.lands(P0, "Wastes", 3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.tap(bears);
    let other = t.battlefield(P0, "Hill Giant");
    t.g.tap(other);
    t.lands(P1, "Forest", 1);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_yes(P0, true);
    let growth = t.hand(P1, "Giant Growth");
    t.cast(P1, growth).target(theirs).go();
    t.resolve_all();
    assert!(!t.obj_now(bears).tapped, "untapped");
    assert!(t.obj_now(other).tapped);
    assert!(wastes.iter().all(|w| t.obj_now(*w).tapped), "paid {{3}}");
    // A nongreen spell doesn't trigger it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Malachite Talisman");
    t.lands(P0, "Wastes", 3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.tap(bears);
    t.lands(P1, "Mountain", 1);
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_yes(P0, true);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(P0).go();
    t.resolve_all();
    assert!(t.obj_now(bears).tapped);
    assert_eq!(t.life(P0), 17);
    // Declining to pay leaves it tapped.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Malachite Talisman");
    t.lands(P0, "Wastes", 3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.tap(bears);
    t.lands(P1, "Forest", 1);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_yes(P0, false);
    let growth = t.hand(P1, "Giant Growth");
    t.cast(P1, growth).target(theirs).go();
    t.resolve_all();
    assert!(t.obj_now(bears).tapped);
}

#[test]
fn shrine_of_limitless_power_charges_on_upkeep_and_black_spells() {
    cr!("603.2", "503.1a", "122.6");
    // "At the beginning of your upkeep and whenever you cast a black spell, put a charge
    // counter on this artifact."
    supported("Shrine of Limitless Power");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Shrine of Limitless Power");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve_all();
    assert_eq!(t.counters(s, "charge"), 0, "a red spell");
    let rite = t.hand(P0, "Dark Ritual");
    t.cast(P0, rite).go();
    t.resolve_all();
    assert_eq!(t.counters(s, "charge"), 1);
    // An opponent's black spell doesn't count.
    t.lands(P1, "Swamp", 1);
    let rite = t.hand(P1, "Dark Ritual");
    t.cast(P1, rite).go();
    t.resolve_all();
    assert_eq!(t.counters(s, "charge"), 1);
    // The opponent's upkeep doesn't count; P0's does.
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(s, "charge"), 1);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(s, "charge"), 2);
}

// ---------------------------------------------------------------------------
// Upkeep and draw-step triggers
// ---------------------------------------------------------------------------

#[test]
fn brass_gnat_may_pay_1_to_untap_at_upkeep() {
    cr!("503.1a", "118.12", "502.3");
    // "This creature doesn't untap during your untap step. / At the beginning of your
    // upkeep, you may pay {1}. If you do, untap this creature."
    supported("Brass Gnat");
    for pay in [true, false] {
        let mut t = TestGame::new(2);
        let g = t.battlefield(P0, "Brass Gnat");
        let w = t.lands(P0, "Wastes", 1)[0];
        t.g.tap(g);
        t.answer_yes(P0, pay);
        t.advance_to(P1, Step::Upkeep);
        t.advance_to(P0, Step::Upkeep);
        assert!(t.obj_now(g).tapped, "doesn't untap in the untap step");
        t.resolve_all();
        assert_eq!(t.obj_now(g).tapped, !pay, "pay: {pay}");
        assert_eq!(t.obj_now(w).tapped, pay);
    }
}

#[test]
fn ring_of_valkas_counters_the_equipped_creature_if_red() {
    cr!("503.1a", "603.2", "122.6");
    // "At the beginning of your upkeep, put a +1/+1 counter on equipped creature if it's
    // red."
    supported("Ring of Valkas");
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P0, "Ring of Valkas");
    let red = t.battlefield(P0, "Raging Goblin");
    let other = t.battlefield(P0, "Grizzly Bears");
    t.g.attach(ring, obj(red));
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(red, "+1/+1"), 0, "not P0's upkeep");
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(red, "+1/+1"), 1);
    assert_eq!(t.counters(other, "+1/+1"), 0);
    // A green equipped creature gets nothing.
    t.g.attach(ring, obj(other));
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(other, "+1/+1"), 0);
    assert_eq!(t.counters(red, "+1/+1"), 1);
}

#[test]
fn bog_elemental_is_sacrificed_unless_you_sacrifice_a_land() {
    cr!("503.1a", "118.12a", "701.21a");
    // "At the beginning of your upkeep, sacrifice this creature unless you sacrifice a
    // land."
    supported("Bog Elemental");
    // Paying: a land is sacrificed, the Elemental stays.
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Bog Elemental");
    let swamp = t.battlefield(P0, "Swamp");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(swamp)]);
    t.set_step(P0, Step::Untap);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.on_battlefield(e));
    assert!(!t.on_battlefield(swamp));
    assert!(t.in_graveyard(P0, "Swamp"));
    // Declining: the Elemental is sacrificed.
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Bog Elemental");
    let swamp = t.battlefield(P0, "Swamp");
    t.answer_yes(P0, false);
    t.set_step(P0, Step::Untap);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(!t.on_battlefield(e));
    assert!(t.on_battlefield(swamp));
    // With no land, it's sacrificed.
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Bog Elemental");
    t.answer_yes(P0, true);
    t.set_step(P0, Step::Untap);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(!t.on_battlefield(e));
}

#[test]
fn well_of_ideas_draws_two_additional_cards_in_your_draw_step() {
    cr!("504.1", "603.2b", "121.2");
    // "At the beginning of your draw step, draw two additional cards." (and "At the
    // beginning of each other player's draw step, that player draws an additional card.")
    supported("Well of Ideas");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Well of Ideas");
    // Another player's draw step: one additional card for that player.
    t.advance_to(P1, Step::Draw);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 2);
    assert_eq!(t.hand_size(P0), 0);
    // Your draw step: two additional cards.
    t.advance_to(P0, Step::Draw);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 3, "one normal draw and two additional");
    assert_eq!(t.hand_size(P1), 2);
}

// ---------------------------------------------------------------------------
// Draw triggers
// ---------------------------------------------------------------------------

#[test]
fn ethereal_investigator_makes_a_spirit_on_your_second_draw_each_turn() {
    cr!("603.2", "111.1", "121.2");
    // "Whenever you draw your second card each turn, create a 1/1 white Spirit creature
    // token with flying."
    supported("Ethereal Investigator");
    let mut t = TestGame::new(2);
    let inv = etb(&mut t, P0, "Ethereal Investigator");
    let spirits = |t: &TestGame| {
        subtype_ids(t, P0, "Spirit")
            .into_iter()
            .filter(|s| *s != t.g.current(inv))
            .collect::<Vec<_>>()
    };
    t.g.draw_cards(P0, 1);
    t.resolve_all();
    assert!(spirits(&t).is_empty(), "first draw");
    t.g.draw_cards(P1, 2);
    t.resolve_all();
    assert!(spirits(&t).is_empty(), "an opponent's draws");
    t.g.draw_cards(P0, 1);
    t.resolve_all();
    let s = spirits(&t);
    assert_eq!(s.len(), 1);
    let o = t.obj_now(s[0]);
    assert!(o.is_token() && o.has_keyword(KeywordKind::Flying));
    assert!(o.chars.colors.contains(Color::White));
    assert_eq!(t.pt(s[0]), (1, 1));
    t.g.draw_cards(P0, 2);
    t.resolve_all();
    assert_eq!(spirits(&t).len(), 1, "only the second card");
    assert_eq!(count_subtype(&t, P1, "Spirit"), 0);
}

// ---------------------------------------------------------------------------
// Combat triggers
// ---------------------------------------------------------------------------

#[test]
fn infectious_horror_makes_each_opponent_lose_2_when_it_attacks() {
    cr!("508.1m", "119.3");
    // "Whenever this creature attacks, each opponent loses 2 life."
    supported("Infectious Horror");
    let mut t = TestGame::new(3);
    let h = t.battlefield(P0, "Infectious Horror");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(h, Entity::Player(P1))], &[]);
    // P1 also took 2 combat damage.
    assert_eq!((t.life(P0), t.life(P1), t.life(P2)), (20, 16, 18));
}

#[test]
fn captain_lannery_storm_creates_a_treasure_when_it_attacks() {
    cr!("508.1m", "111.10a");
    // "Whenever this creature attacks, create a Treasure token."
    supported("Captain Lannery Storm");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Captain Lannery Storm");
    assert_eq!(count_subtype(&t, P0, "Treasure"), 0);
    fight(&mut t, &[c], &[]);
    assert_eq!(count_subtype(&t, P0, "Treasure"), 1);
    assert_eq!(count_subtype(&t, P1, "Treasure"), 0);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn slimefoot_and_squee_makes_a_saproling_on_entering_and_attacking() {
    cr!("603.6a", "508.1m", "111.1");
    // "Whenever Slimefoot and Squee enters or attacks, create a 1/1 green Saproling
    // creature token."
    supported("Slimefoot and Squee");
    let mut t = TestGame::new(2);
    let s = etb(&mut t, P0, "Slimefoot and Squee");
    let saps = subtype_ids(&t, P0, "Saproling");
    assert_eq!(saps.len(), 1);
    assert_eq!(t.pt(saps[0]), (1, 1));
    assert!(t.obj_now(saps[0]).chars.colors.contains(Color::Green));
    let cur = t.g.current(s);
    t.g.objects[cur.0 as usize].summoning_sick = false;
    fight(&mut t, &[s], &[]);
    assert_eq!(count_subtype(&t, P0, "Saproling"), 2);
    assert_eq!(count_subtype(&t, P1, "Saproling"), 0);
}

#[test]
fn vaultbreaker_may_rummage_when_it_attacks() {
    cr!("508.1m", "118.12", "701.9a");
    // "Whenever this creature attacks, you may discard a card. If you do, draw a card."
    supported("Vaultbreaker");
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Vaultbreaker");
    let bolt = t.hand(P0, "Lightning Bolt");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bolt)]);
    fight(&mut t, &[v], &[]);
    assert_eq!(t.zone(bolt), Zone::Graveyard(P0));
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.library_size(P0), 29);
    // Declining: nothing happens.
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Vaultbreaker");
    t.hand(P0, "Lightning Bolt");
    t.answer_yes(P0, false);
    fight(&mut t, &[v], &[]);
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.library_size(P0), 30);
    // With an empty hand, nothing is discarded so nothing is drawn.
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Vaultbreaker");
    t.answer_yes(P0, true);
    fight(&mut t, &[v], &[]);
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(t.library_size(P0), 30);
}

#[test]
fn cenns_heir_grows_for_each_other_attacking_kithkin() {
    cr!("508.1m", "611.2a");
    // "Whenever this creature attacks, it gets +1/+1 until end of turn for each other
    // attacking Kithkin."
    supported("Cenn's Heir");
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Cenn's Heir");
    let k1 = t.battlefield(P0, "Cenn's Heir");
    let k2 = t.battlefield(P0, "Goldmeadow Harrier");
    let home = t.battlefield(P0, "Goldmeadow Harrier");
    let bears = t.battlefield(P0, "Grizzly Bears");
    fight(&mut t, &[h, k1, k2, bears], &[]);
    // Two other attacking Kithkin (not the one at home, not the Bears).
    assert_eq!(t.pt(h), (3, 3));
    assert_eq!(t.pt(k1), (3, 3));
    assert_eq!(t.pt(home), (1, 1));
    assert_eq!(t.life(P1), 20 - 3 - 3 - 1 - 2);
    next_turn(&mut t);
    assert_eq!(t.pt(h), (1, 1));
}

#[test]
fn inferno_elemental_deals_3_to_each_creature_it_blocks_or_is_blocked_by() {
    cr!("509.1i", "509.3b", "509.3d", "120.3e");
    ruling!(
        "Inferno Elemental",
        "will trigger for each creature it blocks or becomes blocked by"
    );
    ruling!(
        "Inferno Elemental",
        "that creature will be destroyed before it can deal its combat damage"
    );
    ruling!(
        "Inferno Elemental",
        "Inferno Elemental will still deal its combat damage as normal"
    );
    // "Whenever this creature blocks or becomes blocked by a creature, this creature deals
    // 3 damage to that creature."
    supported("Inferno Elemental");
    // Blocked by two creatures: 3 damage to each blocker before combat damage.
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Inferno Elemental");
    let a = t.battlefield(P1, "Hill Giant");
    let b = t.battlefield(P1, "Hill Giant");
    let idle = t.battlefield(P1, "Hill Giant");
    t.answer(P0, DecisionKind::Damage, Answer::Default);
    fight(&mut t, &[e], &[(a, e), (b, e)]);
    assert!(!t.on_battlefield(a) && !t.on_battlefield(b));
    assert!(t.on_battlefield(e), "the blockers died before dealing combat damage");
    assert_eq!(t.obj_now(e).damage, 0);
    assert_eq!(t.obj_now(idle).damage, 0);
    // A blocker that survives the 3 damage is still dealt combat damage (and deals its
    // own): Giant Spider (2/4) dies from 3 + 4.
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Inferno Elemental");
    let spider = t.battlefield(P1, "Giant Spider");
    fight(&mut t, &[e], &[(spider, e)]);
    assert!(!t.on_battlefield(spider));
    assert_eq!(t.obj_now(e).damage, 2, "the Spider dealt its combat damage");
    assert_eq!(t.life(P1), 20);
    // Blocking: 3 damage to the attacker.
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Inferno Elemental");
    let giant = t.battlefield(P1, "Hill Giant");
    t.set_step(P1, Step::BeginningOfCombat);
    t.attack(&[(giant, Entity::Player(P0))], &[(e, giant)]);
    assert!(!t.on_battlefield(giant));
    assert_eq!(t.obj_now(e).damage, 0);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn clockwork_beetle_loses_a_counter_at_end_of_combat_after_attacking_or_blocking() {
    cr!("508.1m", "509.1i", "603.7a");
    // "This creature enters with two +1/+1 counters on it. / Whenever this creature attacks
    // or blocks, remove a +1/+1 counter from it at end of combat."
    supported("Clockwork Beetle");
    let mut t = TestGame::new(2);
    let b = etb(&mut t, P0, "Clockwork Beetle");
    assert_eq!(t.counters(b, "+1/+1"), 2);
    let cur = t.g.current(b);
    t.g.objects[cur.0 as usize].summoning_sick = false;
    fight(&mut t, &[b], &[]);
    // The delayed trigger fires at the beginning of the end of combat step.
    t.resolve_all();
    assert_eq!(t.life(P1), 18, "dealt combat damage as a 2/2");
    assert_eq!(t.counters(b, "+1/+1"), 1);
    // Blocking a 0/3 on the opponent's turn: it survives combat damage, then loses its
    // last counter and dies as a 0/0.
    let walker = t.battlefield(P1, "Phyrexian Walker");
    let cur = t.g.current(b);
    t.g.untap(cur);
    t.advance_to(P1, Step::BeginningOfCombat);
    t.attack(&[(walker, Entity::Player(P0))], &[(b, walker)]);
    assert_eq!(t.obj_now(walker).damage, 1);
    t.resolve_all();
    assert!(!t.on_battlefield(b), "a 0/0 without counters");
    // Not attacking or blocking: no counter removed.
    let mut t = TestGame::new(2);
    let b = etb(&mut t, P0, "Clockwork Beetle");
    let bears = t.battlefield(P0, "Grizzly Bears");
    fight(&mut t, &[bears], &[]);
    t.resolve_all();
    assert_eq!(t.counters(b, "+1/+1"), 2);
}

#[test]
fn mardu_blazebringer_is_sacrificed_at_end_of_combat() {
    cr!("508.1m", "509.1i", "603.7a", "701.21a");
    // "When this creature attacks or blocks, sacrifice it at end of combat."
    supported("Mardu Blazebringer");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Mardu Blazebringer");
    fight(&mut t, &[m], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 16, "it dealt its damage first");
    assert!(!t.on_battlefield(m));
    assert!(t.in_graveyard(P0, "Mardu Blazebringer"));
    // Blocking.
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Mardu Blazebringer");
    let gob = t.battlefield(P1, "Raging Goblin");
    t.set_step(P1, Step::BeginningOfCombat);
    t.attack(&[(gob, Entity::Player(P0))], &[(m, gob)]);
    t.resolve_all();
    assert!(!t.on_battlefield(m));
    // Staying home: it survives combat.
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Mardu Blazebringer");
    let bears = t.battlefield(P0, "Grizzly Bears");
    fight(&mut t, &[bears], &[]);
    t.resolve_all();
    assert!(t.on_battlefield(m));
}

#[test]
fn abyssal_specter_makes_the_damaged_player_discard() {
    cr!("603.2", "510.2", "701.9a");
    // "Whenever this creature deals damage to a player, that player discards a card."
    supported("Abyssal Specter");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Abyssal Specter");
    t.hand(P1, "Lightning Bolt");
    t.hand(P0, "Lightning Bolt");
    fight(&mut t, &[s], &[]);
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.hand_size(P1), 0);
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    assert_eq!(t.hand_size(P0), 1);
    // Damage to a creature doesn't trigger it.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Abyssal Specter");
    let spider = t.battlefield(P1, "Giant Spider");
    t.hand(P1, "Lightning Bolt");
    fight(&mut t, &[s], &[(spider, s)]);
    assert_eq!(t.hand_size(P1), 1);
}

#[test]
fn the_indomitable_draws_when_your_creature_deals_combat_damage_to_a_player() {
    cr!("510.2", "603.2");
    // "Whenever a creature you control deals combat damage to a player, draw a card."
    supported("The Indomitable");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Indomitable");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let c = t.battlefield(P0, "Grizzly Bears");
    let wall = t.battlefield(P1, "Wall of Wood");
    fight(&mut t, &[a, b, c], &[(wall, c)]);
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.hand_size(P0), 2, "one card per creature that dealt damage to P1");
    assert_eq!(t.hand_size(P1), 0);
    // An opponent's creature hitting P0 doesn't trigger it.
    let gob = t.battlefield(P1, "Raging Goblin");
    t.advance_to(P1, Step::BeginningOfCombat);
    t.attack(&[(gob, Entity::Player(P0))], &[]);
    assert_eq!(t.life(P0), 19);
    assert_eq!(t.hand_size(P0), 2);
}

#[test]
fn ingenious_infiltrator_draws_when_a_ninja_you_control_deals_combat_damage() {
    cr!("510.2", "603.2", "603.10a");
    ruling!(
        "Ingenious Infiltrator",
        "If Ingenious Infiltrator is dealt lethal combat damage at the same time a Ninja you control deals combat damage to a player, its ability triggers."
    );
    // "Whenever a Ninja you control deals combat damage to a player, draw a card."
    supported("Ingenious Infiltrator");
    let mut t = TestGame::new(2);
    let inf = t.battlefield(P0, "Ingenious Infiltrator");
    let ninja = t.battlefield(P0, "Ninja of the Deep Hours");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Ninja of the Deep Hours' own "may draw" is declined.
    t.answer_yes(P0, false);
    fight(&mut t, &[inf, ninja, bears], &[]);
    assert_eq!(t.life(P1), 20 - 2 - 2 - 2);
    // One draw for each Ninja (Ingenious Infiltrator is one itself), none for the Bears.
    assert_eq!(t.hand_size(P0), 2);
    // Dealt lethal combat damage at the same time as another Ninja hits a player, it
    // still triggers (it "looks back").
    let mut t = TestGame::new(2);
    let inf = t.battlefield(P0, "Ingenious Infiltrator");
    let ninja = t.battlefield(P0, "Ninja of the Deep Hours");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_yes(P0, false);
    fight(&mut t, &[inf, ninja], &[(giant, inf)]);
    assert!(!t.on_battlefield(inf));
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.hand_size(P0), 1);
    // Non-Ninjas don't draw.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ingenious Infiltrator");
    let bears = t.battlefield(P0, "Grizzly Bears");
    fight(&mut t, &[bears], &[]);
    assert_eq!(t.hand_size(P0), 0);
}

#[test]
fn hoard_robber_creates_a_treasure_on_combat_damage_to_a_player() {
    cr!("510.2", "603.2", "111.10a");
    // "Whenever this creature deals combat damage to a player, create a Treasure token."
    supported("Hoard Robber");
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Hoard Robber");
    fight(&mut t, &[r], &[]);
    assert_eq!(t.life(P1), 19);
    assert_eq!(count_subtype(&t, P0, "Treasure"), 1);
    // Blocked: no Treasure.
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Hoard Robber");
    let wall = t.battlefield(P1, "Wall of Wood");
    fight(&mut t, &[r], &[(wall, r)]);
    assert_eq!(count_subtype(&t, P0, "Treasure"), 0);
}

// ---------------------------------------------------------------------------
// Turned face up; Saga chapters
// ---------------------------------------------------------------------------

#[test]
fn stormwing_dragon_counters_other_dragons_when_turned_face_up() {
    cr!("702.37b", "702.37e", "708.8");
    // "When this creature is turned face up, put a +1/+1 counter on each other Dragon
    // creature you control."
    supported("Stormwing Dragon");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Shivan Dragon");
    let theirs = t.battlefield(P1, "Shivan Dragon");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 3);
    let card = t.hand(P0, "Stormwing Dragon");
    t.cast(P0, card)
        .method(CastMethod::FaceDown(KeywordKind::Morph))
        .go();
    t.resolve_all();
    let d = t.g.current(card);
    assert!(t.obj_now(d).face_down);
    assert_eq!(t.counters(mine, "+1/+1"), 0);
    t.lands(P0, "Mountain", 7);
    t.g.turn.priority = Some(P0);
    t.g
        .take_action(P0, Action::Special(SpecialAction::TurnFaceUp { obj: d }));
    t.resolve_all();
    assert!(!t.obj_now(d).face_down);
    assert_eq!(t.counters(mine, "+1/+1"), 1);
    assert_eq!(t.counters(theirs, "+1/+1"), 0);
    assert_eq!(t.counters(bears, "+1/+1"), 0);
    assert_eq!(t.counters(d, "+1/+1"), 1, "only megamorph's counter");
}

#[test]
fn invasion_of_the_giants_chapter_one_scries_2() {
    cr!("714.2b", "714.3a", "701.22a");
    // "I — Scry 2."
    supported("Invasion of the Giants");
    let mut t = TestGame::new(2);
    let saga = etb(&mut t, P0, "Invasion of the Giants");
    assert_eq!(t.counters(saga, "lore"), 1);
    assert_eq!(scries(&t), vec![2]);
    assert_eq!(t.hand_size(P0), 0, "chapter II hasn't triggered");
}
