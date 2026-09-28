//! Rulings batch S18 — ward (CR 702.21), and cards with ward: several ward abilities
//! triggering for one spell, and Dusk Rose Reliquary's "until" exile of a double-faced
//! card.

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use mtg_engine::object::{FaceState, StackKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

/// Ward triggers on the stack.
fn ward_triggers(t: &TestGame) -> usize {
    t.g.stack
        .iter()
        .filter(|id| {
            t.g.obj(**id).stack.as_deref().is_some_and(
                |si| matches!(&si.kind, StackKind::Triggered { ability, .. } if ability.text == "Ward"),
            )
        })
        .count()
}

/// P1 casts Arc Trail (2 damage to one target, 1 to another) at P0's Patchwork Automaton
/// (1/1, ward {2}) and Adrix and Nev, Twincasters (2/2, ward {2}), with `extra` lands
/// beyond Arc Trail's cost, trying to pay every ward cost.
fn arc_trail_at_two_ward_creatures(extra: usize) -> (TestGame, ObjectId, ObjectId) {
    let mut t = TestGame::new(2);
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    let automaton = t.battlefield(P0, "Patchwork Automaton");
    let adrix = t.battlefield(P0, "Adrix and Nev, Twincasters");
    give_mana_for(&mut t, P1, "Arc Trail");
    t.lands(P1, "Wastes", extra);
    let arc = t.hand(P1, "Arc Trail");
    t.cast(P1, arc)
        .targets(&[Entity::Object(automaton)])
        .targets(&[Entity::Object(adrix)])
        .go();
    t.settle();
    assert_eq!(ward_triggers(&t), 2);
    t.answer_yes(P1, true);
    t.answer_yes(P1, true);
    t.resolve_all();
    (t, automaton, adrix)
}

#[test]
fn each_ward_ability_triggers_and_all_must_be_paid() {
    cr!("702.21a");
    ruling!(
        "Adrix and Nev, Twincasters",
        "If a player casts a spell that targets multiple permanents their opponent controls with ward, each of those ward abilities will trigger. If that player doesn't pay for all of them, the spell will be countered."
    );
    supported("Patchwork Automaton");
    supported("Adrix and Nev, Twincasters");
    supported("Arc Trail");
    // P1 can pay one {2} but not both: Arc Trail is countered.
    let (t, automaton, adrix) = arc_trail_at_two_ward_creatures(2);
    assert!(t.in_graveyard(P1, "Arc Trail"));
    assert!(t.on_battlefield(automaton));
    assert!(t.on_battlefield(adrix));
    assert_eq!(t.obj_now(adrix).damage, 0);
    // Paying both, it resolves.
    let (t, automaton, adrix) = arc_trail_at_two_ward_creatures(4);
    assert!(!t.on_battlefield(automaton));
    assert_eq!(t.obj_now(adrix).damage, 1);
}

/// `p`'s Delver of Secrets, transformed into Insectile Aberration.
fn aberration(t: &mut TestGame, p: PlayerId) -> ObjectId {
    let delver = t.battlefield(p, "Delver of Secrets");
    assert!(mtg_engine::dfc::transform(&mut t.g, delver));
    t.g.recompute();
    assert_eq!(t.obj_now(delver).chars.name, "Insectile Aberration");
    delver
}

#[test]
fn an_exiled_double_faced_card_returns_front_face_up() {
    cr!("712.14", "610.3", "400.7");
    ruling!(
        "Dusk Rose Reliquary",
        "If a double-faced card is exiled this way, it will return with its front face up, no matter which face was up when it left the battlefield."
    );
    ruling!(
        "Abuelo, Ancestral Echo",
        "If a double-faced card is exiled this way, it will return with its front face up, no matter which face was up when it left the battlefield."
    );
    supported("Dusk Rose Reliquary");
    supported("Abuelo, Ancestral Echo");
    // Dusk Rose Reliquary exiles P1's Insectile Aberration until the Reliquary leaves.
    let mut t = TestGame::new(2);
    let delver = aberration(&mut t, P1);
    t.answer_targets(P0, &[Entity::Object(delver)]);
    let reliquary = t.enter(P0, "Dusk Rose Reliquary");
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.zone(delver), Zone::Exile);
    destroy(&mut t, reliquary);
    t.resolve_all();
    let back = t.g.current(delver);
    assert!(t.on_battlefield(back));
    assert_eq!(t.obj_now(back).face, FaceState::Front);
    assert_eq!(t.obj_now(back).chars.name, "Delver of Secrets");
    assert_eq!(t.pt(back), (1, 1));
    // Abuelo ("{1}{W}{U}: Exile another target creature or artifact you control. Return
    // it to the battlefield under its owner's control at the beginning of the next end
    // step.") flickers P0's own Aberration.
    let mut t = TestGame::new(2);
    let abuelo = t.battlefield(P0, "Abuelo, Ancestral Echo");
    let delver = aberration(&mut t, P0);
    give_mana_for(&mut t, P0, "Abuelo, Ancestral Echo");
    t.activate(P0, abuelo, 0, &[Entity::Object(delver)])
        .expect("activate");
    t.resolve_all();
    assert_eq!(t.zone(delver), Zone::Exile);
    t.advance_to(P0, mtg_engine::turn::Step::End);
    t.resolve_all();
    let back = t.g.current(delver);
    assert!(t.on_battlefield(back));
    assert_eq!(t.obj_now(back).face, FaceState::Front);
    assert_eq!(t.obj_now(back).chars.name, "Delver of Secrets");
}
