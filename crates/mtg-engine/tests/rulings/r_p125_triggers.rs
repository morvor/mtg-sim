//! Rulings batch P125 — triggered abilities and resolving effects: cast triggers resolve
//! before the spell, even if it's countered (CR 603.3, 113.7a); replacement effects as a
//! permanent enters (CR 614.12); base power/toughness set by a later effect (CR 613.4b);
//! effects that last indefinitely (CR 611.2a); copies of tokens (CR 707.2); targets of
//! one ability (CR 115.3); Aura spells with illegal targets (CR 608.3b); cumulative skip
//! effects (CR 614.10).

use crate::r_p125_common::*;
use crate::r_p130_common::loyalty;
use crate::r_s01_common::watch;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

fn modes(t: &mut TestGame, p: PlayerId, m: &[usize]) {
    t.answer(p, DecisionKind::Modes, Answer::Indices(m.to_vec()));
}

#[test]
fn radagast_trigger_resolves_even_if_the_spell_is_countered() {
    cr!("603.3", "113.7a");
    ruling!(
        "Radagast, Wizard of Wilds",
        "It will resolve even if the spell that caused it to trigger is countered or has otherwise left the stack."
    );
    supported("Radagast, Wizard of Wilds");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Radagast, Wizard of Wilds");
    modes(&mut t, P0, &[0]);
    let wurm = cast_new(&mut t, P0, "Craw Wurm", &[]);
    t.settle();
    assert_eq!(t.stack_len(), 2, "the trigger is on top of the Wurm");
    // P1 counters the Wurm while the trigger waits.
    cast_new(&mut t, P1, "Counterspell", &[obj(wurm)]);
    t.resolve();
    assert!(t.in_graveyard(P0, "Craw Wurm"));
    t.resolve_all();
    assert_eq!(crate::r_p108_common::tokens_with(&t, P0, "Beast"), 1);
}

#[test]
fn soulblade_djinn_triggers_only_on_noncreature_spells() {
    cr!("603.2", "305.1");
    ruling!(
        "Soulblade Djinn",
        "Any spell you cast that doesn’t have the type creature will cause Soulblade Djinn’s ability to trigger."
    );
    supported("Soulblade Djinn");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Soulblade Djinn");
    // An artifact creature spell: no trigger.
    cast_new(&mut t, P0, "Ornithopter", &[]);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    // A land: no trigger.
    let forest = t.hand(P0, "Forest");
    t.play_land(P0, forest).expect("land");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // An artifact: it triggers.
    cast_new(&mut t, P0, "Darksteel Relic", &[]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
}

#[test]
fn soulblade_djinn_trigger_resolves_before_the_spell() {
    cr!("603.3", "405.5");
    ruling!(
        "Soulblade Djinn",
        "Soulblade Djinn’s ability goes on the stack on top of the spell that caused it to trigger. It will resolve before that spell."
    );
    let mut t = TestGame::new(2);
    let djinn = t.battlefield(P0, "Soulblade Djinn");
    cast_new(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    t.resolve();
    assert_eq!(t.pt(djinn), (5, 4));
    assert_eq!(t.life(P1), 20, "the Shock hasn't resolved yet");
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

// --- Dragon's Disciple ---------------------------------------------------------------------

#[test]
fn dragons_disciple_reveals_as_it_resolves_not_as_a_cost() {
    cr!("614.12", "601.2f");
    ruling!(
        "Dragon's Disciple",
        "Revealing a Dragon card happens as Dragon's Disciple resolves. It is not an additional cost for casting the spell."
    );
    supported("Dragon's Disciple");
    let mut t = TestGame::new(2);
    t.hand(P0, "Shivan Dragon");
    let seen = watch(
        &mut t,
        P0,
        |d| !matches!(d, Decision::Priority { .. }),
        |g| g.stack.len(),
    );
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Entities, Answer::Default);
    let disciple = cast_new(&mut t, P0, "Dragon's Disciple", &[]);
    assert!(seen.lock().unwrap().is_empty(), "nothing asked while casting");
    t.resolve_all();
    assert!(!seen.lock().unwrap().is_empty(), "asked as it resolved");
    assert_eq!(t.counters(disciple, "+1/+1"), 1);
}

#[test]
fn dragons_disciple_checks_your_dragons_as_it_enters() {
    cr!("614.12", "117.3");
    ruling!(
        "Dragon's Disciple",
        "Players can respond to Dragon's Disciple while it is on the stack, but they cannot respond to the choice of whether or not to reveal a Dragon as it enters the battlefield."
    );
    // P0 controls a Dragon as the Disciple is cast, but P1 destroys it in response: no
    // counter. Without that response, the Disciple gets one without revealing.
    for respond in [false, true] {
        let mut t = TestGame::new(2);
        let dragon = t.battlefield(P0, "Shivan Dragon");
        t.answer_yes(P0, false);
        let disciple = cast_new(&mut t, P0, "Dragon's Disciple", &[]);
        if respond {
            cast_new(&mut t, P1, "Murder", &[obj(dragon)]);
            t.resolve();
        }
        t.resolve_all();
        assert_eq!(t.counters(disciple, "+1/+1"), u32::from(!respond));
    }
}

// --- Storvald, Frost Giant Jarl -------------------------------------------------------------

/// P0's Storvald enters; its trigger (modes `m`, targets `targets`) resolves.
fn storvald_enters(t: &mut TestGame, m: &[usize], targets: &[ObjectId]) {
    supported("Storvald, Frost Giant Jarl");
    modes(t, P0, m);
    let es: Vec<Entity> = targets.iter().map(|x| obj(*x)).collect();
    for e in es {
        t.answer_targets(P0, &[e]);
    }
    t.enter(P0, "Storvald, Frost Giant Jarl");
    t.resolve_all();
}

#[test]
fn storvald_overwrites_earlier_setting_effects_but_keeps_modifications() {
    cr!("613.4b", "613.4c", "613.7");
    ruling!(
        "Storvald, Frost Giant Jarl",
        "Storvald's triggered ability will overwrite any previous effects that set the creature's power and toughness to specific numbers."
    );
    supported("Ovinize");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    put_counters(&mut t, bears, "+1/+1", 1);
    cast_new(&mut t, P0, "Ovinize", &[obj(bears)]);
    t.resolve_all();
    cast_new(&mut t, P0, "Giant Growth", &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 5));
    storvald_enters(&mut t, &[0], &[bears]);
    assert_eq!(t.pt(bears), (11, 11), "7/7 base, +3/+3, one +1/+1 counter");
}

#[test]
fn storvald_can_target_the_same_creature_for_both_modes() {
    cr!("115.3", "700.2d", "613.7");
    ruling!(
        "Storvald, Frost Giant Jarl",
        "You can choose the same creature as the target for both modes if you wanted to"
    );
    let mut t = TestGame::new(2);
    let ogre = t.battlefield(P1, "Gray Ogre");
    storvald_enters(&mut t, &[0, 1], &[ogre, ogre]);
    assert_eq!(t.pt(ogre), (1, 1), "the 1/1 mode applies last");
}

#[test]
fn nissa_who_shakes_the_world_plus_one_lasts_indefinitely() {
    cr!("611.2a", "514.2");
    ruling!(
        "Nissa, Who Shakes the World",
        "The effect of Nissa's first loyalty ability lasts indefinitely. It doesn't wear off during the cleanup step."
    );
    supported("Nissa, Who Shakes the World");
    let mut t = TestGame::new(2);
    let nissa = t.battlefield(P0, "Nissa, Who Shakes the World");
    loyalty(&mut t, nissa, 5);
    let land = t.battlefield(P0, "Wastes");
    t.activate(P0, nissa, 0, &[obj(land)]).expect("+1");
    t.resolve_all();
    assert_eq!(t.pt(land), (3, 3));
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    assert!(t.obj_now(land).is(CardType::Creature));
    assert!(t.obj_now(land).is(CardType::Land));
    assert_eq!(t.pt(land), (3, 3));
    assert!(has(&t, land, mtg_engine::keywords::KeywordKind::Haste));
}

#[test]
fn for_the_common_good_copies_only_the_copiable_values() {
    cr!("707.2", "111.4", "707.3");
    ruling!(
        "For the Common Good",
        "The new tokens copy the characteristics of the original token as stated by the effect that created the original token."
    );
    ruling!(
        "For the Common Good",
        "The new tokens don’t copy whether the original token is tapped or untapped, whether it has any counters on it or Auras and Equipment attached to it, or any non-copy effects that have changed its power, toughness, color, and so on."
    );
    supported("For the Common Good");
    let mut t = TestGame::new(2);
    let soldier = create_token(&mut t, P0, "Soldier");
    put_counters(&mut t, soldier, "+1/+1", 2);
    cast_new(&mut t, P0, "Giant Growth", &[obj(soldier)]);
    t.resolve_all();
    t.g.tap(soldier);
    assert_eq!(t.pt(soldier), (6, 6));
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 2);
    let spell = t.hand(P0, "For the Common Good");
    t.cast(P0, spell).target(soldier).x(1).go();
    t.resolve_all();
    let copies: Vec<ObjectId> = crate::r_s01_common::tokens(&t, P0)
        .into_iter()
        .filter(|id| *id != soldier)
        .collect();
    assert_eq!(copies.len(), 1);
    let c = t.obj_now(copies[0]);
    assert_eq!(c.chars.name, "Soldier");
    assert!(c.chars.has_subtype("Soldier"));
    assert_eq!(t.pt(copies[0]), (1, 1));
    assert!(!c.tapped);
    assert_eq!(t.counters(copies[0], "+1/+1"), 0);
}

#[test]
fn domri_rade_second_target_can_be_yours_but_not_the_same_creature() {
    cr!("115.3", "701.14a");
    ruling!(
        "Domri Rade",
        "The second target of Domri's second ability can be another creature you control, but it can't be the same creature as the first target."
    );
    let mut t = TestGame::new(2);
    let domri = t.battlefield(P0, "Domri Rade");
    loyalty(&mut t, domri, 3);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let from = t.asked().len();
    // The -2, found by its text.
    let minus_two = t
        .obj_now(domri)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, mtg_engine::ability::AbilityKind::Activated(_)))
        .position(|a| a.text.contains("fights"))
        .expect("the -2 ability");
    t.activate(P0, domri, minus_two, &[obj(giant), obj(bears)]).expect("-2");
    let cands = crate::r_s02_common::target_candidates(&t, P0, from);
    assert_eq!(cands.len(), 2);
    assert!(cands[0].contains(&obj(giant)));
    assert!(cands[1].contains(&obj(bears)));
    assert!(!cands[1].contains(&obj(giant)), "not the first target again");
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(damage_marked(&t, giant), 2);
}

#[test]
fn shielding_plax_with_an_illegal_target_doesnt_enter() {
    cr!("608.3b", "303.4a");
    ruling!(
        "Shielding Plax",
        "If the target creature is an illegal target by the time Shielding Plax tries to resolve, the Aura spell doesn't resolve."
    );
    supported("Shielding Plax");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let hand = t.hand_size(P0);
    lands_for_cost(&mut t, P0, "Shielding Plax");
    let plax = t.hand(P0, "Shielding Plax");
    t.cast(P0, plax).target(bears).go();
    exile_now(&mut t, bears);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Shielding Plax"));
    assert_eq!(t.hand_size(P0), hand, "no card drawn");
}

#[test]
fn blinding_angel_skips_are_cumulative() {
    cr!("614.10", "500.11");
    ruling!(
        "Blinding Angel",
        "If you are damaged by more than one, the effects are cumulative and you skip multiple combat phases."
    );
    supported("Blinding Angel");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Blinding Angel");
    let b = t.battlefield(P0, "Blinding Angel");
    fight_it_out(&mut t, &[a, b], &[]);
    t.resolve_all();
    let combat_skips = |t: &TestGame| {
        t.g.player(P1)
            .skips
            .iter()
            .filter(|k| **k == mtg_engine::ability::StepKind::Combat)
            .count()
    };
    assert_eq!(combat_skips(&t), 2);
    // P1's next turn: its combat phase is skipped, one skip remains.
    t.advance_to(P1, Step::PostcombatMain);
    assert_eq!(combat_skips(&t), 1);
}

#[test]
fn domri_rade_plus_one_puts_back_an_unwanted_card_unrevealed() {
    cr!("401.4", "701.20a");
    ruling!(
        "Domri Rade",
        "When resolving Domri's first ability, if the card you look at isn't a creature card, or if it's a creature card you don't want to put into your hand, you simply put it back on top of your library. You don't reveal it or say why you're putting it back."
    );
    supported("Domri Rade");
    let revealed = |t: &TestGame| {
        t.g.turn_events.iter().any(|e| {
            matches!(e, mtg_engine::events::Event::Custom { name, .. }
                if name.as_str() == mtg_engine::reveal::REVEALED)
        })
    };
    // (top card, take it?) -> whether it ends up in P0's hand.
    for (top, take) in [("Forest", true), ("Grizzly Bears", false), ("Grizzly Bears", true)] {
        let mut t = TestGame::new(2);
        let domri = t.battlefield(P0, "Domri Rade");
        loyalty(&mut t, domri, 3);
        let card = t.library_top(P0, top);
        t.answer_yes(P0, take);
        t.activate(P0, domri, 0, &[]).expect("+1");
        t.resolve_all();
        let to_hand = take && top == "Grizzly Bears";
        assert_eq!(t.in_hand(P0, top), to_hand, "{top} {take}");
        if !to_hand {
            assert_eq!(t.g.player(P0).library.last().copied(), Some(t.g.current(card)), "back on top");
            assert!(!revealed(&t));
        }
    }
}
