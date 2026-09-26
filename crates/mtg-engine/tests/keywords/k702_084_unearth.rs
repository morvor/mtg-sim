//! CR 702.84 Unearth.

use crate::common_k702_011_017::{assert_supported, attack_with};
use crate::common_k702_018_026::declare_blocks;
use crate::common_k702_027_037::activate_named;
use crate::common_k702_052_066::{destroy, run_effect};
use mtg_engine::ability::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{StackKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn unearth(t: &mut TestGame, p: PlayerId, card: ObjectId) -> Result<(), casting::Illegal> {
    activate_named(t, p, card, "Unearth", 0).map(|_| ())
}

/// Unearths Dregscape Zombie ({B}) from `p`'s graveyard and returns the permanent.
fn unearthed_zombie(t: &mut TestGame, p: PlayerId) -> ObjectId {
    let zombie = t.graveyard(p, "Dregscape Zombie");
    t.lands(p, "Swamp", 1);
    unearth(t, p, zombie).expect("unearth");
    t.resolve();
    let now = t.g.current(zombie);
    assert_eq!(t.g.obj(now).zone, Zone::Battlefield);
    now
}

#[test]
fn unearth_returns_the_card_with_haste_and_exiles_it_at_the_next_end_step() {
    cr!("702.84", "702.84a");
    assert_supported("Hellspark Elemental");
    let mut t = TestGame::new(2);
    let elemental = t.graveyard(P0, "Hellspark Elemental");
    t.lands(P0, "Mountain", 2);
    unearth(&mut t, P0, elemental).expect("unearth");
    // It's an activated ability on the stack; the card is still in the graveyard.
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.zone(elemental), Zone::Graveyard(P0));
    t.resolve();
    let now = t.g.current(elemental);
    assert!(t.on_battlefield(now));
    assert!(t.g.obj(now).summoning_sick);
    assert!(t.g.obj(now).has_keyword(KeywordKind::Haste));
    // It can attack this turn.
    attack_with(&mut t, &[(now, Entity::Player(P1))]);
    declare_blocks(&mut t, P1, &[]);
    t.advance_to(P0, Step::End);
    assert_eq!(t.life(P1), 17);
    // At the beginning of the end step it's exiled (its own sacrifice trigger also
    // triggers; whichever resolves first, it ends up exiled).
    t.resolve_all();
    assert!(t.in_exile("Hellspark Elemental"));
    assert!(!t.in_graveyard(P0, "Hellspark Elemental"));
}

#[test]
fn unearth_can_be_activated_only_from_the_graveyard_as_a_sorcery() {
    cr!("702.84a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    let in_hand = t.hand(P0, "Dregscape Zombie");
    assert!(unearth(&mut t, P0, in_hand).is_err());
    let on_bf = t.battlefield(P0, "Dregscape Zombie");
    assert!(unearth(&mut t, P0, on_bf).is_err());
    let in_gy = t.graveyard(P0, "Dregscape Zombie");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(unearth(&mut t, P0, in_gy).is_err());
    t.set_step(P1, Step::PrecombatMain);
    assert!(unearth(&mut t, P0, in_gy).is_err());
    t.set_step(P0, Step::PostcombatMain);
    assert!(unearth(&mut t, P0, in_gy).is_ok());
}

#[test]
fn an_unearthed_creature_that_would_leave_the_battlefield_is_exiled_instead() {
    cr!("702.84a", "614.1a");
    ruling!(
        "Dregscape Zombie",
        "If a creature returned to the battlefield with unearth would leave the battlefield for any reason, it’s exiled instead"
    );
    let mut t = TestGame::new(2);
    let zombie = unearthed_zombie(&mut t, P0);
    destroy(&mut t, zombie);
    t.settle();
    assert!(t.in_exile("Dregscape Zombie"));
    assert!(!t.in_graveyard(P0, "Dregscape Zombie"));
    // Bounced: exiled too.
    let mut t = TestGame::new(2);
    let zombie = unearthed_zombie(&mut t, P0);
    run_effect(
        &mut t,
        None,
        P1,
        Effect::Move {
            what: Sel::Target(0),
            to: Destination::zone(ZoneKind::Hand),
        },
        &[Entity::Object(zombie)],
    );
    assert!(t.in_exile("Dregscape Zombie"));
    assert!(!t.in_hand(P0, "Dregscape Zombie"));
}

#[test]
fn unearths_effects_apply_even_if_the_creature_loses_all_abilities() {
    cr!("702.84a", "603.7c");
    ruling!(
        "Dregscape Zombie",
        "If that creature loses all its abilities, it will still be exiled at the beginning of the end step, and if it would leave the battlefield, it is still exiled instead."
    );
    let mut t = TestGame::new(2);
    let zombie = unearthed_zombie(&mut t, P0);
    run_effect(
        &mut t,
        None,
        P1,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::RemoveAllAbilities],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(zombie)],
    );
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.in_exile("Dregscape Zombie"));
}

#[test]
fn if_the_delayed_trigger_is_countered_the_replacement_still_applies() {
    cr!("702.84a");
    ruling!(
        "Dregscape Zombie",
        "If the ability is countered, the creature will stay on the battlefield and the delayed trigger won’t trigger again. However, the replacement effect will still exile the creature when it eventually leaves the battlefield."
    );
    let mut t = TestGame::new(2);
    let zombie = unearthed_zombie(&mut t, P0);
    t.advance_to(P0, Step::End);
    t.settle();
    let trigger = *t.g.stack.last().expect("the delayed trigger");
    assert!(matches!(
        t.g.obj(trigger).stack.as_ref().unwrap().kind,
        StackKind::Triggered { .. }
    ));
    assert!(t.g.counter(trigger, None));
    t.resolve_all();
    assert!(t.on_battlefield(zombie));
    // It doesn't trigger again at the next end step.
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert!(t.on_battlefield(zombie));
    destroy(&mut t, zombie);
    t.settle();
    assert!(t.in_exile("Dregscape Zombie"));
}

#[test]
fn unearth_does_nothing_if_the_card_left_the_graveyard() {
    cr!("702.84a");
    ruling!(
        "Dregscape Zombie",
        "If you activate a card’s unearth ability but that card is removed from your graveyard before the ability resolves, that unearth ability will resolve and do nothing."
    );
    let mut t = TestGame::new(2);
    let zombie = t.graveyard(P0, "Dregscape Zombie");
    t.lands(P0, "Swamp", 1);
    unearth(&mut t, P0, zombie).unwrap();
    run_effect(
        &mut t,
        None,
        P1,
        Effect::Exile {
            what: Sel::Target(0),
            face_down: false,
            link: false,
        },
        &[Entity::Object(zombie)],
    );
    t.resolve_all();
    assert!(t.named_on_battlefield("Dregscape Zombie").is_empty());
    assert!(t.in_exile("Dregscape Zombie"));
}

#[test]
fn activating_unearth_isnt_casting_the_card() {
    cr!("702.84a", "602.2");
    ruling!(
        "Dregscape Zombie",
        "Activating a creature card’s unearth ability isn’t the same as casting the creature card. The unearth ability is put on the stack, but the creature card is not."
    );
    let mut t = TestGame::new(2);
    let zombie = t.graveyard(P0, "Dregscape Zombie");
    t.lands(P0, "Swamp", 1);
    unearth(&mut t, P0, zombie).unwrap();
    let top = *t.g.stack.last().unwrap();
    assert!(matches!(
        t.g.obj(top).stack.as_ref().unwrap().kind,
        StackKind::Activated { .. }
    ));
    assert!(!t.g.obj(top).is_spell());
    t.resolve();
    let now = t.g.current(zombie);
    assert!(t.on_battlefield(now));
    assert!(t.g.obj(now).cast.as_ref().is_none_or(|c| !c.was_cast));
}

#[test]
fn a_permanent_that_left_and_returned_is_a_new_object_unaffected_by_unearth() {
    cr!("702.84a", "400.7");
    ruling!(
        "Dregscape Zombie",
        "the creature card will return to the battlefield as a new object with no relation to its previous existence. The unearth effect will no longer apply to it."
    );
    let mut t = TestGame::new(2);
    let zombie = unearthed_zombie(&mut t, P0);
    // Exiled (it succeeds) and returned to the battlefield.
    run_effect(
        &mut t,
        None,
        P1,
        Effect::Exile {
            what: Sel::Target(0),
            face_down: false,
            link: false,
        },
        &[Entity::Object(zombie)],
    );
    let exiled = t.g.current(zombie);
    assert_eq!(t.g.obj(exiled).zone, Zone::Exile);
    run_effect(
        &mut t,
        None,
        P0,
        Effect::Move {
            what: Sel::Target(0),
            to: Destination::battlefield(),
        },
        &[Entity::Object(exiled)],
    );
    let back = t.g.current(zombie);
    assert!(t.on_battlefield(back));
    assert!(!t.g.obj(back).has_keyword(KeywordKind::Haste));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.on_battlefield(back));
    destroy(&mut t, back);
    t.settle();
    assert!(t.in_graveyard(P0, "Dregscape Zombie"));
}
