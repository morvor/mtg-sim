//! Structure-coverage tests (round 2): activated-ability structures shared by many cards
//! that no other test exercised (see `docs/STRUCTURE_COVERAGE.md`). Each test checks that
//! a real card's ability does what its text says. Also holds the helpers the other
//! `structure_*_2.rs` files share.

#![allow(dead_code)]

use super::structure_seed_common::*;
use mtg_engine::ability::AbilityKind;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::types::{CardType, Color};
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Activates the activated ability of `src` whose text contains `text` (case-insensitive),
/// answering its targets with `targets` (one per slot).
pub fn act(
    t: &mut TestGame,
    p: PlayerId,
    src: ObjectId,
    text: &str,
    targets: &[Entity],
) -> Result<Option<ObjectId>, mtg_engine::casting::Illegal> {
    t.g.recompute();
    let acts: Vec<_> = t
        .obj_now(src)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .map(|a| a.text.to_lowercase())
        .collect();
    let i = acts
        .iter()
        .position(|a| a.contains(&text.to_lowercase()))
        .unwrap_or_else(|| panic!("no activated ability with {text:?} among {acts:?}"));
    t.activate(p, src, i, targets)
}

/// The number of `p`'s untapped permanents named `name`.
pub fn untapped(t: &TestGame, p: PlayerId, name: &str) -> usize {
    t.named_on_battlefield(name)
        .into_iter()
        .filter(|id| t.obj_now(*id).controller == p && !t.obj_now(*id).tapped)
        .count()
}

/// Puts a real creature that enters with +1/+1 counters onto the battlefield with `n` of
/// them (`TestGame::battlefield` skips "enters with" replacement effects).
pub fn with_p1p1(t: &mut TestGame, p: PlayerId, name: &str, n: u32) -> ObjectId {
    let id = t.battlefield(p, name);
    t.g.add_counters(Entity::Object(id), "+1/+1", n, None);
    t.g.recompute();
    id
}

/// Advances to the next turn's upkeep (so "until end of turn" effects have ended).
pub fn next_turn(t: &mut TestGame) {
    t.advance_to(P1, Step::Upkeep);
}

#[test]
fn treasure_trove_draws_a_card_for_mana() {
    cr!("602.2", "121.1");
    // "{2}{U}{U}: Draw a card."
    supported("Treasure Trove");
    let mut t = TestGame::new(2);
    let trove = t.battlefield(P0, "Treasure Trove");
    t.lands(P0, "Island", 3);
    assert!(act(&mut t, P0, trove, "draw", &[]).is_err(), "{{2}}{{U}}{{U}} needs four mana");
    t.lands(P0, "Island", 1);
    act(&mut t, P0, trove, "draw", &[]).unwrap();
    assert_eq!(untapped(&t, P0, "Island"), 0, "the cost was paid");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.hand_size(P1), 0);
    assert!(!t.obj_now(trove).tapped, "no {{T}} in the cost");
}

#[test]
fn vial_of_dragonfire_sacrifices_to_damage_a_creature() {
    cr!("602.2", "118.3", "120.3e");
    // "{2}, {T}, Sacrifice this artifact: It deals 2 damage to target creature."
    supported("Vial of Dragonfire");
    let mut t = TestGame::new(2);
    let vial = t.battlefield(P0, "Vial of Dragonfire");
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Wastes", 2);
    act(&mut t, P0, vial, "damage", &[Entity::Object(giant)]).unwrap();
    assert!(!t.on_battlefield(vial), "sacrificed as a cost");
    assert!(t.in_graveyard(P0, "Vial of Dragonfire"));
    t.resolve_all();
    assert_eq!(t.obj_now(giant).damage, 2);
    assert!(t.on_battlefield(giant), "2 damage doesn't kill a 3/3");
    assert_eq!(t.obj_now(bears).damage, 0);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn blossom_dryad_untaps_target_land() {
    cr!("602.2", "701.26b", "115.1");
    // "{T}: Untap target land."
    supported("Blossom Dryad");
    let mut t = TestGame::new(2);
    let dryad = t.battlefield(P0, "Blossom Dryad");
    let forest = t.battlefield(P0, "Forest");
    let other = t.battlefield(P0, "Forest");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.tap(forest);
    t.g.tap(other);
    t.g.tap(bears);
    // A creature isn't a legal target: the requested target is replaced by a land.
    act(&mut t, P0, dryad, "untap", &[Entity::Object(forest)]).unwrap();
    assert!(t.obj_now(dryad).tapped);
    t.resolve_all();
    assert!(!t.obj_now(forest).tapped);
    assert!(t.obj_now(other).tapped, "only the target");
    assert!(t.obj_now(bears).tapped);
}

#[test]
fn blighted_bat_gains_haste_until_end_of_turn() {
    cr!("602.2", "611.2a", "302.6", "702.10b");
    // "{1}: This creature gains haste until end of turn."
    supported("Blighted Bat");
    let mut t = TestGame::new(2);
    let bat = t.battlefield_sick(P0, "Blighted Bat");
    t.lands(P0, "Wastes", 1);
    assert!(!t.obj_now(bat).has_keyword(KeywordKind::Haste));
    act(&mut t, P0, bat, "haste", &[]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(bat).has_keyword(KeywordKind::Haste));
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bat, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 18, "a summoning-sick creature with haste attacks");
    next_turn(&mut t);
    assert!(!t.obj_now(bat).has_keyword(KeywordKind::Haste));
}

#[test]
fn wayfarers_bauble_fetches_a_basic_land_tapped() {
    cr!("602.2", "701.23a", "701.24a");
    // "{2}, {T}, Sacrifice this artifact: Search your library for a basic land card, put
    // that card onto the battlefield tapped, then shuffle."
    supported("Wayfarer's Bauble");
    let mut t = TestGame::new(2);
    let bauble = t.battlefield(P0, "Wayfarer's Bauble");
    t.lands(P0, "Wastes", 2);
    t.library_top(P0, "Forest");
    t.library_top(P0, "Bayou");
    let lib = t.library_size(P0);
    act(&mut t, P0, bauble, "search", &[]).unwrap();
    assert!(!t.on_battlefield(bauble));
    t.resolve_all();
    let forests = t.named_on_battlefield("Forest");
    assert_eq!(forests.len(), 1);
    assert!(t.obj_now(forests[0]).tapped, "enters tapped");
    assert!(t.named_on_battlefield("Bayou").is_empty(), "not a basic land");
    assert_eq!(t.library_size(P0), lib - 1);
}

#[test]
fn marble_chalice_gains_one_life() {
    cr!("602.2", "119.3");
    // "{T}: You gain 1 life."
    supported("Marble Chalice");
    let mut t = TestGame::new(2);
    let chalice = t.battlefield(P0, "Marble Chalice");
    act(&mut t, P0, chalice, "gain", &[]).unwrap();
    assert!(t.obj_now(chalice).tapped);
    assert!(act(&mut t, P0, chalice, "gain", &[]).is_err(), "already tapped");
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn hungry_ghoul_sacrifices_another_creature_for_a_counter() {
    cr!("602.2", "118.3", "122.1");
    // "{1}, Sacrifice another creature: Put a +1/+1 counter on this creature."
    supported("Hungry Ghoul");
    let mut t = TestGame::new(2);
    let ghoul = t.battlefield(P0, "Hungry Ghoul");
    t.lands(P0, "Wastes", 2);
    assert!(
        act(&mut t, P0, ghoul, "counter", &[]).is_err(),
        "it can't sacrifice itself"
    );
    let theirs = t.battlefield(P1, "Grizzly Bears");
    assert!(
        act(&mut t, P0, ghoul, "counter", &[]).is_err(),
        "only a creature you control can be sacrificed"
    );
    let bears = t.battlefield(P0, "Grizzly Bears");
    act(&mut t, P0, ghoul, "counter", &[]).unwrap();
    assert!(!t.on_battlefield(bears));
    assert!(t.on_battlefield(theirs));
    t.resolve_all();
    assert_eq!(t.counters(ghoul, "+1/+1"), 1);
    assert_eq!(t.pt(ghoul), (3, 3));
}

#[test]
fn staff_of_the_storyteller_removes_a_story_counter_to_draw() {
    cr!("602.2", "118.3", "122.8");
    // "{W}, {T}, Remove a story counter from this artifact: Draw a card."
    supported("Staff of the Storyteller");
    let mut t = TestGame::new(2);
    let staff = t.battlefield(P0, "Staff of the Storyteller");
    t.lands(P0, "Plains", 1);
    assert!(
        act(&mut t, P0, staff, "draw", &[]).is_err(),
        "no story counter to remove"
    );
    t.g.add_counters(Entity::Object(staff), "story", 2, None);
    act(&mut t, P0, staff, "draw", &[]).unwrap();
    assert_eq!(t.counters(staff, "story"), 1);
    assert!(t.obj_now(staff).tapped);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn zephyr_charge_grants_flying_until_end_of_turn() {
    cr!("602.2", "611.2a", "702.9b");
    // "{1}{U}: Target creature gains flying until end of turn."
    supported("Zephyr Charge");
    let mut t = TestGame::new(2);
    let charge = t.battlefield(P0, "Zephyr Charge");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Island", 2);
    act(&mut t, P0, charge, "flying", &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Flying));
    assert!(!t.obj_now(other).has_keyword(KeywordKind::Flying));
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P1))], &[(other, bears)]);
    assert_eq!(t.life(P1), 18, "a creature without flying or reach can't block it");
    next_turn(&mut t);
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::Flying));
}

#[test]
fn garruk_wildspeaker_minus_one_makes_a_beast() {
    cr!("606.3", "606.4", "111.1");
    // "−1: Create a 3/3 green Beast creature token."
    supported("Garruk Wildspeaker");
    let mut t = TestGame::new(2);
    let garruk = t.battlefield(P0, "Garruk Wildspeaker");
    assert_eq!(t.counters(garruk, "loyalty"), 3);
    act(&mut t, P0, garruk, "beast", &[]).unwrap();
    assert_eq!(t.counters(garruk, "loyalty"), 2, "the loyalty cost is paid");
    t.resolve_all();
    let beasts = subtype_ids(&t, P0, "Beast");
    assert_eq!(beasts.len(), 1);
    assert_eq!(t.pt(beasts[0]), (3, 3));
    assert!(t.obj_now(beasts[0]).is_token());
    assert_eq!(count_subtype(&t, P1, "Beast"), 0);
    assert!(
        act(&mut t, P0, garruk, "beast", &[]).is_err(),
        "one loyalty ability per turn"
    );
}

#[test]
fn darksteel_brute_becomes_a_beast_artifact_creature() {
    cr!("602.2", "611.2a", "613.1d", "613.4b");
    // "{3}: This artifact becomes a 2/2 Beast artifact creature until end of turn."
    supported("Darksteel Brute");
    let mut t = TestGame::new(2);
    let brute = t.battlefield(P0, "Darksteel Brute");
    t.lands(P0, "Wastes", 3);
    assert!(!t.obj_now(brute).is(CardType::Creature));
    act(&mut t, P0, brute, "becomes", &[]).unwrap();
    t.resolve_all();
    let o = t.obj_now(brute);
    assert!(o.is(CardType::Creature) && o.is(CardType::Artifact));
    assert!(o.chars.has_subtype("Beast"));
    assert_eq!(t.pt(brute), (2, 2));
    assert!(t.obj_now(brute).has_keyword(KeywordKind::Indestructible));
    next_turn(&mut t);
    assert!(!t.obj_now(brute).is(CardType::Creature));
    assert!(t.obj_now(brute).is(CardType::Artifact));
    assert!(!t.obj_now(brute).chars.has_subtype("Beast"));
}

#[test]
fn prized_elephant_gains_trample() {
    cr!("602.2", "611.2a", "702.19b");
    // "{G}: This creature gains trample until end of turn."
    supported("Prized Elephant");
    let mut t = TestGame::new(2);
    let elephant = t.battlefield(P0, "Prized Elephant");
    let chump = t.battlefield(P1, "Llanowar Elves");
    t.lands(P0, "Bayou", 1);
    act(&mut t, P0, elephant, "trample", &[]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(elephant).has_keyword(KeywordKind::Trample));
    // +1/+1 as long as you control a Forest (Bayou is one): 4/4.
    assert_eq!(t.pt(elephant), (4, 4));
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(elephant, Entity::Player(P1))], &[(chump, elephant)]);
    assert_eq!(t.life(P1), 17, "excess damage tramples over");
    next_turn(&mut t);
    assert!(!t.obj_now(elephant).has_keyword(KeywordKind::Trample));
}

#[test]
fn skinbrand_goblin_bloodrush_pumps_an_attacking_creature() {
    cr!("602.2", "702.117a", "118.3");
    // "Bloodrush — {R}, Discard this card: Target attacking creature gets +2/+1 until end
    // of turn."
    supported("Skinbrand Goblin");
    let mut t = TestGame::new(2);
    let attacker = t.battlefield(P0, "Grizzly Bears");
    let idle = t.battlefield(P0, "Hill Giant");
    let goblin = t.hand(P0, "Skinbrand Goblin");
    t.lands(P0, "Mountain", 1);
    assert!(
        act(&mut t, P0, goblin, "attacking", &[Entity::Object(idle)]).is_err(),
        "no attacking creature to target"
    );
    t.clear_answers();
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(attacker, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    act(&mut t, P0, goblin, "attacking", &[Entity::Object(attacker)]).unwrap();
    assert!(t.in_graveyard(P0, "Skinbrand Goblin"), "discarded as a cost");
    t.resolve_all();
    assert_eq!(t.pt(attacker), (4, 3));
    assert_eq!(t.pt(idle), (3, 3));
    next_turn(&mut t);
    assert_eq!(t.pt(attacker), (2, 2));
}

#[test]
fn spike_weaver_moves_a_counter_to_target_creature() {
    cr!("602.2", "118.3", "122.1");
    // "{2}, Remove a +1/+1 counter from this creature: Put a +1/+1 counter on target
    // creature."
    supported("Spike Weaver");
    let mut t = TestGame::new(2);
    let spike = with_p1p1(&mut t, P0, "Spike Weaver", 1);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Wastes", 4);
    act(&mut t, P0, spike, "on target creature", &[Entity::Object(bears)]).unwrap();
    assert_eq!(t.counters(spike, "+1/+1"), 0, "removed as a cost");
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    assert_eq!(t.pt(bears), (3, 3));
    assert!(!t.on_battlefield(spike), "a 0/0 with no counters dies");
}

#[test]
fn thermo_alchemist_pings_each_opponent() {
    cr!("602.2", "120.3a", "102.2");
    // "{T}: This creature deals 1 damage to each opponent."
    supported("Thermo-Alchemist");
    let mut t = TestGame::new(3);
    let alch = t.battlefield(P0, "Thermo-Alchemist");
    act(&mut t, P0, alch, "each opponent", &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.life(PlayerId(2)), 19);
    assert!(t.obj_now(alch).tapped);
}

#[test]
fn nantuko_disciple_pumps_target_creature() {
    cr!("602.2", "611.2a");
    // "{G}, {T}: Target creature gets +2/+2 until end of turn."
    supported("Nantuko Disciple");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Nantuko Disciple");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    act(&mut t, P0, d, "+2/+2", &[Entity::Object(bears)]).unwrap();
    assert!(t.obj_now(d).tapped);
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 4));
    assert_eq!(t.pt(d), (2, 2));
    next_turn(&mut t);
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn compulsion_discards_to_draw() {
    cr!("602.2", "118.3", "701.9a");
    // "{1}{U}, Discard a card: Draw a card."
    supported("Compulsion");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Compulsion");
    t.lands(P0, "Island", 2);
    assert!(
        act(&mut t, P0, c, "discard", &[]).is_err(),
        "no card to discard"
    );
    t.hand(P0, "Hill Giant");
    act(&mut t, P0, c, "discard", &[]).unwrap();
    assert!(t.in_graveyard(P0, "Hill Giant"), "discarded as a cost");
    assert_eq!(t.hand_size(P0), 0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    assert!(t.on_battlefield(c), "not sacrificed");
}

#[test]
fn torch_fiend_sacrifices_to_destroy_an_artifact() {
    cr!("602.2", "701.8a", "701.21a");
    // "{R}, Sacrifice this creature: Destroy target artifact."
    supported("Torch Fiend");
    let mut t = TestGame::new(2);
    let fiend = t.battlefield(P0, "Torch Fiend");
    let obelisk = t.battlefield(P1, "Obelisk of Naya");
    let other = t.battlefield(P1, "Marble Chalice");
    t.lands(P0, "Mountain", 1);
    act(&mut t, P0, fiend, "destroy", &[Entity::Object(obelisk)]).unwrap();
    assert!(!t.on_battlefield(fiend));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Obelisk of Naya"));
    assert!(t.on_battlefield(other));
}

#[test]
fn iron_lance_grants_first_strike() {
    cr!("602.2", "611.2a", "702.7b");
    // "{3}, {T}: Target creature gains first strike until end of turn."
    supported("Iron Lance");
    let mut t = TestGame::new(2);
    let lance = t.battlefield(P0, "Iron Lance");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let blocker = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Wastes", 3);
    act(&mut t, P0, lance, "first strike", &[Entity::Object(bears)]).unwrap();
    assert!(t.obj_now(lance).tapped);
    t.resolve_all();
    assert!(t.obj_now(bears).has_keyword(KeywordKind::FirstStrike));
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P1))], &[(blocker, bears)]);
    assert!(t.on_battlefield(bears), "first strike killed the blocker first");
    assert!(!t.on_battlefield(blocker));
    next_turn(&mut t);
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::FirstStrike));
}

#[test]
fn armillary_sphere_puts_up_to_two_basic_lands_into_hand() {
    cr!("602.2", "701.23a", "701.20a");
    // "{2}, {T}, Sacrifice this artifact: Search your library for up to two basic land
    // cards, reveal them, put them into your hand, then shuffle."
    supported("Armillary Sphere");
    let mut t = TestGame::new(2);
    let sphere = t.battlefield(P0, "Armillary Sphere");
    t.lands(P0, "Wastes", 2);
    t.library_top(P0, "Forest");
    t.library_top(P0, "Bayou");
    t.library_top(P0, "Island");
    t.library_top(P0, "Mountain");
    act(&mut t, P0, sphere, "search", &[]).unwrap();
    assert!(!t.on_battlefield(sphere));
    t.resolve_all();
    let hand: Vec<String> = t.g.player(P0).hand.iter().map(|c| t.obj_now(*c).chars.name.to_string()).collect();
    assert_eq!(hand.len(), 2, "{hand:?}");
    assert!(!hand.iter().any(|n| n == "Bayou"), "only basic lands");
    assert!(t.named_on_battlefield("Forest").is_empty(), "into the hand, not onto the battlefield");
}

#[test]
fn murmuring_volume_taps_and_discards_to_draw() {
    cr!("602.2", "118.3", "701.9a");
    // "{2}, {T}, Discard a card: Draw a card."
    supported("Murmuring Volume");
    let mut t = TestGame::new(2);
    let vol = t.battlefield(P0, "Murmuring Volume");
    t.lands(P0, "Wastes", 2);
    t.hand(P0, "Hill Giant");
    act(&mut t, P0, vol, "discard", &[]).unwrap();
    assert!(t.obj_now(vol).tapped);
    assert!(t.in_graveyard(P0, "Hill Giant"));
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    assert!(!t.in_hand(P0, "Hill Giant"));
}

#[test]
fn metrognome_makes_a_gnome_artifact_creature() {
    cr!("602.2", "111.1", "111.4");
    // "{4}, {T}: Create a 1/1 colorless Gnome artifact creature token."
    supported("Metrognome");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Metrognome");
    t.lands(P0, "Wastes", 4);
    act(&mut t, P0, m, "gnome", &[]).unwrap();
    assert!(t.obj_now(m).tapped);
    t.resolve_all();
    let g = subtype_ids(&t, P0, "Gnome");
    assert_eq!(g.len(), 1);
    let o = t.obj_now(g[0]);
    assert!(o.is_token() && o.is(CardType::Artifact) && o.is(CardType::Creature));
    assert!(o.chars.colors.is_colorless(), "colorless");
    assert_eq!(t.pt(g[0]), (1, 1));
}

#[test]
fn perpetual_timepiece_mills_two() {
    cr!("602.2", "701.17a");
    // "{T}: Mill two cards."
    supported("Perpetual Timepiece");
    let mut t = TestGame::new(2);
    let tp = t.battlefield(P0, "Perpetual Timepiece");
    let lib = t.library_size(P0);
    act(&mut t, P0, tp, "mill", &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.library_size(P0), lib - 2);
    assert_eq!(t.graveyard_size(P0), 2);
    assert_eq!(t.graveyard_size(P1), 0);
}

#[test]
fn crystalline_crawler_taps_to_grow() {
    cr!("602.2", "122.1");
    // "{T}: Put a +1/+1 counter on this creature."
    supported("Crystalline Crawler");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Crystalline Crawler");
    act(&mut t, P0, c, "{t}: put", &[]).unwrap();
    assert!(t.obj_now(c).tapped);
    t.resolve_all();
    assert_eq!(t.counters(c, "+1/+1"), 1);
    assert_eq!(t.pt(c), (2, 2));
}

#[test]
fn barkform_harvester_puts_a_graveyard_card_on_the_bottom() {
    cr!("602.2", "115.1", "401.3");
    // "{2}: Put target card from your graveyard on the bottom of your library."
    supported("Barkform Harvester");
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Barkform Harvester");
    let mine = t.graveyard(P0, "Hill Giant");
    let theirs = t.graveyard(P1, "Grizzly Bears");
    t.lands(P0, "Wastes", 2);
    // The opponent's graveyard card isn't a legal target: replaced by the legal one.
    act(&mut t, P0, h, "bottom", &[Entity::Object(theirs)]).unwrap();
    t.resolve_all();
    assert!(!t.in_graveyard(P0, "Hill Giant"));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    let bottom = t.g.player(P0).library[0];
    assert_eq!(t.obj_now(bottom).chars.name, "Hill Giant");
    let _ = mine;
}

#[test]
fn blistering_dieflyn_pumps_with_hybrid_mana() {
    cr!("602.2", "611.2a", "107.4e");
    // "{B/R}: This creature gets +1/+0 until end of turn."
    supported("Blistering Dieflyn");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Blistering Dieflyn");
    t.lands(P0, "Island", 1);
    assert!(act(&mut t, P0, d, "+1/+0", &[]).is_err(), "blue can't pay {{B/R}}");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Mountain", 1);
    act(&mut t, P0, d, "+1/+0", &[]).unwrap();
    act(&mut t, P0, d, "+1/+0", &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(d), (2, 1));
    next_turn(&mut t);
    assert_eq!(t.pt(d), (0, 1));
}

#[test]
fn brutal_deceiver_looks_at_the_top_card() {
    cr!("602.2", "401.2");
    // "{1}: Look at the top card of your library."
    supported("Brutal Deceiver");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Brutal Deceiver");
    let top = t.library_top(P0, "Hill Giant");
    t.lands(P0, "Wastes", 1);
    let lib = t.library_size(P0);
    act(&mut t, P0, d, "look", &[]).unwrap();
    assert_eq!(untapped(&t, P0, "Wastes"), 0, "{{1}} was paid");
    t.resolve_all();
    // Looking moves nothing.
    assert_eq!(t.library_size(P0), lib);
    assert_eq!(*t.g.player(P0).library.last().unwrap(), top);
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(t.pt(d), (2, 2));
}

#[test]
fn vodalian_hypnotist_makes_target_player_discard_as_a_sorcery() {
    cr!("602.2", "602.5d", "701.9a");
    // "{2}{B}, {T}: Target player discards a card. Activate only as a sorcery."
    supported("Vodalian Hypnotist");
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Vodalian Hypnotist");
    t.lands(P0, "Swamp", 3);
    t.hand(P1, "Hill Giant");
    t.hand(P0, "Grizzly Bears");
    t.set_step(P0, Step::Upkeep);
    assert!(
        act(&mut t, P0, h, "discards", &[Entity::Player(P1)]).is_err(),
        "only as a sorcery"
    );
    t.clear_answers();
    t.set_step(P0, Step::PrecombatMain);
    act(&mut t, P0, h, "discards", &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn ceta_disciple_gives_plus_two_power() {
    cr!("602.2", "611.2a");
    // "{R}, {T}: Target creature gets +2/+0 until end of turn."
    supported("Ceta Disciple");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Ceta Disciple");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 1);
    act(&mut t, P0, d, "+2/+0", &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 2));
    assert_eq!(t.pt(d), (1, 1));
    next_turn(&mut t);
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn sunbeam_spellbomb_sacrifices_for_five_life() {
    cr!("602.2", "119.3", "701.21a");
    // "{W}, Sacrifice this artifact: You gain 5 life."
    supported("Sunbeam Spellbomb");
    let mut t = TestGame::new(2);
    let bomb = t.battlefield(P0, "Sunbeam Spellbomb");
    t.lands(P0, "Plains", 1);
    act(&mut t, P0, bomb, "gain", &[]).unwrap();
    assert!(t.in_graveyard(P0, "Sunbeam Spellbomb"));
    t.resolve_all();
    assert_eq!(t.life(P0), 25);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn chitterspitter_makes_a_squirrel() {
    cr!("602.2", "111.1");
    // "{G}, {T}: Create a 1/1 green Squirrel creature token."
    supported("Chitterspitter");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Chitterspitter");
    t.lands(P0, "Forest", 1);
    act(&mut t, P0, c, "squirrel", &[]).unwrap();
    assert!(t.obj_now(c).tapped);
    t.resolve_all();
    let s = subtype_ids(&t, P0, "Squirrel");
    assert_eq!(s.len(), 1);
    assert_eq!(t.pt(s[0]), (1, 1));
    assert!(t.obj_now(s[0]).chars.colors.contains(Color::Green));
}

#[test]
fn bloodlust_inciter_grants_haste_to_target_creature() {
    cr!("602.2", "611.2a", "302.6");
    // "{T}: Target creature gains haste until end of turn."
    supported("Bloodlust Inciter");
    let mut t = TestGame::new(2);
    let inciter = t.battlefield(P0, "Bloodlust Inciter");
    let giant = t.battlefield_sick(P0, "Hill Giant");
    act(&mut t, P0, inciter, "haste", &[Entity::Object(giant)]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(giant).has_keyword(KeywordKind::Haste));
    assert!(!t.obj_now(inciter).has_keyword(KeywordKind::Haste));
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(giant, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 17);
    next_turn(&mut t);
    assert!(!t.obj_now(giant).has_keyword(KeywordKind::Haste));
}
