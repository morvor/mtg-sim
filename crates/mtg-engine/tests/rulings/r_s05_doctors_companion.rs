//! Rulings batch S05 — Doctor's companion (CR 702.124m), a partner ability: two
//! commanders, one with Doctor's companion and the other a Time Lord Doctor.

use crate::r_s01_common::*;
use crate::r_s04_common::*;
use mtg_engine::card::{card, CardDef};
use mtg_engine::deck::DeckProblem;
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::kw::partner::{check_commander_deck, commanders_problem};
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;
use std::sync::Arc;

const DOCTOR: &str = "The Thirteenth Doctor";
const COMPANION: &str = "Barbara Wright";

fn commander_game() -> TestGame {
    TestGame::with_config(
        2,
        GameConfig {
            variant: Variant::Commander,
            ..Default::default()
        },
    )
}

/// Puts `name` into `p`'s command zone as one of their commanders.
fn commander(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = t.command(p, name);
    t.g.objects[id.0 as usize].is_commander = true;
    t.g.players[p.idx()].commander_names.push(name.into());
    id
}

#[test]
fn a_doctor_and_companion_deck_uses_their_combined_color_identity() {
    cr!("702.124c", "702.124m", "903.4");
    ruling!(
        "Barbara Wright",
        "If your Commander deck has two commanders, you can include only cards whose own color identities are also found in your commanders' combined color identities."
    );
    supported(DOCTOR);
    supported(COMPANION);
    let pair: Vec<Arc<CardDef>> = vec![card(DOCTOR), card(COMPANION)];
    assert!(commanders_problem(&[&pair[0], &pair[1]]).is_none());
    // The Thirteenth Doctor is green-blue, Barbara Wright white.
    let mut deck: Vec<Arc<CardDef>> = pair.clone();
    deck.extend((0..95).map(|_| card("Island")));
    deck.extend(["Plains", "Forest", "Island"].map(card));
    assert!(check_commander_deck(&deck, &pair, &[], false).is_empty());
    // A black card is outside it.
    deck.pop();
    deck.push(card("Swamp"));
    assert!(check_commander_deck(&deck, &pair, &[], false).contains(
        &DeckProblem::OutsideColorIdentity {
            name: "Swamp".into()
        }
    ));
}

#[test]
fn a_doctor_and_companion_are_taxed_and_deal_commander_damage_separately() {
    cr!("702.124d", "903.8", "903.10a");
    ruling!(
        "Barbara Wright",
        "Once the game begins, your two commanders are tracked separately. If you cast one, you won't have to pay an additional {2} the first time you cast the other. A player loses the game after having been dealt 21 combat damage from any one of them, not from both of them combined."
    );
    // Commander tax: casting Barbara Wright ({1}{W}) doesn't make the Doctor cost more.
    let mut t = commander_game();
    let barbara = commander(&mut t, P0, COMPANION);
    let doctor = commander(&mut t, P0, DOCTOR);
    t.lands(P0, "Plains", 2);
    t.cast(P0, barbara).go();
    t.resolve_all();
    t.answer_yes(P0, true);
    t.g.destroy(t.g.current(barbara), None);
    t.settle();
    assert_eq!(t.zone(barbara), Zone::Command);
    // The Doctor ({1}{G}{U}) for three mana.
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 1);
    t.cast(P0, doctor).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    t.resolve_all();
    // Barbara again: {1}{W} plus {2}.
    let barbara = t.g.current(barbara);
    t.lands(P0, "Plains", 2);
    assert!(!crate::r_s02_common::can_cast(
        &mut t,
        P0,
        barbara,
        CastMethod::Normal
    ));
    t.lands(P0, "Wastes", 2);
    t.cast(P0, barbara).go();
    assert_eq!(untapped_lands(&t, P0), 0);

    // Commander damage: 20 from each commander isn't 21 from one.
    let mut t = commander_game();
    t.g.players[1].life = 60;
    let barbara = t.battlefield(P0, COMPANION);
    t.g.objects[barbara.0 as usize].is_commander = true;
    let doctor = t.battlefield(P0, DOCTOR);
    t.g.objects[doctor.0 as usize].is_commander = true;
    t.g.players[1].commander_damage.insert(DOCTOR.into(), 20);
    t.g.players[1].commander_damage.insert(COMPANION.into(), 19);
    // Barbara Wright deals 1: 20 from each.
    attack_with(&mut t, &[(barbara, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    t.settle();
    assert!(!t.has_lost(P1));
    // The Doctor deals 2 more: 22 from the Doctor.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    attack_with(&mut t, &[(doctor, Entity::Player(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        mtg_engine::decision::Answer::Blockers(vec![]),
    );
    assert!(t.g.run_until(10_000, |g| g.result.is_some()));
    assert!(t.has_lost(P1));
    assert!(t.life(P1) > 0);
}
