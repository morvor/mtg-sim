//! Rulings batch P158 — copies of spells and self-replacement effects: a copy of a spell
//! wasn't cast (CR 707.10), so "if you cast this spell during your main phase", "if this
//! spell was cast from ...", and "if {R}{G} was spent to cast this spell" are false for
//! the copy; copies keep the original's X and modes (CR 707.10, 707.2); copy effects
//! copy any instant or sorcery spell, and one that has left the stack (CR 608.2h).

use crate::r_p158_common::*;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn a_copy_of_careful_consideration_was_not_cast_during_a_main_phase() {
    cr!("707.10", "614.1a");
    ruling!(
        "Careful Consideration",
        "If a spell or ability copies Careful Consideration, the target player discards three cards for the copy after drawing four cards. This is because the copy wasn't cast at all."
    );
    supported("Careful Consideration");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let cc = cast_at(&mut t, P0, "Careful Consideration", &[pl(P0)]);
    vengeance(&mut t, P0, cc);
    t.resolve(); // Vengeance: the copy
    assert_eq!(spell_copies(&t).len(), 1);
    assert_eq!(t.hand_size(P0), 0);
    t.resolve(); // the copy: draw four, discard three
    assert_eq!(t.hand_size(P0), 1);
    t.resolve(); // the original, cast during the main phase: draw four, discard two
    assert_eq!(t.hand_size(P0), 3);
}

#[test]
fn a_copy_of_might_of_old_krosa_gives_only_plus_two() {
    cr!("707.10", "614.1a");
    ruling!(
        "Might of Old Krosa",
        "If an effect copies Might of Old Krosa while it's on the stack, the copy will give the creature only +2/+2, even if the original spell was cast during your main phase. This is because the copy is created on the stack and wasn't cast at all."
    );
    supported("Might of Old Krosa");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let m = cast_at(&mut t, P0, "Might of Old Krosa", &[obj(bears)]);
    vengeance(&mut t, P0, m);
    t.resolve();
    t.resolve(); // the copy
    assert_eq!(t.pt(bears), (4, 4));
    t.resolve(); // the original
    assert_eq!(t.pt(bears), (8, 8));
}

#[test]
fn a_copy_of_see_the_truth_was_not_cast_from_anywhere() {
    cr!("707.10", "614.1a");
    ruling!(
        "See the Truth",
        "If an effect copies See the Truth while it's on the stack, the copy wasn't cast at all, so you only get one of the cards it has you look at."
    );
    supported("See the Truth");
    supported("Snapcaster Mage");
    // The original is cast from the graveyard (Snapcaster Mage's flashback): it puts all
    // three cards into your hand; its copy puts only one.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let stt = t.graveyard(P0, "See the Truth");
    t.answer_targets(P0, &[obj(stt)]);
    t.enter(P0, "Snapcaster Mage");
    t.resolve_all();
    let stt = cast_from_graveyard(&mut t, P0, stt, &[]);
    vengeance(&mut t, P0, stt);
    t.resolve();
    t.resolve(); // the copy: one card
    assert_eq!(t.hand_size(P0), 1);
    t.resolve(); // the original, cast from the graveyard: all three
    assert_eq!(t.hand_size(P0), 4);
}

/// P0 casts Mythos of Illuna at `target` with exactly {U}{U}{R}{G} (or {U}{U}{C}{C})
/// in their pool, and copies it with Increasing Vengeance when `copy`; the token's fight
/// trigger, if any, targets `fight`. Returns the tokens P0 ends with after everything
/// resolves.
fn mythos(
    t: &mut TestGame,
    target: ObjectId,
    red_green: bool,
    copy: bool,
    fight: Option<ObjectId>,
) -> Vec<ObjectId> {
    supported("Mythos of Illuna");
    let before = t.g.battlefield.clone();
    mana(t, P0, ManaType::U, 2);
    if red_green {
        mana(t, P0, ManaType::R, 1);
        mana(t, P0, ManaType::G, 1);
    } else {
        mana(t, P0, ManaType::C, 2);
    }
    let m = t.hand(P0, "Mythos of Illuna");
    let m = t.cast(P0, m).target(obj(target)).go();
    assert_eq!(t.g.players[0].mana_pool.total(), 0, "all the mana was spent");
    if copy {
        vengeance(t, P0, m);
    }
    if let Some(f) = fight {
        t.answer_targets(P0, &[obj(f)]);
    }
    t.resolve_all();
    new_tokens(t, P0, &before)
}

const FIGHT: &str = "fights up to one target creature";

#[test]
fn mythos_of_illuna_checks_the_mana_spent_and_a_copy_spent_none() {
    cr!("707.10", "614.1a", "601.2h", "707.9b");
    ruling!(
        "Mythos of Illuna",
        "If an effect copies the Mythos spell, no mana was spent to cast the copy, so the copy won't receive the bonus."
    );
    ruling!(
        "Mythos of Illuna",
        "The abilities of the Mythos check what colors of mana were spent to cast the spell. It's not an alternative cost to cast the spell."
    );
    ruling!(
        "Mythos of Illuna",
        "If {R}{G} was spent to cast the spell, the token has the extra ability. If something copies the token later, the copy will also have that ability."
    );
    // {R}{G} spent: the token has the fight ability (and fights); {C}{C}: it doesn't.
    for red_green in [true, false] {
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P1, "Hill Giant");
        let ogre = t.battlefield(P1, "Gray Ogre");
        let toks = mythos(&mut t, giant, red_green, false, Some(ogre));
        assert_eq!(toks.len(), 1);
        assert_eq!(has_ability(&t, toks[0], FIGHT), red_green, "{red_green}");
        assert_eq!(!t.on_battlefield(ogre), red_green, "{red_green}");
    }
    // Copied: the copy's token has no fight ability; the original's has.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let toks = mythos(&mut t, bears, true, true, None);
    assert_eq!(toks.len(), 2);
    let with: Vec<bool> = toks.iter().map(|k| has_ability(&t, *k, FIGHT)).collect();
    assert_eq!(with.iter().filter(|b| **b).count(), 1, "{with:?}");
    // A copy of the token with the extra ability has it too (CR 707.2: the ability is
    // part of its copiable values).
    let with_ability = toks[with.iter().position(|b| *b).unwrap()];
    let before = t.g.battlefield.clone();
    lots(&mut t, P0);
    let m = t.hand(P0, "Mythos of Illuna");
    t.g.players[0].mana_pool.mana.clear();
    mana(&mut t, P0, ManaType::U, 4);
    t.cast(P0, m).target(obj(with_ability)).go();
    t.resolve_all();
    let copies = new_tokens(&t, P0, &before);
    assert_eq!(copies.len(), 1);
    assert!(has_ability(&t, copies[0], FIGHT));
}

#[test]
fn copies_keep_the_originals_x() {
    cr!("707.10", "707.2");
    ruling!(
        "Increasing Vengeance",
        "If the spell Increasing Vengeance copies has an X whose value was determined as it was cast (like Earthquake does), the copy has the same value of X."
    );
    ruling!(
        "Tomb of Horrors Adventurer",
        "If the spell Tomb of Horrors Adventurer copies has an X whose value was determined as it was cast (like Fireball does), the copy has the same value of X."
    );
    supported("Earthquake");
    supported("Tomb of Horrors Adventurer");
    for copier in ["Increasing Vengeance", "Tomb of Horrors Adventurer"] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        if copier == "Tomb of Horrors Adventurer" {
            t.battlefield(P0, copier);
            cast_at(&mut t, P0, "Opt", &[]);
            t.resolve_all();
        }
        lots(&mut t, P0);
        let q = t.hand(P0, "Earthquake");
        let q = t.cast(P0, q).x(2).go();
        t.settle();
        if copier == "Increasing Vengeance" {
            vengeance(&mut t, P0, q);
        }
        let from = t.asked().len();
        t.resolve(); // the copier: creates the copy
        let copies = spell_copies(&t);
        assert_eq!(copies.len(), 1, "{copier}");
        assert_eq!(x_of(&t, copies[0]), Some(2), "{copier}");
        assert!(!asked_x(&t, P0, from), "{copier}: X is copied, not chosen");
        t.resolve_all();
        assert_eq!(t.life(P1), 16, "{copier}");
        assert_eq!(t.life(P0), 16, "{copier}");
    }
}

#[test]
fn copies_keep_the_originals_modes() {
    cr!("707.10", "707.2");
    ruling!(
        "Increasing Vengeance",
        "If the spell Increasing Vengeance copies is modal (that is, it says “Choose one —” or the like), the copies will have the same mode(s). You can't choose different ones."
    );
    ruling!(
        "Tomb of Horrors Adventurer",
        "If the spell Tomb of Horrors Adventurer's ability copies is modal (that is, it has a bulleted list of choices), the copies will have the same mode(s). You can't choose different ones."
    );
    supported("Boros Charm");
    for copier in ["Increasing Vengeance", "Tomb of Horrors Adventurer"] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        if copier == "Tomb of Horrors Adventurer" {
            t.battlefield(P0, copier);
            cast_at(&mut t, P0, "Opt", &[]);
            t.resolve_all();
        }
        lots(&mut t, P0);
        // Mode 0: Boros Charm deals 4 damage to target player or planeswalker.
        let c = t.hand(P0, "Boros Charm");
        let c = t.cast(P0, c).modes(&[0]).target(pl(P1)).go();
        t.settle();
        if copier == "Increasing Vengeance" {
            vengeance(&mut t, P0, c);
        }
        let from = t.asked().len();
        t.resolve();
        let copies = spell_copies(&t);
        assert_eq!(copies.len(), 1, "{copier}");
        assert_eq!(modes_on_stack(&t, copies[0]), vec![0], "{copier}");
        assert!(!asked_modes(&t, P0, from), "{copier}: modes are copied, not chosen");
        t.resolve_all();
        assert_eq!(t.life(P1), 12, "{copier}");
    }
}

#[test]
fn copy_effects_copy_spells_without_targets_too() {
    cr!("707.10", "115.1");
    ruling!(
        "Increasing Vengeance",
        "Increasing Vengeance can target (and copy) any instant or sorcery spell you control, not just one with targets."
    );
    ruling!(
        "Sea Gate Stormcaller",
        "Sea Gate Stormcaller's delayed triggered ability will copy any instant or sorcery spell, not just one with targets."
    );
    supported("Divination");
    supported("Sea Gate Stormcaller");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let d = cast_at(&mut t, P0, "Divination", &[]);
    assert!(targets_of(&t, d).is_empty());
    vengeance(&mut t, P0, d);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 4);
    // Sea Gate Stormcaller copies the next Opt (no targets).
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.enter(P0, "Sea Gate Stormcaller");
    t.resolve_all();
    cast_at(&mut t, P0, "Opt", &[]);
    t.settle();
    assert_eq!(spell_copies(&t).len(), 0);
    t.resolve(); // the delayed trigger
    assert_eq!(spell_copies(&t).len(), 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2);
}

#[test]
fn tomb_of_horrors_adventurer_copies_a_spell_that_was_countered() {
    cr!("707.10", "608.2h", "603.2");
    ruling!(
        "Tomb of Horrors Adventurer",
        "The triggered ability will copy the spell that caused it to trigger even if that spell has been countered by the time that ability resolves."
    );
    supported("Counterspell");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Tomb of Horrors Adventurer");
    cast_at(&mut t, P0, "Opt", &[]);
    t.resolve_all();
    let shock = cast_at(&mut t, P0, "Shock", &[pl(P1)]);
    t.settle();
    assert_eq!(t.stack_len(), 2, "the trigger is above Shock");
    // P1 counters Shock while the trigger waits.
    cast_at(&mut t, P1, "Counterspell", &[obj(shock)]);
    t.resolve();
    assert_eq!(t.zone(shock), Zone::Graveyard(P0));
    assert_eq!(t.stack_len(), 1);
    t.resolve(); // the trigger: a copy of the countered Shock
    let copies = spell_copies(&t);
    assert_eq!(copies.len(), 1);
    assert_eq!(name_of(&t, copies[0]), "Shock");
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}
