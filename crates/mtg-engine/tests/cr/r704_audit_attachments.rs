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

#[test]
fn an_aura_or_equipment_that_becomes_a_creature_becomes_unattached() {
    cr!("704.5m", "704.5p", "303.4d", "301.5c", "704.5f");
    supported("Mycosynth Lattice");
    supported("March of the Machines");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let strength = t.battlefield(P0, "Holy Strength");
    let splitter = t.battlefield(P0, "Bonesplitter");
    assert!(t.g.attach(strength, Entity::Object(bears)));
    assert!(t.g.attach(splitter, Entity::Object(bears)));
    let land = t.battlefield(P1, "Forest");
    t.battlefield(P1, "Mycosynth Lattice");
    t.battlefield(P1, "March of the Machines");
    // Every permanent is an artifact; every noncreature artifact a creature with power
    // and toughness equal to its mana value. The Aura (mana value 1) is a 1/1 creature:
    // it becomes unattached, then goes to the graveyard. The Equipment becomes unattached
    // and stays a 1/1 creature. The land is a 0/0 and dies.
    t.settle();
    assert!(t.in_graveyard(P0, "Holy Strength"));
    assert!(t.on_battlefield(splitter));
    assert_eq!(t.obj_now(splitter).attached_to, None);
    assert_eq!(t.pt(splitter), (1, 1));
    assert!(!t.on_battlefield(land));
    assert_eq!(t.pt(bears), (2, 2));
}

/// Gives `id` protection from green until end of turn.
fn protect_from_green(t: &mut TestGame, id: ObjectId) {
    let mut prot =
        mtg_engine::keywords::Keyword::new(mtg_engine::keywords::KeywordKind::Protection);
    prot.filter = Some(Filter::Color(Color::Green));
    let mut ctx = Ctx::new(None, P0);
    ctx.targets = vec![vec![Entity::Object(id)]];
    t.g.exec(
        &Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::AddKeyword(prot)],
            duration: Duration::EndOfTurn,
        },
        &mut ctx,
    );
    t.g.recompute();
}

#[test]
fn a_creature_dying_as_its_bestowed_aura_becomes_unattached_dies_with_the_auras_bonus() {
    cr!("704.3", "704.8", "704.5g", "702.103f");
    supported("Boon Satyr");
    supported("Death's Presence");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.battlefield(P0, "Death's Presence");
    t.lands(P0, "Forest", 5);
    let satyr = t.hand(P0, "Boon Satyr");
    t.cast(P0, satyr)
        .method(CastMethod::Keyword(
            mtg_engine::keywords::KeywordKind::Bestow,
        ))
        .target(bears)
        .go();
    t.resolve();
    assert_eq!(t.pt(bears), (6, 4));
    // In one check: the Bears have lethal damage, and the Satyr (green) is attached to a
    // creature with protection from green. Both happen at once; the Bears die as a 6/4.
    let source = t.battlefield(P1, "Hill Giant");
    t.g.deal_damage(source, Entity::Object(bears), 4, false);
    protect_from_green(&mut t, bears);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.settle();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.g.obj(bears).power(), 6, "last known information");
    t.resolve_all();
    assert_eq!(t.counters(giant, counters::PLUS1), 6);
    // The Satyr ceased to be bestowed: it's a creature now.
    let satyr = t.named_on_battlefield("Boon Satyr")[0];
    assert!(t.obj_now(satyr).is_creature());
}
