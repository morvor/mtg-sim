//! CR 707.2 / 707.4 audit: copies of permanents. A copy of a token isn't a token; the
//! choices made for a permanent survive a copy effect that ends (707.4: the copy effect
//! doesn't change what isn't copiable); and the legend rule puts a permanent another
//! player controls into its owner's graveyard.

use crate::r603_audit_common::*;
use crate::r613_audit_common::choose_creature_type;
use mtg_engine::ability::*;
use mtg_engine::eval::Ctx;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// `obj` becomes a copy of `of` for `duration`.
fn become_copy(t: &mut TestGame, obj: ObjectId, of: ObjectId, duration: Duration) {
    let mut ctx = Ctx::new(None, P0);
    ctx.targets = vec![vec![Entity::Object(obj)], vec![Entity::Object(of)]];
    t.g.exec(
        &Effect::BecomeCopy {
            what: Sel::Target(0),
            of: Sel::Target(1),
            duration,
        },
        &mut ctx,
    );
    t.g.recompute();
    t.g.flush_events();
}

#[test]
fn a_clone_of_a_token_isnt_a_token() {
    cr!("707.2", "111.1", "704.5d");
    ruling!(
        "Clone",
        "If the copied creature is a token, Clone copies the original characteristics of that token as stated by the effect that created the token."
    );
    supported("Raise the Alarm");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 2);
    let alarm = t.hand(P0, "Raise the Alarm");
    t.cast(P0, alarm).go();
    t.resolve();
    let soldier =
        t.g.battlefield
            .iter()
            .copied()
            .find(|id| t.g.obj(*id).kind == ObjKind::Token)
            .unwrap();
    t.answer_choose(P0, &[Entity::Object(soldier)]);
    let clone = t.enter(P0, "Clone");
    let clone = t.g.current(clone);
    assert_eq!(t.g.obj(clone).kind, ObjKind::Card);
    assert_eq!(t.pt(clone), (1, 1));
    assert!(t.obj_now(clone).chars.has_subtype("Soldier"));
    // Returned to its owner's hand, it's the Clone card; it doesn't cease to exist.
    t.g.move_object(
        clone,
        Zone::Hand(P0),
        mtg_engine::events::MoveCause::Effect,
        None,
    );
    t.settle();
    assert!(t.in_hand(P0, "Clone"));
}

#[test]
fn a_permanent_keeps_its_choices_through_a_copy_effect_that_ends() {
    cr!("707.4", "707.2", "611.2a");
    supported("Adaptive Automaton");
    let mut t = TestGame::new(2);
    choose_creature_type(&mut t, P0, "Bear");
    let automaton = t.enter(P0, "Adaptive Automaton");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.settle();
    assert_eq!(t.pt(bears), (3, 3));
    become_copy(&mut t, automaton, giant, Duration::EndOfTurn);
    assert_eq!(t.obj_now(automaton).chars.name.as_str(), "Hill Giant");
    assert_eq!(t.pt(bears), (2, 2));
    // The copy effect ends: it's an Adaptive Automaton that chose Bear again.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(
        t.obj_now(automaton).chars.name.as_str(),
        "Adaptive Automaton"
    );
    assert_eq!(t.pt(bears), (3, 3));
    assert!(t.obj_now(automaton).chars.has_subtype("Bear"));
}

#[test]
fn the_legend_rule_puts_a_stolen_legend_into_its_owners_graveyard() {
    cr!("704.5j", "400.3");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Isamaru, Hound of Konda");
    let theirs = t.battlefield(P1, "Isamaru, Hound of Konda");
    t.settle();
    assert!(t.on_battlefield(mine) && t.on_battlefield(theirs));
    // P1 gains control of P0's Isamaru: P1 chooses which to keep.
    let mut ctx = Ctx::new(None, P1);
    ctx.targets = vec![vec![Entity::Object(mine)]];
    t.g.exec(
        &Effect::GainControl {
            what: Sel::Target(0),
            who: PlayerRef::You,
            duration: Duration::EndOfTurn,
        },
        &mut ctx,
    );
    t.g.recompute();
    t.answer_choose(P1, &[Entity::Object(theirs)]);
    t.settle();
    assert!(t.on_battlefield(theirs));
    assert!(!t.on_battlefield(mine));
    assert!(t.in_graveyard(P0, "Isamaru, Hound of Konda"));
    assert!(!t.in_graveyard(P1, "Isamaru, Hound of Konda"));
}
