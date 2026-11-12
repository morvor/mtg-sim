//! CR 704.5m / 704.5n / 704.5p audit: an attachment's legality is checked against the
//! game as it is now, including who controls what ("enchant creature you control" after a
//! control change) and what a permanent has become (a copy, a creature); and a copy of an
//! Aura that can't be attached to anything doesn't enter at all (CR 303.4f, 608.3e).

use crate::r603_audit_common::*;
use mtg_engine::ability::*;
use mtg_engine::eval::Ctx;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// `p` gains control of `what` until end of turn (Threaten's effect, minus the untap).
fn steal(t: &mut TestGame, p: PlayerId, what: ObjectId) {
    let mut ctx = Ctx::new(None, p);
    ctx.targets = vec![vec![Entity::Object(what)]];
    t.g.exec(
        &Effect::GainControl {
            what: Sel::Target(0),
            who: PlayerRef::You,
            duration: Duration::EndOfTurn,
        },
        &mut ctx,
    );
    t.g.recompute();
    t.g.flush_events();
}

#[test]
fn an_enchant_creature_you_control_aura_goes_when_its_creature_changes_control() {
    cr!("704.5m", "303.4c", "109.5", "301.5d");
    supported("Inferno Fist");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let fist = t.battlefield(P0, "Inferno Fist");
    let splitter = t.battlefield(P0, "Bonesplitter");
    assert!(t.g.attach(fist, Entity::Object(bears)));
    assert!(t.g.attach(splitter, Entity::Object(bears)));
    t.settle();
    assert_eq!(t.pt(bears), (6, 2));
    steal(&mut t, P1, bears);
    assert_eq!(t.obj_now(bears).controller, P1);
    t.settle();
    // "You" is the Aura's controller: P0 no longer controls the creature.
    assert!(t.in_graveyard(P0, "Inferno Fist"));
    // An Equipment may equip a creature another player controls (CR 301.5d).
    assert_eq!(t.obj_now(splitter).attached_to, Some(Entity::Object(bears)));
    assert_eq!(t.pt(bears), (4, 2));
}

#[test]
fn copy_enchantment_copying_an_aura_with_nothing_to_enchant_doesnt_enter() {
    cr!("303.4f", "608.3e", "707.5");
    ruling!(
        "Copy Enchantment",
        "The chosen recipient must be able to legally be enchanted by"
    );
    supported("Setessan Training");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let training = t.battlefield(P0, "Setessan Training");
    assert!(t.g.attach(training, Entity::Object(bears)));
    t.settle();
    // P1 controls no creature: the copy of "Enchant creature you control" can't be
    // attached to anything as it would enter. It stays where it was (the stack) and is
    // put into its owner's graveyard; it never enters, so nothing triggers.
    t.lands(P1, "Island", 3);
    let copy = t.hand(P1, "Copy Enchantment");
    let hand = t.hand_size(P1);
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    t.cast(P1, copy).go();
    t.answer_choose(P1, &[Entity::Object(training)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Copy Enchantment"));
    assert_eq!(t.hand_size(P1), hand - 1, "no card drawn: it never entered");
    // With a creature of its own, it enters attached to it and draws a card.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let training = t.battlefield(P0, "Setessan Training");
    assert!(t.g.attach(training, Entity::Object(bears)));
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.lands(P1, "Island", 3);
    let copy = t.hand(P1, "Copy Enchantment");
    let hand = t.hand_size(P1);
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    t.cast(P1, copy).go();
    t.answer_choose(P1, &[Entity::Object(training)]);
    t.answer_choose(P1, &[Entity::Object(elves)]);
    t.resolve_all();
    let on = t.named_on_battlefield("Setessan Training");
    assert_eq!(on.len(), 2);
    let mine = on
        .into_iter()
        .find(|id| t.obj_now(*id).controller == P1)
        .unwrap();
    assert_eq!(t.obj_now(mine).attached_to, Some(Entity::Object(elves)));
    assert_eq!(t.hand_size(P1), hand - 1 + 1);
}
