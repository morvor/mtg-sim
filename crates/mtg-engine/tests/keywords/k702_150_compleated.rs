//! CR 702.150 Compleated.

use crate::common_k702_052_066::{choose_replacement, run_effect};
use crate::common_k702_140_152::*;
use mtg_engine::ability::*;
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Answers the "How will you pay {B/P}?" choice: 1 = with the colored mana, 2 = with 2
/// life (0 leaves it to the payment).
fn pay_phyrexian(t: &mut TestGame, how: usize) {
    t.answer(P0, DecisionKind::Option, Answer::Index(how));
}

fn loyalty(t: &TestGame, id: ObjectId) -> u32 {
    t.counters(id, "loyalty")
}

#[test]
fn paying_life_for_a_phyrexian_symbol_means_two_fewer_loyalty_counters() {
    cr!("702.150", "702.150a");
    ruling!(
        "Vraska, Betrayal's Sting",
        "A Phyrexian mana symbol contributes 1 toward the mana value of a card, even if life is paid for it. Specifically, Vraska's mana value is always 6."
    );
    let mut t = TestGame::new(2);
    // Vraska, Betrayal's Sting: {4}{B}{B/P}, loyalty 6, compleated.
    let vraska = t.hand(P0, "Vraska, Betrayal's Sting");
    add_mana(&mut t, P0, ManaType::B, 1);
    add_mana(&mut t, P0, ManaType::C, 4);
    pay_phyrexian(&mut t, 2);
    let spell = t.cast(P0, vraska).go();
    assert_eq!(t.life(P0), 18);
    assert_eq!(pool(&t, P0), 0);
    assert_eq!(t.g.mana_value_of(spell), 6);
    t.resolve_all();
    assert!(t.on_battlefield(vraska));
    assert_eq!(loyalty(&t, vraska), 4);
}

#[test]
fn paying_mana_for_it_it_enters_with_its_printed_loyalty() {
    cr!("702.150a");
    let mut t = TestGame::new(2);
    let vraska = t.hand(P0, "Vraska, Betrayal's Sting");
    add_mana(&mut t, P0, ManaType::B, 2);
    add_mana(&mut t, P0, ManaType::C, 4);
    pay_phyrexian(&mut t, 1);
    t.cast(P0, vraska).go();
    assert_eq!(t.life(P0), 20);
    t.resolve_all();
    assert_eq!(loyalty(&t, vraska), 6);
}

#[test]
fn life_paid_by_the_payment_for_a_phyrexian_symbol_counts_too() {
    cr!("702.150a");
    let mut t = TestGame::new(2);
    let vraska = t.hand(P0, "Vraska, Betrayal's Sting");
    // Not enough mana for {B/P}: the payment uses 2 life for it.
    add_mana(&mut t, P0, ManaType::B, 1);
    add_mana(&mut t, P0, ManaType::C, 4);
    t.cast(P0, vraska).go();
    assert_eq!(t.life(P0), 18);
    t.resolve_all();
    assert_eq!(loyalty(&t, vraska), 4);
}

#[test]
fn two_fewer_for_each_symbol_paid_with_life() {
    cr!("702.150a");
    // Nissa, Ascended Animist: {3}{G}{G}{G/P}{G/P}, loyalty 7.
    let mut t = TestGame::new(2);
    let nissa = t.hand(P0, "Nissa, Ascended Animist");
    add_mana(&mut t, P0, ManaType::G, 2);
    add_mana(&mut t, P0, ManaType::C, 3);
    pay_phyrexian(&mut t, 2);
    pay_phyrexian(&mut t, 2);
    t.cast(P0, nissa).go();
    assert_eq!(t.life(P0), 16);
    t.resolve_all();
    assert_eq!(loyalty(&t, nissa), 3);

    let mut t = TestGame::new(2);
    let nissa = t.hand(P0, "Nissa, Ascended Animist");
    add_mana(&mut t, P0, ManaType::G, 3);
    add_mana(&mut t, P0, ManaType::C, 3);
    pay_phyrexian(&mut t, 1);
    pay_phyrexian(&mut t, 2);
    t.cast(P0, nissa).go();
    assert_eq!(t.life(P0), 18);
    t.resolve_all();
    assert_eq!(loyalty(&t, nissa), 5);
}

#[test]
fn life_paid_for_other_reasons_doesnt_count() {
    cr!("702.150a");
    ruling!(
        "Tamiyo, Compleated Sage",
        "If a player paid life for some other reason while casting the spell, that will not reduce the number of loyalty counters the planeswalker enters the battlefield with."
    );
    let mut t = TestGame::new(2);
    // Mana Confluence: "{T}, Pay 1 life: Add one mana of any color."
    t.battlefield(P0, "Mana Confluence");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Wastes", 4);
    let vraska = t.hand(P0, "Vraska, Betrayal's Sting");
    pay_phyrexian(&mut t, 1);
    t.cast(P0, vraska).go();
    assert_eq!(t.life(P0), 19);
    t.resolve_all();
    assert_eq!(loyalty(&t, vraska), 6);
}

#[test]
fn it_isnt_reduced_unless_it_was_cast() {
    cr!("702.150a");
    let mut t = TestGame::new(2);
    let vraska = t.enter(P0, "Vraska, Betrayal's Sting");
    assert_eq!(loyalty(&t, vraska), 6);
}

#[test]
fn other_replacement_effects_apply_as_normal() {
    cr!("702.150a", "616.1");
    ruling!(
        "Vraska, Betrayal's Sting",
        "Other replacement effects that would change the number of loyalty counters Vraska enters with will apply as normal."
    );
    for (first, expected) in [(0, 8), (1, 10)] {
        let mut t = TestGame::new(2);
        // "If an effect would put one or more counters on a permanent you control, it puts
        // twice that many of those counters on that permanent instead."
        run_effect(
            &mut t,
            None,
            P0,
            Effect::AddReplacement {
                def: ReplacementDef {
                    event: ReplacementEvent::PutCounters {
                        on_objects: Some(Filter::ControlledBy(PlayerRel::You)),
                        on_players: None,
                        kind: None,
                    },
                    action: ReplacementAction::Multiply(2),
                    self_replacement: false,
                    optional: false,
                },
                duration: Duration::EndOfTurn,
                uses: None,
            },
            &[],
        );
        let vraska = t.hand(P0, "Vraska, Betrayal's Sting");
        add_mana(&mut t, P0, ManaType::B, 1);
        add_mana(&mut t, P0, ManaType::C, 4);
        pay_phyrexian(&mut t, 2);
        t.cast(P0, vraska).go();
        // Its controller chooses the order: compleated first gives (6 - 2) × 2; doubling
        // first gives 6 × 2 - 2.
        choose_replacement(&mut t, P0, first);
        t.resolve_all();
        assert_eq!(loyalty(&t, vraska), expected, "order {first}");
    }
}
