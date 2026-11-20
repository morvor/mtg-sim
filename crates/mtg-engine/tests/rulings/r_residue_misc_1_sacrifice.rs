//! Item `rulings-residue-misc-1` (pass C) — sacrifices, delayed destruction and values on
//! resolution (batch P116): Nicol Bolas, the Deceiver's punisher (CR 118.12a), God-Eternal
//! Bontu and Pitiless Carnage (sacrificing on resolution; triggers wait, CR 603.3, 608.2;
//! the legend rule first, CR 704.5j), Disciple of Bolas (CR 608.2h), Chaos Mutation's
//! targets (CR 115.3), Vibrating Sphere (state-based actions in the upkeep, CR 502.4, 704.3),
//! Desperate Charge (the affected set is locked in, CR 611.2c), Berserk and Fatal Frenzy's
//! delayed triggers (CR 603.7), Thickest in the Thicket (CR 608.2h, 603.4-like check on
//! resolution), Driftgloom Coyote (CR 610.3c), Drop of Honey and Porphyry Nodes (state
//! triggers, CR 603.8; indestructible, CR 702.12b) and Wyll's Reversal's copies (CR 707.10,
//! 111.12).

use crate::r_p103_common::{obj, pl};
use crate::r_residue_misc_1_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// The candidates of the entity choices `p` was asked since `from`.
fn choice_candidates(t: &TestGame, p: PlayerId, from: usize) -> Vec<Vec<Entity>> {
    asked_since(t, from)
        .into_iter()
        .filter_map(|(q, d)| match d {
            Decision::ChooseEntities { candidates, .. } if q == p => Some(candidates),
            _ => None,
        })
        .collect()
}

// ---------------------------------------------------------------------------------------
// Nicol Bolas, the Deceiver
// ---------------------------------------------------------------------------------------

#[test]
fn nicol_bolas_opponent_may_always_lose_3_life() {
    cr!("118.12a", "608.2c");
    ruling!(
        "Nicol Bolas, the Deceiver",
        "While resolving Nicol Bolas's first ability, your opponent chooses a card to be discarded without revealing it, chooses a nonland permanent to be sacrificed, or chooses to do neither. Then that player discards that card, sacrifices that permanent, or loses 3 life. Your opponent can always choose to lose 3 life, even if they have cards to discard or nonland permanents to sacrifice."
    );
    supported("Nicol Bolas, the Deceiver");
    for choice in ["life", "discard"] {
        let mut t = TestGame::new(2);
        let bolas = t.battlefield(P0, "Nicol Bolas, the Deceiver");
        let bears = t.battlefield(P1, "Grizzly Bears");
        let card = t.hand(P1, "Hill Giant");
        t.advance_to(P0, Step::PrecombatMain);
        t.answer_yes(P1, false);
        if choice == "discard" {
            t.answer_yes(P1, true);
            t.answer_choose(P1, &[obj(card)]);
        } else {
            t.answer_yes(P1, false);
        }
        t.activate(P0, bolas, 0, &[]).unwrap();
        t.resolve_all();
        assert!(t.on_battlefield(bears), "{choice}");
        if choice == "life" {
            assert_eq!(t.life(P1), 17);
            assert!(t.in_hand(P1, "Hill Giant"));
        } else {
            assert_eq!(t.life(P1), 20);
            assert!(t.in_graveyard(P1, "Hill Giant"));
        }
    }
}

// ---------------------------------------------------------------------------------------
// God-Eternal Bontu, Pitiless Carnage
// ---------------------------------------------------------------------------------------

#[test]
fn bontu_and_pitiless_carnage_may_sacrifice_nothing() {
    cr!("608.2c", "701.21a");
    ruling!(
        "God-Eternal Bontu",
        "As Bontu's first triggered ability resolves, you may choose to sacrifice zero other permanents."
    );
    ruling!(
        "Pitiless Carnage",
        "You can choose to sacrifice zero permanents. (That does sound an awful lot like pity, though.)"
    );
    supported("God-Eternal Bontu");
    supported("Pitiless Carnage");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_choose(P0, &[]);
    let hand = t.hand_size(P0);
    enter(&mut t, P0, "God-Eternal Bontu");
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.hand_size(P0), hand);
    t.answer_choose(P0, &[]);
    cast_new(&mut t, P0, "Pitiless Carnage", &[]);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn bontu_and_pitiless_carnage_draw_before_the_dies_triggers() {
    cr!("603.3", "608.2c", "117.5");
    ruling!(
        "God-Eternal Bontu",
        "If any abilities trigger as you sacrifice other permanents, those abilities won't be put onto the stack until after you've drawn cards."
    );
    ruling!(
        "Pitiless Carnage",
        "If any abilities trigger as you sacrifice permanents, those abilities won’t be put onto the stack until after you’ve drawn the appropriate number of cards."
    );
    supported("Blood Artist");
    for bontu in [true, false] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Blood Artist");
        let a = t.battlefield(P0, "Grizzly Bears");
        let b = t.battlefield(P0, "Llanowar Elves");
        t.advance_to(P0, Step::PrecombatMain);
        t.answer_choose(P0, &[obj(a), obj(b)]);
        let hand = t.hand_size(P0);
        if bontu {
            enter(&mut t, P0, "God-Eternal Bontu");
            t.settle();
        } else {
            cast_new(&mut t, P0, "Pitiless Carnage", &[]);
        }
        let hand = if bontu { hand } else { t.hand_size(P0) };
        // Resolve only the sacrificing spell or ability: the cards are drawn, and only
        // then are the two Blood Artist triggers put on the stack.
        t.resolve();
        assert_eq!(t.hand_size(P0), hand + 2, "bontu {bontu}");
        assert_eq!(triggers_on_stack(&t, "loses 1 life"), 2, "bontu {bontu}");
        t.resolve_all();
    }
}

#[test]
fn a_second_bontu_meets_the_legend_rule_before_its_sacrifice() {
    cr!("704.5j", "603.3", "117.5");
    ruling!(
        "God-Eternal Bontu",
        "If a second God-Eternal Bontu enters the battlefield under your control, you'll put one into your graveyard before you can sacrifice anything to the new Bontu's triggered ability."
    );
    let mut t = TestGame::new(2);
    let old = t.battlefield(P0, "God-Eternal Bontu");
    t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P0, Step::PrecombatMain);
    let from = t.asked().len();
    let new = enter(&mut t, P0, "God-Eternal Bontu");
    // The legend rule: P0 keeps the new one; the old one dies (P0 doesn't put it into the
    // library).
    t.answer_choose(P0, &[obj(new)]);
    t.answer_yes(P0, false);
    t.answer_choose(P0, &[]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "God-Eternal Bontu"));
    assert!(!t.on_battlefield(old));
    let sac = choice_candidates(&t, P0, from)
        .into_iter()
        .last()
        .unwrap_or_default();
    assert!(!sac.contains(&obj(old)), "{sac:?}");
}

#[test]
fn pitiless_carnage_sacrifices_on_resolution_not_as_a_cost() {
    cr!("608.2c", "601.2h", "701.6a");
    ruling!(
        "Pitiless Carnage",
        "You sacrifice the permanents as part of the resolution of Pitiless Carnage. It isn’t an additional cost. If Pitiless Carnage is countered or never resolves, you won’t sacrifice anything."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P0, Step::PrecombatMain);
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(bears)]);
    let spell = cast_new(&mut t, P0, "Pitiless Carnage", &[]);
    assert!(choice_candidates(&t, P0, from).is_empty(), "nothing chosen to cast it");
    t.g.turn.priority = Some(P1);
    cast_new(&mut t, P1, "Cancel", &[obj(spell)]);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert!(t.in_graveyard(P0, "Pitiless Carnage"));
}

// ---------------------------------------------------------------------------------------
// Disciple of Bolas
// ---------------------------------------------------------------------------------------

#[test]
fn disciple_of_bolas_must_sacrifice_another_creature_if_it_can() {
    cr!("608.2c", "701.21a");
    ruling!(
        "Disciple of Bolas",
        "If you control at least one other creature when the ability resolves, you must sacrifice one. If you don't control any other creatures at that time, the ability won't do anything."
    );
    supported("Disciple of Bolas");
    for other in [true, false] {
        let mut t = TestGame::new(2);
        let bears = other.then(|| t.battlefield(P0, "Grizzly Bears"));
        t.advance_to(P0, Step::PrecombatMain);
        // An empty answer isn't allowed when a creature can be sacrificed.
        t.answer_choose(P0, &[]);
        let (life, hand) = (t.life(P0), t.hand_size(P0));
        let d = enter(&mut t, P0, "Disciple of Bolas");
        t.resolve_all();
        assert!(t.on_battlefield(d));
        if let Some(b) = bears {
            assert!(!t.on_battlefield(b));
            assert_eq!((t.life(P0), t.hand_size(P0)), (life + 2, hand + 2));
        } else {
            assert_eq!((t.life(P0), t.hand_size(P0)), (life, hand));
        }
    }
}

#[test]
fn disciple_of_bolas_uses_the_sacrificed_creatures_last_power() {
    cr!("608.2h");
    ruling!(
        "Disciple of Bolas",
        "The creature's power as it last existed on the battlefield is the amount of life you gain and the number of cards you draw."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P0, Step::PrecombatMain);
    enter(&mut t, P0, "Disciple of Bolas");
    t.settle();
    cast_new(&mut t, P0, "Giant Growth", &[obj(bears)]);
    t.resolve();
    t.answer_choose(P0, &[obj(bears)]);
    let (life, hand) = (t.life(P0), t.hand_size(P0));
    t.resolve_all();
    assert_eq!((t.life(P0), t.hand_size(P0)), (life + 5, hand + 5));
}

// ---------------------------------------------------------------------------------------
// Chaos Mutation, Vibrating Sphere, Desperate Charge
// ---------------------------------------------------------------------------------------

#[test]
fn chaos_mutation_targets_have_different_controllers() {
    cr!("115.3", "601.2c");
    ruling!(
        "Chaos Mutation",
        "The phrase \"target creatures controlled by different players\" means that no two targets may be controlled by the same player."
    );
    supported("Chaos Mutation");
    let mut t = TestGame::new(3);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let c = t.battlefield(P2, "Craw Wurm");
    t.advance_to(P0, Step::PrecombatMain);
    lands_for_cost(&mut t, P0, "Chaos Mutation");
    let s = t.hand(P0, "Chaos Mutation");
    t.answer_targets(P0, &[obj(a), obj(b)]);
    let r = t.cast(P0, s).try_go();
    if let Ok(id) = r {
        let ts = crate::r_s25_common::targets_of(&t, id);
        assert!(!(ts.contains(&obj(a)) && ts.contains(&obj(b))), "{ts:?}");
    }
    // One of P1's and P2's is fine.
    let mut t = TestGame::new(3);
    let a = t.battlefield(P1, "Grizzly Bears");
    let c2 = t.battlefield(P2, "Craw Wurm");
    t.advance_to(P0, Step::PrecombatMain);
    lands_for_cost(&mut t, P0, "Chaos Mutation");
    let s = t.hand(P0, "Chaos Mutation");
    t.answer_targets(P0, &[obj(a), obj(c2)]);
    let id = t.cast(P0, s).go();
    assert_eq!(crate::r_s25_common::targets_of(&t, id), vec![obj(a), obj(c2)]);
    let _ = c;
}

#[test]
fn vibrating_sphere_kills_at_the_next_state_based_check() {
    cr!("704.3", "704.5f", "502.4");
    ruling!(
        "Vibrating Sphere",
        "Any creature which is lowered below a toughness of 1 will die at the beginning of the upkeep step, since that is the first time State-Based Actions are checked."
    );
    supported("Vibrating Sphere");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vibrating Sphere");
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.advance_to(P0, Step::End);
    assert!(t.on_battlefield(elves));
    t.advance_to(P1, Step::Upkeep);
    assert!(t.in_graveyard(P0, "Llanowar Elves"));
    assert_eq!(t.g.turn.active, P1);
}

#[test]
fn desperate_charge_affects_only_the_creatures_there_as_it_resolves() {
    cr!("611.2c");
    ruling!(
        "Desperate Charge",
        "Desperate Charge affects only creatures you control at the time it resolves. A creature that enters later in the turn won't get +2/+0."
    );
    supported("Desperate Charge");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P0, Step::PrecombatMain);
    cast_new(&mut t, P0, "Desperate Charge", &[]);
    t.resolve_all();
    let b = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(pt_now(&mut t, a), (4, 2));
    assert_eq!(pt_now(&mut t, b), (2, 2));
}

// ---------------------------------------------------------------------------------------
// Delayed triggers: Berserk, Fatal Frenzy
// ---------------------------------------------------------------------------------------

#[test]
fn berserk_destroys_it_even_if_it_stopped_being_a_creature() {
    cr!("603.7c", "610.3");
    ruling!(
        "Berserk",
        "If the target permanent stops being a creature before the next end step, it is still destroyed when the delayed trigger resolves."
    );
    supported("Berserk");
    let mut t = TestGame::new(2);
    let land = t.battlefield(P0, "Forest");
    run_from(
        &mut t,
        P0,
        None,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![
                Modification::AddTypes(vec![CardType::Creature]),
                Modification::SetPT(Some(Value::Const(2)), Some(Value::Const(2))),
            ],
            duration: Duration::EndOfCombat,
        },
        &[obj(land)],
    );
    t.g.objects[land.0 as usize].summoning_sick = false;
    t.advance_to(P0, Step::PrecombatMain);
    attack_with(&mut t, &[(land, pl(P1))]);
    cast_new(&mut t, P0, "Berserk", &[obj(land)]);
    t.resolve_all();
    block_and_finish(&mut t, P1, &[]);
    t.advance_to(P0, Step::PostcombatMain);
    assert!(!t.obj_now(land).is_creature());
    through_end_step(&mut t, P0);
    assert!(t.in_graveyard(P0, "Forest"));
}

#[test]
fn fatal_frenzy_doesnt_sacrifice_a_creature_you_no_longer_control() {
    cr!("603.7c", "701.21a");
    ruling!(
        "Fatal Frenzy",
        "If you don’t control the affected creature at end of turn, it isn’t sacrificed."
    );
    supported("Fatal Frenzy");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P0, Step::PrecombatMain);
    cast_new(&mut t, P0, "Fatal Frenzy", &[obj(bears)]);
    t.resolve_all();
    give_control(&mut t, bears, P1);
    through_end_step(&mut t, P0);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).controller, P1);
}

// ---------------------------------------------------------------------------------------
// Thickest in the Thicket, Driftgloom Coyote
// ---------------------------------------------------------------------------------------

#[test]
fn thickest_in_the_thicket_x_is_the_power_as_it_resolves() {
    cr!("608.2h");
    ruling!(
        "Thickest in the Thicket",
        "The value of X is calculated only once, as Thickest in the Thicket's first ability resolves."
    );
    supported("Thickest in the Thicket");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_targets(P0, &[obj(bears)]);
    enter(&mut t, P0, "Thickest in the Thicket");
    t.settle();
    cast_new(&mut t, P0, "Giant Growth", &[obj(bears)]);
    t.resolve();
    t.resolve_all();
    assert_eq!(p1(&t, bears), 5);
}

#[test]
fn thickest_in_the_thicket_triggers_anyway_and_checks_on_resolution() {
    cr!("603.2", "608.2c");
    ruling!(
        "Thickest in the Thicket",
        "Thickest in the Thicket's last ability will trigger at the beginning of your end step regardless of who controls the creature with the greatest power. As long as you control the creature with the greatest power or tied for the greatest power when the ability resolves, you'll draw two cards."
    );
    for pump in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Thickest in the Thicket");
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.battlefield(P1, "Craw Wurm");
        t.advance_to(P0, Step::End);
        t.settle();
        assert_eq!(triggers_on_stack(&t, "draw two cards"), 1, "pump {pump}");
        if pump {
            // Two Giant Growths: the Bears are 8/8, more than the Wurm's 6.
            cast_new(&mut t, P0, "Giant Growth", &[obj(bears)]);
            t.resolve();
            cast_new(&mut t, P0, "Giant Growth", &[obj(bears)]);
            t.resolve();
        }
        let hand = t.hand_size(P0);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + if pump { 2 } else { 0 });
    }
}

#[test]
fn driftgloom_coyote_gone_before_its_trigger_resolves_exiles_nothing() {
    cr!("610.3c", "603.2");
    ruling!(
        "Driftgloom Coyote",
        "If Driftgloom Coyote leaves the battlefield before its triggered ability resolves, the target creature won’t be exiled."
    );
    supported("Driftgloom Coyote");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_targets(P0, &[obj(bears)]);
    let c = enter(&mut t, P0, "Driftgloom Coyote");
    t.settle();
    destroy(&mut t, c);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
}

// ---------------------------------------------------------------------------------------
// Drop of Honey, Porphyry Nodes
// ---------------------------------------------------------------------------------------

#[test]
fn drop_of_honey_sacrificed_after_the_only_creature_blinks() {
    cr!("603.8", "603.2");
    ruling!(
        "Drop of Honey",
        "If all creatures on the battlefield leave and immediately return (perhaps because Momentary Blink exiled and returned the only creature), the last ability of Drop of Honey triggers. You'll sacrifice it even though a creature's on the battlefield by the time the ability resolves."
    );
    supported("Drop of Honey");
    supported("Momentary Blink");
    let mut t = TestGame::new(2);
    let drop = t.battlefield(P0, "Drop of Honey");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P0, Step::PrecombatMain);
    cast_new(&mut t, P0, "Momentary Blink", &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert!(!t.on_battlefield(drop));
    assert!(t.in_graveyard(P0, "Drop of Honey"));
}

#[test]
fn drop_of_honey_and_porphyry_nodes_skip_indestructible_least_power() {
    cr!("702.12b", "608.2d");
    ruling!(
        "Drop of Honey",
        "If one of the creatures with the least power has indestructible, you must destroy one without indestructible. If all creatures with the least power have indestructible, you don't destroy any creature."
    );
    ruling!(
        "Porphyry Nodes",
        "If one of the creatures with the least power has indestructible, you must destroy one without indestructible. If all creatures with the least power have indestructible, you don't destroy any creature."
    );
    for name in ["Drop of Honey", "Porphyry Nodes"] {
        supported(name);
        // Two 2/2s, one indestructible: P0 asks to destroy the indestructible one; the
        // other is destroyed.
        let mut t = TestGame::new(2);
        t.battlefield(P0, name);
        let a = t.battlefield(P1, "Grizzly Bears");
        let b = t.battlefield(P1, "Grizzly Bears");
        t.battlefield(P1, "Hill Giant");
        run_from(
            &mut t,
            P1,
            None,
            Effect::Modify {
                what: Sel::Target(0),
                mods: vec![Modification::AddKeyword(mtg_engine::keywords::Keyword::new(
                    KeywordKind::Indestructible,
                ))],
                duration: Duration::Permanent,
            },
            &[obj(a)],
        );
        t.advance_to(P1, Step::End);
        t.answer_choose(P0, &[obj(a)]);
        t.advance_to(P0, Step::Upkeep);
        t.resolve_all();
        assert!(t.on_battlefield(a), "{name}");
        assert!(!t.on_battlefield(b), "{name}: the other 2/2 is destroyed");
        // The only least-power creature is indestructible: nothing is destroyed.
        let mut t = TestGame::new(2);
        t.battlefield(P0, name);
        let elves = t.battlefield(P1, "Llanowar Elves");
        let bears = t.battlefield(P1, "Grizzly Bears");
        run_from(
            &mut t,
            P1,
            None,
            Effect::Modify {
                what: Sel::Target(0),
                mods: vec![Modification::AddKeyword(mtg_engine::keywords::Keyword::new(
                    KeywordKind::Indestructible,
                ))],
                duration: Duration::Permanent,
            },
            &[obj(elves)],
        );
        t.advance_to(P1, Step::End);
        t.advance_to(P0, Step::Upkeep);
        t.resolve_all();
        assert!(t.on_battlefield(elves) && t.on_battlefield(bears), "{name}");
    }
}

// ---------------------------------------------------------------------------------------
// Wyll's Reversal
// ---------------------------------------------------------------------------------------

#[test]
fn wylls_reversal_copy_of_an_aura_spell_becomes_a_token_not_created() {
    cr!("707.10f", "111.12", "614.1a");
    ruling!(
        "Wyll's Reversal",
        "If the spell is a permanent spell with a target (such as an Aura spell), and you create a copy of it, the copy will become a token that you control. Because the spell becomes a token, the token isn't \"created.\" Effects that care about a token being created won't interact with a token that enters the battlefield because Wyll's Reversal copied a permanent spell."
    );
    supported("Wyll's Reversal");
    supported("Anointed Procession");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Anointed Procession");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.advance_to(P1, Step::PrecombatMain);
    let pac = cast_new(&mut t, P1, "Pacifism", &[obj(bears)]);
    // d20: 18 + 2 (the Bears) = 20: copy it; the copy enchants the Giant.
    t.g.dice.loaded.push_back(18);
    t.answer_yes(P0, false);
    crate::r_s25_common::change_copy_targets(&mut t, P0, &[Some(obj(giant))]);
    t.g.turn.priority = Some(P0);
    cast_new(&mut t, P0, "Wyll's Reversal", &[obj(pac)]);
    t.resolve_all();
    let tokens: Vec<ObjectId> = t
        .g
        .permanents()
        .filter(|o| o.is_token() && o.chars.name == "Pacifism")
        .map(|o| o.id)
        .collect();
    assert_eq!(tokens.len(), 1, "one token, not doubled");
    assert_eq!(t.obj_now(tokens[0]).controller, P0);
    assert_eq!(attached_to(&t, tokens[0]), Some(obj(giant)));
}

#[test]
fn wylls_reversal_copy_keeps_x() {
    cr!("707.10", "107.3");
    ruling!(
        "Wyll's Reversal",
        "If the target spell or ability has an X whose value was determined as it was cast, activated, or put on the stack (like Fireball does), the copy has the same value of X."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 4);
    let fb = t.hand(P1, "Fireball");
    let fireball = t.cast(P1, fb).x(3).target(pl(P0)).go();
    t.g.dice.loaded.push_back(18);
    t.answer_yes(P0, false);
    crate::r_s25_common::change_copy_targets(&mut t, P0, &[Some(pl(P1))]);
    t.g.turn.priority = Some(P0);
    cast_new(&mut t, P0, "Wyll's Reversal", &[obj(fireball)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 17, "the copy's X is 3");
    assert_eq!(t.life(P0), 17);
}
