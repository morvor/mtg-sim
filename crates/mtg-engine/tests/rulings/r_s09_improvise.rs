//! Rulings batch S09 — improvise (CR 702.126): "For each generic mana in this spell's
//! total cost, you may tap an untapped artifact you control rather than pay that mana."
//! It isn't an additional or alternative cost and applies once the total cost is
//! determined (CR 702.126b).

use crate::r_s01_common::*;
use crate::r_s02_common::create_token;
use crate::r_s04_common::untapped_lands;
use crate::r_s06_common::attach_new;
use crate::r_s09_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Whether `p` could begin to cast `card` normally now (it's among the cast options whose
/// costs look payable).
fn castable(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    t.g.recompute();
    t.g.turn.priority = Some(p);
    let card = t.g.current(card);
    t.g.cast_options(p, card)
        .into_iter()
        .any(|o| o.method == CastMethod::Normal && t.g.can_begin_cast(p, card, &o))
}

fn artifacts(t: &mut TestGame, p: PlayerId, n: usize) -> Vec<ObjectId> {
    (0..n).map(|_| t.battlefield(p, "Ornithopter")).collect()
}

fn tapped(t: &TestGame, ids: &[ObjectId]) -> usize {
    ids.iter().filter(|id| t.obj_now(**id).tapped).count()
}

fn tap_for_improvise(t: &mut TestGame, p: PlayerId, ids: &[ObjectId]) {
    t.answer_choose(p, &ids.iter().map(|a| Entity::Object(*a)).collect::<Vec<_>>());
}

/// The maximum of each improvise choice asked so far.
fn improvise_prompts(t: &TestGame) -> Vec<u32> {
    t.asked()
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { prompt, max, .. } if prompt.contains("improvise") => {
                Some(max)
            }
            _ => None,
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Tapping artifacts and Equipment
// ---------------------------------------------------------------------------

#[test]
fn a_tapped_artifacts_abilities_keep_applying() {
    cr!("702.126a", "110.5");
    ruling!(
        "Inspiring Statuary",
        "Tapping an artifact won't cause its abilities to stop applying unless those abilities say so."
    );
    supported("Inspiring Statuary");
    let mut t = TestGame::new(2);
    // Inspiring Statuary: "Nonartifact spells you cast have improvise."
    let statuary = t.battlefield(P0, "Inspiring Statuary");
    let orn = artifacts(&mut t, P0, 3);
    t.lands(P0, "Island", 2);
    let d1 = t.hand(P0, "Divination");
    let d2 = t.hand(P0, "Divination");
    // The Statuary taps to help cast the first Divination ({2}{U}).
    tap_for_improvise(&mut t, P0, &[statuary, orn[0]]);
    t.cast(P0, d1).go();
    t.resolve_all();
    assert!(t.obj_now(statuary).tapped);
    assert_eq!(untapped_lands(&t, P0), 1);
    // Tapped, it still gives the second Divination improvise.
    assert!(castable(&mut t, P0, d2));
    tap_for_improvise(&mut t, P0, &[orn[1], orn[2]]);
    t.cast(P0, d2).go();
    assert_eq!(tapped(&t, &orn), 3);
    assert_eq!(untapped_lands(&t, P0), 0);
}

#[test]
fn equipment_and_the_equipped_creature_tap_independently() {
    cr!("702.126a", "110.5c", "301.5a");
    ruling!(
        "Reverse Engineer",
        "Equipment attached to a creature doesn't become tapped when that creature becomes tapped, and tapping that Equipment doesn't cause the creature to become tapped."
    );
    supported("Reverse Engineer");
    supported("Bonesplitter");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let on_bears = attach_new(&mut t, P0, "Bonesplitter", bears);
    let on_elves = attach_new(&mut t, P0, "Bonesplitter", elves);
    // The Bears attack and become tapped; their Equipment doesn't.
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    assert!(t.obj_now(bears).tapped);
    assert!(!t.obj_now(on_bears).tapped);
    assert_eq!(t.life(P1), 16);
    // Both Equipment can be tapped for improvise; the Elves don't become tapped.
    t.advance_to(P0, Step::PostcombatMain);
    t.lands(P0, "Island", 3);
    let re = t.hand(P0, "Reverse Engineer");
    tap_for_improvise(&mut t, P0, &[on_bears, on_elves]);
    t.cast(P0, re).go();
    assert!(t.obj_now(on_bears).tapped && t.obj_now(on_elves).tapped);
    assert!(!t.obj_now(elves).tapped);
    assert_eq!(untapped_lands(&t, P0), 0);
    // The tapped Equipment still applies.
    assert_eq!(t.pt(elves), (3, 1));
}

#[test]
fn tapped_equipment_still_applies_battle_at_the_bridge() {
    cr!("702.126a", "110.5c", "301.5a");
    ruling!(
        "Battle at the Bridge",
        "Tapping an artifact won’t cause its abilities to stop applying unless those abilities say so."
    );
    ruling!(
        "Battle at the Bridge",
        "Equipment attached to a creature doesn’t become tapped when that creature becomes tapped, and tapping that Equipment doesn’t cause the creature to become tapped."
    );
    supported("Battle at the Bridge");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let blade = attach_new(&mut t, P0, "Bonesplitter", bears);
    let orn = artifacts(&mut t, P0, 2);
    let giant = t.battlefield(P1, "Hill Giant");
    // The Bears attack: the Bonesplitter stays untapped.
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    assert!(!t.obj_now(blade).tapped);
    // Battle at the Bridge with X = 3 ({3}{B}): the Bonesplitter and two Ornithopters pay
    // {3}.
    t.advance_to(P0, Step::PostcombatMain);
    t.lands(P0, "Swamp", 1);
    let battle = t.hand(P0, "Battle at the Bridge");
    tap_for_improvise(&mut t, P0, &[blade, orn[0], orn[1]]);
    t.cast(P0, battle).x(3).target(giant).go();
    t.resolve_all();
    assert!(t.obj_now(blade).tapped);
    assert_eq!(t.zone(giant), Zone::Graveyard(P1));
    assert_eq!(t.life(P0), 23);
    // The Bears still get +2/+0.
    assert_eq!(t.pt(bears), (4, 2));
}

// ---------------------------------------------------------------------------
// Mana abilities of artifacts
// ---------------------------------------------------------------------------

#[test]
fn an_artifact_tapped_or_sacrificed_for_mana_cant_also_improvise() {
    cr!("702.126a", "601.2g", "601.2h");
    ruling!(
        "Reverse Engineer",
        "If an artifact you control has a mana ability with {T} in the cost, activating that ability while casting a spell with improvise will result in the artifact being tapped when you pay the spell's costs. You won't be able to tap it again for improvise."
    );
    supported("Sol Ring");
    // Reverse Engineer ({3}{U}{U}) with two Islands and Sol Ring: Sol Ring pays {2} for
    // mana or {1} by improvise, not both, so the spell can't be cast.
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P0, "Sol Ring");
    t.lands(P0, "Island", 2);
    let re = t.hand(P0, "Reverse Engineer");
    assert!(t.cast(P0, re).try_go().is_err());
    assert_eq!(t.zone(re), Zone::Hand(P0));
    assert!(!t.obj_now(ring).tapped);
    assert_eq!(untapped_lands(&t, P0), 2);
    // With one more artifact, Sol Ring's mana and the Ornithopter pay for {3}.
    let orn = t.battlefield(P0, "Ornithopter");
    t.cast(P0, re).go();
    assert!(t.obj_now(orn).tapped && t.obj_now(ring).tapped);
    assert_eq!(untapped_lands(&t, P0), 0);
    // A Treasure sacrificed for mana isn't there to be tapped for improvise.
    let mut t = TestGame::new(2);
    create_token(&mut t, P0, "Treasure");
    t.battlefield(P0, "Ornithopter");
    t.lands(P0, "Island", 2);
    let re = t.hand(P0, "Reverse Engineer");
    assert!(t.cast(P0, re).try_go().is_err());
    assert_eq!(t.zone(re), Zone::Hand(P0));
    assert_eq!(with_subtype(&t, P0, "Treasure").len(), 1);
    t.battlefield(P0, "Ornithopter");
    t.cast(P0, re).go();
    assert!(with_subtype(&t, P0, "Treasure").is_empty());
}

#[test]
fn an_artifact_tapped_for_mana_cant_also_improvise_bastion_inventor() {
    cr!("702.126a", "601.2g");
    ruling!(
        "Bastion Inventor",
        "If an artifact you control has a mana ability with {T} in the cost, activating that ability while casting a spell with improvise will result in the artifact being tapped when you pay the spell’s costs."
    );
    supported("Bastion Inventor");
    // Bastion Inventor ({5}{U}): an Island, Sol Ring ({2}) and two Ornithopters pay only
    // {U} and four generic mana.
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P0, "Sol Ring");
    let orn = artifacts(&mut t, P0, 2);
    t.lands(P0, "Island", 1);
    let bi = t.hand(P0, "Bastion Inventor");
    assert!(t.cast(P0, bi).try_go().is_err());
    assert_eq!(tapped(&t, &orn), 0);
    assert!(!t.obj_now(ring).tapped);
    let more = t.battlefield(P0, "Ornithopter");
    t.cast(P0, bi).go();
    assert_eq!(tapped(&t, &[orn[0], orn[1], more, ring]), 4);
}

// ---------------------------------------------------------------------------
// Improvise with alternative costs and cost changes
// ---------------------------------------------------------------------------

#[test]
fn improvise_works_with_an_alternative_cost() {
    cr!("702.126b", "702.74a");
    ruling!(
        "Inspiring Statuary",
        "Because improvise isn't an alternative cost, it can be used in conjunction with alternative costs."
    );
    supported("Mulldrifter");
    let mut t = TestGame::new(2);
    let statuary = t.battlefield(P0, "Inspiring Statuary");
    let orn = t.battlefield(P0, "Ornithopter");
    t.lands(P0, "Island", 1);
    let md = t.hand(P0, "Mulldrifter");
    // Evoke {2}{U}: the Statuary and the Ornithopter pay {2}.
    tap_for_improvise(&mut t, P0, &[statuary, orn]);
    let hand = t.hand_size(P0);
    t.cast(P0, md)
        .method(CastMethod::Keyword(KeywordKind::Evoke))
        .go();
    assert_eq!(tapped(&t, &[statuary, orn]), 2);
    assert_eq!(untapped_lands(&t, P0), 0);
    t.resolve_all();
    // Evoked: it's sacrificed, and its controller draws two cards.
    assert!(t.in_graveyard(P0, "Mulldrifter"));
    assert_eq!(t.hand_size(P0), hand - 1 + 2);
}

#[test]
fn improvise_works_with_flashback_battle_at_the_bridge() {
    cr!("702.126b", "702.34a");
    ruling!(
        "Battle at the Bridge",
        "Because improvise isn’t an alternative cost, it can be used in conjunction with alternative costs."
    );
    supported("Snapcaster Mage");
    let mut t = TestGame::new(2);
    let battle = t.graveyard(P0, "Battle at the Bridge");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // Snapcaster Mage gives it flashback {X}{B}.
    t.answer_targets(P0, &[Entity::Object(battle)]);
    t.enter(P0, "Snapcaster Mage");
    t.resolve_all();
    let orn = artifacts(&mut t, P0, 2);
    t.lands(P0, "Swamp", 1);
    tap_for_improvise(&mut t, P0, &orn);
    t.cast(P0, battle)
        .method(CastMethod::Keyword(KeywordKind::Flashback))
        .x(2)
        .target(bears)
        .go();
    assert_eq!(tapped(&t, &orn), 2);
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Graveyard(P1));
    assert_eq!(t.life(P0), 22);
    assert!(t.in_exile("Battle at the Bridge"));
}

#[test]
fn improvise_can_pay_a_cost_increase_battle_at_the_bridge() {
    cr!("702.126b", "601.2f");
    ruling!(
        "Battle at the Bridge",
        "When calculating a spell’s total cost, include any alternative costs, additional costs, or anything else that increases or reduces the cost to cast the spell. Improvise applies after the total cost is calculated."
    );
    supported("Thalia, Guardian of Thraben");
    let mut t = TestGame::new(2);
    // Thalia: noncreature spells cost {1} more. X = 1: the total cost is {2}{B}.
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let orn = artifacts(&mut t, P0, 2);
    t.lands(P0, "Swamp", 1);
    let battle = t.hand(P0, "Battle at the Bridge");
    tap_for_improvise(&mut t, P0, &orn);
    t.cast(P0, battle).x(1).target(bears).go();
    assert_eq!(tapped(&t, &orn), 2);
    assert_eq!(improvise_prompts(&t), vec![2]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (1, 1));
}

#[test]
fn improvise_doesnt_change_the_mana_value_battle_at_the_bridge() {
    cr!("702.126a", "202.3e");
    ruling!(
        "Battle at the Bridge",
        "Improvise doesn’t change a spell’s mana cost or mana value."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let orn = artifacts(&mut t, P0, 2);
    t.lands(P0, "Swamp", 1);
    let battle = t.hand(P0, "Battle at the Bridge");
    tap_for_improvise(&mut t, P0, &orn);
    let spell = t.cast(P0, battle).x(2).target(bears).go();
    // {X}{B} with X = 2: mana value 3, though only {B} was paid in mana.
    assert_eq!(t.g.mana_value_of(spell), 3);
    assert_eq!(
        t.g.obj(spell).chars.mana_cost.as_ref().map(|m| m.to_string()),
        Some("{X}{B}".to_string())
    );
}

#[test]
fn improvise_cant_pay_colored_mana_bastion_inventor() {
    cr!("702.126a");
    ruling!(
        "Bastion Inventor",
        "Improvise can’t pay for {W}, {U}, {B}, {R}, {G}, or {C} mana symbols in a spell’s total cost."
    );
    let mut t = TestGame::new(2);
    artifacts(&mut t, P0, 6);
    let bi = t.hand(P0, "Bastion Inventor");
    assert!(!castable(&mut t, P0, bi));
    assert!(t.cast(P0, bi).try_go().is_err());
    t.lands(P0, "Island", 1);
    assert!(castable(&mut t, P0, bi));
    t.cast(P0, bi).go();
    // Five artifacts pay {5}; the Island pays {U}.
    assert_eq!(improvise_prompts(&t).last(), Some(&5));
}

#[test]
fn improvise_with_x_uses_the_total_cost_after_choosing_x() {
    cr!("702.126b", "107.3a", "601.2f");
    ruling!(
        "Reverse Engineer",
        "For example, if you cast Saheeli's Directive and choose X to be 3, the total cost is {3}{R}{R}{R}. If you tap two artifacts, you'll have to pay {1}{R}{R}{R}."
    );
    supported("Saheeli's Directive");
    let mut t = TestGame::new(2);
    let orn = artifacts(&mut t, P0, 2);
    t.lands(P0, "Mountain", 4);
    // The top three cards: Sol Ring and Bonesplitter (mana value 1) and Grizzly Bears.
    let top = stack_library(&mut t, P0, &["Sol Ring", "Grizzly Bears", "Bonesplitter"]);
    let sd = t.hand(P0, "Saheeli's Directive");
    tap_for_improvise(&mut t, P0, &orn);
    t.answer_choose(P0, &[Entity::Object(top[0]), Entity::Object(top[2])]);
    t.cast(P0, sd).x(3).go();
    // Two artifacts and four Mountains: {3}{R}{R}{R}.
    assert_eq!(tapped(&t, &orn), 2);
    assert_eq!(untapped_lands(&t, P0), 0);
    assert_eq!(improvise_prompts(&t), vec![2]);
    t.resolve_all();
    assert!(t.on_battlefield(top[0]));
    assert!(t.on_battlefield(top[2]));
    assert_eq!(t.zone(top[1]), Zone::Graveyard(P0));
}

// ---------------------------------------------------------------------------
// Improvise pays only the spell's own cost.
// ---------------------------------------------------------------------------

/// P0 casts `name` paying with `lands` while controlling three untapped Ornithopters it
/// doesn't tap; P1 responds with Mana Leak ("Counter target spell unless its controller
/// pays {3}."). The artifacts can't pay the {3}: the spell is countered.
fn improvise_cant_pay_for_mana_leak(name: &str, land: &str, n: usize) {
    supported(name);
    supported("Mana Leak");
    let mut t = TestGame::new(2);
    let orn = artifacts(&mut t, P0, 3);
    t.lands(P0, land, n);
    let card = t.hand(P0, name);
    tap_for_improvise(&mut t, P0, &[]);
    let spell = t.cast(P0, card).go();
    assert_eq!(tapped(&t, &orn), 0);
    assert_eq!(untapped_lands(&t, P0), 0);
    t.lands(P1, "Island", 2);
    let leak = t.hand(P1, "Mana Leak");
    t.answer_yes(P0, true);
    t.cast(P1, leak).target(spell).go();
    t.resolve();
    assert_eq!(t.zone(spell), Zone::Graveyard(P0));
    assert_eq!(tapped(&t, &orn), 0);
}

#[test]
fn improvise_cant_pay_for_counter_unless_pays() {
    cr!("702.126a", "118.12a");
    ruling!(
        "Freejam Regent",
        "Improvise can't be used to pay for anything other than the cost of casting the spell. For example, it can't be used during the resolution of an ability that says \"Counter target spell unless its controller pays {3}.\""
    );
    improvise_cant_pay_for_mana_leak("Freejam Regent", "Mountain", 6);
}

#[test]
fn improvise_cant_pay_for_counter_unless_pays_bastion_inventor() {
    cr!("702.126a", "118.12a");
    ruling!(
        "Bastion Inventor",
        "Improvise can’t be used to pay for anything other than the cost of casting the spell. For example, it can’t be used during the resolution of an ability that says “Counter target spell unless its controller pays {3}.”"
    );
    improvise_cant_pay_for_mana_leak("Bastion Inventor", "Island", 6);
}

#[test]
fn improvise_cant_pay_for_counter_unless_pays_reverse_engineer() {
    cr!("702.126a", "118.12a");
    ruling!(
        "Reverse Engineer",
        "Improvise can't be used to pay for anything other than the cost of casting the spell. For example, it can't be used during the resolution of an ability that says “Counter target spell unless its controller pays {3}.”"
    );
    improvise_cant_pay_for_mana_leak("Reverse Engineer", "Island", 5);
}
