//! Rulings batch P033 — reductions for spells and abilities that target something
//! (CR 601.2f, 602.2b, 115.3): "Spells that target this creature cost {2} less to cast"
//! counts the creature once however often it's targeted, other targets don't matter, and
//! such an ability of a spell doesn't affect activated abilities.

use crate::r_p033_common::*;
use crate::r_s01_common::{supported, tapped_lands};
use crate::r_p213_common::equip_nth;
use crate::r_s04_common::activate_named;
use crate::r_s06_common::{activate_containing, attach_new, attached_to};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// `p` casts `name` from hand with the given targets (one entity per target slot),
/// paying from a full pool; returns what it cost.
fn cast_targeting(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) -> Paid {
    for e in targets {
        t.answer_targets(p, &[*e]);
    }
    let card = t.hand(p, name);
    cast_normal(t, p, card)
}

#[test]
fn spellwild_ouphe_counts_each_ouphe_once_for_any_caster() {
    cr!("601.2f", "115.3", "601.2c");
    ruling!(
        "Spellwild Ouphe",
        "The cost reduction will be {2} no matter how many times the spell targets the same Spellwild Ouphe."
    );
    ruling!(
        "Spellwild Ouphe",
        "The spell may have any number of other targets; that won’t affect the cost reduction unless the spell targets other Spellwild Ouphes as well. In that case, the spell costs {2} less to cast for each different Spellwild Ouphe it targets."
    );
    ruling!("Spellwild Ouphe", "The cost reduction applies to spells cast by any player.");
    supported("Spellwild Ouphe");
    supported("Seeds of Strength");
    // Seeds of Strength ({G}{W}): "Target creature gets +1/+1 until end of turn." three
    // times. P1's two God-Pharaoh's Statues ("Spells your opponents cast cost {2} more")
    // make it {4}{G}{W} for P0.
    let setup = || {
        let mut t = main_phase(2);
        t.battlefield(P1, "God-Pharaoh's Statue");
        t.battlefield(P1, "God-Pharaoh's Statue");
        let a = t.battlefield(P0, "Spellwild Ouphe");
        let b = t.battlefield(P0, "Spellwild Ouphe");
        let bears = t.battlefield(P0, "Grizzly Bears");
        (t, a, b, bears)
    };
    // The same Ouphe three times: {2} less.
    let (mut t, a, _, _) = setup();
    let (p, q) = t.pt(a);
    let paid = cast_targeting(&mut t, P0, "Seeds of Strength", &[obj(a), obj(a), obj(a)]);
    assert_spent(&paid.spent, "{2}{G}{W}", "one Ouphe three times");
    t.resolve_all();
    assert_eq!(t.pt(a), (p + 3, q + 3));
    // One Ouphe and the Bears twice: {2} less.
    let (mut t, a, _, bears) = setup();
    let paid = cast_targeting(&mut t, P0, "Seeds of Strength", &[obj(a), obj(bears), obj(bears)]);
    assert_spent(&paid.spent, "{2}{G}{W}", "one Ouphe and other targets");
    // Two different Ouphes and the Bears: {4} less.
    let (mut t, a, b, bears) = setup();
    let paid = cast_targeting(&mut t, P0, "Seeds of Strength", &[obj(a), obj(b), obj(bears)]);
    assert_spent(&paid.spent, "{G}{W}", "two Ouphes");
    // No Ouphe: {4}{G}{W}.
    let (mut t, _, _, bears) = setup();
    let paid = cast_targeting(&mut t, P0, "Seeds of Strength", &[obj(bears), obj(bears), obj(bears)]);
    assert_spent(&paid.spent, "{4}{G}{W}", "no Ouphe");
    // P1's Murder ({1}{B}{B}) targeting P0's Ouphe: {B}{B}.
    let mut t = main_phase(2);
    let ouphe = t.battlefield(P0, "Spellwild Ouphe");
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    let paid = cast_targeting(&mut t, P1, "Murder", &[obj(ouphe)]);
    assert_spent(&paid.spent, "{B}{B}", "P1's Murder");
    t.resolve_all();
    assert!(!t.on_battlefield(ouphe));
}

#[test]
fn elderwood_scion_changes_spells_with_other_targets_but_not_abilities() {
    cr!("601.2f", "602.2b", "113.3b");
    ruling!(
        "Elderwood Scion",
        "Spells you cast that target Elderwood Scion will cost {2} less to cast even if those spells have additional targets. The same is true for spells your opponents cast costing {2} more."
    );
    ruling!(
        "Elderwood Scion",
        "None of Elderwood Scion’s abilities affect abilities that target it."
    );
    supported("Elderwood Scion");
    // "Spells you cast that target this creature cost {2} less to cast. Spells your
    // opponents cast that target this creature cost {2} more to cast."
    // Ram Through ({1}{G}): "Target creature you control deals damage equal to its power
    // to target creature you don't control."
    let mut t = main_phase(2);
    let scion = t.battlefield(P0, "Elderwood Scion");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let paid = cast_targeting(&mut t, P0, "Ram Through", &[obj(scion), obj(bears)]);
    assert_spent(&paid.spent, "{G}", "P0's Ram Through");
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    let mut t = main_phase(2);
    let scion = t.battlefield(P0, "Elderwood Scion");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    let paid = cast_targeting(&mut t, P1, "Ram Through", &[obj(wurm), obj(scion)]);
    assert_spent(&paid.spent, "{3}{G}", "P1's Ram Through");
    t.resolve_all();
    assert!(!t.on_battlefield(scion));
    // Icy Manipulator's "{1}, {T}: Tap target artifact, creature, or land." targeting the
    // Scion costs {1} for either player (one land pays it).
    for p in [P0, P1] {
        let mut t = main_phase(2);
        t.set_step(p, mtg_engine::turn::Step::PrecombatMain);
        let scion = t.battlefield(P0, "Elderwood Scion");
        let icy = t.battlefield(p, "Icy Manipulator");
        t.lands(p, "Wastes", 1);
        t.answer_targets(p, &[obj(scion)]);
        activate_containing(&mut t, p, icy, "Tap target").expect("Icy for {1}");
        assert_eq!(tapped_lands(&t, p), 1);
        t.lands(p, "Wastes", 3);
        t.resolve_all();
        assert!(t.obj_now(scion).tapped, "{p:?}");
        // It wasn't free for P0 either: with no mana, no activation.
        let mut t = main_phase(2);
        t.set_step(p, mtg_engine::turn::Step::PrecombatMain);
        let scion = t.battlefield(P0, "Elderwood Scion");
        let icy = t.battlefield(p, "Icy Manipulator");
        t.answer_targets(p, &[obj(scion)]);
        assert!(activate_containing(&mut t, p, icy, "Tap target").is_err());
    }
}

#[test]
fn strong_back_reduces_equip_abilities_and_other_auras_but_not_reconfigure_or_itself() {
    cr!("601.2f", "602.2b", "702.6a", "702.151a");
    ruling!(
        "Strong Back",
        "Some Equipment creature cards in other sets have reconfigure, a different ability that attaches them to a creature. Reconfigure is not an equip ability, and reconfigure costs are not reduced by Strong Back’s second ability. Equip variants or equip abilities with restrictions, like the “equip Citizen” ability of Ceremonial Groundbreaker from the Streets of New Capenna® set, are still equip abilities, and their costs will be reduced by Strong Back’s second ability."
    );
    ruling!("Strong Back", "Strong Back’s third ability won’t reduce its own cost.");
    supported("Strong Back");
    supported("Ceremonial Groundbreaker");
    supported("Lion Sash");
    // "Equip abilities you activate that target enchanted creature cost {3} less to
    // activate. Aura spells you cast that target enchanted creature cost {3} less to
    // cast." High-Rise Sawjack is a Citizen.
    let setup = || {
        let mut t = main_phase(2);
        let citizen = t.battlefield(P0, "High-Rise Sawjack");
        attach_new(&mut t, P0, "Strong Back", obj(citizen));
        (t, citizen)
    };
    // Ceremonial Groundbreaker's "Equip Citizen {1}" (the first) and "Equip {3}": free.
    for nth in [0, 1] {
        let (mut t, citizen) = setup();
        let gb = t.battlefield(P0, "Ceremonial Groundbreaker");
        equip_nth(&mut t, P0, gb, nth, citizen).expect("free equip");
        t.resolve_all();
        assert_eq!(attached_to(&t, gb), Some(obj(citizen)), "equip ability {nth}");
        // Not attached to another creature, it isn't reduced: {3} with no mana fails.
        let bears = t.battlefield(P0, "Grizzly Bears");
        assert!(equip_nth(&mut t, P0, gb, 1, bears).is_err());
    }
    // Lion Sash's reconfigure {2}: not free, {2} pays it.
    let (mut t, citizen) = setup();
    let sash = t.battlefield(P0, "Lion Sash");
    t.answer_targets(P0, &[obj(citizen)]);
    assert!(activate_named(&mut t, P0, sash, "Reconfigure", 0).is_err());
    t.clear_answers();
    t.lands(P0, "Wastes", 2);
    t.answer_targets(P0, &[obj(citizen)]);
    activate_named(&mut t, P0, sash, "Reconfigure", 0).expect("reconfigure for {2}");
    assert_eq!(tapped_lands(&t, P0), 2);
    t.resolve_all();
    assert_eq!(attached_to(&t, sash), Some(obj(citizen)));
    // Strong Back ({2}{G}) cast onto a creature costs {2}{G}; a second one onto the
    // creature the first enchants costs {G}.
    let mut t = main_phase(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let paid = cast_targeting(&mut t, P0, "Strong Back", &[obj(bears)]);
    assert_spent(&paid.spent, "{2}{G}", "the first Strong Back");
    t.resolve_all();
    let paid = cast_targeting(&mut t, P0, "Strong Back", &[obj(bears)]);
    assert_spent(&paid.spent, "{G}", "a second Strong Back");
}

#[test]
fn starfield_mystics_aura_on_an_opponents_creature_stays_under_your_control() {
    cr!("303.4e", "601.2f");
    ruling!(
        "Starfield Mystic",
        "If you cast an Aura spell targeting an opponent's permanent, you still control the Aura after it resolves."
    );
    supported("Starfield Mystic");
    // "Enchantment spells you cast cost {1} less to cast." Pacifism ({1}{W}) on P1's
    // Bears: {W}, and P0 controls it.
    let mut t = main_phase(2);
    t.battlefield(P0, "Starfield Mystic");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let paid = cast_targeting(&mut t, P0, "Pacifism", &[obj(bears)]);
    assert_spent(&paid.spent, "{W}", "Pacifism");
    let spell = paid.spell;
    t.resolve_all();
    let aura = t.g.current(spell);
    assert!(t.on_battlefield(aura));
    assert_eq!(t.obj(aura).controller, P0);
    assert_eq!(attached_to(&t, aura), Some(obj(bears)));
}
