//! Rulings batch P162 — counting the spells cast this turn (spells cast, not resolved;
//! copies aren't cast, CR 707.10), and "for each color of mana spent to cast it" counting
//! the colors of mana actually spent (CR 609.4b).

use crate::r_p076_common::mana;
use crate::r_p119_common::counter_spell;
use crate::r_s01_common::{supported, triggers_on_stack};
use crate::r_s02_common::can_cast;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

/// P0 casts Lightning Bolt at P1 and it's countered.
fn countered_bolt(t: &mut TestGame) {
    mana(t, P0, ManaType::R, 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    let bolt = t.cast(P0, bolt).target(Entity::Player(P1)).go();
    counter_spell(t, bolt);
}

#[test]
fn spells_cast_this_turn_count_whatever_happened_to_them() {
    cr!("601.2i", "701.6a");
    ruling!(
        "Lock and Load",
        "Lock and Load will count any other instant and sorcery spells you’ve cast this turn. It doesn’t matter if the other spells resolved, didn’t resolve, were countered, or are still on the stack."
    );
    ruling!(
        "Thunder Salvo",
        "Thunder Salvo will count any other spells you’ve cast this turn. It doesn’t matter if the other spells resolved, didn’t resolve, were countered, or are still on the stack."
    );
    ruling!(
        "Thunder Salvo",
        "The value of X is determined only once, as Thunder Salvo resolves."
    );
    supported("Lock and Load");
    supported("Thunder Salvo");
    // Lock and Load: a countered Lightning Bolt and a resolved Opt count; a creature
    // spell doesn't.
    let mut t = TestGame::new(2);
    countered_bolt(&mut t);
    mana(&mut t, P0, ManaType::U, 1);
    let opt = t.hand(P0, "Opt");
    t.cast(P0, opt).go();
    t.resolve_all();
    mana(&mut t, P0, ManaType::G, 2);
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.resolve_all();
    let hand = t.hand_size(P0);
    mana(&mut t, P0, ManaType::U, 3);
    let lnl = t.hand(P0, "Lock and Load");
    t.cast(P0, lnl).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 3);
    // Thunder Salvo: a countered Bolt, a creature spell still on the stack, and Opt cast
    // after Thunder Salvo was cast but before it resolved: X = 2 + 3.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Colossal Dreadmaw");
    countered_bolt(&mut t);
    mana(&mut t, P0, ManaType::G, 2);
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    mana(&mut t, P0, ManaType::R, 2);
    let salvo = t.hand(P0, "Thunder Salvo");
    t.cast(P0, salvo).target(wurm).go();
    mana(&mut t, P0, ManaType::U, 1);
    let opt = t.hand(P0, "Opt");
    t.cast(P0, opt).go();
    t.resolve(); // Opt
    t.resolve(); // Thunder Salvo
    assert_eq!(crate::r_s07_common::damage_on(&t, wurm), 5);
}

#[test]
fn storm_entity_counts_spells_cast_while_it_was_on_the_stack_but_not_copies() {
    cr!("707.10", "614.1c");
    ruling!(
        "Storm Entity",
        "The number of other spells cast this turn is determined as Storm Entity enters the battlefield. It will count spells that were cast while Storm Entity was on the stack. It will not count copies of spells that were created directly on the stack."
    );
    supported("Storm Entity");
    supported("Reverberate");
    let mut t = TestGame::new(2);
    // Shock, copied by Reverberate: two spells cast, one copy.
    mana(&mut t, P0, ManaType::R, 1);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(Entity::Player(P1)).go();
    mana(&mut t, P0, ManaType::R, 2);
    let rev = t.hand(P0, "Reverberate");
    t.cast(P0, rev).target(shock).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    mana(&mut t, P0, ManaType::R, 2);
    let entity = t.hand(P0, "Storm Entity");
    t.cast(P0, entity).go();
    // Opt cast while Storm Entity is on the stack counts.
    mana(&mut t, P0, ManaType::U, 1);
    let opt = t.hand(P0, "Opt");
    t.cast(P0, opt).go();
    t.resolve_all();
    assert_eq!(t.counters(entity, "+1/+1"), 3);
}

#[test]
fn another_spell_cast_this_turn_may_have_been_countered_or_be_on_the_stack() {
    cr!("601.3");
    ruling!(
        "Illusory Angel",
        "It doesn't matter whether the other spell resolved. It could have been countered or, if you've somehow cast Illusory Angel as though it had flash, it could still be on the stack."
    );
    ruling!(
        "Hewed Stone Retainers",
        "It doesn’t matter whether the other spell resolved. It could have been countered or, if you’ve somehow cast Hewed Stone Retainers as though it had flash, it could still be on the stack."
    );
    supported("Vedalken Orrery");
    for card in ["Illusory Angel", "Hewed Stone Retainers"] {
        supported(card);
        // Countered.
        let mut t = TestGame::new(2);
        t.lands(P0, "Island", 3);
        let c = t.hand(P0, card);
        assert!(!can_cast(&mut t, P0, c, CastMethod::Normal), "{card}");
        countered_bolt(&mut t);
        assert!(can_cast(&mut t, P0, c, CastMethod::Normal), "{card}");
        // Still on the stack, with Vedalken Orrery letting it be cast as though it had
        // flash.
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Vedalken Orrery");
        t.lands(P0, "Island", 3);
        let c = t.hand(P0, card);
        mana(&mut t, P0, ManaType::U, 1);
        let opt = t.hand(P0, "Opt");
        t.cast(P0, opt).go();
        assert_eq!(t.stack_len(), 1);
        t.cast(P0, c).go();
        assert_eq!(t.stack_len(), 2, "{card}");
    }
}

#[test]
fn converge_counts_only_colored_mana_actually_spent_to_cast_it() {
    cr!("614.1c", "106.1b", "609.4b");
    ruling!(
        "Springmantle Cleric",
        "Colorless mana won’t give Springmantle Cleric another +1/+1 counter. Colorless is not a color."
    );
    ruling!(
        "Springmantle Cleric",
        "If Springmantle Cleric enters the battlefield without being cast, it receives no +1/+1 counters."
    );
    ruling!(
        "Springmantle Cleric",
        "If an effect allows you to spend mana as though it were mana of any color to cast Springmantle Cleric, you consider only the actual mana spent to determine how many counters it receives."
    );
    supported("Springmantle Cleric");
    supported("Mycosynth Lattice");
    // {G} and four colorless: one counter.
    let mut t = TestGame::new(2);
    mana(&mut t, P0, ManaType::G, 1);
    mana(&mut t, P0, ManaType::C, 4);
    let c = t.hand(P0, "Springmantle Cleric");
    t.cast(P0, c).go();
    t.resolve_all();
    assert_eq!(t.counters(c, "+1/+1"), 1);
    // {W}{U}{B}{R}{G}: five.
    let mut t = TestGame::new(2);
    for ty in [ManaType::W, ManaType::U, ManaType::B, ManaType::R, ManaType::G] {
        mana(&mut t, P0, ty, 1);
    }
    let c = t.hand(P0, "Springmantle Cleric");
    t.cast(P0, c).go();
    t.resolve_all();
    assert_eq!(t.counters(c, "+1/+1"), 5);
    // Put onto the battlefield without being cast: none.
    let mut t = TestGame::new(2);
    let c = t.enter(P0, "Springmantle Cleric");
    assert_eq!(t.counters(c, "+1/+1"), 0);
    // Mycosynth Lattice: five colorless mana spent as though it were green: none.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mycosynth Lattice");
    mana(&mut t, P0, ManaType::C, 5);
    let c = t.hand(P0, "Springmantle Cleric");
    t.cast(P0, c).go();
    t.resolve_all();
    assert_eq!(t.zone(c), Zone::Battlefield);
    assert_eq!(t.counters(c, "+1/+1"), 0);
}

#[test]
fn a_copy_of_a_converge_creature_counts_the_mana_spent_on_itself() {
    cr!("614.1c", "707.2");
    ruling!(
        "Springmantle Cleric",
        "If another creature enters the battlefield as a copy of Springmantle Cleric, consider what mana was spent to cast that creature to determine how many +1/+1 counters it enters with."
    );
    supported("Clone");
    let mut t = TestGame::new(2);
    let cleric = t.battlefield(P0, "Springmantle Cleric");
    // Clone cast with {U} and {G}{C}{C}: two colors.
    mana(&mut t, P0, ManaType::U, 1);
    mana(&mut t, P0, ManaType::G, 1);
    mana(&mut t, P0, ManaType::C, 2);
    let clone = t.hand(P0, "Clone");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(cleric)]);
    t.cast(P0, clone).go();
    t.resolve_all();
    assert_eq!(t.obj_now(clone).chars.name, "Springmantle Cleric");
    assert_eq!(t.counters(clone, "+1/+1"), 2);
}

#[test]
fn skyrider_elf_x_lets_you_spend_more_colors() {
    cr!("107.3a", "614.1c");
    ruling!(
        "Skyrider Elf",
        "You can choose any value for X. The value chosen for X doesn't directly affect the number of +1/+1 counters Skyrider Elf enters the battlefield with, but it does let you pay more mana and thus spend more colors of mana to cast it."
    );
    supported("Skyrider Elf");
    for (x, pay, n) in [
        (0, vec![ManaType::G, ManaType::U], 2),
        (
            3,
            vec![
                ManaType::W,
                ManaType::U,
                ManaType::B,
                ManaType::R,
                ManaType::G,
            ],
            5,
        ),
    ] {
        let mut t = TestGame::new(2);
        for ty in pay {
            mana(&mut t, P0, ty, 1);
        }
        let elf = t.hand(P0, "Skyrider Elf");
        t.cast(P0, elf).x(x).go();
        t.resolve_all();
        assert_eq!(t.counters(elf, "+1/+1"), n, "X={x}");
    }
}

#[test]
fn magmablood_archaic_trigger_resolves_first_even_if_the_spell_is_countered() {
    cr!("603.3", "405.5");
    ruling!(
        "Magmablood Archaic",
        "Magmablood Archaic's last ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack."
    );
    supported("Magmablood Archaic");
    let mut t = TestGame::new(2);
    let archaic = t.battlefield(P0, "Magmablood Archaic");
    let base = t.pt(archaic).0;
    mana(&mut t, P0, ManaType::R, 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    let bolt = t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    assert_eq!(triggers_on_stack(&t, "+1/+0"), 1);
    assert_eq!(*t.g.stack.first().unwrap(), bolt);
    counter_spell(&mut t, bolt);
    t.resolve_all();
    assert_eq!(t.pt(archaic).0, base + 1);
    assert_eq!(t.life(P1), 20);
}
