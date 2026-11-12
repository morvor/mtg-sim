//! Rulings batch P157 — copies: Narset's Reversal (copying a spell, then returning it to
//! its owner's hand, CR 707.10, 704.5e), Sea Gate Stormcaller's delayed trigger, copies of
//! permanent spells becoming tokens (CR 707.10f, 608.3f), token copies of tokens (X is
//! 0, CR 107.3g), copied enters abilities (Mythos of Illuna), last known information of
//! a copied permanent (Scute Swarm), new targets for copies (Increasing Vengeance), and
//! base power and toughness set by Will, Scholar of Frost (CR 613.4b).

use crate::r_p157_common::*;
use crate::r_p180_common::plus;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The copies of spells on the stack, bottom first.
fn stack_copies(t: &TestGame) -> Vec<ObjectId> {
    crate::r_s25_common::spell_copies(t)
}

#[test]
fn narsets_reversal_returns_even_an_uncounterable_spell() {
    cr!("707.10", "608.2c", "701.6a");
    ruling!(
        "Narset's Reversal",
        "If a spell is returned to its owner’s hand, it’s removed from the stack and thus will not resolve. The spell isn’t countered; it just no longer exists. This works against a spell that can’t be countered."
    );
    supported("Narset's Reversal");
    let mut t = TestGame::new(2);
    let rage = cast_at(&mut t, P1, "Urza's Rage", &[pl(P0)]);
    // The copy is P0's, aimed at P1.
    cast_at(&mut t, P0, "Narset's Reversal", &[obj(rage)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[pl(P1)]);
    t.resolve();
    assert!(t.in_hand(P1, "Urza's Rage"), "returned to its owner's hand");
    assert!(!on_stack(&t, rage));
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 17);
    // It wasn't countered: no "countered" event for it.
    assert!(!t.g.turn_events.iter().any(|e| matches!(e,
        mtg_engine::events::Event::Countered { .. })));
}

#[test]
fn narsets_reversal_returning_a_copy_makes_it_cease_to_exist() {
    cr!("704.5e", "707.10a");
    ruling!(
        "Narset's Reversal",
        "If a copy of a spell is returned to its owner’s hand, it’s moved there, then it will cease to exist as a state-based action."
    );
    let mut t = TestGame::new(2);
    let shock = cast_at(&mut t, P1, "Shock", &[pl(P0)]);
    // P1 copies its own Shock with Twincast (the copy also at P0).
    t.answer_yes(P1, false);
    cast_at(&mut t, P1, "Twincast", &[obj(shock)]);
    t.resolve();
    let copies = stack_copies(&t);
    assert_eq!(copies.len(), 1);
    let copy = copies[0];
    let hand = t.hand_size(P1);
    // P0 reverses the copy: P0's own copy of it is aimed at P1, and the copy returns to
    // P1's hand.
    cast_at(&mut t, P0, "Narset's Reversal", &[obj(copy)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[pl(P1)]);
    t.resolve();
    assert!(!on_stack(&t, copy));
    assert_eq!(t.hand_size(P1), hand, "the copy ceased to exist");
    assert!(!t.in_hand(P1, "Shock"));
    t.resolve_all();
    // P0's copy hit P1; the original Shock hit P0.
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 18);
}

#[test]
fn narsets_reversal_copies_any_instant_or_sorcery_and_keeps_x() {
    cr!("707.10", "707.10c", "107.3a");
    ruling!(
        "Narset's Reversal",
        "Narset’s Reversal can copy any instant or sorcery spell, not just one with targets."
    );
    ruling!(
        "Narset's Reversal",
        "If the spell that’s copied has an X whose value was determined as it was cast (like Finale of Promise does), the copy will have the same value of X."
    );
    // No targets: Divination.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let div = cast_at(&mut t, P1, "Divination", &[]);
    let hand = t.hand_size(P0);
    cast_at(&mut t, P0, "Narset's Reversal", &[obj(div)]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    assert!(t.in_hand(P1, "Divination"));
    // X: P1's Blaze with X = 3 at P0's Craw Wurm; P0's copy at P1 deals 3.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Craw Wurm");
    t.set_step(P1, Step::PrecombatMain);
    lots(&mut t, P1);
    let b = t.hand(P1, "Blaze");
    let blaze = t.cast(P1, b).x(3).target(wurm).go();
    cast_at(&mut t, P0, "Narset's Reversal", &[obj(blaze)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.obj_now(wurm).damage, 0);
    assert!(t.in_hand(P1, "Blaze"));
}

#[test]
fn hazel_and_octomancer_token_copies_have_x_zero() {
    cr!("107.3g", "202.3e", "707.2");
    ruling!(
        "Hazel of the Rootbloom",
        "If the copied token has {X} in its mana cost, X is 0."
    );
    ruling!("Octomancer", "If the copied token has {X} in its mana cost, X is 0.");
    supported("Hazel of the Rootbloom");
    supported("Octomancer");
    for name in ["Hazel of the Rootbloom", "Octomancer"] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, name);
        // A token copy of Ingenious Prodigy ({X}{U}), made this turn by Mythos of Illuna.
        let prodigy = t.battlefield(P0, "Ingenious Prodigy");
        cast_at(&mut t, P0, "Mythos of Illuna", &[obj(prodigy)]);
        t.resolve_all();
        let tk = tokens(&t, P0);
        assert_eq!(tk.len(), 1);
        t.answer_targets(P0, &[obj(tk[0])]);
        end_step(&mut t);
        let tk: Vec<_> = tokens(&t, P0)
            .into_iter()
            .filter(|id| *id != tk[0])
            .collect();
        assert_eq!(tk.len(), 1, "{name}");
        assert_eq!(t.g.mana_value_of(tk[0]), 1, "{name}: {{X}}{{U}} with X = 0");
        assert_eq!(plus(&t, tk[0]), 0, "{name}: X = 0 counters");
    }
}

#[test]
fn sea_gate_stormcaller_copies_even_if_it_left_or_the_spell_was_countered() {
    cr!("113.7a", "603.7a", "707.10", "608.2h");
    ruling!(
        "Sea Gate Stormcaller",
        "A copy is created even if Sea Gate Stormcaller leaves the battlefield before its delayed triggered ability is created."
    );
    ruling!(
        "Sea Gate Stormcaller",
        "A copy is created even if the spell that caused Sea Gate Stormcaller's delayed triggered ability to trigger has been countered by the time that ability resolves. The copy resolves before the original spell."
    );
    supported("Sea Gate Stormcaller");
    // It leaves before its enters ability resolves.
    let mut t = TestGame::new(2);
    let s = t.enter(P0, "Sea Gate Stormcaller");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    kill(&mut t, s);
    t.resolve_all();
    t.answer_yes(P0, false);
    let shock = cast_at(&mut t, P0, "Shock", &[pl(P1)]);
    t.settle();
    t.resolve();
    // The copy is above the original: it resolves first.
    let copies = stack_copies(&t);
    assert_eq!(copies.len(), 1);
    assert_eq!(t.g.stack, vec![shock, copies[0]]);
    t.resolve();
    assert_eq!(t.life(P1), 18);
    assert!(on_stack(&t, shock));
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    // The spell is countered before the delayed trigger resolves.
    let mut t = TestGame::new(2);
    t.enter(P0, "Sea Gate Stormcaller");
    t.resolve_all();
    t.answer_yes(P0, false);
    let shock = cast_at(&mut t, P0, "Shock", &[pl(P1)]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    cast_at(&mut t, P1, "Counterspell", &[obj(shock)]);
    t.resolve();
    assert!(!on_stack(&t, shock));
    t.resolve_all();
    assert_eq!(t.life(P1), 18, "the copy still resolved");
}

#[test]
fn sea_gate_stormcaller_x_counts_for_mana_value_and_the_copy_keeps_it() {
    cr!("202.3e", "707.10c");
    ruling!(
        "Sea Gate Stormcaller",
        "For spells with {X} in their mana costs, use the value chosen for X to determine the spell's mana value. If it's copied, the copy has the same value of X."
    );
    // Blaze with X = 2 has mana value 3: not copied, and the trigger is still waiting.
    let mut t = TestGame::new(2);
    t.enter(P0, "Sea Gate Stormcaller");
    t.resolve_all();
    lots(&mut t, P0);
    let b = t.hand(P0, "Blaze");
    t.cast(P0, b).x(2).target(P1).go();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // With X = 1 (mana value 2): copied, the copy with X = 1.
    t.answer_yes(P0, false);
    lots(&mut t, P0);
    let b = t.hand(P0, "Blaze");
    t.cast(P0, b).x(1).target(P1).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn tomb_of_horrors_adventurer_copied_permanent_spells_arent_created_tokens() {
    cr!("707.10f", "608.3f", "614.1a");
    ruling!(
        "Tomb of Horrors Adventurer",
        "A resolving copy of a permanent spell becomes a token, so the token isn't “created.” Effects that care about a token being created won't interact with a token that enters the battlefield because the triggered ability copied a permanent spell."
    );
    supported("Tomb of Horrors Adventurer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tomb of Horrors Adventurer");
    t.battlefield(P0, "Anointed Procession");
    cast_at(&mut t, P0, "Shock", &[pl(P1)]);
    t.resolve_all();
    cast_at(&mut t, P0, "Grizzly Bears", &[]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    let bears = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(bears.len(), 2, "the card and one token, not doubled");
    assert_eq!(tokens(&t, P0).len(), 1);
}

#[test]
fn mythos_of_illuna_copied_enters_abilities_work() {
    cr!("707.2", "603.6a", "614.1c");
    ruling!(
        "Mythos of Illuna",
        "Any enters-the-battlefield abilities of the copied permanent will trigger when the token enters the battlefield. Any \"As [this permanent] enters the battlefield\" or \"[This permanent] enters the battlefield with\" abilities of the copied permanent will also work."
    );
    supported("Mythos of Illuna");
    // "When ~ enters, draw a card."
    let mut t = TestGame::new(2);
    let v = t.battlefield(P1, "Elvish Visionary");
    let hand = t.hand_size(P0);
    cast_at(&mut t, P0, "Mythos of Illuna", &[obj(v)]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // "This artifact enters with three wish counters on it."
    let mut t = TestGame::new(2);
    let w = t.battlefield(P1, "Wishclaw Talisman");
    cast_at(&mut t, P0, "Mythos of Illuna", &[obj(w)]);
    t.resolve_all();
    let tk = tokens(&t, P0);
    assert_eq!(tk.len(), 1);
    assert_eq!(t.counters(tk[0], "wish"), 3);
    // "As this creature enters, choose a creature type."
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Adaptive Automaton");
    let m = mark(&t);
    cast_at(&mut t, P0, "Mythos of Illuna", &[obj(a)]);
    t.resolve_all();
    let named = t.asked()[m..].iter().any(|(p, d)| {
        *p == P0
            && matches!(d, mtg_engine::decision::Decision::ChooseOption { prompt, .. }
                if prompt.to_lowercase().contains("type"))
    });
    assert!(named, "the creature type was chosen as the token entered");
}

#[test]
fn scute_swarm_copy_uses_last_known_values() {
    cr!("707.2", "608.2h", "113.7a");
    ruling!(
        "Scute Swarm",
        "If Scute Swarm leaves the battlefield before its triggered ability resolves, the token will still enter the battlefield as a copy of Scute Swarm, using Scute Swarm's copiable values from when it was last on the battlefield."
    );
    supported("Scute Swarm");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Scute Swarm");
    t.lands(P0, "Forest", 5);
    let f = t.hand(P0, "Forest");
    t.play_land(P0, f).unwrap();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    kill(&mut t, s);
    t.resolve_all();
    let tk = tokens(&t, P0);
    assert_eq!(tk.len(), 1);
    assert_eq!(t.obj_now(tk[0]).chars.name, "Scute Swarm");
}

#[test]
fn increasing_vengeance_copies_new_targets_each() {
    cr!("707.10c", "115.7");
    ruling!(
        "Increasing Vengeance",
        "Each of the copies will have the same targets as the spell it's copying unless you choose new ones. You may change any number of the targets, including all of them or none of them. If, for one of the targets, you can't choose a new legal target, then it remains unchanged (even if the current target is illegal). If there are two copies, you may change the targets of each of them to different legal targets."
    );
    supported("Increasing Vengeance");
    // Flashback: two copies of Shock (at P1): one to the Giant, one to the Wurm.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let shock = cast_at(&mut t, P0, "Shock", &[pl(P1)]);
    t.graveyard(P0, "Increasing Vengeance");
    lots(&mut t, P0);
    let iv = t.g.player(P0).graveyard[0];
    t.answer_targets(P0, &[obj(shock)]);
    t.cast(P0, iv)
        .method(CastMethod::Keyword(
            mtg_engine::keywords::KeywordKind::Flashback,
        ))
        .go();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(giant)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(wurm)]);
    t.resolve();
    let copies = stack_copies(&t);
    assert_eq!(copies.len(), 2);
    let mut targets: Vec<_> = copies.iter().map(|c| stack_targets(&t, *c)).collect();
    targets.sort_by_key(|v| format!("{v:?}"));
    let mut want = vec![vec![obj(giant)], vec![obj(wurm)]];
    want.sort_by_key(|v| format!("{v:?}"));
    assert_eq!(targets, want, "different new targets");
    t.resolve_all();
    assert_eq!(t.obj_now(giant).damage, 2);
    assert_eq!(t.obj_now(wurm).damage, 2);
    assert_eq!(t.life(P1), 18, "the original kept its target");
    // No new legal target: the copy keeps the (now illegal) one.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let gg = cast_at(&mut t, P0, "Giant Growth", &[obj(bears)]);
    cast_at(&mut t, P0, "Increasing Vengeance", &[obj(gg)]);
    t.answer_yes(P0, true);
    kill(&mut t, bears);
    t.resolve();
    let copies = stack_copies(&t);
    assert_eq!(copies.len(), 1);
    assert_eq!(stack_targets(&t, copies[0]), vec![obj(bears)]);
}

#[test]
fn rowans_emblem_pays_two_only_once() {
    cr!("118.12", "707.10");
    ruling!(
        "Rowan, Scholar of Sparks // Will, Scholar of Frost",
        "For Rowan’s emblem, you can pay {2} only once. You can’t pay more to copy the spell more than once."
    );
    supported("Rowan, Scholar of Sparks // Will, Scholar of Frost");
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Rowan, Scholar of Sparks // Will, Scholar of Frost");
    t.g.objects[r.0 as usize]
        .counters
        .insert(mtg_engine::types::counters::LOYALTY.into(), 5);
    act(&mut t, P0, r, 1, &[]);
    assert_eq!(t.counters(r, "loyalty"), 1, "the emblem's ability was activated");
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    cast_at(&mut t, P0, "Shock", &[pl(P1)]);
    t.settle();
    t.resolve();
    assert_eq!(stack_copies(&t).len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn wills_base_power_and_toughness_under_other_modifications() {
    cr!("613.4", "613.4b", "613.4c");
    ruling!(
        "Rowan, Scholar of Sparks // Will, Scholar of Frost",
        "For Will’s first loyalty ability, any effects that modify a creature’s power and/or toughness without setting them to a specific value (i.e. ones that don’t affect base power and/or toughness) will apply after its base power and toughness are set, regardless of the order those effects were created. The same is true for counters that modify its power and toughness."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    cast_at(&mut t, P0, "Giant Growth", &[obj(giant)]);
    t.resolve_all();
    t.g.objects[giant.0 as usize]
        .counters
        .insert(mtg_engine::types::counters::PLUS1.into(), 1);
    t.g.recompute();
    assert_eq!(t.pt(giant), (7, 7));
    lots(&mut t, P0);
    let card = t.hand(P0, "Rowan, Scholar of Sparks // Will, Scholar of Frost");
    t.cast(P0, card).method(CastMethod::Half(1)).go();
    t.resolve_all();
    let will = t.named_on_battlefield("Will, Scholar of Frost");
    assert_eq!(will.len(), 1);
    act(&mut t, P0, will[0], 0, &[obj(giant)]);
    // 0/2, then +3/+3 and the +1/+1 counter.
    assert_eq!(t.pt(giant), (4, 6));
}
