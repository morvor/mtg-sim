//! Foretold split cards and adventurer cards with a foretell cost "equal to its mana cost
//! reduced by {2}" (Dream Devourer's grant in the hand, Ethereal Valkyrie's "It becomes
//! foretold"): the cost is that of the half or Adventure cast, not the split card's
//! combined mana cost (CR 709.4b) or the adventurer card's normal one (CR 715.4) it has in
//! the hand and in exile. Interpretation `foretell-reduced-cost-of-the-half-cast`
//! (rulings-P167), by analogy with the two cards' ruling on modal double-faced cards.

use crate::r_s01_common::*;
use crate::r_s04_common::add_mana;
use crate::r_s05_common::tokens_with_subtype;
use crate::r_s08_common::legal_cast_methods;
use mtg_engine::decision::{Action, Answer, SpecialAction};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const FORETELL: CastMethod = CastMethod::Keyword(KeywordKind::Foretell);

/// What gives the card its foretell cost "equal to its mana cost reduced by {2}".
#[derive(Clone, Copy, Debug)]
enum Grant {
    /// Dream Devourer: "Each nonland card in your hand without foretell has foretell. Its
    /// foretell cost is equal to its mana cost reduced by {2}." P0 foretells the card.
    DreamDevourer,
    /// Ethereal Valkyrie: "Whenever this creature enters or attacks, draw a card, then exile
    /// a card from your hand face down. It becomes foretold. Its foretell cost is its mana
    /// cost reduced by {2}."
    EtherealValkyrie,
}

/// P0's `name` in their hand becomes a foretold card in exile through `grant`; then the
/// game moves on to P0's next main phase, when it may be cast. Returns the card in exile.
fn foretold(t: &mut TestGame, grant: Grant, name: &str) -> ObjectId {
    let card = t.hand(P0, name);
    match grant {
        Grant::DreamDevourer => {
            t.battlefield(P0, "Dream Devourer");
            add_mana(t, P0, ManaType::C, 2);
            t.g.turn.priority = Some(P0);
            t.g.perform_action(P0, Action::Special(SpecialAction::Foretell { card }))
                .expect("foretell");
            t.g.flush_events();
        }
        Grant::EtherealValkyrie => {
            t.answer_choose(P0, &[Entity::Object(card)]);
            t.enter(P0, "Ethereal Valkyrie");
        }
    }
    t.resolve_all();
    let exiled = t.g.current(card);
    assert_eq!(t.obj(exiled).zone, Zone::Exile, "{grant:?}");
    assert!(t.obj(exiled).face_down, "{grant:?}");
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    exiled
}

#[test]
fn a_foretold_split_card_costs_its_cast_halfs_mana_cost_reduced_by_2() {
    cr!("702.143a", "702.143d", "709.3", "709.3b", "601.2f");
    interp!("foretell-reduced-cost-of-the-half-cast");
    supported("Dream Devourer");
    supported("Ethereal Valkyrie");
    supported("Alive // Well");
    // Alive ({3}{G}) for {1}{G}; Well ({W}) for {W}. The card's combined mana cost,
    // {3}{G}{W} (CR 709.4b), reduced by {2} would be {1}{G}{W} for either half.
    for grant in [Grant::DreamDevourer, Grant::EtherealValkyrie] {
        // Alive: {G} alone isn't enough; {1}{G} is.
        let mut t = TestGame::new(2);
        let card = foretold(&mut t, grant, "Alive // Well");
        add_mana(&mut t, P0, ManaType::G, 1);
        assert!(legal_cast_methods(&mut t, P0, card).is_empty(), "{grant:?}");
        add_mana(&mut t, P0, ManaType::C, 1);
        assert_eq!(legal_cast_methods(&mut t, P0, card), vec![FORETELL], "{grant:?}");
        t.answer(P0, DecisionKind::Option, Answer::Index(0));
        let spell = t.cast(P0, card).method(FORETELL).go();
        assert_eq!(t.obj(spell).chars.name, "Alive", "{grant:?}");
        assert_eq!(t.g.player(P0).mana_pool.total(), 0, "{grant:?}: paid all of it");
        t.resolve_all();
        assert_eq!(tokens_with_subtype(&t, P0, "Centaur").len(), 1, "{grant:?}");
        // Well: {W}.
        let mut t = TestGame::new(2);
        let card = foretold(&mut t, grant, "Alive // Well");
        add_mana(&mut t, P0, ManaType::W, 1);
        assert_eq!(legal_cast_methods(&mut t, P0, card), vec![FORETELL], "{grant:?}");
        t.answer(P0, DecisionKind::Option, Answer::Index(1));
        let spell = t.cast(P0, card).method(FORETELL).go();
        assert_eq!(t.obj(spell).chars.name, "Well", "{grant:?}");
        assert_eq!(t.g.player(P0).mana_pool.total(), 0, "{grant:?}: paid all of it");
        let life = t.life(P0);
        t.resolve_all();
        // Dream Devourer or Ethereal Valkyrie: one creature, 2 life.
        assert_eq!(t.life(P0), life + 2, "{grant:?}");
    }
}

#[test]
fn a_foretold_adventurer_card_costs_its_adventures_mana_cost_reduced_by_2() {
    cr!("702.143a", "702.143d", "715.3", "715.3b", "715.3d", "601.2f");
    interp!("foretell-reduced-cost-of-the-half-cast");
    supported("Dream Devourer");
    supported("Ethereal Valkyrie");
    supported("Beanstalk Giant // Fertile Footsteps");
    // Fertile Footsteps ({2}{G}) for {G}; Beanstalk Giant ({6}{G}) for {4}{G}. The card's
    // own mana cost in exile, {6}{G} (CR 715.4), reduced by {2} would be {4}{G} for the
    // Adventure too.
    for grant in [Grant::DreamDevourer, Grant::EtherealValkyrie] {
        let mut t = TestGame::new(2);
        let card = foretold(&mut t, grant, "Beanstalk Giant // Fertile Footsteps");
        add_mana(&mut t, P0, ManaType::G, 1);
        assert_eq!(legal_cast_methods(&mut t, P0, card), vec![FORETELL], "{grant:?}");
        t.answer(P0, DecisionKind::Option, Answer::Index(1));
        let spell = t.cast(P0, card).method(FORETELL).go();
        assert_eq!(t.obj(spell).chars.name, "Fertile Footsteps", "{grant:?}");
        assert_eq!(t.g.player(P0).mana_pool.total(), 0, "{grant:?}: paid all of it");
        t.resolve_all();
        // An Adventure that resolves is exiled, and the creature may be cast from there
        // (CR 715.3d).
        let card = t.g.current(card);
        assert_eq!(t.obj(card).zone, Zone::Exile, "{grant:?}");
        assert!(!t.obj(card).face_down, "{grant:?}");
        // The creature: {4}{G}.
        let mut t = TestGame::new(2);
        let card = foretold(&mut t, grant, "Beanstalk Giant // Fertile Footsteps");
        add_mana(&mut t, P0, ManaType::G, 1);
        add_mana(&mut t, P0, ManaType::C, 4);
        t.answer(P0, DecisionKind::Option, Answer::Index(0));
        let spell = t.cast(P0, card).method(FORETELL).go();
        assert_eq!(t.obj(spell).chars.name, "Beanstalk Giant", "{grant:?}");
        assert_eq!(t.g.player(P0).mana_pool.total(), 0, "{grant:?}: paid all of it");
        // Its power and toughness are the number of lands P0 controls.
        t.lands(P0, "Forest", 1);
        t.resolve_all();
        assert_eq!(t.named_on_battlefield("Beanstalk Giant").len(), 1, "{grant:?}");
    }
}
