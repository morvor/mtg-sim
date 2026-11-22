//! Rulings batch S07 — escape (CR 702.138): "You may cast this card from your graveyard
//! by paying [cost] rather than paying its mana cost."

use crate::r_s01_common::*;
use crate::r_s02_common::*;
use crate::r_s04_common::*;
use crate::r_s07_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::game::Game;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::*;

const ESCAPE: CastMethod = CastMethod::Keyword(KeywordKind::Escape);
const FLASHBACK: CastMethod = CastMethod::Keyword(KeywordKind::Flashback);
const EVOKE: CastMethod = CastMethod::Keyword(KeywordKind::Evoke);

#[test]
fn with_two_permissions_you_choose_which_one_to_apply() {
    cr!("702.138a", "702.34a", "601.2b");
    ruling!(
        "Underworld Breach",
        "If a card has multiple abilities giving you permission to cast it, such as two escape abilities or an escape ability and a flashback ability, you choose which one to apply. The others have no effect."
    );
    supported("Underworld Breach");
    supported("Think Twice");
    // Underworld Breach gives Think Twice ({1}{U}, flashback {2}{U}) escape: its mana cost
    // plus exiling three other cards from the graveyard.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Underworld Breach");
    t.lands(P0, "Island", 3);
    let tt = t.graveyard(P0, "Think Twice");
    graveyard_n(&mut t, P0, "Grizzly Bears", 3);
    let methods = cast_methods(&mut t, P0, tt);
    assert!(methods.contains(&ESCAPE) && methods.contains(&FLASHBACK));
    // Cast with escape: flashback has no effect, so it goes back to the graveyard.
    let hand = t.hand_size(P0);
    t.cast(P0, tt).method(ESCAPE).go();
    assert_eq!(t.g.exile.len(), 3);
    assert_eq!(untapped_lands(&t, P0), 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert!(t.in_graveyard(P0, "Think Twice"));
    // Cast with flashback: escape has no effect (no cards are exiled for it), and it's
    // exiled as it resolves.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Underworld Breach");
    t.lands(P0, "Island", 3);
    let tt = t.graveyard(P0, "Think Twice");
    graveyard_n(&mut t, P0, "Grizzly Bears", 3);
    t.cast(P0, tt).method(FLASHBACK).go();
    assert_eq!(t.g.exile.len(), 0);
    assert_eq!(untapped_lands(&t, P0), 0);
    t.resolve_all();
    assert!(t.in_exile("Think Twice"));
    assert_eq!(t.graveyard_size(P0), 3);
}

/// Whether Woe Strider is on the stack (and not in P0's graveyard).
fn strider_on_stack(g: &Game) -> bool {
    g.stack
        .iter()
        .any(|s| g.obj(*s).chars.name.as_str() == "Woe Strider")
        && !g
            .player(P0)
            .graveyard
            .iter()
            .any(|c| g.obj(*c).chars.name.as_str() == "Woe Strider")
}

#[test]
fn an_escaping_spell_moves_to_the_stack_as_you_begin_casting_it() {
    cr!("601.2a", "601.2h", "702.138a");
    ruling!(
        "Woe Strider",
        "Once you begin casting a spell with escape, it immediately moves to the stack. Players can't take any other actions until you're done casting the spell."
    );
    supported("Woe Strider");
    // Woe Strider: escape—{3}{B}{B}, exile four other cards from your graveyard. Five other
    // cards: P0 chooses the four to exile while Woe Strider is already on the stack.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 5);
    let strider = t.graveyard(P0, "Woe Strider");
    graveyard_n(&mut t, P0, "Grizzly Bears", 5);
    let is_choice = |d: &Decision| matches!(d, Decision::ChooseEntities { .. });
    let seen = watch(&mut t, P0, is_choice, strider_on_stack);
    let from = t.asked().len();
    t.cast(P0, strider).method(ESCAPE).go();
    let seen = seen.lock().unwrap().clone();
    assert!(!seen.is_empty());
    assert!(seen.iter().all(|x| *x));
    // Nobody got priority while it was being cast.
    assert_eq!(count_asked(&t, from, is_priority), 0);
    assert_eq!(t.g.exile.len(), 4);
    assert_eq!(t.graveyard_size(P0), 1);
    t.resolve_all();
    assert!(t.on_battlefield(strider));
}

#[test]
fn escape_cant_be_combined_with_other_alternative_costs_but_additional_costs_are_paid() {
    cr!("118.9a", "601.2f", "702.138a");
    ruling!(
        "Underworld Breach",
        "If you cast a spell with its escape permission, you can't choose to apply any other alternative costs or to cast it without paying its mana cost. If it has any additional costs, you must pay those."
    );
    supported("Mulldrifter");
    supported("Village Rites");
    // Mulldrifter (evoke {2}{U}) in the graveyard with Underworld Breach: it can be cast
    // with escape, not for its evoke cost.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Underworld Breach");
    t.lands(P0, "Island", 5);
    let drifter = t.graveyard(P0, "Mulldrifter");
    graveyard_n(&mut t, P0, "Grizzly Bears", 3);
    let methods = cast_methods(&mut t, P0, drifter);
    assert_eq!(methods, vec![ESCAPE]);
    assert!(!methods.contains(&EVOKE));
    // Village Rites ({B} instant; "As an additional cost to cast this spell, sacrifice a
    // creature.") cast with escape: the additional cost is paid too.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Underworld Breach");
    t.lands(P0, "Swamp", 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let rites = t.graveyard(P0, "Village Rites");
    graveyard_n(&mut t, P0, "Llanowar Elves", 3);
    let hand = t.hand_size(P0);
    t.cast(P0, rites).method(ESCAPE).go();
    assert!(!t.on_battlefield(bears));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.g.exile.len(), 3);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    // Without a creature to sacrifice, it can't be cast with escape.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Underworld Breach");
    t.lands(P0, "Swamp", 1);
    let rites = t.graveyard(P0, "Village Rites");
    graveyard_n(&mut t, P0, "Llanowar Elves", 3);
    assert!(!cast_methods(&mut t, P0, rites).contains(&ESCAPE));
}

#[test]
fn you_can_escape_a_card_right_after_it_is_put_into_your_graveyard() {
    cr!("117.3b", "117.3c", "702.138a");
    ruling!(
        "Woe Strider",
        "If a card with escape is put into your graveyard during your turn, you'll be able to cast it right away if it's legal to do so, before an opponent can take any actions."
    );
    supported("Murder");
    // P0 casts Murder on their own Woe Strider in their main phase. After Murder resolves,
    // P0 gets priority before P1 does and escapes the Strider.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Crook of Condemnation");
    t.lands(P1, "Wastes", 1);
    let strider = t.battlefield(P0, "Woe Strider");
    graveyard_n(&mut t, P0, "Grizzly Bears", 4);
    give_mana_for(&mut t, P0, "Murder");
    let murder = t.hand(P0, "Murder");
    t.cast(P0, murder).target(strider).go();
    let in_graveyard = |g: &Game| {
        g.player(P0)
            .graveyard
            .iter()
            .any(|c| g.obj(*c).chars.name.as_str() == "Woe Strider")
    };
    let seen1 = watch(&mut t, P1, is_priority, in_graveyard);
    let ok = t.g.run_until(1000, |g| {
        in_graveyard(g) && g.stack.is_empty() && g.turn.priority.is_some()
    });
    assert!(ok);
    assert_eq!(t.g.turn.priority, Some(P0));
    assert!(!seen1.lock().unwrap().iter().any(|x| *x));
    let card = t.g.current(strider);
    t.lands(P0, "Swamp", 5);
    assert!(can_cast(&mut t, P0, card, ESCAPE));
    t.cast(P0, card).method(ESCAPE).go();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(t.on_battlefield(strider));
    // It escaped with two +1/+1 counters.
    assert_eq!(t.pt(strider), (5, 4));
}

#[test]
fn an_escaped_permanent_returns_to_the_graveyard_and_can_escape_again() {
    cr!("702.138a", "702.138b", "400.7");
    ruling!(
        "Phlage, Titan of Fire's Fury",
        "After an escaped spell resolves, it returns to its owner's graveyard if it's not a permanent spell. If it is a permanent spell, it enters the battlefield and will return to its owner's graveyard if it dies later. It can escape again."
    );
    supported("Phlage, Titan of Fire's Fury");
    // Phlage: "When Phlage enters, sacrifice it unless it escaped. Whenever Phlage enters or
    // attacks, it deals 3 damage to any target and you gain 3 life. Escape—{R}{R}{W}{W},
    // Exile five other cards from your graveyard."
    let mut t = TestGame::new(2);
    let phlage = t.graveyard(P0, "Phlage, Titan of Fire's Fury");
    graveyard_n(&mut t, P0, "Grizzly Bears", 5);
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Plains", 2);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, phlage).method(ESCAPE).go();
    t.resolve_all();
    // It escaped: it stays on the battlefield.
    assert!(t.on_battlefield(phlage));
    assert_eq!(t.life(P1), 17);
    // It dies and returns to its owner's graveyard.
    destroy(&mut t, phlage);
    let card = t.g.current(phlage);
    assert!(t.in_graveyard(P0, "Phlage, Titan of Fire's Fury"));
    // With five other cards and the mana again, it escapes again.
    graveyard_n(&mut t, P0, "Llanowar Elves", 5);
    for l in t.g.battlefield.clone() {
        t.g.objects[l.0 as usize].tapped = false;
    }
    assert!(can_cast(&mut t, P0, card, ESCAPE));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, card).method(ESCAPE).go();
    t.resolve_all();
    assert!(t.on_battlefield(card));
    assert_eq!(t.life(P1), 14);
    assert_eq!(t.g.exile.len(), 10);
}
