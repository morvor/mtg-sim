//! Rulings batch P036 — counters-matter cards: counts checked on resolution and last known
//! information (CR 608.2h), choices made as a trigger resolves (CR 608.2c), optional
//! targets (CR 115.1, 608.2b), playing exiled cards with normal timing (CR 305.1, 601.3),
//! regeneration (CR 701.19), and attack costs versus requirements (CR 508.1d, 508.1h).

use crate::r_p036_common::*;
use crate::r_s01_common::{attack_with, stack_library, supported};
use crate::r_s04_common::top_of_stack;
use crate::r_s06_common::activate_containing;
use crate::r_s09_common::{legal_attack, to_combat};
use crate::r_s13_common::add;
use crate::r_s25_common::{cast_new, targets_of};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// From the declare attackers step, no blocks: advances to P0's combat damage step (combat
/// damage dealt, its triggers on the stack).
fn to_damage(t: &mut TestGame) {
    t.answer(
        P1,
        DecisionKind::Blockers,
        mtg_engine::decision::Answer::Blockers(vec![]),
    );
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
}

/// P0's The War Doctor with `n` time counters attacks P1; its attack trigger ("it deals
/// damage equal to the number of time counters on it to any target") targets P1 and is on
/// the stack. Returns (the War Doctor, the trigger).
fn war_doctor_attacks(t: &mut TestGame, n: u32) -> (ObjectId, ObjectId) {
    supported("The War Doctor");
    let doc = t.battlefield(P0, "The War Doctor");
    add(t, obj(doc), "time", n);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    attack_with(t, &[(doc, Entity::Player(P1))]);
    let trig = top_of_stack(t);
    assert_eq!(targets_of(t, trig), vec![Entity::Player(P1)]);
    (doc, trig)
}

#[test]
fn the_war_doctor_counts_time_counters_on_resolution_or_last_known() {
    cr!("603.3d", "608.2h", "113.7a");
    ruling!(
        "The War Doctor",
        "You choose the target of the second ability as it attacks. The number of time counters is checked at the time the ability resolves. If The War Doctor has left the battlefield by that time, use the number of time counters it had on it the last time it was on the battlefield to determine how much damage it deals to that target."
    );
    // A third counter arrives before the trigger resolves: 3 damage.
    let mut t = TestGame::new(2);
    let (doc, _) = war_doctor_attacks(&mut t, 2);
    add(&mut t, obj(doc), "time", 1);
    t.resolve();
    assert_eq!(t.life(P1), 17);
    // It gets a third counter, then leaves the battlefield: its last known count, 3.
    let mut t = TestGame::new(2);
    let (doc, _) = war_doctor_attacks(&mut t, 2);
    add(&mut t, obj(doc), "time", 1);
    crate::r_s02_common::destroy(&mut t, doc);
    assert!(!t.on_battlefield(doc));
    t.resolve();
    assert_eq!(t.life(P1), 17);
}

/// P0's Rotisserie Elemental (menace) with one skewer counter deals combat damage to P1,
/// with Forest and Hill Giant on top of P0's library; its trigger ("put a skewer counter
/// on this creature. Then you may sacrifice it. If you do, exile the top X cards of your
/// library, where X is the number of skewer counters on this creature. You may play those
/// cards this turn.") is on the stack. Returns (the Elemental, the library's top cards).
fn rotisserie(t: &mut TestGame) -> (ObjectId, Vec<ObjectId>) {
    supported("Rotisserie Elemental");
    let rot = t.battlefield(P0, "Rotisserie Elemental");
    add(t, obj(rot), "skewer", 1);
    let top = stack_library(t, P0, &["Forest", "Hill Giant", "Mountain"]);
    attack_with(t, &[(rot, Entity::Player(P1))]);
    to_damage(t);
    assert_eq!(t.life(P1), 19);
    (rot, top)
}

#[test]
fn rotisserie_elemental_sacrifices_as_its_trigger_resolves() {
    cr!("608.2c", "608.2h", "117.1");
    ruling!(
        "Rotisserie Elemental",
        "You choose whether to sacrifice Rotisserie Elemental as its triggered ability resolves. No player may respond between the time you sacrifice it and the time you exile cards from the top of your library."
    );
    let mut t = TestGame::new(2);
    let seen = crate::r_s01_common::watch(
        &mut t,
        P0,
        |d| matches!(d, mtg_engine::decision::Decision::YesNo { .. }),
        |g| {
            (
                g.permanents()
                    .find(|o| o.chars.name == "Rotisserie Elemental")
                    .map(|o| o.counter("skewer")),
                g.stack.len(),
            )
        },
    );
    let (rot, top) = rotisserie(&mut t);
    t.settle();
    let trig = top_of_stack(&t);
    t.answer_yes(P0, true);
    let from = t.asked().len();
    t.g.resolve_top();
    // The choice was asked while the trigger resolved (the second counter already on
    // it); the sacrifice and the exile happened in that same resolution, with no
    // priority in between.
    assert!(!on_stack(&t, trig));
    assert!(!t.on_battlefield(rot));
    assert!(t.in_graveyard(P0, "Rotisserie Elemental"));
    assert_eq!(t.zone(t.g.current(top[0])), Zone::Exile);
    assert_eq!(t.zone(t.g.current(top[1])), Zone::Exile);
    assert_eq!(t.zone(t.g.current(top[2])), Zone::Library(P0));
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, mtg_engine::decision::Decision::Priority { .. })));
    let seen = seen.lock().unwrap();
    assert!(seen.iter().any(|(c, _)| *c == Some(2)), "{seen:?}");
}

#[test]
fn rotisserie_elemental_cards_follow_normal_timing_and_costs() {
    cr!("305.1", "305.2", "601.2h", "307.1");
    ruling!(
        "Rotisserie Elemental",
        "You pay all costs and follow all normal timing rules for cards played from exile this way. For example, if the exiled card is a land card, you may play it only during your main phase while the stack is empty."
    );
    let mut t = TestGame::new(2);
    let (_, top) = rotisserie(&mut t);
    t.settle();
    t.answer_yes(P0, true);
    t.resolve();
    let forest = t.g.current(top[0]);
    let giant = t.g.current(top[1]);
    assert_eq!(t.zone(forest), Zone::Exile);
    // Combat damage step: neither can be played now.
    assert!(!crate::r_s02_common::can_play_land(&mut t, P0, forest));
    t.lands(P0, "Mountain", 4);
    assert!(!crate::r_s21_common::castable(&mut t, P0, giant));
    t.advance_to(P0, Step::PostcombatMain);
    // Main phase, stack not empty: no land play.
    let bolt = cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    assert!(on_stack(&t, bolt));
    assert!(!crate::r_s02_common::can_play_land(&mut t, P0, forest));
    t.resolve();
    // Main phase, empty stack: the land can be played, and Hill Giant cast by paying
    // {3}{R} (four Mountains).
    assert!(crate::r_s02_common::can_play_land(&mut t, P0, forest));
    assert!(crate::r_s21_common::castable(&mut t, P0, giant));
    let gs = t.cast(P0, giant).go();
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 5);
    t.resolve();
    assert!(t.on_battlefield(t.g.current(gs)));
    t.play_land(P0, forest).unwrap();
    assert!(t.on_battlefield(t.g.current(forest)));
}

#[test]
fn rotisserie_elemental_without_mana_cant_cast_the_exiled_card() {
    cr!("601.2h");
    ruling!(
        "Rotisserie Elemental",
        "You pay all costs and follow all normal timing rules for cards played from exile this way."
    );
    let mut t = TestGame::new(2);
    let (_, top) = rotisserie(&mut t);
    t.settle();
    t.answer_yes(P0, true);
    t.resolve();
    t.advance_to(P0, Step::PostcombatMain);
    let giant = t.g.current(top[1]);
    t.lands(P0, "Mountain", 3);
    assert!(!crate::r_s21_common::castable(&mut t, P0, giant));
    t.lands(P0, "Mountain", 1);
    assert!(crate::r_s21_common::castable(&mut t, P0, giant));
}

/// P0's Wakka, Devoted Guardian deals combat damage to P1 (`artifact`: P1 controls an
/// Ornithopter, targeted). Returns (Wakka, the Ornithopter).
fn wakka(t: &mut TestGame, artifact: bool) -> (ObjectId, Option<ObjectId>) {
    supported("Wakka, Devoted Guardian");
    let w = t.battlefield(P0, "Wakka, Devoted Guardian");
    let orni = artifact.then(|| t.battlefield(P1, "Ornithopter"));
    if let Some(o) = orni {
        t.answer_targets(P0, &[obj(o)]);
    }
    attack_with(t, &[(w, Entity::Player(P1))]);
    to_damage(t);
    assert_eq!(t.life(P1), 16);
    (w, orni)
}

#[test]
fn wakka_needs_no_target_but_an_illegal_one_stops_the_counter() {
    cr!("115.1d", "608.2b", "603.3d");
    ruling!(
        "Wakka, Devoted Guardian",
        "You don't have to choose a target for Wakka's second ability. However, if you do and the target is illegal as the ability tries to resolve, it won't resolve and none of its effects will happen. You won't put a counter on Wakka."
    );
    // No artifact: no target, still a counter.
    let mut t = TestGame::new(2);
    let (w, _) = wakka(&mut t, false);
    t.resolve_all();
    assert_eq!(t.counters(w, "+1/+1"), 1);
    // A target that's gone by resolution: no counter.
    let mut t = TestGame::new(2);
    let (w, orni) = wakka(&mut t, true);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    crate::r_s02_common::destroy(&mut t, orni.unwrap());
    t.resolve_all();
    assert_eq!(t.counters(w, "+1/+1"), 0);
    // Control: the target is still there: destroyed, and a counter.
    let mut t = TestGame::new(2);
    let (w, _) = wakka(&mut t, true);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Ornithopter"));
    assert_eq!(t.counters(w, "+1/+1"), 1);
}

#[test]
fn malcolm_cant_play_a_discarded_land() {
    cr!("305.9", "601.1", "608.2c");
    ruling!(
        "Malcolm, Alluring Scoundrel",
        "You may not play land cards discarded with Malcolm, Alluring Scoundrel's last ability."
    );
    supported("Malcolm, Alluring Scoundrel");
    // (the card drawn and discarded, whether it ends up played)
    for (card, played) in [("Forest", false), ("Lightning Bolt", true)] {
        let mut t = TestGame::new(2);
        let m = t.battlefield(P0, "Malcolm, Alluring Scoundrel");
        add(&mut t, obj(m), "chorus", 3);
        let drawn = t.library_top(P0, card);
        attack_with(&mut t, &[(m, Entity::Player(P1))]);
        to_damage(&mut t);
        t.answer_choose_moved(P0, &[obj(drawn)]);
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.resolve_all();
        assert_eq!(t.counters(m, "chorus"), 4);
        assert_eq!(t.life(P1), if played { 20 - 2 - 3 } else { 18 }, "{card}");
        // Either way the card ends in the graveyard (discarded, or cast and resolved).
        assert!(t.in_graveyard(P0, card), "{card}");
        assert!(t.named_on_battlefield("Forest").is_empty());
    }
}

#[test]
fn altar_of_shadows_gets_its_counter_even_if_the_creature_regenerates() {
    cr!("701.19a", "608.2c");
    ruling!(
        "Altar of Shadows",
        "You put the counter on Altar of Shadows even if the creature regenerates."
    );
    supported("Altar of Shadows");
    supported("River Boa");
    let mut t = TestGame::new(2);
    let altar = t.battlefield(P0, "Altar of Shadows");
    let boa = t.battlefield(P1, "River Boa");
    t.lands(P1, "Forest", 1);
    activate_containing(&mut t, P1, boa, "Regenerate").unwrap();
    t.resolve();
    t.lands(P0, "Wastes", 7);
    t.answer_targets(P0, &[obj(boa)]);
    activate_containing(&mut t, P0, altar, "Destroy target creature").unwrap();
    t.resolve();
    assert!(t.on_battlefield(boa));
    assert!(t.obj_now(boa).tapped);
    assert_eq!(t.counters(altar, "charge"), 1);
}

#[test]
fn nils_attack_tax_versus_attacks_if_able() {
    cr!("508.1d", "508.1h", "506.4");
    ruling!(
        "Nils, Discipline Enforcer",
        "Your opponents can choose not to pay to attack with a creature with one or more counters on it that attacks \"if able.\" If there's no other player or planeswalker to attack, the creature simply doesn't attack."
    );
    supported("Nils, Discipline Enforcer");
    supported("Juggernaut");
    // Two players: P1's Juggernaut ("attacks each combat if able") with a counter needn't
    // attack P0 (Nils's controller); attacking P0 would cost {1}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Nils, Discipline Enforcer");
    let jug = t.battlefield(P1, "Juggernaut");
    add(&mut t, obj(jug), "+1/+1", 1);
    to_combat(&mut t, P1);
    assert!(legal_attack(&mut t, &[]));
    let attacks = crate::r_s09_common::declare(&mut t, P1, &[]);
    assert!(attacks.is_empty());
    // Control: without a counter it must attack.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Nils, Discipline Enforcer");
    let jug = t.battlefield(P1, "Juggernaut");
    to_combat(&mut t, P1);
    assert!(!legal_attack(&mut t, &[]));
    assert!(legal_attack(&mut t, &[(jug, Entity::Player(P0))]));
    // Three players: with another opponent to attack for free, it must attack (that
    // player).
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Nils, Discipline Enforcer");
    let jug = t.battlefield(P1, "Juggernaut");
    add(&mut t, obj(jug), "+1/+1", 1);
    to_combat(&mut t, P1);
    assert!(!legal_attack(&mut t, &[]));
    assert!(legal_attack(&mut t, &[(jug, Entity::Player(P2))]));
}
