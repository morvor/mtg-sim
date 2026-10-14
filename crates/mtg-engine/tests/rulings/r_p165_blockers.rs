//! Rulings batch P165 — "whenever this creature blocks / attacks or blocks" and
//! "can block an additional creature": Wildwood Tracker, Sustainer of the Realm, Wall of
//! Nets, Lairwatch Giant, Loyal Sentry, Fibrous Entangler, Elder Gargaroth, Foriysian
//! Interceptor, Assembled Alphas and Merfolk Skyscout.

use crate::r_p189_common::bounce;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s03_common::to_blockers;
use crate::r_s06_common::has_kw;
use crate::r_s21_common::{blocks_now, legal_blocks};
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn o(id: ObjectId) -> Entity {
    Entity::Object(id)
}

fn p1() -> Entity {
    Entity::Player(P1)
}

fn blocked(t: &TestGame, id: ObjectId) -> bool {
    t.g.combat.as_ref().unwrap().is_blocked(t.g.current(id))
}

/// Triggered abilities on the stack whose source is `source`.
fn triggers_from(t: &TestGame, source: ObjectId) -> usize {
    crate::r_p189_common::triggers_of(t, source)
}

/// P1's `blocker` can block any number of creatures this turn (Valor Made Real).
fn valor_made_real(t: &mut TestGame, blocker: ObjectId) {
    supported("Valor Made Real");
    let p = t.g.obj(blocker).controller;
    t.lands(p, "Plains", 1);
    let v = t.hand(p, "Valor Made Real");
    t.cast(p, v).target(blocker).go();
    t.resolve_all();
}

#[test]
fn wildwood_tracker_checks_for_another_non_human_on_trigger_and_resolution() {
    cr!("603.4");
    ruling!(
        "Wildwood Tracker",
        "If you don't control another non-Human creature immediately after Wildwood Tracker attacks or blocks, its ability doesn't trigger at all."
    );
    supported("Wildwood Tracker");
    // No other non-Human creature (only a Human): no trigger.
    let mut t = TestGame::new(2);
    let tracker = t.battlefield(P0, "Wildwood Tracker");
    t.battlefield(P0, "Benalish Knight");
    attack_with(&mut t, &[(tracker, p1())]);
    assert_eq!(triggers_from(&t, tracker), 0);
    t.resolve_all();
    assert_eq!(t.pt(tracker), (1, 1));
    // A non-Human at both times: +1/+1.
    let mut t = TestGame::new(2);
    let tracker = t.battlefield(P0, "Wildwood Tracker");
    t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(tracker, p1())]);
    assert_eq!(triggers_from(&t, tracker), 1);
    t.resolve_all();
    assert_eq!(t.pt(tracker), (2, 2));
    // It leaves before the ability resolves: no bonus.
    let mut t = TestGame::new(2);
    let tracker = t.battlefield(P0, "Wildwood Tracker");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(tracker, p1())]);
    assert_eq!(triggers_from(&t, tracker), 1);
    bounce(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.pt(tracker), (1, 1));
    // Gaining one later in the turn doesn't help either.
    t.battlefield(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(t.pt(tracker), (1, 1));
    // The same when it blocks.
    let mut t = TestGame::new(2);
    let attacker = t.battlefield(P0, "Grizzly Bears");
    let tracker = t.battlefield(P1, "Wildwood Tracker");
    t.battlefield(P1, "Llanowar Elves");
    to_blockers(&mut t, &[(attacker, p1())], &[(tracker, attacker)]);
    assert_eq!(triggers_from(&t, tracker), 1);
    t.resolve_all();
    assert_eq!(t.pt(tracker), (2, 2));
}

#[test]
fn sustainer_of_the_realm_gets_the_bonus_once_however_many_it_blocks() {
    cr!("509.1i", "603.2");
    ruling!(
        "Sustainer of the Realm",
        "It gets the bonus only once, not once per creature it blocks."
    );
    supported("Sustainer of the Realm");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Llanowar Elves");
    let s = t.battlefield(P1, "Sustainer of the Realm");
    valor_made_real(&mut t, s);
    to_blockers(&mut t, &[(a, p1()), (b, p1())], &[(s, a), (s, b)]);
    assert!(blocked(&t, a) && blocked(&t, b));
    assert_eq!(triggers_from(&t, s), 1);
    t.resolve_all();
    assert_eq!(t.pt(s), (2, 5));
}

#[test]
fn wall_of_nets_exiles_only_if_it_is_still_on_the_battlefield_at_end_of_combat() {
    cr!("603.2", "511.1");
    ruling!(
        "Wall of Nets",
        "It only exiles creatures if this card is still on the battlefield at the end of combat."
    );
    supported("Wall of Nets");
    for leaves in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let wall = t.battlefield(P1, "Wall of Nets");
        to_blockers(&mut t, &[(bears, p1())], &[(wall, bears)]);
        if leaves {
            bounce(&mut t, wall);
            t.settle();
        }
        t.advance_to(P0, Step::EndOfCombat);
        t.resolve_all();
        assert_eq!(t.on_battlefield(bears), leaves, "wall left: {leaves}");
    }
}

#[test]
fn lairwatch_giant_gains_first_strike_only_blocking_two_or_more() {
    cr!("509.1i", "603.2");
    ruling!(
        "Lairwatch Giant",
        "Lairwatch Giant’s second ability triggers if it blocks at least two creatures when blockers are declared."
    );
    supported("Lairwatch Giant");
    for two in [false, true] {
        let mut t = TestGame::new(2);
        let a = t.battlefield(P0, "Grizzly Bears");
        let b = t.battlefield(P0, "Llanowar Elves");
        let giant = t.battlefield(P1, "Lairwatch Giant");
        let blocks = if two {
            vec![(giant, a), (giant, b)]
        } else {
            vec![(giant, a)]
        };
        to_blockers(&mut t, &[(a, p1()), (b, p1())], &blocks);
        assert_eq!(triggers_from(&t, giant), usize::from(two));
        t.resolve_all();
        assert_eq!(has_kw(&t, giant, KeywordKind::FirstStrike), two);
    }
}

#[test]
fn loyal_sentry_destroys_the_blocked_creature_even_if_it_left() {
    cr!("603.2", "608.2b");
    ruling!(
        "Loyal Sentry",
        "Loyal Sentry and the creature it blocks are destroyed before combat damage is dealt. The blocked creature is destroyed even if Loyal Sentry leaves the battlefield before its triggered ability resolves."
    );
    supported("Loyal Sentry");
    for leaves in [false, true] {
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P0, "Hill Giant");
        let sentry = t.battlefield(P1, "Loyal Sentry");
        to_blockers(&mut t, &[(giant, p1())], &[(sentry, giant)]);
        assert_eq!(triggers_from(&t, sentry), 1);
        if leaves {
            bounce(&mut t, sentry);
        }
        t.resolve_all();
        // Both are gone before the combat damage step.
        assert_eq!(t.g.turn.step, Step::DeclareBlockers);
        assert!(!t.on_battlefield(giant));
        assert!(t.in_graveyard(P0, "Hill Giant"));
        if leaves {
            assert!(t.in_hand(P1, "Loyal Sentry"));
        } else {
            assert!(t.in_graveyard(P1, "Loyal Sentry"));
        }
    }
}

#[test]
fn fibrous_entangler_needs_one_blocker_chosen_by_the_defending_player() {
    cr!("509.1c", "509.1a");
    ruling!(
        "Tangleclaw Werewolf // Fibrous Entangler",
        "Only one creature is required to block Fibrous Entangler. Other creatures may also block it, and are free to block other creatures or not block at all."
    );
    ruling!(
        "Tangleclaw Werewolf // Fibrous Entangler",
        "The defending player, not you, chooses which creature blocks Fibrous Entangler."
    );
    supported("Tangleclaw Werewolf // Fibrous Entangler");
    let mut t = TestGame::new(2);
    let ent = t.battlefield(P0, "Tangleclaw Werewolf // Fibrous Entangler");
    assert!(mtg_engine::dfc::transform(&mut t.g, ent));
    t.g.recompute();
    let ent = t.g.current(ent);
    assert_eq!(t.obj_now(ent).chars.name, "Fibrous Entangler");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wall = t.battlefield(P1, "Wall of Stone");
    let elves = t.battlefield(P1, "Llanowar Elves");
    let spider = t.battlefield(P1, "Giant Spider");
    attack_with(&mut t, &[(ent, p1()), (bears, p1())]);
    assert!(!legal_blocks(&mut t, P1, &[]));
    assert!(!legal_blocks(&mut t, P1, &[(wall, bears)]));
    // Any one of them suffices; the others may block it too, block something else, or
    // not block.
    assert!(legal_blocks(&mut t, P1, &[(elves, ent)]));
    assert!(legal_blocks(&mut t, P1, &[(wall, ent)]));
    assert!(legal_blocks(&mut t, P1, &[(wall, ent), (elves, ent), (spider, ent)]));
    assert!(legal_blocks(&mut t, P1, &[(wall, ent), (spider, bears)]));
    // The defending player makes the choice: P1 picks the Spider.
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(spider, ent)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    assert_eq!(blocks_now(&t), vec![(spider, ent)]);
    let blocker_decisions: Vec<PlayerId> = t
        .asked()
        .iter()
        .filter(|(_, d)| matches!(d, mtg_engine::decision::Decision::DeclareBlockers { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(blocker_decisions, vec![P1]);
}

#[test]
fn players_can_act_after_elder_gargaroths_trigger_resolves() {
    cr!("508.2", "509.2", "117.3b");
    ruling!(
        "Elder Gargaroth",
        "Players can cast spells after Elder Gargaroth's triggered ability has resolved but before blockers are declared (if it's attacking) or before combat damage is dealt (if it's blocking)."
    );
    supported("Elder Gargaroth");
    // Attacking: after the trigger (gain 3 life) resolves, P1 Bolts P0 in the declare
    // attackers step, before blockers are declared.
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Elder Gargaroth");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    attack_with(&mut t, &[(g, p1())]);
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    assert_eq!(t.g.turn.step, Step::DeclareAttackers);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(Entity::Player(P0)).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert!(t.g.combat.as_ref().unwrap().blockers.is_empty());
    // Blocking: P0 Bolts it in the declare blockers step, before combat damage.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let g = t.battlefield(P1, "Elder Gargaroth");
    t.answer(P1, DecisionKind::Modes, Answer::Indices(vec![1]));
    to_blockers(&mut t, &[(giant, p1())], &[(g, giant)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 23);
    assert_eq!(t.g.turn.step, Step::DeclareBlockers);
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(g).go();
    t.resolve_all();
    assert_eq!(t.obj_now(g).damage, 3);
    t.advance_to(P0, Step::EndOfCombat);
    assert!(!t.on_battlefield(giant));
}

#[test]
fn foriysian_interceptor_extra_blocks_are_cumulative() {
    cr!("509.1b", "509.1a");
    ruling!(
        "Foriysian Interceptor",
        "The third ability normally lets Foriysian Interceptor block two creatures. If another effect also lets it block an additional creature, the effects are cumulative."
    );
    supported("Foriysian Interceptor");
    supported("Act of Heroism");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Llanowar Elves");
    let c = t.battlefield(P0, "Hill Giant");
    let fi = t.battlefield(P1, "Foriysian Interceptor");
    attack_with(&mut t, &[(a, p1()), (b, p1()), (c, p1())]);
    assert!(legal_blocks(&mut t, P1, &[(fi, a), (fi, b)]));
    assert!(!legal_blocks(&mut t, P1, &[(fi, a), (fi, b), (fi, c)]));
    // Act of Heroism: "... can block an additional creature this turn": three.
    t.lands(P1, "Plains", 2);
    let act = t.hand(P1, "Act of Heroism");
    t.cast(P1, act).target(fi).go();
    t.resolve_all();
    assert!(legal_blocks(&mut t, P1, &[(fi, a), (fi, b), (fi, c)]));
}

#[test]
fn assembled_alphas_damages_the_controller_even_if_the_creature_isnt_dealt_damage() {
    cr!("603.2", "608.2b");
    ruling!(
        "Assembled Alphas",
        "The triggered ability deals damage to the creature’s controller even if the creature isn’t dealt damage, perhaps because it has left the battlefield or because the damage was prevented."
    );
    supported("Assembled Alphas");
    let mut t = TestGame::new(2);
    let alphas = t.battlefield(P0, "Assembled Alphas");
    let wall = t.battlefield(P1, "Wall of Stone");
    to_blockers(&mut t, &[(alphas, p1())], &[(wall, alphas)]);
    assert_eq!(triggers_from(&t, alphas), 1);
    bounce(&mut t, wall);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(t.in_hand(P1, "Wall of Stone"));
}

#[test]
fn assembled_alphas_triggers_per_creature_and_stays_blocked() {
    cr!("603.2", "509.1h", "506.4");
    ruling!(
        "Assembled Alphas",
        "The triggered ability triggers once for each creature blocking or blocked by Assembled Alphas. The ability resolves and deals damage to that creature before combat damage is dealt. If that damage destroys all creatures blocking Assembled Alphas, Assembled Alphas doesn’t become unblocked."
    );
    let mut t = TestGame::new(2);
    let alphas = t.battlefield(P0, "Assembled Alphas");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Llanowar Elves");
    to_blockers(&mut t, &[(alphas, p1())], &[(a, alphas), (b, alphas)]);
    assert_eq!(triggers_from(&t, alphas), 2);
    t.resolve_all();
    assert_eq!(t.g.turn.step, Step::DeclareBlockers);
    assert!(!t.on_battlefield(a) && !t.on_battlefield(b));
    assert_eq!(t.life(P1), 14);
    assert!(blocked(&t, alphas));
    t.advance_to(P0, Step::EndOfCombat);
    // No combat damage to P1 (no trample).
    assert_eq!(t.life(P1), 14);
    // Blocking: one trigger per creature it blocks.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Llanowar Elves");
    let alphas = t.battlefield(P1, "Assembled Alphas");
    valor_made_real(&mut t, alphas);
    to_blockers(&mut t, &[(a, p1()), (b, p1())], &[(alphas, a), (alphas, b)]);
    assert_eq!(triggers_from(&t, alphas), 2);
    t.resolve_all();
    assert_eq!(t.life(P0), 14);
}

#[test]
fn merfolk_skyscout_triggers_once_and_can_untap_any_permanent() {
    cr!("509.1i", "603.2", "115.1");
    ruling!(
        "Merfolk Skyscout",
        "When Merfolk Skyscout blocks, its second ability triggers just once, even if it somehow blocked multiple attacking creatures."
    );
    ruling!(
        "Merfolk Skyscout",
        "You may target any permanent with the ability, including Merfolk Skyscout itself or a permanent that’s already untapped."
    );
    supported("Merfolk Skyscout");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Llanowar Elves");
    let sky = t.battlefield(P1, "Merfolk Skyscout");
    valor_made_real(&mut t, sky);
    t.answer_targets(P1, &[o(sky)]);
    to_blockers(&mut t, &[(a, p1()), (b, p1())], &[(sky, a), (sky, b)]);
    assert_eq!(triggers_from(&t, sky), 1);
    assert_eq!(
        crate::r_s25_common::targets_of(&t, t.g.stack[0]),
        vec![o(sky)]
    );
    t.resolve_all();
    // Attacking: it may target itself (tapped by attacking) and untap it, or an
    // untapped permanent.
    for target_self in [true, false] {
        let mut t = TestGame::new(2);
        let sky = t.battlefield(P0, "Merfolk Skyscout");
        let island = t.battlefield(P1, "Island");
        let target = if target_self { sky } else { island };
        t.answer_targets(P0, &[o(target)]);
        attack_with(&mut t, &[(sky, p1())]);
        assert!(t.obj_now(sky).tapped);
        assert_eq!(triggers_from(&t, sky), 1);
        t.resolve_all();
        assert_eq!(t.obj_now(sky).tapped, !target_self);
        assert!(!t.obj_now(island).tapped);
    }
}
