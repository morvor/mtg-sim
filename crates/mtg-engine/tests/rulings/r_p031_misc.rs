//! Rulings batch P031 — the other abilities of self-copying cards: last known information
//! (CR 608.2h), entering as a copy (CR 614.1c, 616.1, 707.5), "doesn't untap during its
//! controller's next untap step" (CR 502.3), triggers that resolve before the spell that
//! caused them (CR 603.3), text and type changes (CR 612, 205.1b), characteristic-defining
//! abilities in every zone (CR 604.3), and the legend rule (CR 704.5j).

use crate::r_p031_common::*;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s06_common::{activate_containing, attach_new, attached_to, give_control};
use crate::r_s11_common::empty_library;
use crate::r_s17_common::token_copy;
use crate::r_s25_common::{abilities_from, cast_new};
use crate::r_s26_common::modify_until_eot;
use mtg_engine::ability::{Modification, Value};
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn pump(t: &mut TestGame, id: ObjectId, p: i32, x: i32) {
    modify_until_eot(t, id, vec![Modification::ModifyPT(Value::c(p), Value::c(x))]);
}

/// `name` enters under `p`'s control (a real zone change), and triggers are put on the
/// stack.
fn enter(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = t.enter(p, name);
    t.g.flush_events();
    t.settle();
    t.g.current(id)
}

// --- Sunscourge Champion --------------------------------------------------------------------

#[test]
fn sunscourge_champion_gains_life_equal_to_its_power_on_resolution() {
    cr!("608.2h", "107.1b");
    ruling!(
        "Sunscourge Champion",
        "The amount of life you gain is determined as Sunscourge Champion's triggered ability resolves. Players may respond to the ability by attempting to change its power."
    );
    ruling!(
        "Sunscourge Champion",
        "If Sunscourge Champion leaves the battlefield before its triggered ability resolves, use its power as it last existed on the battlefield to determine how much life to gain. If that number is negative, you don't gain any life and you don't lose any life."
    );
    supported("Sunscourge Champion");
    // (pump, then leaves?) → life gained
    for (p, leaves, gain) in [(0, false, 2), (2, false, 4), (2, true, 4), (-5, true, 0)] {
        let mut t = TestGame::new(2);
        let c = enter(&mut t, P0, "Sunscourge Champion");
        assert_eq!(t.stack_len(), 1);
        pump(&mut t, c, p, 0);
        if leaves {
            kill(&mut t, c);
        }
        t.resolve_all();
        assert_eq!(t.life(P0), 20 + gain, "pump {p}, leaves {leaves}");
    }
}

// --- Essence of the Wild --------------------------------------------------------------------

/// Asserts the permanent is a plain, untapped Essence of the Wild with no counters.
fn plain_essence(t: &TestGame, id: ObjectId) {
    let o = t.obj_now(id);
    assert_eq!(o.chars.name, "Essence of the Wild");
    assert_eq!(t.pt(id), (6, 6));
    assert!(!o.tapped);
    assert!(o.counters.values().all(|n| *n == 0));
}

#[test]
fn essence_of_the_wild_wins_over_clone_and_printed_enters_abilities() {
    cr!("614.1c", "616.1", "707.5");
    ruling!(
        "Essence of the Wild",
        "If a creature such as Clone is entering under your control, there will be two copy effects to apply: the creature's own and Essence of the Wild's. No matter what order these effects are applied, the creature will be a copy of Essence of the Wild when it enters."
    );
    ruling!(
        "Essence of the Wild",
        "Other \"enters\" replacement abilities printed on the creature entering won't be applied because the creature will already be Essence of the Wild at that point (and therefore it won't have those abilities)."
    );
    supported("Essence of the Wild");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Essence of the Wild");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_choose(P0, &[obj(giant)]);
    let c = enter(&mut t, P0, "Clone");
    plain_essence(&t, c);
    // Diregraf Ghoul ("enters tapped") and Spike Colony ("enters with four +1/+1
    // counters").
    for name in ["Diregraf Ghoul", "Spike Colony"] {
        let id = enter(&mut t, P0, name);
        plain_essence(&t, id);
    }
}

#[test]
fn essence_of_the_wild_applies_after_control_changes_and_before_other_effects() {
    cr!("616.1", "614.12");
    ruling!(
        "Essence of the Wild",
        "Replacement effects that modify how a creature enters are applied in the following order: first control-changing effects (such as Gather Specimens), then copy effects (such as the abilities of Essence of the Wild and Clone), then all other effects."
    );
    supported("Grumgully, the Generous");
    // Grumgully: "Each other non-Human creature you control enters with an additional
    // +1/+1 counter on it." Elite Vanguard is a Human, but it enters as an Essence (an
    // Avatar) first.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Essence of the Wild");
    t.battlefield(P0, "Grumgully, the Generous");
    let v = enter(&mut t, P0, "Elite Vanguard");
    assert_eq!(t.obj_now(v).chars.name, "Essence of the Wild");
    assert_eq!(t.counters(v, counters::PLUS1), 1);
    // Gather Specimens: P1's creature enters under P0's control, so it's an Essence.
    supported("Gather Specimens");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Essence of the Wild");
    cast_new(&mut t, P0, "Gather Specimens", &[]);
    t.resolve_all();
    let b = enter(&mut t, P1, "Grizzly Bears");
    assert_eq!(t.obj_now(b).controller, P0);
    assert_eq!(t.obj_now(b).chars.name, "Essence of the Wild");
}

/// P0's Cryptoplasm becomes a copy of P1's Essence of the Wild at P0's next upkeep
/// (keeping its own ability). Returns the Cryptoplasm.
fn cryptoplasm_essence(t: &mut TestGame) -> ObjectId {
    supported("Cryptoplasm");
    let e = t.battlefield(P1, "Essence of the Wild");
    let c = t.battlefield(P0, "Cryptoplasm");
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(e)]);
    t.resolve_all();
    assert_eq!(t.obj_now(c).chars.name, "Essence of the Wild");
    t.advance_to(P0, Step::PrecombatMain);
    c
}

fn has_cryptoplasm_ability(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .any(|a| a.text.contains("become a copy of another target creature"))
}

#[test]
fn essence_copied_by_cryptoplasm_passes_on_the_modification() {
    cr!("707.9b", "707.3");
    ruling!(
        "Essence of the Wild",
        "If a creature such as Cryptoplasm that's already on the battlefield copies Essence of the Wild and modifies its own copy effect, that modification is also copied by creatures entering under your control."
    );
    let mut t = TestGame::new(2);
    cryptoplasm_essence(&mut t);
    let b = enter(&mut t, P0, "Grizzly Bears");
    assert_eq!(t.obj_now(b).chars.name, "Essence of the Wild");
    assert!(has_cryptoplasm_ability(&t, b));
}

#[test]
fn two_essences_the_one_applied_last_wins() {
    cr!("616.1", "616.1e");
    ruling!(
        "Essence of the Wild",
        "If you control more than one Essence of the Wild, creatures you control will enter as a copy of the one whose copy effect you apply last."
    );
    // P0 controls a real Essence and the Cryptoplasm-Essence (which has the extra
    // ability): which one the Bears copy depends on the order P0 applies them in.
    let mut outcomes = vec![];
    for first in [0usize, 1] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Essence of the Wild");
        cryptoplasm_essence(&mut t);
        let from = t.asked().len();
        t.answer(P0, DecisionKind::Replacement, Answer::Index(first));
        let b = enter(&mut t, P0, "Grizzly Bears");
        assert_eq!(t.obj_now(b).chars.name, "Essence of the Wild");
        let asked = t.asked()[from..].to_vec();
        assert!(!asked.is_empty(), "P0 chooses the order: {asked:?}");
        outcomes.push(has_cryptoplasm_ability(&t, b));
    }
    outcomes.sort();
    assert_eq!(outcomes, vec![false, true]);
}

// --- Hands of Binding -------------------------------------------------------------------------

/// P0 casts Hands of Binding on P1's Grizzly Bears (encoding it on nothing). Returns the
/// Bears, tapped.
fn bind(t: &mut TestGame) -> ObjectId {
    supported("Hands of Binding");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_new(t, P0, "Hands of Binding", &[obj(bears)]);
    t.resolve_all();
    assert!(t.obj_now(bears).tapped);
    bears
}

#[test]
fn hands_of_binding_follows_the_creature_to_its_new_controller() {
    cr!("502.3", "611.2c");
    ruling!(
        "Hands of Binding",
        "If a different player gains control of the creature that was tapped by Hands of Binding, Hands of Binding will stop that creature from untapping during its new controller's next untap step."
    );
    let mut t = TestGame::new(2);
    let bears = bind(&mut t);
    give_control(&mut t, bears, P0);
    t.advance_to(P0, Step::Upkeep);
    assert!(t.obj_now(bears).tapped, "P0's next untap step");
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert!(!t.obj_now(bears).tapped);
}

#[test]
fn hands_of_binding_does_nothing_later_if_the_creature_was_untapped() {
    cr!("502.3");
    ruling!(
        "Hands of Binding",
        "If the creature that was tapped by Hands of Binding is untapped during its controller's next untap step (perhaps because a spell untapped it), Hands of Binding has no effect at that time. It won't apply at some later time when the creature is tapped."
    );
    let mut t = TestGame::new(2);
    let bears = bind(&mut t);
    t.g.untap(bears);
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.obj_now(bears).tapped);
    t.g.tap(bears);
    t.advance_to(P0, Step::Upkeep);
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.obj_now(bears).tapped, "it untaps normally");
}

// --- Triggers that resolve before the spell that caused them ---------------------------

#[test]
fn pawpatch_recruit_triggers_once_per_creature_targeted() {
    cr!("603.2c", "115.3");
    ruling!(
        "Pawpatch Recruit",
        "If a spell or ability an opponent controls targets a single creature you control more than once, Pawpatch Recruit’s last ability will trigger only once. However, if a spell or ability an opponent controls targets multiple creatures you control, Pawpatch Recruit’s last ability will trigger once for each of those permanents."
    );
    supported("Pawpatch Recruit");
    supported("Seeds of Strength");
    for (spread, want) in [(false, 1), (true, 2)] {
        let mut t = TestGame::new(2);
        let r = t.battlefield(P0, "Pawpatch Recruit");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let third = if spread { r } else { bears };
        cast_new(&mut t, P1, "Seeds of Strength", &[obj(bears), obj(bears), obj(third)]);
        t.settle();
        assert_eq!(abilities_from(&t, r).len(), want, "spread: {spread}");
    }
}

#[test]
fn these_triggers_resolve_even_if_the_spell_is_countered() {
    cr!("603.3", "701.6a");
    ruling!(
        "Pawpatch Recruit",
        "Pawpatch Recruit’s last ability resolves before the spell or ability that caused it to trigger. It resolves even if that spell or ability is countered."
    );
    ruling!(
        "Steelburr Champion",
        "Steelburr Champion's last ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    ruling!(
        "Leitmotif Composer",
        "Leitmotif Composer's second ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack without resolving."
    );
    // (card, its controller, the spell's caster, the spell, its target)
    for name in ["Pawpatch Recruit", "Steelburr Champion", "Leitmotif Composer"] {
        supported(name);
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, name);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let spell = match name {
            "Pawpatch Recruit" => cast_new(&mut t, P1, "Shock", &[obj(bears)]),
            "Steelburr Champion" => cast_new(&mut t, P1, "Shock", &[Entity::Player(P0)]),
            _ => cast_new(&mut t, P0, "Lava Axe", &[Entity::Player(P1)]),
        };
        t.settle();
        assert_eq!(t.stack_len(), 2);
        assert_eq!(t.g.stack[0], spell, "{name}: the trigger is above the spell");
        // The spell's caster's opponent counters it.
        let counterer = if t.obj(spell).controller == P0 { P1 } else { P0 };
        cast_new(&mut t, counterer, "Cancel", &[obj(spell)]);
        t.resolve_all();
        assert!(t.zone(t.g.current(spell)) != Zone::Stack);
        assert_eq!(t.life(P0), 20, "{name}");
        assert_eq!(t.life(P1), 20, "{name}");
        match name {
            "Leitmotif Composer" => {
                assert_eq!(named(&t, P0, name).len(), 2, "{name}: a token")
            }
            _ => assert_eq!(t.counters(c, counters::PLUS1), 1, "{name}: a counter"),
        }
    }
}

// --- Text and type changes ------------------------------------------------------------------

#[test]
fn trait_doctoring_changes_a_plains_mana_ability_but_not_its_name() {
    cr!("612.2", "305.7");
    ruling!(
        "Trait Doctoring",
        "If the basic land type of a land is changed, the associated mana ability of that land will also change, but its name will not."
    );
    supported("Trait Doctoring");
    let mut t = TestGame::new(2);
    let plains = t.battlefield(P0, "Plains");
    // Plains is the 6th word offered (white, blue, black, red, green, Plains, ...); Island
    // the first of the remaining land words.
    t.answer(P0, DecisionKind::Option, Answer::Index(5));
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    // (Its own Island pays for Trait Doctoring.)
    t.lands(P0, "Island", 1);
    let td = t.hand(P0, "Trait Doctoring");
    t.cast_with(P0, td, &[obj(plains)]).unwrap();
    t.resolve_all();
    let o = t.obj_now(plains);
    assert_eq!(o.chars.name, "Plains");
    assert!(o.chars.subtypes.iter().any(|s| s == "Island"));
    assert!(!o.chars.subtypes.iter().any(|s| s == "Plains"));
    let salve = t.hand(P0, "Healing Salve");
    assert!(t.cast_with(P0, salve, &[Entity::Player(P0)]).is_err(), "no {{W}}");
    let opt = t.hand(P0, "Opt");
    assert!(t.cast_with(P0, opt, &[]).is_ok(), "{{U}} from the Plains");
}

#[test]
fn enchanted_creature_loses_its_non_creature_subtypes() {
    cr!("205.1b", "613.1d");
    ruling!(
        "Amphibian Downpour",
        "If the enchanted creature has any subtypes other than creature types, such as Equipment, Vehicle, or Urza's, it loses those as well."
    );
    ruling!(
        "Trickster's Elk",
        "If the enchanted creature has any subtypes other than creature types, such as Equipment, Vehicle, or Urza's, it loses those as well."
    );
    supported("Amphibian Downpour");
    supported("Trickster's Elk");
    // Lizard Blades: "Artifact Creature — Equipment Lizard".
    let mut t = TestGame::new(2);
    let blades = t.battlefield(P1, "Lizard Blades");
    assert!(t.obj_now(blades).chars.subtypes.iter().any(|s| s == "Equipment"));
    attach_new(&mut t, P0, "Amphibian Downpour", blades);
    assert_eq!(t.obj_now(blades).chars.subtypes.to_vec(), vec!["Frog"]);
    let mut t = TestGame::new(2);
    let blades = t.battlefield(P1, "Lizard Blades");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 1);
    let elk = t.hand(P0, "Trickster's Elk");
    t.cast(P0, elk)
        .method(CastMethod::Keyword(KeywordKind::Bestow))
        .target(obj(blades))
        .go();
    t.resolve_all();
    assert_eq!(t.obj_now(blades).chars.subtypes.to_vec(), vec!["Elk"]);
}

#[test]
fn the_flood_of_mars_on_a_land_creature_makes_a_copy_not_an_island() {
    cr!("707.2", "205.1b");
    ruling!(
        "The Flood of Mars",
        "If the target is both a creature and a land as the triggered ability resolves, it will become a copy of The Flood of Mars. Excepting some very unusual cases, that copy won't be a land, so it won't also become an Island."
    );
    supported("The Flood of Mars");
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "The Flood of Mars");
    let arbor = t.battlefield(P1, "Dryad Arbor");
    t.answer_targets(P0, &[obj(arbor)]);
    attack_with(&mut t, &[(f, Entity::Player(P1))]);
    t.resolve_all();
    let o = t.obj_now(arbor);
    assert_eq!(o.chars.name, "The Flood of Mars");
    assert!(!o.is(CardType::Land));
    assert!(!o.chars.subtypes.iter().any(|s| s == "Island" || s == "Forest"));
    assert_eq!(t.counters(arbor, "flood"), 1);
}

// --- Joo Dee, One of Many --------------------------------------------------------------------

#[test]
fn joo_dee_sacrifices_as_part_of_the_resolution() {
    cr!("608.2c", "701.21a");
    ruling!(
        "Joo Dee, One of Many",
        "If you control no creatures other than Joo Dee and no artifacts as the ability resolves, you'll need to sacrifice either Joo Dee or the token copy."
    );
    ruling!(
        "Joo Dee, One of Many",
        "The sacrifice part of Joo Dee's ability happens as part of the resolution of the ability. Players cannot do anything between creating the copy and sacrificing an artifact or creature."
    );
    supported("Joo Dee, One of Many");
    // Only Joo Dee: the choice is between it and its token.
    let mut t = TestGame::new(2);
    let joo = t.battlefield(P0, "Joo Dee, One of Many");
    t.lands(P0, "Swamp", 1);
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(joo)]);
    activate_containing(&mut t, P0, joo, "Surveil").unwrap();
    t.resolve_all();
    let offered: Vec<Vec<Entity>> = crate::r_s03_common::choice_candidates(&t, from, "")
        .into_iter()
        .filter(|c| c.contains(&obj(joo)))
        .collect();
    assert_eq!(offered.len(), 1);
    assert_eq!(offered[0].len(), 2, "Joo Dee and its token");
    let left = named(&t, P0, "Joo Dee, One of Many");
    assert_eq!(left.len(), 1);
    assert!(t.obj_now(left[0]).is_token());
    // With Soul Warden: the token enters and Joo Dee is sacrificed before Soul Warden's
    // trigger is even put on the stack.
    let mut t = TestGame::new(2);
    let joo = t.battlefield(P0, "Joo Dee, One of Many");
    t.battlefield(P0, "Soul Warden");
    t.lands(P0, "Swamp", 1);
    t.answer_choose(P0, &[obj(joo)]);
    activate_containing(&mut t, P0, joo, "Surveil").unwrap();
    t.resolve();
    assert!(t.in_graveyard(P0, "Joo Dee, One of Many"));
    assert_eq!(named(&t, P0, "Joo Dee, One of Many").len(), 1);
    assert_eq!(t.stack_len(), 1, "Soul Warden's trigger");
    assert_eq!(t.life(P0), 20);
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
}
