//! Rulings batch S08 — flying: the omen cards (CR 720) of Tarkir: Dragonstorm, flying
//! Dragons with an Omen spell inset. Casting as an Omen isn't an alternative cost; the
//! spell uses only the Omen's characteristics; only a resolving Omen is shuffled into its
//! owner's library.

use crate::r_s01_common::*;
use crate::r_s02_common::*;
use crate::r_s04_common::*;
use crate::r_s05_common::*;
use crate::r_s07_common::*;
use crate::r_s08_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const OMEN: CastMethod = CastMethod::Half(1);
const SAGU: &str = "Sagu Wildling // Roost Seek";
const TWINMAW: &str = "Twinmaw Stormbrood // Charring Bite";
const MARANG: &str = "Marang River Regent // Coil and Catch";

#[test]
fn an_omen_can_be_cast_without_paying_its_mana_cost() {
    cr!("720.3", "701.57a", "118.9");
    ruling!(
        "Sagu Wildling // Roost Seek",
        "Casting a card as an Omen isn’t casting it for an alternative cost. Effects that allow you to cast a spell for an alternative cost or without paying its mana cost may allow you to apply those to the Omen."
    );
    supported(SAGU);
    supported("Trumpeting Carnosaur");
    // Trumpeting Carnosaur: "When this creature enters, discover 5." It finds Sagu
    // Wildling ({4}{G} creature // Roost Seek, {G} sorcery — Omen: "Search your library
    // for a basic land card, reveal it, put it into your hand, then shuffle."), and P0
    // casts it as an Omen without paying its mana cost.
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &[SAGU, "Forest"]);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    enter(&mut t, P0, "Trumpeting Carnosaur");
    t.resolve();
    // Roost Seek is on the stack, cast without mana.
    assert_eq!(t.stack_len(), 1);
    let spell = top_of_stack(&t);
    assert_eq!(t.obj(spell).chars.name, "Roost Seek");
    assert_eq!(
        t.obj(spell).stack.as_ref().unwrap().cast.method,
        CastMethod::Free
    );
    t.resolve_all();
    assert!(t.in_hand(P0, "Forest"));
    assert!(t.named_on_battlefield("Sagu Wildling").is_empty());
    assert_eq!(t.g.find_in_zone(Zone::Library(P0), "Sagu Wildling").len(), 1);
}

/// P0 casts Charring Bite ({1}{R} sorcery — Omen: "deals 5 damage to target creature
/// without flying.") targeting P1's Grizzly Bears. Returns the spell and the Bears.
fn cast_charring_bite(t: &mut TestGame) -> (ObjectId, ObjectId) {
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    let card = t.hand(P0, TWINMAW);
    let spell = t.cast(P0, card).method(OMEN).target(bears).go();
    (spell, bears)
}

#[test]
fn an_omen_spell_with_only_illegal_targets_goes_to_the_graveyard() {
    cr!("720.3d", "608.2b");
    ruling!(
        "Twinmaw Stormbrood // Charring Bite",
        "If an Omen spell has one or more targets and all of those targets are illegal when the spell tries to resolve, it won’t resolve. None of its effects will happen, and it will be put into its owner’s graveyard. It won’t be shuffled into its owner’s library."
    );
    supported(TWINMAW);
    let mut t = TestGame::new(2);
    let lib = t.library_size(P0);
    let (spell, bears) = cast_charring_bite(&mut t);
    // The Bears get flying: no longer a legal target.
    t.lands(P1, "Plains", 2);
    let wing = t.hand(P1, "Wing It");
    t.cast(P1, wing).target(bears).go();
    t.resolve();
    assert!(!resolved(&t, spell));
    t.resolve_all();
    assert!(!resolved(&t, spell));
    assert_eq!(t.obj_now(bears).damage, 0);
    assert!(t.in_graveyard(P0, "Twinmaw Stormbrood"));
    assert_eq!(t.library_size(P0), lib);
}

#[test]
fn an_omen_spell_has_only_the_omens_characteristics() {
    cr!("720.3b", "720.4", "202.3");
    ruling!(
        "Twinmaw Stormbrood // Charring Bite",
        "When casting a spell as an Omen, use the alternative characteristics and ignore all of the card’s normal characteristics. The spell’s color, mana cost, mana value, and so on are determined by only those alternative characteristics. If the spell leaves the stack, it immediately resumes using its normal characteristics."
    );
    let mut t = TestGame::new(2);
    let (spell, _) = cast_charring_bite(&mut t);
    // On the stack: a red sorcery with mana cost {1}{R} (Twinmaw Stormbrood is a white
    // {5}{W} Dragon creature).
    let o = t.obj(spell);
    assert_eq!(o.chars.name, "Charring Bite");
    assert_eq!(o.chars.colors, ColorSet::single(Color::Red));
    assert!(o.is(CardType::Sorcery) && !o.is(CardType::Creature));
    assert_eq!(format!("{}", o.chars.mana_cost.clone().unwrap()), "{1}{R}");
    assert_eq!(mana_value(&t, spell), 2);
    // Countered, it's the white creature card again in the graveyard.
    t.lands(P1, "Island", 2);
    let cs = t.hand(P1, "Counterspell");
    t.cast(P1, cs).target(spell).go();
    t.resolve_all();
    let card = t.g.find_in_zone(Zone::Graveyard(P0), "Twinmaw Stormbrood")[0];
    let c = &t.obj(card).chars;
    assert_eq!(c.colors, ColorSet::single(Color::White));
    assert!(c.is_creature() && !c.is(CardType::Sorcery));
    assert_eq!(mana_value(&t, card), 6);
}

#[test]
fn whether_an_omen_can_be_cast_depends_only_on_the_omens_characteristics() {
    cr!("720.3a", "601.3", "304.1");
    ruling!(
        "Sagu Wildling // Roost Seek",
        "If you cast an omen card as an Omen, use only its alternative characteristics to determine whether it’s legal to cast that spell."
    );
    supported("Steel Golem");
    supported(MARANG);
    // Steel Golem: "You can't cast creature spells." Sagu Wildling can't be cast, but
    // Roost Seek (a sorcery) can.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Steel Golem");
    t.lands(P0, "Forest", 5);
    let sagu = t.hand(P0, SAGU);
    assert!(can_cast(&mut t, P0, sagu, OMEN));
    assert!(!can_cast(&mut t, P0, sagu, CastMethod::Normal));
    // Marang River Regent ({4}{U}{U} creature // Coil and Catch, {3}{U} instant — Omen)
    // in P1's turn: only the instant can be cast.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P0, "Island", 6);
    let marang = t.hand(P0, MARANG);
    assert!(can_cast(&mut t, P0, marang, OMEN));
    assert!(!can_cast(&mut t, P0, marang, CastMethod::Normal));
}

#[test]
fn a_countered_or_bounced_omen_isnt_shuffled_into_the_library() {
    cr!("720.3d", "701.6a");
    ruling!(
        "Marang River Regent // Coil and Catch",
        "If an Omen spell is countered or an effect causes it to otherwise leave the stack, it won’t be shuffled into its owner’s library."
    );
    for bounce in [false, true] {
        let mut t = TestGame::new(2);
        let lib = t.library_size(P0);
        t.lands(P0, "Island", 4);
        let marang = t.hand(P0, MARANG);
        let spell = t.cast(P0, marang).method(OMEN).go();
        t.lands(P1, "Island", 3);
        if bounce {
            let un = t.hand(P1, "Unsubstantiate");
            t.cast(P1, un).target(spell).go();
        } else {
            let cs = t.hand(P1, "Counterspell");
            t.cast(P1, cs).target(spell).go();
        }
        t.resolve_all();
        assert!(!resolved(&t, spell));
        assert_eq!(t.library_size(P0), lib);
        if bounce {
            assert!(t.in_hand(P0, "Marang River Regent"));
        } else {
            assert!(t.in_graveyard(P0, "Marang River Regent"));
        }
    }
}
