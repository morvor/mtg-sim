//! CR 701.43: exert.

use crate::a701_028_071_common::*;
use mtg_engine::ability::{KeywordAction, Sel};
use mtg_engine::kwa::exert::EXERTED_EVENT;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Has `p` exert `obj` with an effect.
fn exert(t: &mut TestGame, p: PlayerId, obj: ObjectId) {
    run(
        t,
        p,
        None,
        ka(KeywordAction::Exert, Sel::Target(0), 1),
        &[Entity::Object(obj)],
    );
}

fn cats(t: &TestGame) -> usize {
    t.g.permanents()
        .filter(|o| o.is_token() && o.chars.has_subtype("Cat"))
        .count()
}

#[test]
fn an_exerted_permanent_doesnt_untap_during_your_next_untap_step() {
    cr!("701.43a");
    supported("Pride Sovereign");
    // "{W}, {T}, Exert this creature: Create two 1/1 white Cat creature tokens with
    // lifelink."
    let mut t = TestGame::new(2);
    let sovereign = t.battlefield(P0, "Pride Sovereign");
    t.lands(P0, "Plains", 1);
    t.activate(P0, sovereign, 0, &[]).expect("activate");
    t.resolve_all();
    assert_eq!(cats(&t), 2);
    assert!(t.obj_now(sovereign).exerted);
    assert_eq!(
        custom_events(&t, EXERTED_EVENT),
        vec![(Some(P0), Some(sovereign), 0)]
    );
    // Not the opponent's untap step either (it's not theirs), nor yours.
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj_now(sovereign).tapped);
    t.advance_to(P0, Step::Upkeep);
    assert!(t.obj_now(sovereign).tapped);
    // Only the next one.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert!(!t.obj_now(sovereign).tapped);
}

#[test]
fn a_creature_you_exert_while_controlling_it_until_end_of_turn_untaps_for_its_owner() {
    cr!("701.43a");
    ruling!(
        "Watchful Naga",
        "If you gain control of another player’s creature until end of turn and exert it, it will untap during that player’s untap step."
    );
    // Watchful Naga: "You may exert this creature as it attacks. When you do, draw a card."
    supported("Watchful Naga");
    supported("Act of Treason");
    let mut t = TestGame::new(2);
    let naga = t.battlefield(P1, "Watchful Naga");
    t.lands(P0, "Mountain", 3);
    let spell = t.hand(P0, "Act of Treason");
    t.answer_targets(P0, &[Entity::Object(naga)]);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(t.obj_now(naga).controller, P0);
    t.answer_yes(P0, true);
    t.attack(&[(naga, Entity::Player(P1))], &[]);
    assert!(t.obj_now(naga).exerted && t.obj_now(naga).tapped);
    // Back under its owner's control, it untaps during their untap step.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.obj_now(naga).controller, P1);
    assert!(!t.obj_now(naga).tapped);
}

#[test]
fn a_permanent_can_be_exerted_while_untapped_or_again() {
    cr!("701.43b");
    ruling!(
        "Fervent Paincaster",
        "Exerting it multiple times will keep it tapped only during your next untap step."
    );
    supported("Fervent Paincaster");
    // "{T}, Exert this creature: It deals 1 damage to target creature."
    let mut t = TestGame::new(2);
    let caster = t.battlefield(P0, "Fervent Paincaster");
    let giant = t.battlefield(P1, "Hill Giant");
    t.activate(P0, caster, 1, &[Entity::Object(giant)])
        .expect("activate");
    t.resolve_all();
    // Untapped, then exerted again to activate it again.
    t.g.untap(caster);
    t.activate(P0, caster, 1, &[Entity::Object(giant)])
        .expect("activate again");
    t.resolve_all();
    assert_eq!(t.obj_now(giant).damage, 2);
    assert_eq!(custom_events(&t, EXERTED_EVENT).len(), 2);
    t.advance_to(P0, Step::Upkeep);
    assert!(t.obj_now(caster).tapped);
    // Both expire in that same untap step.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert!(!t.obj_now(caster).tapped);
}

#[test]
fn an_untapped_exerted_permanent() {
    cr!("701.43b");
    ruling!(
        "Watchful Naga",
        "If an exerted creature is already untapped during your next untap step (most likely because it had vigilance or an effect untapped it), exert’s effect preventing it from untapping expires without having done anything."
    );
    // Exerted while untapped, then tapped: it stays tapped in the next untap step.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    exert(&mut t, P0, bears);
    assert!(t.obj_now(bears).exerted);
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    t.advance_to(P0, Step::Upkeep);
    assert!(t.obj_now(bears).tapped);
    // Untapped during that untap step: the effect expires; tapped afterward, it untaps as
    // usual during the following untap step.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    exert(&mut t, P0, bears);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert!(!t.obj_now(bears).exerted);
    t.g.tap(bears);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert!(!t.obj_now(bears).tapped);
}

#[test]
fn a_creature_with_vigilance_can_be_exerted_as_it_attacks() {
    cr!("701.43b", "701.43d");
    ruling!(
        "Trial of Solidarity",
        "If an effect allows you to exert a creature as it attacks, you may do so even if it has vigilance. It won’t be tapped."
    );
    supported("Trial of Solidarity");
    // "When this enchantment enters, creatures you control get +2/+1 and gain vigilance
    // until end of turn."
    let mut t = TestGame::new(2);
    let naga = t.battlefield(P0, "Watchful Naga");
    t.enter(P0, "Trial of Solidarity");
    t.resolve_all();
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.attack(&[(naga, Entity::Player(P1))], &[]);
    assert!(t.obj_now(naga).exerted);
    assert!(!t.obj_now(naga).tapped);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn an_object_not_on_the_battlefield_cant_be_exerted() {
    cr!("701.43c");
    supported("Trueheart Twins");
    let mut t = TestGame::new(2);
    let twins = t.battlefield(P0, "Trueheart Twins");
    let card = t.graveyard(P0, "Grizzly Bears");
    exert(&mut t, P0, card);
    t.resolve_all();
    assert!(!t.obj_now(card).exerted);
    assert!(custom_events(&t, EXERTED_EVENT).is_empty());
    // No "whenever you exert a creature" trigger.
    assert_eq!(t.pt(twins), (4, 4));
}

#[test]
fn exert_as_it_attacks_is_an_optional_cost_to_attack_with_a_linked_trigger() {
    cr!("701.43d");
    ruling!(
        "Glorybringer",
        "If a creature has a targeted triggered ability that triggers when you exert it, you can exert it even if there isn’t a legal target for that triggered ability."
    );
    supported("Glorybringer");
    // "You may exert this creature as it attacks. When you do, it deals 4 damage to target
    // non-Dragon creature an opponent controls."
    let mut t = TestGame::new(2);
    let dragon = t.battlefield(P0, "Glorybringer");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.attack(&[(dragon, Entity::Player(P1))], &[]);
    assert!(t.obj_now(dragon).exerted);
    assert!(!t.on_battlefield(giant));
    // Not exerted: no damage.
    let mut t = TestGame::new(2);
    let dragon = t.battlefield(P0, "Glorybringer");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_yes(P0, false);
    t.attack(&[(dragon, Entity::Player(P1))], &[]);
    assert!(!t.obj_now(dragon).exerted);
    assert!(t.on_battlefield(giant));
    // No legal target: it can still be exerted.
    let mut t = TestGame::new(2);
    let dragon = t.battlefield(P0, "Glorybringer");
    t.answer_yes(P0, true);
    t.attack(&[(dragon, Entity::Player(P1))], &[]);
    assert!(t.obj_now(dragon).exerted);
    assert_eq!(t.life(P1), 20 - 4);
    t.advance_to(P0, Step::Upkeep);
    assert!(t.obj_now(dragon).tapped);
}

#[test]
fn whenever_you_exert_a_creature() {
    cr!("701.43a", "701.43d");
    ruling!(
        "Trueheart Twins",
        "Some cards have abilities that trigger whenever you exert any creature. These abilities trigger when you exert that creature or any other creature you control."
    );
    supported("Trueheart Twins");
    // "Whenever you exert a creature, creatures you control get +1/+0 until end of turn."
    let mut t = TestGame::new(2);
    let twins = t.battlefield(P0, "Trueheart Twins");
    let naga = t.battlefield(P0, "Watchful Naga");
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.attack(&[(twins, Entity::Player(P1)), (naga, Entity::Player(P1))], &[]);
    // Both exerted: +2/+0.
    assert_eq!(custom_events(&t, EXERTED_EVENT).len(), 2);
    assert_eq!(t.pt(naga), (2 + 2, 2));
    // Exerting as a cost counts too; an opponent's exertion doesn't.
    let mut t = TestGame::new(2);
    let twins = t.battlefield(P0, "Trueheart Twins");
    let sovereign = t.battlefield(P0, "Pride Sovereign");
    t.battlefield(P1, "Trueheart Twins");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 1);
    t.activate(P0, sovereign, 0, &[]).expect("activate");
    t.resolve_all();
    assert_eq!(t.pt(twins), (4 + 1, 4));
    assert_eq!(t.pt(theirs), (2, 2));
}
