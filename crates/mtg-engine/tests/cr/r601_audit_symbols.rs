//! CR 107.4 / 118.13 / 601.2b audit: paying the symbols that can be paid in more than one
//! way, with real cards. Phyrexian symbols (life or mana, CR 107.4f, 119.4, 119.8),
//! colorless symbols (CR 107.4c), monocolored hybrid symbols and the reductions that
//! reach them (CR 107.4e, 118.7a, 601.2b), snow symbols (CR 107.4h), and colored
//! reductions meeting Phyrexian symbols of their color.

use crate::r601_audit_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn a_phyrexian_symbol_cant_be_paid_with_life_when_the_life_total_cant_change() {
    cr!("107.4f", "119.8", "118.3");
    ruling!(
        "Platinum Emperion",
        "You can’t pay a cost that includes the payment of any amount of life other than 0 life."
    );
    supported("Platinum Emperion");
    supported("Gitaxian Probe");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Platinum Emperion");
    let probe = t.hand(P0, "Gitaxian Probe");
    assert!(!can_cast(&mut t, P0, probe, CastMethod::Normal));
    assert!(t.cast(P0, probe).target(P1).try_go().is_err());
    assert!(t.in_hand(P0, "Gitaxian Probe"));
    assert_eq!(t.life(P0), 20);
    // With an Island, {U} pays it.
    t.lands(P0, "Island", 1);
    let probe = t.g.current(probe);
    assert!(can_cast(&mut t, P0, probe, CastMethod::Normal));
    t.cast(P0, probe).target(P1).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn two_life_cant_be_paid_at_one_life_but_can_at_two() {
    cr!("107.4f", "119.4", "118.3");
    ruling!(
        "Gitaxian Probe",
        "If you're at 1 life or less, you can't pay 2 life."
    );
    let mut t = TestGame::new(2);
    t.g.players[P0.idx()].life = 1;
    let probe = t.hand(P0, "Gitaxian Probe");
    assert!(!can_cast(&mut t, P0, probe, CastMethod::Normal));
    assert!(t.cast(P0, probe).target(P1).try_go().is_err());
    assert_eq!(t.life(P0), 1);
    // At 2 life the whole life total may be paid (CR 119.4).
    let mut t = TestGame::new(2);
    t.g.players[P0.idx()].life = 2;
    let probe = t.hand(P0, "Gitaxian Probe");
    assert!(can_cast(&mut t, P0, probe, CastMethod::Normal));
    t.cast(P0, probe).target(P1).go();
    assert_eq!(t.life(P0), 0);
}

#[test]
fn phyrexian_symbols_may_be_split_between_mana_and_life_and_count_toward_mana_value() {
    cr!("107.4f", "202.3g", "601.2b", "601.2h");
    ruling!(
        "Dismember",
        "A Phyrexian mana symbol contributes 1 toward the mana value of a card, even if life is paid for it. Specifically, Dismember's mana value is always 3."
    );
    supported("Dismember");
    // Dismember {1}{B/P}{B/P} with a Swamp and a Wastes: the Wastes pays {1}, the Swamp
    // one {B/P}, and 2 life the other.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Wastes", 1);
    let dismember = t.hand(P0, "Dismember");
    assert!(can_cast(&mut t, P0, dismember, CastMethod::Normal));
    let spell = t.cast(P0, dismember).target(giant).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.g.mana_value_of(spell), 3);
    assert!(t.obj(spell).chars.colors.contains(Color::Black));
    t.resolve();
    assert!(t.in_graveyard(P1, "Hill Giant"));
}

#[test]
fn colorless_symbols_need_colorless_mana_but_generic_mana_takes_any() {
    cr!("107.4b", "107.4c");
    supported("Thought-Knot Seer");
    // Thought-Knot Seer {3}{C}: four Mountains can't pay the {C}; three Mountains and a
    // Wastes can, the Wastes paying the {C}.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    let seer = t.hand(P0, "Thought-Knot Seer");
    assert!(!can_cast(&mut t, P0, seer, CastMethod::Normal));
    assert!(t.cast(P0, seer).try_go().is_err());
    assert_eq!(untapped_lands(&t, P0), 4);
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let wastes = t.lands(P0, "Wastes", 1)[0];
    let seer = t.hand(P0, "Thought-Knot Seer");
    assert!(can_cast(&mut t, P0, seer, CastMethod::Normal));
    t.cast(P0, seer).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    assert!(is_tapped(&t, wastes));
}

#[test]
fn monocolored_hybrid_symbols_are_paid_either_way_and_reduced_through_their_generic_half() {
    cr!("107.4e", "118.7a", "601.2b", "202.3f");
    ruling!(
        "Spectral Procession",
        "you can pay for Spectral Procession by spending {W}{W}{W}, {2}{W}{W}, {4}{W}, or {6}"
    );
    ruling!(
        "Spectral Procession",
        "it applies to a monocolored hybrid spell only if you've chosen a method of paying for it that includes generic mana"
    );
    interp!("hybrid-half-for-a-reduction");
    supported("Spectral Procession");
    // {2/W}{2/W}{2/W}: three Plains pay it, six Mountains pay it, five Mountains don't.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 3);
    let procession = t.hand(P0, "Spectral Procession");
    assert_eq!(t.g.mana_value_of(procession), 6);
    assert!(can_cast(&mut t, P0, procession, CastMethod::Normal));
    t.cast(P0, procession).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    t.resolve();
    assert_eq!(permanents_with_subtype(&t, "Spirit"), 3);
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 5);
    let procession = t.hand(P0, "Spectral Procession");
    assert!(!can_cast(&mut t, P0, procession, CastMethod::Normal));
    t.lands(P0, "Mountain", 1);
    assert!(can_cast(&mut t, P0, procession, CastMethod::Normal));
    t.cast(P0, procession).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    // With Helm of Awakening, {W}{W}{2} - {1} = {W}{W}{1}: two Plains and a Mountain.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Helm of Awakening");
    t.lands(P0, "Plains", 2);
    t.lands(P0, "Mountain", 1);
    let procession = t.hand(P0, "Spectral Procession");
    assert!(can_cast(&mut t, P0, procession, CastMethod::Normal));
    t.cast(P0, procession).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    // Without the Helm, those three lands can't pay it.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 2);
    t.lands(P0, "Mountain", 1);
    let procession = t.hand(P0, "Spectral Procession");
    assert!(!can_cast(&mut t, P0, procession, CastMethod::Normal));
}

#[test]
fn a_snow_symbol_is_paid_only_with_mana_from_a_snow_source_and_isnt_reduced() {
    cr!("107.4h");
    supported("Icehide Golem");
    // Icehide Golem {S}: a Mountain's mana can't pay it, even with Helm of Awakening (a
    // generic reduction doesn't touch {S}); a Snow-Covered Forest's mana can.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Helm of Awakening");
    t.lands(P0, "Mountain", 1);
    let golem = t.hand(P0, "Icehide Golem");
    assert!(!can_cast(&mut t, P0, golem, CastMethod::Normal));
    assert!(t.cast(P0, golem).try_go().is_err());
    assert!(t.in_hand(P0, "Icehide Golem"));
    let snow = t.lands(P0, "Snow-Covered Forest", 1)[0];
    let golem = t.g.current(golem);
    assert!(can_cast(&mut t, P0, golem, CastMethod::Normal));
    t.cast(P0, golem).go();
    assert!(is_tapped(&t, snow));
    assert_eq!(untapped_lands(&t, P0), 1, "the Mountain stays untapped");
}

#[test]
fn a_colored_reduction_reduces_a_phyrexian_symbol_of_its_color() {
    cr!("107.4f", "118.7c", "601.2b", "601.2f");
    interp!("phyrexian-symbol-colored-reduction");
    supported("Morophon, the Boundless");
    supported("Vault Skirge");
    // Morophon naming Imp: Imp spells cost {W}{U}{B}{R}{G} less, colored mana only.
    // Vault Skirge {1}{B/P}: its {B/P} is a black mana symbol, so the reduction removes
    // it and the Skirge costs {1}, with no life paid.
    let mut t = TestGame::new(2);
    let i = subtype_lists()
        .creature
        .iter()
        .position(|x| x == "Imp")
        .expect("a creature type");
    t.answer(P0, DecisionKind::Option, Answer::Index(i));
    t.enter(P0, "Morophon, the Boundless");
    t.lands(P0, "Wastes", 1);
    let skirge = t.hand(P0, "Vault Skirge");
    assert!(can_cast(&mut t, P0, skirge, CastMethod::Normal));
    t.cast(P0, skirge).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    assert_eq!(t.life(P0), 20);
    // Without Morophon, a Wastes alone leaves the {B/P} to be paid with 2 life.
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 1);
    let skirge = t.hand(P0, "Vault Skirge");
    t.cast(P0, skirge).go();
    assert_eq!(t.life(P0), 18);
}
