//! Rulings batch P114 — partner and companion commanders that copy spells and abilities:
//! Bill Potts, The Twelfth Doctor (demonstrate), The Sixth Doctor, Rowan and Will
//! Kenrith's emblems, Adric (CR 707.10, 607.2).

use crate::r_p114_common::*;
use crate::r_s01_common::asked_since;
use crate::r_s07_common::chosen_modes;
use crate::r_s25_common::{abilities_from, change_copy_targets, lands_for_cost, spell_copies};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::{StackKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

// --- Bill Potts -------------------------------------------------------------------------------

#[test]
fn bill_potts_copies_a_spell_that_could_have_targeted_more() {
    cr!("707.10", "601.2c", "707.10c");
    ruling!(
        "Bill Potts",
        "The first ability of Bill Potts will trigger as long as it is the only target of that spell or ability, even if that spell or ability could target multiple permanents."
    );
    ruling!(
        "Bill Potts",
        "The copy made by Bill's triggered ability is created on the stack, so it's not \"cast\" or \"activated.\""
    );
    supported("Bill Potts");
    let mut t = TestGame::new(2);
    let bill = t.battlefield(P0, "Bill Potts");
    // Young Pyromancer: "Whenever you cast an instant or sorcery spell, create a 1/1 red
    // Elemental creature token."
    t.battlefield(P0, "Young Pyromancer");
    // Blessings of Nature: "Distribute four +1/+1 counters among any number of target
    // creatures." All four on Bill.
    t.answer_targets(P0, &[ent(bill)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![4]));
    lands_for_cost(&mut t, P0, "Blessings of Nature");
    let card = t.hand(P0, "Blessings of Nature");
    t.cast_with(P0, card, &[]).unwrap();
    t.settle();
    assert_eq!(abilities_from(&t, bill).len(), 1);
    t.resolve_all();
    assert_eq!(t.counters(bill, counters::PLUS1), 8);
    assert_eq!(
        tokens_named(&t, P0, "Elemental").len(),
        1,
        "only one spell cast"
    );
}

#[test]
fn bill_potts_copy_has_the_same_x() {
    cr!("707.10", "107.3c");
    ruling!(
        "Bill Potts",
        "If the spell or ability that's copied has an X whose value was determined as it was cast, the copy will have the same value of X."
    );
    // Untamed Might ({X}{G} instant): "Target creature gets +X/+X until end of turn." X=2.
    supported("Untamed Might");
    let mut t = TestGame::new(2);
    let bill = t.battlefield(P0, "Bill Potts");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 2);
    let card = t.hand(P0, "Untamed Might");
    t.cast(P0, card).target(bill).x(2).go();
    t.settle();
    t.resolve();
    let copy = spell_copies(&t)[0];
    assert_eq!(crate::r_s25_common::x_of(&t, copy), Some(2));
    t.resolve_all();
    assert_eq!(t.pt(bill), (6, 8));
}

// --- The Twelfth Doctor: "The first spell you cast from anywhere other than your hand each
// turn has demonstrate." ------------------------------------------------------------------

/// P0 controls The Twelfth Doctor and Future Sight ("You may play lands and cast spells
/// from the top of your library") with the real card `name` on top of the library, and
/// lands for it.
fn twelfth_doctor_game(name: &str) -> (TestGame, ObjectId) {
    supported("The Twelfth Doctor");
    supported("Future Sight");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Twelfth Doctor");
    t.battlefield(P0, "Future Sight");
    let card = t.library_top(P0, name);
    lands_for_cost(&mut t, P0, name);
    (t, card)
}

#[test]
fn twelfth_doctor_demonstrate_copies_see_kicker() {
    cr!("702.144a", "707.10", "702.33d");
    ruling!(
        "The Twelfth Doctor",
        "If any additional or alternative costs were paid to cast the spell with demonstrate, the copies will see those choices as well. For example, if the spell has a kicker ability and was kicked, the copies will also be kicked."
    );
    // Burst Lightning ({R}, kicker {4}): "Burst Lightning deals 2 damage to any target. If
    // this spell was kicked, it deals 4 damage instead."
    let (mut t, card) = twelfth_doctor_game("Burst Lightning");
    t.lands(P0, "Wastes", 4);
    // Gorm the Great (2/7) and Hill Giant (3/3).
    let gorm = t.battlefield(P1, "Gorm the Great");
    let a = t.battlefield(P1, "Hill Giant");
    t.answer_yes(P0, true); // demonstrate
    change_copy_targets(&mut t, P0, &[Some(ent(a))]);
    t.cast(P0, card).target(gorm).kicked(true).go();
    t.settle();
    t.resolve_all();
    // P0's copy (aimed at the Hill Giant) dealt 4; the original and the opponent's copy
    // (aimed at Gorm, kept) 4 each.
    assert!(!t.on_battlefield(a), "4 damage to the Hill Giant");
    assert!(!t.on_battlefield(gorm), "8 damage to Gorm");
}

#[test]
fn twelfth_doctor_demonstrate_copies_have_the_same_modes() {
    cr!("702.144a", "707.10", "700.2");
    ruling!(
        "The Twelfth Doctor",
        "If the spell with demonstrate had any modes chosen for it as it was cast, the copies will have the same modes chosen for it."
    );
    // Kolaghan's Command (instant, choose two): "Target player discards a card" and "deals
    // 2 damage to any target".
    supported("Kolaghan's Command");
    let (mut t, card) = twelfth_doctor_game("Kolaghan's Command");
    t.answer_yes(P0, true); // demonstrate
    let spell = t
        .cast(P0, card)
        .modes(&[1, 3])
        .target(Entity::Player(P1))
        .target(Entity::Player(P1))
        .go();
    let from = t.asked().len();
    t.settle();
    // The demonstrate trigger resolves.
    t.resolve();
    let copies = spell_copies(&t);
    assert_eq!(copies.len(), 2, "P0's copy and the opponent's");
    for c in copies {
        assert_eq!(chosen_modes(&t, c), chosen_modes(&t, spell));
    }
    assert!(!asked_since(&t, from)
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseModes { .. })));
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
}

// --- The Sixth Doctor ---------------------------------------------------------------------

#[test]
fn the_sixth_doctor_copy_keeps_the_targets() {
    cr!("707.10", "707.10c", "303.4a");
    ruling!(
        "The Sixth Doctor",
        "Usually, a historic spell won't have any targets. However, if it does have one or more targets, those targets will remain the same for the copy. You won't get to choose new targets."
    );
    supported("The Sixth Doctor");
    supported("On Serra's Wings");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Sixth Doctor");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    // On Serra's Wings: a legendary Aura.
    lands_for_cost(&mut t, P0, "On Serra's Wings");
    let card = t.hand(P0, "On Serra's Wings");
    t.cast(P0, card).target(bears).go();
    let from = t.asked().len();
    t.settle();
    t.resolve_all();
    assert!(!asked_since(&t, from)
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseTargets { .. })));
    let auras: Vec<_> = t
        .named_on_battlefield("On Serra's Wings")
        .into_iter()
        .filter(|a| t.obj(*a).attached_to == Some(ent(bears)))
        .collect();
    assert_eq!(auras.len(), 2, "both enchant the Bears");
    assert_eq!(t.pt(bears), (4, 4));
}

// --- Rowan's and Will's emblems -----------------------------------------------------------

#[test]
fn rowan_and_will_emblems_copy_any_spell_or_nonmana_ability() {
    cr!("114.4", "605.1a", "707.10");
    ruling!(
        "Rowan Kenrith",
        "The ability of Will's emblem can copy any instant or sorcery spell, not just one with targets. Similarly, the ability of Rowan's emblem can copy any activated ability that isn't a mana ability. A mana ability is an ability that produces mana, not an ability that costs mana."
    );
    supported("Rowan Kenrith");
    supported("Will Kenrith");
    supported("Mind Stone");
    for name in ["Rowan Kenrith", "Will Kenrith"] {
        let mut t = TestGame::new(2);
        let pw = t.battlefield(P0, name);
        let have = t.counters(pw, counters::LOYALTY);
        t.g.add_counters(ent(pw), counters::LOYALTY, 8 - have, None);
        t.activate(P0, pw, 2, &[Entity::Player(P0)]).expect("-8");
        t.resolve_all();
        let hand = t.hand_size(P0);
        if name == "Rowan Kenrith" {
            // Mind Stone: "{T}: Add {C}." isn't copied; "{1}, {T}, Sacrifice Mind Stone:
            // Draw a card." (it costs mana) is.
            let stone = t.battlefield(P0, "Mind Stone");
            t.activate(P0, stone, 0, &[]).expect("mana");
            t.settle();
            assert_eq!(t.stack_len(), 0);
            let stone = t.battlefield(P0, "Mind Stone");
            t.lands(P0, "Wastes", 1);
            t.activate(P0, stone, 1, &[]).expect("draw");
            t.settle();
            t.resolve_all();
            assert_eq!(t.hand_size(P0), hand + 2);
        } else {
            // Divination (no targets) is copied.
            lands_for_cost(&mut t, P0, "Divination");
            let d = t.hand(P0, "Divination");
            t.cast(P0, d).go();
            t.settle();
            t.resolve_all();
            assert_eq!(t.hand_size(P0), hand + 4);
        }
    }
}

// --- Adric, Mathematical Genius: "{2}{U}, {T}: Copy target activated or triggered ability
// you control. You may choose new targets for the copy." -------------------------------------

#[test]
fn adric_copies_of_a_linked_ability_are_linked_and_have_the_same_source() {
    cr!("607.2a", "707.10", "113.7a");
    ruling!(
        "Adric, Mathematical Genius",
        "If an ability is linked to a second ability, copies of that first ability are also linked to that second ability. If the second ability refers to \"the exiled card,\" it refers to all cards exiled by the first ability and the copy. For example, if Fiend Hunter's enters-the-battlefield ability is copied and two creatures are exiled, they both return when Fiend Hunter leaves the battlefield."
    );
    ruling!(
        "Adric, Mathematical Genius",
        "The source of the copy from Adric, Mathematical Genius's first ability is the same as the source of the original ability."
    );
    supported("Fiend Hunter");
    let mut t = TestGame::new(2);
    let adric = t.battlefield(P0, "Adric, Mathematical Genius");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 2);
    t.answer_targets(P0, &[ent(bears)]);
    let hunter = t.enter(P0, "Fiend Hunter");
    t.settle();
    let trigger = t.g.stack[0];
    t.activate(P0, adric, 0, &[ent(trigger)]).expect("Adric");
    change_copy_targets(&mut t, P0, &[Some(ent(giant))]);
    t.resolve();
    assert_eq!(
        abilities_from(&t, hunter).len(),
        2,
        "the copy's source is Fiend Hunter"
    );
    let copy = *t.g.stack.last().unwrap();
    assert!(matches!(
        t.obj(copy).stack.as_deref().map(|s| &s.kind),
        Some(StackKind::Triggered { source, .. }) if *source == hunter
    ));
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(bears)), Zone::Exile);
    assert_eq!(t.zone(t.g.current(giant)), Zone::Exile);
    crate::r_s02_common::destroy(&mut t, hunter);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
}
