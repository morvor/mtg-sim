//! Item `rulings-residue-misc-1` (pass C) — Harold and Bob, First Numens returning as an
//! Aura (CR 303.4f-g, 603.6c, 613.1d-f; copies, CR 707.2, 111.7), Convergence of Dominion's
//! reduction for abilities of cards in graveyards (CR 602.1, 601.2f), and Manabond putting
//! lands onto the battlefield as it resolves (CR 305.4).

use crate::r_p103_common::{obj, pl};
use crate::r_residue_misc_1_common::*;
use mtg_engine::ability::AbilityKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const HB: &str = "Harold and Bob, First Numens";

/// The number of activated abilities the permanent (followed) has now.
fn activated(t: &mut TestGame, id: ObjectId) -> usize {
    t.g.recompute();
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .count()
}

#[test]
fn harold_and_bob_returns_under_its_controller_or_stays_in_the_graveyard() {
    cr!("603.6c", "303.4f", "303.4g", "110.2");
    ruling!(
        "Harold and Bob, First Numens",
        "If there's nothing for Harold and Bob to legally enchant, it remains in its owner's graveyard"
    );
    ruling!(
        "Harold and Bob, First Numens",
        "If you control Harold and Bob, you'll return it to the battlefield when it dies while it's a creature. It doesn't matter who owns the card. You'll choose which Forest you control it will enchant as it returns to the battlefield."
    );
    supported(HB);
    // No Forest P0 controls (only P1's): it stays in the graveyard.
    let mut t = TestGame::new(2);
    let hb = t.battlefield(P0, HB);
    t.battlefield(P1, "Forest");
    destroy(&mut t, hb);
    t.resolve_all();
    assert!(t.in_graveyard(P0, HB));
    // P1 controls P0's Harold and Bob: it returns under P1's control, on P1's chosen
    // Forest.
    let mut t = TestGame::new(2);
    let hb = t.battlefield(P0, HB);
    t.battlefield(P0, "Forest");
    t.battlefield(P1, "Forest");
    let f2 = t.battlefield(P1, "Forest");
    give_control(&mut t, hb, P1);
    t.answer_choose(P1, &[obj(f2)]);
    destroy(&mut t, hb);
    t.resolve_all();
    let aura = t.g.current(hb);
    assert!(t.on_battlefield(aura));
    assert_eq!(t.obj_now(aura).controller, P1);
    assert_eq!(t.obj_now(aura).owner, P0);
    assert_eq!(attached_to(&t, aura), Some(obj(f2)));
}

#[test]
fn harold_and_bob_copies_a_token_stays_a_clone_returns_as_a_bare_aura() {
    cr!("707.2", "111.7", "613.1d", "613.1f", "205.1b");
    ruling!(
        "Harold and Bob, First Numens",
        "A token that's a copy of Harold and Bob won't return to the battlefield from its owner's graveyard."
    );
    ruling!(
        "Harold and Bob, First Numens",
        "If a nontoken permanent that's a copy of Harold and Bob dies while it's a creature, it will return to the battlefield as an Aura with only the abilities granted by the effect that returned it to the battlefield. It will also retain its name, colors, and supertypes, but it will be an Aura enchantment with no other types or subtypes."
    );
    supported("Clone");
    // A token copy (P1's, away from the legend rule): nothing returns.
    let mut t = TestGame::new(2);
    let hb = t.battlefield(P0, HB);
    t.battlefield(P1, "Forest");
    let copy = crate::r_s17_common::token_copy(&mut t, P1, hb)[0];
    destroy(&mut t, copy);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield(HB).len(), 1);
    assert_eq!(
        t.g.permanents()
            .filter(|o| o.is(CardType::Enchantment))
            .count(),
        0
    );
    // P1's Clone copying it returns as Clone's own card, a blue Aura enchantment with no
    // other types or subtypes and only the two granted abilities (Clone's own is gone).
    let mut t = TestGame::new(2);
    let hb = t.battlefield(P0, HB);
    let forest = t.battlefield(P1, "Forest");
    t.answer_choose(P1, &[obj(hb)]);
    let clone = t.enter(P1, "Clone");
    t.resolve_all();
    assert_eq!(name_now(&t, clone), HB);
    let before = activated(&mut t, forest);
    t.answer_choose(P1, &[]);
    t.answer_choose(P1, &[obj(forest)]);
    destroy(&mut t, clone);
    t.resolve_all();
    let back = t.g.current(clone);
    assert!(t.on_battlefield(back), "{}", t.dump_log());
    let o = t.obj_now(back);
    assert_eq!(o.chars.name.as_str(), "Clone");
    assert_eq!(o.chars.colors, ColorSet::single(Color::Blue));
    assert!(!o.chars.supertypes.contains(Supertype::Legendary));
    assert!(o.is(CardType::Enchantment) && !o.is(CardType::Creature));
    assert_eq!(o.chars.card_types.iter().count(), 1);
    assert_eq!(
        o.chars.subtypes.len(),
        1,
        "only Aura: {:?}",
        o.chars.subtypes
    );
    assert!(o.chars.has_subtype("Aura"));
    // Enchant Forest you control and the granted static; nothing else.
    assert_eq!(o.chars.abilities.len(), 2, "{:?}", o.chars.abilities);
    assert_eq!(attached_to(&t, back), Some(obj(forest)));
    assert_eq!(activated(&mut t, forest), before + 1);
    // The original returns legendary and green, an Aura with no Treefolk or Mutant type.
    let mut t = TestGame::new(2);
    let hb = t.battlefield(P0, HB);
    t.battlefield(P0, "Forest");
    destroy(&mut t, hb);
    t.resolve_all();
    let back = t.g.current(hb);
    let o = t.obj_now(back);
    assert!(o.chars.supertypes.contains(Supertype::Legendary));
    assert_eq!(o.chars.colors, ColorSet::single(Color::Green));
    assert!(!o.chars.has_subtype("Treefolk") && !o.chars.has_subtype("Mutant"));
}

#[test]
fn convergence_of_dominion_doesnt_let_graveyard_abilities_be_activated() {
    cr!("602.1", "602.2", "601.2f");
    ruling!(
        "Convergence of Dominion",
        "Convergence of Dominion doesn't allow you to activate abilities of cards in graveyards unless you could already activate those abilities."
    );
    supported("Convergence of Dominion");
    let mut t = TestGame::new(2);
    crate::r_p114_common::commander_on_battlefield(&mut t, P0, "Grizzly Bears");
    t.battlefield(P0, "Convergence of Dominion");
    let pyro = t.graveyard(P0, "Prodigal Pyromancer");
    let skeleton = t.graveyard(P0, "Reassembling Skeleton");
    t.advance_to(P0, Step::PrecombatMain);
    t.lands(P0, "Swamp", 1);
    // The Skeleton's {1}{B} costs {B}; the Pyromancer's {T} ability still can't be
    // activated from the graveyard.
    assert!(can_activate(&mut t, P0, skeleton));
    assert!(!can_activate(&mut t, P0, pyro));
}

#[test]
fn manabond_puts_the_lands_onto_the_battlefield_as_it_resolves() {
    cr!("305.4", "608.2c");
    ruling!(
        "Manabond",
        "The lands are put onto the battlefield during the resolution of the ability."
    );
    supported("Manabond");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Manabond");
    let forest = t.hand(P0, "Forest");
    t.hand(P0, "Grizzly Bears");
    t.advance_to(P0, Step::End);
    t.settle();
    t.answer_yes(P0, true);
    assert_eq!(t.zone(forest), mtg_engine::object::Zone::Hand(P0));
    t.resolve();
    // Put onto the battlefield (not played), and the rest of the hand discarded.
    assert!(t.on_battlefield(forest));
    assert_eq!(t.g.players[0].lands_played_this_turn, 0);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.hand_size(P0), 0);
    let _ = pl(P1);
}
