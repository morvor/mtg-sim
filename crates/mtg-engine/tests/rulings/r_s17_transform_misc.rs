//! Rulings batch S17 — transform: Werewolves (CR 701.27, 603.4), disturb (CR 702.146),
//! a transform ability that already transformed (CR 701.27f), and copying spells with
//! Overloaded Mage-Ring (CR 707.10).

use crate::r_s01_common::*;
use crate::r_s04_common::untapped_lands;
use crate::r_s07_common::chosen_modes;
use crate::r_s11_common::spells_copied;
use crate::r_s17_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, FaceState, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Duskwatch Recruiter ("At the beginning of each upkeep, if no spells were cast last
/// turn, transform this creature.") // Krallenhorde Howler ("At the beginning of each
/// upkeep, if a player cast two or more spells last turn, transform this creature.").
const RECRUITER: &str = "Duskwatch Recruiter // Krallenhorde Howler";
/// Lunarch Veteran ({W}; disturb {1}{W}) // Luminous Phantom.
const VETERAN: &str = "Lunarch Veteran // Luminous Phantom";
/// Invasion of Vryn // Overloaded Mage-Ring ("{1}, {T}, Sacrifice this artifact: Copy
/// target spell you control. You may choose new targets for the copy.").
const VRYN: &str = "Invasion of Vryn // Overloaded Mage-Ring";

/// `p` casts Lightning Bolt at `target` (with a Mountain for it) and it resolves.
fn bolt(t: &mut TestGame, p: PlayerId, target: PlayerId) {
    t.lands(p, "Mountain", 1);
    let b = t.hand(p, "Lightning Bolt");
    t.cast(p, b).target(target).go();
    t.resolve_all();
}

#[test]
fn a_werewolfs_back_face_transforms_only_if_a_single_player_cast_two_spells() {
    cr!("603.4", "701.27a");
    ruling!(
        "Duskwatch Recruiter // Krallenhorde Howler",
        "To trigger the Werewolf's back face's transform ability, a single player must have cast two or more spells during the previous turn."
    );
    supported(RECRUITER);
    // P0 and P1 each cast one spell during P0's turn: at P1's upkeep, the Howler doesn't
    // transform.
    let mut t = TestGame::new(2);
    let howler = enter_transformed(&mut t, P0, RECRUITER);
    assert_eq!(name_of(&t, howler), "Krallenhorde Howler");
    bolt(&mut t, P0, P1);
    bolt(&mut t, P1, P0);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(name_of(&t, howler), "Krallenhorde Howler");
    // P1 casts two spells during their turn: at P0's upkeep, it transforms.
    t.advance_to(P1, Step::PrecombatMain);
    bolt(&mut t, P1, P0);
    bolt(&mut t, P1, P0);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(name_of(&t, howler), "Duskwatch Recruiter");
}

#[test]
fn a_werewolf_looks_at_the_whole_previous_turn_even_if_it_wasnt_on_the_battlefield() {
    cr!("603.4", "701.27a");
    ruling!(
        "Duskwatch Recruiter // Krallenhorde Howler",
        "The abilities that transform a Werewolf back and forth look at the entire previous turn, even if the Werewolf with that ability wasn't on the battlefield for some or all of that turn."
    );
    // P1 casts two spells, then Krallenhorde Howler enters at the end of that turn: it
    // transforms at the next upkeep.
    let mut t = TestGame::new(2);
    t.advance_to(P1, Step::PrecombatMain);
    bolt(&mut t, P1, P0);
    bolt(&mut t, P1, P0);
    t.advance_to(P1, Step::End);
    let howler = enter_transformed(&mut t, P0, RECRUITER);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(name_of(&t, howler), "Duskwatch Recruiter");
    // No spells during P0's turn; Duskwatch Recruiter enters at its end: it transforms at
    // P1's upkeep.
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::End);
    let recruiter = t.battlefield(P0, RECRUITER);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(name_of(&t, recruiter), "Krallenhorde Howler");
}

#[test]
fn disturb_casts_the_card_transformed_from_the_graveyard_for_the_disturb_cost() {
    cr!("702.146a", "702.146b", "601.2b", "118.9");
    ruling!(
        "Lunarch Veteran // Luminous Phantom",
        "\"Disturb [cost]\" means \"You may cast this card transformed from your graveyard by paying [cost] rather than its mana cost.\""
    );
    ruling!(
        "Lunarch Veteran // Luminous Phantom",
        "Disturb is found only on the front faces of some double-faced cards."
    );
    supported(VETERAN);
    // The front face has disturb; the back face doesn't.
    let def = mtg_engine::card::card(VETERAN);
    assert!(def.faces[0].chars.has_keyword(KeywordKind::Disturb));
    assert!(!def.faces[1].chars.has_keyword(KeywordKind::Disturb));
    let disturb = CastMethod::Keyword(KeywordKind::Disturb);
    // Not from the hand.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 2);
    let in_hand = t.hand(P0, VETERAN);
    assert!(t.cast(P0, in_hand).method(disturb.clone()).try_go().is_err());
    // From the graveyard, not for its mana cost {W}...
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 1);
    let vet = t.graveyard(P0, VETERAN);
    assert!(t.cast(P0, vet).try_go().is_err());
    assert!(t.cast(P0, vet).method(disturb.clone()).try_go().is_err());
    // ... but for {1}{W}, transformed.
    t.lands(P0, "Plains", 1);
    let spell = t.cast(P0, vet).method(disturb).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    assert_eq!(t.obj(spell).face, FaceState::Back);
    assert_eq!(t.obj(spell).chars.name, "Luminous Phantom");
    assert!(!t.obj(spell).chars.has_keyword(KeywordKind::Disturb));
    t.resolve_all();
    assert_eq!(name_of(&t, spell), "Luminous Phantom");
    assert_eq!(t.zone(spell), Zone::Battlefield);
    assert!(!t.obj_now(spell).chars.has_keyword(KeywordKind::Disturb));
}

#[test]
fn a_copy_of_a_disturbed_spell_is_a_token_copy_of_the_back_face() {
    cr!("707.10", "707.10g", "111.13", "202.3b");
    ruling!(
        "Lunarch Veteran // Luminous Phantom",
        "If you copy a permanent spell cast this way (perhaps with a card like Double Major), the copy becomes a token that's a copy of the card's back face, even though it isn't itself a double-faced card."
    );
    let mut t = TestGame::new(2);
    let ring = enter_transformed(&mut t, P0, VRYN);
    assert_eq!(name_of(&t, ring), "Overloaded Mage-Ring");
    t.lands(P0, "Plains", 3);
    let vet = t.graveyard(P0, VETERAN);
    let spell = t
        .cast(P0, vet)
        .method(CastMethod::Keyword(KeywordKind::Disturb))
        .go();
    t.activate(P0, ring, 0, &[Entity::Object(spell)]).unwrap();
    t.resolve_all();
    let phantoms = t.named_on_battlefield("Luminous Phantom");
    assert_eq!(phantoms.len(), 2);
    let tok = *phantoms.iter().find(|o| t.obj(**o).is_token()).unwrap();
    let o = t.obj(tok);
    assert_eq!(o.chars.name, "Luminous Phantom");
    assert_eq!(o.face, FaceState::Back);
    assert!(o.chars.has_subtype("Spirit") && o.has_keyword(KeywordKind::Flying));
    assert!(!o.chars.has_keyword(KeywordKind::Disturb));
    // A copy of a back face has mana value 0 (CR 202.3b).
    assert_eq!(t.g.mana_value_of(tok), 0);
    // It isn't a card, but it's a double-faced token (CR 707.10g): it can transform.
    transform(&mut t, tok);
    assert_eq!(name_of(&t, tok), "Lunarch Veteran");
}

#[test]
fn a_permanent_that_transformed_while_its_pay_to_transform_trigger_waited_doesnt_transform_again()
{
    cr!("701.27f");
    ruling!(
        "Ashling, Rekindled // Ashling, Rimebound",
        "In some rare cases, this permanent might transform while its triggered ability that allows you to pay mana to transform it is still on the stack."
    );
    // Ashling, Rekindled: "At the beginning of your first main phase, you may pay {U}. If
    // you do, transform Ashling."
    let mut t = TestGame::new(2);
    let ashling = t.battlefield(P0, "Ashling, Rekindled // Ashling, Rimebound");
    t.lands(P0, "Island", 1);
    t.advance_to(P1, Step::Upkeep);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    let ok = t.g.run_until(10_000, |g| {
        g.turn.active == P0 && g.turn.step == Step::PrecombatMain && !g.stack.is_empty()
    });
    assert!(ok);
    assert_eq!(triggers_on_stack(&t, "you may pay {U}"), 1);
    // It transforms some other way while the ability waits...
    transform(&mut t, ashling);
    assert_eq!(name_of(&t, ashling), "Ashling, Rimebound");
    // ... and even with {U} paid as the ability resolves, it doesn't transform back.
    t.resolve_all();
    assert_eq!(name_of(&t, ashling), "Ashling, Rimebound");
    assert_eq!(face(&t, ashling), FaceState::Back);
    // Without that, paying {U} transforms it.
    let mut t = TestGame::new(2);
    let ashling = t.battlefield(P0, "Ashling, Rekindled // Ashling, Rimebound");
    t.lands(P0, "Island", 1);
    t.advance_to(P1, Step::Upkeep);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    assert_eq!(name_of(&t, ashling), "Ashling, Rimebound");
}

#[test]
fn a_copy_of_a_modal_spell_has_the_same_mode() {
    cr!("707.10", "700.2g");
    ruling!(
        "Invasion of Vryn // Overloaded Mage-Ring",
        "If the spell that’s copied is modal (that is, it includes a choice from a bulleted list of effects), the copy will have the same mode. A different mode can’t be chosen."
    );
    supported(VRYN);
    supported("Boros Charm");
    let mut t = TestGame::new(2);
    let ring = enter_transformed(&mut t, P0, VRYN);
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Plains", 1);
    // Boros Charm: "Boros Charm deals 4 damage to target player or planeswalker."
    let charm = t.hand(P0, "Boros Charm");
    let spell = t.cast(P0, charm).modes(&[0]).target(P1).go();
    let from = t.asked().len();
    t.activate(P0, ring, 0, &[Entity::Object(spell)]).unwrap();
    t.resolve();
    assert_eq!(spells_copied(&t), 1);
    let copy = *t.g.stack.last().unwrap();
    assert_eq!(chosen_modes(&t, copy), vec![0]);
    // No mode was chosen for the copy.
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, mtg_engine::decision::Decision::ChooseModes { .. })));
    t.resolve_all();
    assert_eq!(t.life(P1), 12);
}

#[test]
fn a_copy_of_a_spell_isnt_cast() {
    cr!("707.10", "601.2i", "702.108a");
    ruling!(
        "Invasion of Vryn // Overloaded Mage-Ring",
        "A copy of a spell is created on the stack, so it’s not “cast.” Abilities that trigger when a player casts a spell won’t trigger."
    );
    let mut t = TestGame::new(2);
    let ring = enter_transformed(&mut t, P0, VRYN);
    let swiftspear = t.battlefield(P0, "Monastery Swiftspear");
    t.lands(P0, "Mountain", 2);
    let b = t.hand(P0, "Lightning Bolt");
    let spell = t.cast(P0, b).target(P1).go();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Prowess"), 1);
    t.activate(P0, ring, 0, &[Entity::Object(spell)]).unwrap();
    t.resolve();
    assert_eq!(spells_copied(&t), 1);
    // The copy didn't trigger prowess.
    assert_eq!(triggers_on_stack(&t, "Prowess"), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    assert_eq!(t.pt(swiftspear), (2, 3));
}

#[test]
fn a_copy_has_the_costs_paid_for_the_original_but_none_can_be_paid_for_it() {
    cr!("707.10", "702.33d");
    ruling!(
        "Invasion of Vryn // Overloaded Mage-Ring",
        "You can’t choose to pay any alternative or additional costs for the copy. However, effects based on any alternative or additional costs that were paid for the original spell are copied as though those same costs were paid for the copy."
    );
    supported("Burst Lightning");
    // Burst Lightning, kicked: 4 damage; its copy is kicked too.
    let mut t = TestGame::new(2);
    let ring = enter_transformed(&mut t, P0, VRYN);
    t.lands(P0, "Mountain", 6);
    let b = t.hand(P0, "Burst Lightning");
    let spell = t.cast(P0, b).kicked(true).target(P1).go();
    let from = t.asked().len();
    t.activate(P0, ring, 0, &[Entity::Object(spell)]).unwrap();
    t.resolve();
    // Nobody was asked whether to kick the copy.
    assert!(!t.asked()[from..].iter().any(|(_, d)| matches!(
        d,
        mtg_engine::decision::Decision::OptionalCost { .. }
    )));
    t.resolve_all();
    assert_eq!(t.life(P1), 12);
    // Not kicked: neither is its copy.
    let mut t = TestGame::new(2);
    let ring = enter_transformed(&mut t, P0, VRYN);
    t.lands(P0, "Mountain", 2);
    let b = t.hand(P0, "Burst Lightning");
    let spell = t.cast(P0, b).kicked(false).target(P1).go();
    t.activate(P0, ring, 0, &[Entity::Object(spell)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}
