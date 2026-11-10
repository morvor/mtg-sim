//! Rulings batch P154 — abilities that trigger on casting a spell and use its mana value:
//! the trigger goes on the stack above the spell and resolves first (CR 603.3, 608.1),
//! also when the spell has been countered meanwhile, using the spell's last known mana
//! value (CR 608.2h); what counts as "casting your first/second spell" (CR 601.2i,
//! 603.2); and "as though it had flash" not changing sorcery-speed abilities (CR 702.8,
//! 602.5a).

use crate::r_p154_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// P1 counters the spell `spell` with Cancel (cast and resolved at once).
fn p1_cancels(t: &mut TestGame, spell: ObjectId) {
    mana(t, P1, "1UU");
    let cancel = t.hand(P1, "Cancel");
    t.cast(P1, cancel).target(obj(spell)).go();
    t.resolve();
    assert_eq!(t.zone(spell), Zone::Graveyard(t.g.obj(spell).owner));
}

#[test]
fn veiled_sentry_uses_the_mana_value_of_a_spell_that_left_the_stack() {
    cr!("603.3", "608.2h", "603.4");
    ruling!(
        "Veiled Sentry",
        "If the spell is no longer on the stack when the triggered ability resolves, the mana value of the spell is still remembered."
    );
    supported("Veiled Sentry");
    // The Giant still on the stack: a 4/4.
    let mut t = TestGame::new(2);
    let sentry = t.battlefield(P0, "Veiled Sentry");
    t.set_step(P1, Step::PrecombatMain);
    cast(&mut t, P1, "Hill Giant", "3R", &[]);
    t.resolve();
    assert_eq!(t.pt(sentry), (4, 4), "{}", t.dump_log());
    // Countered before the trigger resolves: still a 4/4.
    let mut t = TestGame::new(2);
    let sentry = t.battlefield(P0, "Veiled Sentry");
    t.set_step(P1, Step::PrecombatMain);
    let giant = cast(&mut t, P1, "Hill Giant", "3R", &[]);
    t.settle();
    assert_eq!(t.stack_len(), 2, "the Sentry's trigger above the Giant");
    // P0 counters the Giant before the trigger resolves (Cancel resolves first).
    mana(&mut t, P0, "1UU");
    let cancel = t.hand(P0, "Cancel");
    t.cast(P0, cancel).target(obj(giant)).go();
    t.resolve();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    t.resolve_all();
    assert!(t.obj_now(sentry).is(CardType::Creature));
    assert_eq!(t.pt(sentry), (4, 4), "{}", t.dump_log());
}

#[test]
fn krile_baldesions_trigger_resolves_before_the_spell_even_if_it_is_countered() {
    cr!("603.3", "608.1", "608.2h");
    ruling!(
        "Krile Baldesion",
        "Krile Baldesion's last ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack."
    );
    supported("Krile Baldesion");
    // Shock (mana value 1) is countered; Llanowar Elves (1) still returns to hand.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Krile Baldesion");
    let elves = t.graveyard(P0, "Llanowar Elves");
    t.graveyard(P0, "Hill Giant");
    let shock = cast(&mut t, P0, "Shock", "R", &[pl(P1)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(elves)]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    p1_cancels(&mut t, shock);
    t.resolve_all();
    assert!(t.in_hand(P0, "Llanowar Elves"));
    assert_eq!(t.life(P1), 20);
    // Not countered: the Elves are in hand while Shock is still on the stack.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Krile Baldesion");
    let elves = t.graveyard(P0, "Llanowar Elves");
    let shock = cast(&mut t, P0, "Shock", "R", &[pl(P1)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(elves)]);
    t.resolve();
    assert!(t.in_hand(P0, "Llanowar Elves"));
    assert_eq!(t.zone(shock), Zone::Stack);
    assert_eq!(t.life(P1), 20);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn cast_triggers_resolve_before_the_spell_and_survive_its_countering() {
    cr!("603.3", "608.1", "608.2h");
    ruling!(
        "Metallurgic Summonings",
        "Metallurgic Summonings's first ability resolves before the spell that caused it to trigger."
    );
    ruling!(
        "Renegade Bull",
        "Renegade Bull's second ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack without resolving."
    );
    ruling!(
        "Skittering Cicada",
        "Skittering Cicada's last ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    supported("Metallurgic Summonings");
    supported("Renegade Bull");
    supported("Skittering Cicada");
    // Metallurgic Summonings: a 1/1 Construct exists while Shock is still on the stack.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Metallurgic Summonings");
    let shock = cast(&mut t, P0, "Shock", "R", &[pl(P1)]);
    t.resolve();
    assert_eq!(t.zone(shock), Zone::Stack);
    let tok = tokens(&t, P0);
    assert_eq!(tok.len(), 1);
    assert_eq!(t.pt(tok[0]), (1, 1));
    // Renegade Bull (0/5): +1/+0 from a Shock that is countered.
    let mut t = TestGame::new(2);
    let bull = t.battlefield(P0, "Renegade Bull");
    let shock = cast(&mut t, P0, "Shock", "R", &[pl(P1)]);
    t.settle();
    p1_cancels(&mut t, shock);
    t.resolve_all();
    assert_eq!(t.pt(bull), (1, 5));
    // Skittering Cicada (2/2): +2/+2 and trample from a Mind Stone that is countered.
    let mut t = TestGame::new(2);
    let cicada = t.battlefield(P0, "Skittering Cicada");
    let stone = cast(&mut t, P0, "Mind Stone", "2", &[]);
    t.settle();
    p1_cancels(&mut t, stone);
    t.resolve_all();
    assert_eq!(t.pt(cicada), (4, 4));
    assert!(t.obj_now(cicada).chars.has_keyword(KeywordKind::Trample));
}

#[test]
fn skittering_cicada_gives_flash_to_colorless_spells_not_to_sorcery_speed_abilities() {
    cr!("702.8a", "602.5a", "117.1a");
    ruling!(
        "Skittering Cicada",
        "Skittering Cicada's second ability applies only to casting spells. It does not, for example, change when you may activate abilities that can be activated \"only as a sorcery.\""
    );
    supported("Skittering Cicada");
    supported("Bonesplitter");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Skittering Cicada");
    let splitter = t.battlefield(P0, "Bonesplitter");
    t.battlefield(P0, "Grizzly Bears");
    let stone = t.hand(P0, "Mind Stone");
    mana(&mut t, P0, "3");
    // On P1's turn: Mind Stone (colorless) can be cast; Bonesplitter's equip can't be
    // activated.
    t.set_step(P1, Step::PrecombatMain);
    assert!(!can_activate(&mut t, P0, splitter), "equip only as a sorcery");
    assert!(can_cast(&mut t, P0, stone, mtg_engine::object::CastMethod::Normal));
    t.cast(P0, stone).go();
    t.resolve_all();
    assert_eq!(count_named(&t, "Mind Stone"), 1);
}

#[test]
fn rashmis_trigger_and_the_spell_it_casts_resolve_before_the_first_spell() {
    cr!("603.3", "608.1", "608.2g");
    ruling!(
        "Rashmi, Eternities Crafter",
        "Rashmi's ability resolves before the spell that caused it to trigger. If you cast a spell using the ability, it will also resolve before the initial spell."
    );
    supported("Rashmi, Eternities Crafter");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rashmi, Eternities Crafter");
    stack_library(&mut t, P0, &["Grizzly Bears", "Island"]);
    t.answer_yes(P0, true);
    let giant = cast(&mut t, P0, "Hill Giant", "3R", &[]);
    t.resolve();
    // The trigger resolved: the Bears are a spell above the Giant.
    assert_eq!(t.stack_len(), 2);
    assert_eq!(t.g.stack[0], giant);
    assert_eq!(count_named(&t, "Grizzly Bears"), 0);
    t.resolve();
    assert_eq!(count_named(&t, "Grizzly Bears"), 1);
    assert_eq!(t.zone(giant), Zone::Stack);
    t.resolve_all();
    assert_eq!(count_named(&t, "Hill Giant"), 1);
}

#[test]
fn rashmi_must_be_on_the_battlefield_as_the_first_spell_is_cast() {
    cr!("603.2", "601.2h", "601.2i");
    ruling!(
        "Rashmi, Eternities Crafter",
        "Rashmi has to be on the battlefield at the moment you cast your first spell. If that spell is Rashmi itself, Rashmi's ability can't trigger. If that spell causes Rashmi to leave the battlefield as an additional cost to cast it, Rashmi's ability can't trigger."
    );
    ruling!(
        "Rashmi, Eternities Crafter",
        "Rashmi's ability triggers when you cast your first spell each turn, not just on your turn."
    );
    supported("Rashmi, Eternities Crafter");
    supported("Eldritch Evolution");
    // Casting Rashmi itself: no trigger (the Bears stay on top of the library).
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Grizzly Bears", "Island"]);
    t.answer_yes(P0, true);
    cast_resolve(&mut t, P0, "Rashmi, Eternities Crafter", "2GU", &[]);
    assert_eq!(count_named(&t, "Grizzly Bears"), 0);
    assert_eq!(t.library_size(P0), 32);
    // Eldritch Evolution sacrificing Rashmi as an additional cost: no trigger.
    let mut t = TestGame::new(2);
    let rashmi = t.battlefield(P0, "Rashmi, Eternities Crafter");
    stack_library(&mut t, P0, &["Grizzly Bears", "Island", "Craw Wurm"]);
    t.answer_choose(P0, &[obj(rashmi)]);
    let from = t.asked().len();
    cast_resolve(&mut t, P0, "Eldritch Evolution", "1GG", &[]);
    assert!(t.in_graveyard(P0, "Rashmi, Eternities Crafter"));
    assert!(t.in_exile("Eldritch Evolution"));
    assert!(asked_of(&t, P0, from)
        .iter()
        .all(|d| !matches!(d, mtg_engine::decision::Decision::YesNo { .. })));
    assert!(!t.in_hand(P0, "Grizzly Bears"), "nothing revealed by Rashmi");
    // On P1's turn: P0's first spell (Shock) triggers; the Ornithopter (0 < 1) is cast.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rashmi, Eternities Crafter");
    stack_library(&mut t, P0, &["Ornithopter", "Island"]);
    t.set_step(P1, Step::PrecombatMain);
    t.answer_yes(P0, true);
    cast(&mut t, P0, "Shock", "R", &[pl(P1)]);
    t.resolve_all();
    assert_eq!(count_named(&t, "Ornithopter"), 1);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn naya_soulbeasts_cast_trigger_resolves_while_it_is_on_the_stack() {
    cr!("603.3", "608.1", "614.1c");
    ruling!(
        "Naya Soulbeast",
        "Naya Soulbeast's last ability triggers while Naya Soulbeast is on the stack. After that ability resolves, players will be able to cast spells and activate abilities knowing how many counters Naya Soulbeast will enter the battlefield with if it resolves."
    );
    supported("Naya Soulbeast");
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Hill Giant", "Island"]);
    stack_library(&mut t, P1, &["Forest", "Island"]);
    let beast = cast(&mut t, P0, "Naya Soulbeast", "6GG", &[]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(t.zone(beast), Zone::Stack, "the trigger resolved first");
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    let b = t.named_on_battlefield("Naya Soulbeast");
    assert_eq!(b.len(), 1);
    assert_eq!(t.counters(b[0], counters::PLUS1), 4, "Hill Giant 4 + Forest 0");
}

#[test]
fn dovescape_creates_the_birds_even_if_the_spell_isnt_countered_by_it() {
    cr!("608.2c", "608.2h", "701.6b");
    ruling!(
        "Dovescape",
        "The Bird tokens are created even if the spell isn't countered by Dovescape's ability. This may happen because the spell can't be countered, because it's already been countered, or because it's otherwise been removed from the stack by the time Dovescape's ability resolves."
    );
    supported("Dovescape");
    supported("Abrupt Decay");
    // Abrupt Decay (can't be countered, mana value 2): it resolves, and two Birds.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Dovescape");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast(&mut t, P0, "Abrupt Decay", "BG", &[obj(bears)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(tokens(&t, P0).len(), 2);
    // Shock, already countered when the trigger resolves (by P1's Draining Whelk, a
    // creature spell, which Dovescape doesn't trigger on): one Bird.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Dovescape");
    let shock = cast(&mut t, P0, "Shock", "R", &[pl(P1)]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    mana(&mut t, P1, "4UU");
    let whelk = t.hand(P1, "Draining Whelk");
    t.answer_targets(P1, &[obj(shock)]);
    t.cast(P1, whelk).go();
    t.resolve();
    t.resolve();
    assert!(t.in_graveyard(P0, "Shock"));
    assert_eq!(t.stack_len(), 1, "Dovescape's trigger for Shock");
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1);
    assert_eq!(t.counters(whelk, counters::PLUS1), 1);
    assert!(tokens(&t, P0).iter().all(|b| t.obj_now(*b).chars.has_subtype("Bird")));
}

#[test]
fn breeches_counts_spells_cast_before_it_entered_including_itself() {
    cr!("603.2", "601.2i");
    ruling!(
        "Breeches, the Blastmaker",
        "Spells that were cast before Breeches entered the battlefield count. If Breeches was the first spell you cast this turn, the next spell you cast this turn is your second spell."
    );
    supported("Breeches, the Blastmaker");
    // Breeches cast as the first spell; Shock is the second: with the Memnite sacrificed
    // and the flip lost, Breeches deals 1 (Shock's mana value) to P1.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Memnite");
    cast_resolve(&mut t, P0, "Breeches, the Blastmaker", "1UR", &[]);
    t.g.dice.loaded_coins.push_back(false);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[pl(P1)]);
    cast_resolve(&mut t, P0, "Shock", "R", &[pl(P1)]);
    assert!(t.in_graveyard(P0, "Memnite"));
    assert_eq!(t.life(P1), 17, "Shock and 1 from Breeches");
    // Opt cast, then Breeches enters (not cast), then Shock: Shock is the second spell.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Memnite");
    cast_resolve(&mut t, P0, "Opt", "U", &[]);
    t.enter(P0, "Breeches, the Blastmaker");
    t.g.dice.loaded_coins.push_back(false);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[pl(P1)]);
    cast_resolve(&mut t, P0, "Shock", "R", &[pl(P1)]);
    assert!(t.in_graveyard(P0, "Memnite"));
    assert_eq!(t.life(P1), 17);
}

#[test]
fn breeches_copy_may_have_new_legal_targets() {
    cr!("707.10", "707.10c", "115.1");
    ruling!(
        "Breeches, the Blastmaker",
        "If you win the flip, the copy created by Breeches's delayed triggered ability will have the same targets as the spell it's copying unless you choose new ones. You may change any number of the targets, including all of them or none of them. The new targets must be legal."
    );
    supported("Breeches, the Blastmaker");
    for (change, bears_dead, life) in [(false, false, 16), (true, true, 18)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Breeches, the Blastmaker");
        t.battlefield(P0, "Memnite");
        let bears = t.battlefield(P1, "Grizzly Bears");
        cast_resolve(&mut t, P0, "Opt", "U", &[]);
        t.g.dice.loaded_coins.push_back(true);
        t.answer_yes(P0, true);
        if change {
            change_copy_targets(&mut t, P0, &[Some(obj(bears))]);
        } else {
            keep_copy_targets(&mut t, P0);
        }
        cast_resolve(&mut t, P0, "Shock", "R", &[pl(P1)]);
        assert_eq!(t.life(P1), life, "change: {change}");
        assert_eq!(!t.on_battlefield(bears), bears_dead, "change: {change}");
    }
}

#[test]
fn rootha_checks_for_an_instant_or_sorcery_as_combat_begins() {
    cr!("603.4", "603.2");
    ruling!(
        "Rootha, Mastering the Moment",
        "Rootha's ability will check as your beginning of combat step starts to see if you've cast an instant or sorcery spell this turn. If you haven't, the ability won't trigger at all. It's not possible to cast an instant or sorcery spell during your beginning of combat step in time to have the ability trigger."
    );
    supported("Rootha, Mastering the Moment");
    // Nothing cast in the main phase: no trigger; a Shock cast in the beginning of combat
    // step is too late.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rootha, Mastering the Moment");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    assert_eq!(t.stack_len(), 0, "no trigger");
    cast_resolve(&mut t, P0, "Shock", "R", &[pl(P1)]);
    assert!(tokens(&t, P0).is_empty());
    // A Shock cast in the main phase: a 1/1 Elemental.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rootha, Mastering the Moment");
    cast_resolve(&mut t, P0, "Shock", "R", &[pl(P1)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    let tok = tokens(&t, P0);
    assert_eq!(tok.len(), 1);
    assert_eq!(t.pt(tok[0]), (1, 1));
}
