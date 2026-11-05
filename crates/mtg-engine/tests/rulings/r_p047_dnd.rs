//! Rulings batch P047 — Baldur's Gate characters and items: Zevlor's copies (CR 707.10),
//! party (CR 700.8), intervening "if" clauses (CR 603.4), triggered abilities that add
//! mana (CR 605.1b), draw replacements and face-down exile (CR 406.3, 614.11), Liara's
//! cost reduction, Wulfgar's extra triggers (CR 603.2d), and the Book of Vile Darkness.

use crate::r_p047_common::*;
use crate::r_s01_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn pl(p: PlayerId) -> Entity {
    Entity::Player(p)
}

// ---------------------------------------------------------------------------
// Zevlor, Elturel Exile
// ---------------------------------------------------------------------------

/// A three-player game with Zevlor's ability activated by P0.
fn zevlor() -> TestGame {
    let mut t = TestGame::new(3);
    let z = t.battlefield(P0, "Zevlor, Elturel Exile");
    t.lands(P0, "Wastes", 2);
    t.activate(P0, z, 0, &[]).unwrap();
    t.resolve_all();
    t
}

#[test]
fn zevlor_copies_are_not_cast_and_resolve_normally() {
    cr!("707.10", "707.10e", "601.2");
    ruling!(
        "Zevlor, Elturel Exile",
        "When the delayed triggered ability of Zevlor, Elturel Exile resolves, it may create one or more copies of a spell. You control each of the copies. Those copies are created on the stack, so they're not “cast.” Abilities that trigger when a player casts a spell won't trigger. The copies will then resolve like normal spells, after players get a chance to cast spells and activate abilities."
    );
    supported("Zevlor, Elturel Exile");
    let mut t = zevlor();
    t.answer_choose(P0, &[pl(P2)]);
    cast_x(&mut t, P0, "Lightning Strike", None, &[pl(P1)]);
    t.resolve();
    // The trigger resolved: P0's copy is on the stack above the original, and players
    // get priority before it resolves.
    assert_eq!(t.stack_len(), 2);
    let copy = *t.g.stack.last().unwrap();
    assert_eq!(t.g.obj(copy).controller, P0);
    t.resolve_all();
    assert_eq!((t.life(P1), t.life(P2)), (17, 17));
    // A "whenever you cast" ability (prowess) triggers for the cast spell only.
    let mut t = zevlor();
    let sw = t.battlefield(P0, "Monastery Swiftspear");
    t.answer_choose(P0, &[pl(P2)]);
    cast_x(&mut t, P0, "Lightning Strike", None, &[pl(P1)]);
    t.resolve_all();
    assert_eq!((t.life(P1), t.life(P2)), (17, 17));
    // Prowess triggered once (for the cast spell only).
    assert_eq!(t.pt(sw), (2, 3));
}

#[test]
fn zevlor_copy_needs_a_legal_target() {
    cr!("707.10e", "115.1");
    ruling!(
        "Zevlor, Elturel Exile",
        "As you make your choices for each other opponent, you can choose that opponent or any permanent they control, but if the chosen player or permanent isn't a legal target for the spell, the copy won't be created."
    );
    // P2 has hexproof and controls only an enchantment: no copy for P2.
    let mut t = zevlor();
    t.battlefield(P2, "Leyline of Sanctity");
    cast_x(&mut t, P0, "Lightning Strike", None, &[pl(P1)]);
    t.resolve();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!((t.life(P1), t.life(P2)), (17, 20));
    // A permanent they control is chosen instead.
    let mut t = zevlor();
    let bears = t.battlefield(P2, "Grizzly Bears");
    t.answer_choose(P0, &[obj(bears)]);
    cast_x(&mut t, P0, "Lightning Strike", None, &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Graveyard(P2));
    assert_eq!(t.life(P2), 20);
}

#[test]
fn zevlor_copies_keep_x_modes_and_additional_costs() {
    cr!("707.10", "707.2", "601.2b");
    ruling!(
        "Zevlor, Elturel Exile",
        "If the spell has an X whose value was determined as it was cast (like Fireball does), the copy has the same value of X."
    );
    ruling!(
        "Zevlor, Elturel Exile",
        "If the spell is modal (that is, it has a bulleted list of choices), the copies will have the same mode(s). You can't choose different ones."
    );
    ruling!(
        "Zevlor, Elturel Exile",
        "You can't choose to pay any additional costs for the copies. However, effects based on any additional costs that were paid for the original spell are copied as though those same costs were paid for the copies too. For example, if you sacrifice a 3/3 creature to cast Fling and then copy it, each copy of Fling will also deal 3 damage to its target."
    );
    // Blaze with X = 4.
    let mut t = zevlor();
    t.answer_choose(P0, &[pl(P2)]);
    spell_x(&mut t, P0, "Blaze", 4, &[pl(P1)]);
    assert_eq!((t.life(P1), t.life(P2)), (16, 16));
    // Izzet Charm's second mode (2 damage to target creature): the copy too.
    let mut t = zevlor();
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P2, "Grizzly Bears");
    t.answer_choose(P0, &[obj(b2)]);
    let ic = t.hand(P0, "Izzet Charm");
    give_mana_for(&mut t, P0, "Izzet Charm");
    let from = t.asked().len();
    t.cast(P0, ic).modes(&[1]).target(b1).go();
    t.resolve_all();
    assert_eq!(t.zone(b1), Zone::Graveyard(P1));
    assert_eq!(t.zone(b2), Zone::Graveyard(P2));
    let mode_choices = asked_since(&t, from)
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseModes { .. }))
        .count();
    assert_eq!(mode_choices, 1, "no mode chosen for the copy");
    // Fling, sacrificing Hill Giant (3 power).
    let mut t = zevlor();
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_choose(P0, &[obj(giant)]);
    t.answer_choose(P0, &[pl(P2)]);
    spell(&mut t, P0, "Fling", &[pl(P1)]);
    assert_eq!((t.life(P1), t.life(P2)), (17, 17));
}

#[test]
fn zevlor_one_target_chosen_and_multiple_targets() {
    cr!("707.10e", "115.1", "603.7");
    ruling!(
        "Zevlor, Elturel Exile",
        "If instant or sorcery spell could have multiple targets but only one target is chosen as it is cast, that spell will cause Zevlor's delayed triggered ability to trigger. The copies will similarly have only one target."
    );
    ruling!(
        "Zevlor, Elturel Exile",
        "Notably, an instant or sorcery spell with multiple targets will not cause the delayed triggered ability to trigger, even if only one of those targets is an opponent or an permanent an opponent controls."
    );
    // Forked Bolt at P1 alone: it triggers; the copy deals its 2 to P2.
    let mut t = zevlor();
    t.answer_choose(P0, &[pl(P2)]);
    let fb = t.hand(P0, "Forked Bolt");
    give_mana_for(&mut t, P0, "Forked Bolt");
    t.answer_targets(P0, &[pl(P1)]);
    t.cast(P0, fb).go();
    t.resolve_all();
    assert_eq!((t.life(P1), t.life(P2)), (18, 18));
    // The ruling's example: Explosive Entry ("Destroy up to one target artifact. Put a
    // +1/+1 counter on up to one target creature.") targeting only P1's Bears: it triggers,
    // and the copy's one target is P2's Ornithopter, as the creature only: it gets the
    // counter and isn't destroyed.
    let mut t = zevlor();
    let bears = t.battlefield(P1, "Grizzly Bears");
    let thopter = t.battlefield(P2, "Ornithopter");
    t.answer_choose(P0, &[obj(thopter)]);
    let ee = t.hand(P0, "Explosive Entry");
    give_mana_for(&mut t, P0, "Explosive Entry");
    t.answer_targets(P0, &[]);
    t.answer_targets(P0, &[obj(bears)]);
    t.cast(P0, ee).go();
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    assert!(t.on_battlefield(thopter));
    assert_eq!(t.counters(thopter, "+1/+1"), 1);
    // Forked Bolt at P1 and P0's own Bears: no copy.
    let mut t = zevlor();
    let bears = t.battlefield(P0, "Grizzly Bears");
    let fb = t.hand(P0, "Forked Bolt");
    give_mana_for(&mut t, P0, "Forked Bolt");
    t.answer_targets(P0, &[pl(P1), obj(bears)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![1, 1]));
    t.cast(P0, fb).go();
    t.resolve();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.life(P2), 20);
}

#[test]
fn zevlor_copies_check_their_targets_and_survive_a_countered_original() {
    cr!("608.2b", "707.10", "701.6a");
    ruling!(
        "Zevlor, Elturel Exile",
        "Like any spell, each copy will check to see if its target is still legal as it tries to resolve."
    );
    ruling!(
        "Zevlor, Elturel Exile",
        "The delayed triggered ability can copy the spell that caused it to trigger even if that spell has been countered by the time that ability resolves."
    );
    // The copy's target gains hexproof (Ranger's Guile, +1/+1): the copy doesn't resolve
    // and the Bears are dealt no damage; the original still resolves.
    let mut t = zevlor();
    let bears = t.battlefield(P2, "Grizzly Bears");
    t.answer_choose(P0, &[obj(bears)]);
    cast_x(&mut t, P0, "Lightning Strike", None, &[pl(P1)]);
    t.resolve();
    assert_eq!(t.stack_len(), 2);
    spell(&mut t, P2, "Ranger's Guile", &[obj(bears)]);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.g.obj(bears).damage, 0);
    assert_eq!(t.life(P1), 17);
    // Without the response, the copy kills them.
    let mut t = zevlor();
    let bears = t.battlefield(P2, "Grizzly Bears");
    t.answer_choose(P0, &[obj(bears)]);
    cast_x(&mut t, P0, "Lightning Strike", None, &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Graveyard(P2));
    // The original is countered while the trigger waits: it's still copied.
    let mut t = zevlor();
    t.answer_choose(P0, &[pl(P2)]);
    let ls = cast_x(&mut t, P0, "Lightning Strike", None, &[pl(P1)]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    cast_x(&mut t, P1, "Counterspell", None, &[obj(ls)]);
    t.resolve();
    assert_eq!(t.zone(ls), Zone::Graveyard(P0));
    t.resolve_all();
    assert_eq!((t.life(P1), t.life(P2)), (20, 17));
}

// ---------------------------------------------------------------------------
// Nalia de'Arnise and full parties
// ---------------------------------------------------------------------------

fn party(t: &mut TestGame, names: &[&str]) -> Vec<ObjectId> {
    names.iter().map(|n| t.battlefield(P0, n)).collect()
}

#[test]
fn nalia_full_party_is_four_different_creatures() {
    cr!("700.8", "700.8a", "603.4");
    ruling!(
        "Nalia de'Arnise",
        "You have a \"full party\" if the number of creatures in your party is four (that is, you control a Cleric, a Rogue, a Warrior, and a Wizard, and they are four different creatures)."
    );
    supported("Nalia de'Arnise");
    for (others, full) in [
        (&["Soulmender", "Oreskos Swiftclaw", "Prodigal Sorcerer"][..], true),
        (&["Soulmender", "Oreskos Swiftclaw", "Changeling Outcast"][..], true),
        // Nalia (Rogue), a Cleric and a changeling: three creatures.
        (&["Soulmender", "Changeling Outcast"][..], false),
    ] {
        let mut t = TestGame::new(2);
        let nalia = t.battlefield(P0, "Nalia de'Arnise");
        party(&mut t, others);
        t.advance_to(P0, Step::BeginningOfCombat);
        t.resolve_all();
        assert_eq!(t.counters(nalia, "+1/+1"), u32::from(full), "{others:?}");
    }
}

#[test]
fn nalia_checks_on_trigger_and_resolution_and_keeps_deathtouch() {
    cr!("603.4", "611.2c");
    ruling!(
        "Nalia de'Arnise",
        "Nalia's last ability will check whether you have a full party at the beginning of combat on your turn. If you don't, the ability won't trigger. If the ability does trigger, it will check again as it tries to resolve. If you don't have a full party at that time, the ability won't resolve and none of its effects will happen."
    );
    ruling!(
        "Nalia de'Arnise",
        "After the last ability has resolved, no longer having a full party won't cause any of the creatures that gained deathtouch to lose it."
    );
    // No full party: no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Nalia de'Arnise");
    party(&mut t, &["Soulmender", "Oreskos Swiftclaw"]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "full party"), 0);
    // Triggers, then the Wizard leaves: nothing happens.
    let mut t = TestGame::new(2);
    let nalia = t.battlefield(P0, "Nalia de'Arnise");
    let ps = party(&mut t, &["Soulmender", "Oreskos Swiftclaw", "Prodigal Sorcerer"]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "full party"), 1);
    to_hand(&mut t, ps[2]);
    t.resolve_all();
    assert_eq!(t.counters(nalia, "+1/+1"), 0);
    // Resolves; the Wizard leaves afterwards: the others keep deathtouch.
    let mut t = TestGame::new(2);
    let nalia = t.battlefield(P0, "Nalia de'Arnise");
    let ps = party(&mut t, &["Soulmender", "Oreskos Swiftclaw", "Prodigal Sorcerer"]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    to_hand(&mut t, ps[2]);
    use mtg_engine::keywords::KeywordKind;
    for c in [nalia, ps[0], ps[1]] {
        assert!(t.g.obj(c).has_keyword(KeywordKind::Deathtouch));
        assert_eq!(t.counters(c, "+1/+1"), 1);
    }
}

#[test]
fn nalia_doesnt_change_when_spells_can_be_cast() {
    cr!("601.3", "302.1");
    ruling!(
        "Nalia de'Arnise",
        "Nalia doesn't change when you can cast spells. Normally, this means during your main phase when the stack is empty, although flash may change this."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Nalia de'Arnise");
    t.lands(P0, "Plains", 1);
    let sm = t.library_top(P0, "Soulmender");
    // On the opponent's turn: no.
    t.set_step(P1, Step::PrecombatMain);
    assert!(t.cast(P0, sm).try_go().is_err());
    // In P0's main phase with an empty stack: yes.
    t.set_step(P0, Step::PrecombatMain);
    t.cast(P0, sm).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Soulmender").len(), 1);
}

// ---------------------------------------------------------------------------
// Single cards
// ---------------------------------------------------------------------------

#[test]
fn gorion_triggers_only_on_adventure_spells() {
    cr!("715.3", "715.4", "707.10");
    ruling!(
        "Gorion, Wise Mentor",
        "An Adventure spell means an instant or sorcery with the Adventure spell type. Permanent spells that you cast which have an Adventure you aren't casting will not cause this ability to trigger."
    );
    // Bonecrusher Giant as a creature: no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gorion, Wise Mentor");
    let bg = t.hand(P0, "Bonecrusher Giant // Stomp");
    t.lands(P0, "Mountain", 3);
    t.cast(P0, bg).method(CastMethod::Normal).go();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // Stomp: it triggers and is copied.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gorion, Wise Mentor");
    let bg = t.hand(P0, "Bonecrusher Giant // Stomp");
    t.lands(P0, "Mountain", 2);
    t.answer_yes(P0, true);
    t.cast(P0, bg)
        .method(CastMethod::Half(1))
        .target(pl(P1))
        .go();
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn trelasarra_triggers_once_per_life_gain_event() {
    cr!("119.9", "603.2c");
    ruling!(
        "Trelasarra, Moon Dancer",
        "An ability that triggers \"whenever you gain life\" triggers just once for each life gain event, no matter how much life you gain."
    );
    let mut t = TestGame::new(2);
    let tr = t.battlefield(P0, "Trelasarra, Moon Dancer");
    let hs = t.hand(P0, "Healing Salve");
    t.lands(P0, "Plains", 1);
    t.cast(P0, hs).modes(&[0]).target(pl(P0)).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    assert_eq!(t.counters(tr, "+1/+1"), 1);
}

#[test]
fn klauth_s_trigger_uses_the_stack() {
    cr!("605.1b", "605.5a", "608.2h");
    ruling!(
        "Klauth, Unrivaled Ancient",
        "Despite that fact that it can produce mana as it resolves, Klauth's triggered ability is not a mana ability because its trigger condition wasn't the activation or resolution of an activated mana ability. This means that it uses the stack and can be responded to. Notably, players can respond by destroying attacking creatures to reduce the amount of mana produced."
    );
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Klauth, Unrivaled Ancient");
    let b = t.battlefield(P0, "Hill Giant");
    attack_with(&mut t, &[(k, pl(P1)), (b, pl(P1))]);
    assert_eq!(triggers_on_stack(&t, "add X mana"), 1);
    spell(&mut t, P1, "Murder", &[obj(b)]);
    let pool = t.g.players[0].mana_pool.total();
    assert_eq!(pool, 4);
}

#[test]
fn drizzt_checks_power_on_death_and_resolution() {
    cr!("603.4", "603.10a", "608.2h");
    ruling!(
        "Drizzt Do'Urden",
        "Drizzt's last ability checks his power both at the time the creature dies and at the time the ability resolves. Notably, this means that if several creatures with power greater than Drizzt's die at the same time, Drizzt's ability will trigger that many times, but some of those abilities may not put counters on him as they resolve."
    );
    supported("Drizzt Do'Urden");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Drizzt Do'Urden");
    t.battlefield(P1, "Elvish Ranger");
    t.battlefield(P1, "Dross Crocodile");
    // -2/-2: Drizzt is 1/1; the 4/1 and 5/1 die with power 2 and 3.
    cast_x(&mut t, P0, "Infest", None, &[]);
    t.resolve();
    assert_eq!(triggers_on_stack(&t, "greater than"), 2);
    t.resolve_all();
    // Whichever resolves first raises Drizzt's power to 3 (or 2), and the other adds
    // only the difference left (or nothing): two counters in all, not three.
    assert_eq!(t.counters(d, "+1/+1"), 2);
}

#[test]
fn dynaheir_ignores_mana_abilities() {
    cr!("605.1a", "603.7", "707.10");
    ruling!(
        "Dynaheir, Invoker Adept",
        "Specifically, the delayed triggered ability doesn't trigger if you activate a mana ability that costs four or more mana to activate."
    );
    let mut t = TestGame::new(2);
    let dy = t.battlefield(P0, "Dynaheir, Invoker Adept");
    let cc = t.battlefield(P0, "Cascading Cataracts");
    let gr = t.battlefield(P0, "Gretchen Titchwillow");
    t.lands(P0, "Wastes", 5);
    t.activate(P0, dy, 0, &[]).unwrap();
    t.resolve_all();
    // Cataracts' {5}, {T}: five mana (a mana ability): no copy.
    let uid = t
        .g
        .obj(cc)
        .chars
        .abilities
        .iter()
        .filter(|a| a.is_mana_ability())
        .nth(1)
        .map(|a| a.uid)
        .unwrap();
    t.g.turn.priority = Some(P0);
    t.g.activate_ability(P0, cc, uid).unwrap();
    t.g.flush_events();
    t.settle();
    assert_eq!(t.g.players[0].mana_pool.total(), 5);
    assert_eq!(t.stack_len(), 0);
    // Gretchen's {2}{G}{U} (four mana): copied, two cards drawn.
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Island", 1);
    t.answer_yes(P0, false);
    t.answer_yes(P0, false);
    let hand = t.hand_size(P0);
    t.activate(P0, gr, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn gretchen_puts_a_land_without_playing_it() {
    cr!("305.4", "305.2");
    ruling!(
        "Gretchen Titchwillow",
        "Putting a land onto the battlefield this way doesn't count as playing a land and can be done even if you've played a land this turn or if it isn't your turn."
    );
    supported("Gretchen Titchwillow");
    for p1_turn in [false, true] {
        let mut t = TestGame::new(2);
        if p1_turn {
            t.set_step(P1, Step::PrecombatMain);
        }
        let gr = t.battlefield(P0, "Gretchen Titchwillow");
        if !p1_turn {
            let f = t.hand(P0, "Forest");
            t.play_land(P0, f).unwrap();
        }
        let played = t.g.players[0].lands_played_this_turn;
        let island = t.hand(P0, "Island");
        t.lands(P0, "Forest", 3);
        t.lands(P0, "Island", 1);
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(island)]);
        t.activate(P0, gr, 0, &[]).unwrap();
        t.resolve_all();
        assert!(t.on_battlefield(island), "p1_turn={p1_turn}");
        assert_eq!(t.g.players[0].lands_played_this_turn, played);
    }
}

#[test]
fn mordenkainen_exchange_isnt_drawing_and_dog_tracks_hand() {
    cr!("121.5", "701.12a", "613.4a");
    ruling!(
        "Mordenkainen",
        "The cards put into your hand due to Mordenkainen's last ability aren't \"drawn.\" Anything that cares about you drawing cards won't notice your hand suddenly becoming very, very well-stocked."
    );
    ruling!(
        "Mordenkainen",
        "The power and toughness of the Dog Illusion token will change as the number of cards in your hand changes."
    );
    supported("Mordenkainen");
    // −2: the Dog.
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Mordenkainen");
    fill_hand(&mut t, P0, "Grizzly Bears", 2);
    t.activate(P0, m, 1, &[]).unwrap();
    t.resolve_all();
    let dog = tokens(&t, P0)[0];
    assert_eq!(t.pt(dog), (4, 4));
    t.hand(P0, "Grizzly Bears");
    assert_eq!(t.pt(dog), (6, 6));
    // −10: thirty cards into the hand, none drawn.
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Mordenkainen");
    t.g.objects[m.0 as usize]
        .counters
        .insert(mtg_engine::types::counters::LOYALTY.into(), 10);
    t.hand(P0, "Grizzly Bears");
    let lib = t.library_size(P0);
    t.activate(P0, m, 2, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), lib);
    assert_eq!(t.library_size(P0), 1);
    assert!(!t
        .g
        .turn_events
        .iter()
        .any(|e| matches!(e, Event::Drew { .. })));
}

#[test]
fn wulfgar_doubles_only_attack_triggers() {
    cr!("603.2d", "702.121a", "508.1f");
    ruling!(
        "Wulfgar of Icewind Dale",
        "Wulfgar of Icewind Dale's last ability affects only triggered abilities with conditions that are directly related to attacking, such as “whenever this creature attacks” or “whenever you attack with one or more creatures.” It does not affect triggered abilities with other trigger conditions, such as “whenever this creature becomes tapped.”"
    );
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Wulfgar of Icewind Dale");
    let nml = t.battlefield(P0, "Night Market Lookout");
    attack_with(&mut t, &[(w, pl(P1)), (nml, pl(P1))]);
    t.resolve_all();
    // Melee twice: +2/+2; the Lookout's "becomes tapped" once: 1 life.
    assert_eq!(t.pt(w), (6, 6));
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.life(P0), 21);
}

#[test]
fn bane_and_bhaal_survive_lethal_damage_dealt_with_the_life_loss() {
    cr!("704.3", "704.5g", "702.12b");
    ruling!(
        "Bane, Lord of Darkness",
        "If Bane, Lord of Darkness is dealt lethal damage at the same time that its controller's life total is reduced to less than or equal to half their starting life total, Bane will have indestructible at the time state-based actions are performed and survive."
    );
    ruling!(
        "Bhaal, Lord of Murder",
        "If Bhaal, Lord of Murder is dealt lethal damage at the same time that its controller's life total is reduced to less than or equal to half their starting life total, Bhaal will have indestructible at the time state-based actions are performed and survive."
    );
    for (name, x) in [("Bane, Lord of Darkness", 2), ("Bhaal, Lord of Murder", 4)] {
        let mut t = TestGame::new(2);
        let god = t.battlefield(P0, name);
        t.set_step(P1, Step::PrecombatMain);
        t.g.players[0].life = 10 + x as i32;
        spell_x(&mut t, P1, "Earthquake", x, &[]);
        assert_eq!(t.life(P0), 10, "{name}");
        assert!(t.on_battlefield(god), "{name}");
        // One more life: it isn't indestructible and dies.
        let mut t = TestGame::new(2);
        let god = t.battlefield(P0, name);
        t.set_step(P1, Step::PrecombatMain);
        t.g.players[0].life = 11 + x as i32;
        spell_x(&mut t, P1, "Earthquake", x, &[]);
        assert!(!t.on_battlefield(god), "{name}");
    }
}

#[test]
fn nine_fingers_keene_hand_with_nine_gates_already() {
    cr!("608.2c", "603.2");
    ruling!(
        "Nine-Fingers Keene",
        "If you do not put a Gate card onto the battlefield this way but already control nine or more Gates, you will put all the cards into your hand."
    );
    let mut t = TestGame::new(2);
    let nk = t.battlefield(P0, "Nine-Fingers Keene");
    for _ in 0..9 {
        t.battlefield(P0, "Azorius Guildgate");
    }
    t.answer_choose(P0, &[]);
    attack_with(&mut t, &[(nk, pl(P1))]);
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.hand_size(P0), 9);
}

#[test]
fn mazzy_exiles_a_non_aura_card_and_its_cast_follows_the_rules() {
    cr!("702.103b", "400.7", "601.3", "302.1", "601.2h");
    ruling!(
        "Mazzy, Truesword Paladin",
        "In some unusual cases, the Aura put into your graveyard isn't an Aura card. This may happen due to copy effects, for example. In these cases, you'll still exile the card and you may cast it."
    );
    ruling!(
        "Mazzy, Truesword Paladin",
        "You must still pay all costs and follow all timing rules when casting the card from exile."
    );
    supported("Mazzy, Truesword Paladin");
    // Leafcrown Dryad bestowed: an Aura on the battlefield, a creature card in the graveyard.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mazzy, Truesword Paladin");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let satyr = t.hand(P0, "Leafcrown Dryad");
    t.lands(P0, "Forest", 5);
    t.cast(P0, satyr)
        .method(CastMethod::Keyword(mtg_engine::keywords::KeywordKind::Bestow))
        .target(bears)
        .go();
    t.resolve_all();
    assert_eq!(t.g.obj(t.g.current(satyr)).attached_to, Some(obj(bears)));
    let aura = t.g.current(satyr);
    spell(&mut t, P1, "Disenchant", &[obj(aura)]);
    assert_eq!(t.zone(satyr), Zone::Exile);
    let exiled = t.g.current(satyr);
    // On the opponent's turn (a creature card, no flash): it can't be cast.
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P0, "Forest", 3);
    assert!(t.cast(P0, exiled).try_go().is_err());
    // In P0's main phase without the mana: it can't be cast either.
    let mut tapped = 0;
    for l in t.g.battlefield.clone() {
        if t.g.obj(l).chars.name == "Forest" && !t.g.obj(l).tapped {
            if tapped < 3 {
                t.g.tap(l);
                tapped += 1;
            }
        }
    }
    t.set_step(P0, Step::PrecombatMain);
    assert!(t.cast(P0, exiled).try_go().is_err());
    // With the mana: it's cast as the creature it is.
    t.lands(P0, "Forest", 3);
    t.cast(P0, exiled).method(CastMethod::Normal).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Leafcrown Dryad").len(), 1);
}

#[test]
fn neera_decline_cast_and_targets_needed() {
    cr!("601.2c", "608.2c", "401.4");
    ruling!(
        "Neera, Wild Mage",
        "Specifically, if you choose not to cast the nonland card, it will be put on the bottom of your library along with the other revealed cards."
    );
    ruling!(
        "Neera, Wild Mage",
        "You must have legal targets to cast a spell with targets, even if you intend to put it onto the bottom of your library this way."
    );
    supported("Neera, Wild Mage");
    // No artifact: Shatter can't be cast.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Neera, Wild Mage");
    let s = t.hand(P0, "Shatter");
    give_mana_for(&mut t, P0, "Shatter");
    assert!(t.cast(P0, s).try_go().is_err());
    // Cast Grizzly Bears, put it on the bottom, decline the revealed Hill Giant.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Neera, Wild Mage");
    let top = stack_library(&mut t, P0, &["Forest", "Hill Giant"]);
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    cast_x(&mut t, P0, "Grizzly Bears", None, &[]);
    t.resolve_all();
    let lib = &t.g.players[0].library;
    let n = lib.len();
    let bottom3: Vec<ObjectId> = lib[..3.min(n)].to_vec();
    let bottom_names: Vec<String> = bottom3
        .iter()
        .map(|c| t.g.obj(*c).chars.name.to_string())
        .collect();
    assert!(bottom_names.contains(&"Hill Giant".to_string()), "{bottom_names:?}");
    assert!(bottom_names.contains(&"Forest".to_string()), "{bottom_names:?}");
    assert!(bottom_names.contains(&"Grizzly Bears".to_string()), "{bottom_names:?}");
    let _ = top;
}

#[test]
fn asmodeus_face_down_exile_and_empty_library() {
    cr!("406.3", "614.11", "704.5b", "400.7");
    ruling!(
        "Asmodeus the Archfiend",
        "If a card is exiled face-down and no player has been given permission to look at it, no player is allowed to look at it as long as it remains exiled face-down."
    );
    ruling!(
        "Asmodeus the Archfiend",
        "If Asmodeus the Archfiend leaves the battlefield before you activate its last ability, any cards exiled by its replacement effect remain exiled face down for the rest of the game. If you somehow return the same Asmodeus the Archfiend card to the battlefield, it will be a different object with no connection to those face-down cards."
    );
    ruling!(
        "Asmodeus the Archfiend",
        "If your library is empty, no cards will be exiled but you won't lose the game due to drawing from an empty library."
    );
    supported("Asmodeus the Archfiend");
    // Two draws exiled face down; the {B} ability returns them and costs 2 life.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Asmodeus the Archfiend");
    t.g.draw_cards(P0, 2);
    let exiled: Vec<ObjectId> = t.g.exile.clone();
    assert_eq!(exiled.len(), 2);
    for c in &exiled {
        assert!(t.g.obj(*c).face_down);
        for p in [P0, P1] {
            assert!(!mtg_engine::facedown::can_look_at(&t.g, p, *c));
        }
    }
    t.lands(P0, "Swamp", 1);
    t.activate(P0, a, 1, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!(t.life(P0), 18);
    // It leaves and the same card comes back: no connection to the earlier cards.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Asmodeus the Archfiend");
    t.g.draw_cards(P0, 2);
    to_graveyard(&mut t, a);
    let card = t.g.current(a);
    assert_eq!(t.zone(card), Zone::Graveyard(P0));
    let a2 = t
        .g
        .move_object(card, Zone::Battlefield, mtg_engine::events::MoveCause::Effect, Some(P0))
        .unwrap();
    assert_eq!(t.g.obj(a2).chars.name, "Asmodeus the Archfiend");
    t.lands(P0, "Swamp", 1);
    t.activate(P0, a2, 1, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.g.exile.iter().filter(|c| t.g.obj(**c).face_down).count(), 2);
    // Empty library: nothing exiled, and the player doesn't lose.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Asmodeus the Archfiend");
    for c in t.g.players[0].library.clone() {
        to_graveyard(&mut t, c);
    }
    t.g.draw_cards(P0, 1);
    t.settle();
    assert!(!t.has_lost(P0));
    assert!(t.g.exile.is_empty());
}

#[test]
fn commander_liara_portyr_reduction_and_players_attacked() {
    cr!("508.1", "601.2f", "715.3d");
    ruling!(
        "Commander Liara Portyr",
        "Commander Liara Portyr doesn't have to be among the attacking creatures."
    );
    ruling!(
        "Commander Liara Portyr",
        "If you attack a planeswalker but don't also attack that planeswalker's controller, that player won't count toward the value of X."
    );
    ruling!(
        "Commander Liara Portyr",
        "Commander Liara Portyr's triggered ability reduces the cost of all spells you cast from exile, even if you are casting them with a different permission."
    );
    // Three players: attack P1 with the Bears and P2's planeswalker with the Giant: X=1.
    let mut t = TestGame::with_config(
        3,
        mtg_engine::game::GameConfig {
            attack_multiple_players: true,
            ..Default::default()
        },
    );
    t.battlefield(P0, "Commander Liara Portyr");
    let b = t.battlefield(P0, "Grizzly Bears");
    let g = t.battlefield(P0, "Hill Giant");
    let pw = t.battlefield(P2, "Chandra, Torch of Defiance");
    let exile_before = t.g.exile.len();
    attack_with(&mut t, &[(b, pl(P1)), (g, obj(pw))]);
    t.resolve_all();
    assert_eq!(t.g.exile.len(), exile_before + 1);
    // Stomp first puts Bonecrusher Giant on an adventure in exile; after the attack, the
    // Giant ({2}{R}) costs {1} less though cast with the Adventure's permission.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Commander Liara Portyr");
    let bg = t.hand(P0, "Bonecrusher Giant // Stomp");
    t.lands(P0, "Mountain", 2);
    t.cast(P0, bg).method(CastMethod::Half(1)).target(pl(P1)).go();
    t.resolve_all();
    let b = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(b, pl(P1))]);
    t.resolve_all();
    block_and_finish(&mut t, P1, &[]);
    t.set_step(P0, Step::PostcombatMain);
    let lands = t.lands(P0, "Mountain", 2);
    let giant = t.g.current(bg);
    assert_eq!(t.zone(giant), Zone::Exile);
    t.cast(P0, giant).method(CastMethod::Normal).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Bonecrusher Giant").len(), 1);
    assert!(lands.iter().all(|l| t.g.obj(*l).tapped));
}
