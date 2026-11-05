//! Rulings batch P159 — the rest: look-back conditions over the turn (CR 603.4 intervening
//! "if" clauses at the end step), life gain replacement per event (CR 119.10, 614.1a),
//! multiplayer counts, "one or more" triggers, once-each-turn triggers, madness during a
//! cost, Void Maw's exile replacement and new objects (CR 400.7), cycling counts, choices
//! made as a spell resolves, and searches that may find nothing (CR 701.23b).

use crate::r_p044_common::{cast_only, cast_spell, kill};
use crate::r_s01_common::*;
use crate::r_s05_common::{enter, move_to};
use crate::r_s06_common::activate_containing;
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::game::GameConfig;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// Advances to `p`'s end step with the triggers on the stack.
fn end_step(t: &mut TestGame, p: PlayerId) {
    t.advance_to(p, Step::End);
    t.settle();
}

// --- Looking back over the turn ----------------------------------------------------------

#[test]
fn knight_of_the_ebon_legion_counts_the_life_lost_over_the_turn() {
    cr!("603.4", "119.3");
    ruling!(
        "Knight of the Ebon Legion",
        "A player doesn't need to have lost 4 life all at once."
    );
    ruling!(
        "Knight of the Ebon Legion",
        "It doesn't matter whether a player has 4 or less life than they started the turn with. If a player lost 4 life but also gained 6 life, that player still lost 4 life."
    );
    ruling!(
        "Knight of the Ebon Legion",
        "Knight of the Ebon Legion looks at your entire turn to determine whether a player lost 4 or more life."
    );
    supported("Knight of the Ebon Legion");
    // (losses of life by player, life gained by P1, Knight arrives after the losses,
    // counter expected)
    let cases: &[(&[(PlayerId, u32)], u32, bool, u32)] = &[
        (&[(P1, 2), (P1, 2)], 0, false, 1),
        (&[(P1, 4)], 6, false, 1),
        (&[(P1, 2), (P0, 3)], 0, false, 0),
        (&[(P0, 4)], 0, false, 1),
        (&[(P1, 4)], 0, true, 1),
        (&[(P1, 3)], 0, false, 0),
    ];
    for &(losses, gain, late, counters) in cases {
        let mut t = TestGame::new(2);
        let knight = (!late).then(|| t.battlefield(P0, "Knight of the Ebon Legion"));
        for (p, n) in losses {
            t.g.lose_life(*p, *n);
        }
        t.g.gain_life(P1, gain);
        let knight = knight.unwrap_or_else(|| t.battlefield(P0, "Knight of the Ebon Legion"));
        end_step(&mut t, P0);
        t.resolve_all();
        assert_eq!(
            t.counters(knight, "+1/+1"),
            counters,
            "{losses:?} gain {gain} late {late}"
        );
    }
}

#[test]
fn end_step_intervening_ifs_check_as_the_end_step_begins() {
    cr!("603.4", "513.1a");
    ruling!(
        "Old Flitterfang",
        "Old Flitterfang's triggered ability will check as the end step starts to see if a creature died this turn. If none did, the ability won't trigger at all."
    );
    ruling!(
        "Resplendent Angel",
        "If you didn't gain life during the turn before the end step begins, Resplendent Angel's triggered ability won't trigger at all. Gaining life during the end step won't cause the ability to trigger."
    );
    supported("Old Flitterfang");
    supported("Resplendent Angel");
    // Nothing yet as the end step begins; dying or gaining life during it is too late.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Old Flitterfang");
    t.battlefield(P0, "Resplendent Angel");
    let bears = t.battlefield(P1, "Grizzly Bears");
    end_step(&mut t, P0);
    assert_eq!(t.stack_len(), 0);
    kill(&mut t, bears);
    t.g.gain_life(P0, 5);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    t.advance_to(P1, Step::Upkeep);
    assert!(tokens(&t, P0).is_empty());
    // A creature died and P0 gained 5 before: both trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Old Flitterfang");
    t.battlefield(P0, "Resplendent Angel");
    let bears = t.battlefield(P1, "Grizzly Bears");
    kill(&mut t, bears);
    t.g.gain_life(P0, 5);
    end_step(&mut t, P0);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 2);
}

/// A Two-Headed Giant game: P0 and P1 against P2 and P3.
fn two_headed_giant() -> TestGame {
    TestGame::with_config(4, GameConfig::two_headed_giant(vec![0, 0, 1, 1]))
}

#[test]
fn a_teammates_life_gain_isnt_yours() {
    cr!("810.9", "119.10", "603.4");
    ruling!(
        "Knight of Dawn's Light",
        "In a Two-Headed Giant game, life gained by your teammate won't cause Knight of Dawn's Light's second ability to apply"
    );
    ruling!(
        "Resplendent Angel",
        "In a Two-Headed Giant game, life gained by your teammate won't satisfy Resplendent Angel's triggered ability"
    );
    let mut t = two_headed_giant();
    t.battlefield(P0, "Knight of Dawn's Light");
    t.battlefield(P0, "Resplendent Angel");
    t.g.gain_life(P1, 5);
    assert_eq!((t.life(P0), t.life(P1)), (35, 35));
    end_step(&mut t, P0);
    assert_eq!(t.stack_len(), 0);
    // P0's own gain is replaced.
    t.g.gain_life(P0, 5);
    assert_eq!(t.life(P0), 41);
}

#[test]
fn knight_of_dawns_light_applies_once_per_life_gain_event() {
    cr!("119.10", "614.1a", "510.2", "702.15b");
    ruling!(
        "Knight of Dawn's Light",
        "Each creature with lifelink dealing combat damage causes a separate life-gaining event."
    );
    ruling!(
        "Knight of Dawn's Light",
        "If you control two Knight of Dawn's Lights and you would gain life, you gain that much life plus 2."
    );
    // Two lifelink attackers unblocked: 2 events, +1 each.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Knight of Dawn's Light");
    let a = t.battlefield(P0, "Mesa Unicorn");
    let b = t.battlefield(P0, "Mesa Unicorn");
    t.attack(&[(a, Entity::Player(P1)), (b, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P0), 26);
    // One lifelink attacker blocked by two creatures: one event, +1 once.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Knight of Dawn's Light");
    let a = t.battlefield(P0, "Mesa Unicorn");
    let x = t.battlefield(P1, "Llanowar Elves");
    let y = t.battlefield(P1, "Llanowar Elves");
    t.answer(
        P0,
        DecisionKind::Damage,
        Answer::Numbers(vec![1, 1]),
    );
    t.attack(&[(a, Entity::Player(P1))], &[(x, a), (y, a)]);
    assert!(!t.on_battlefield(x) && !t.on_battlefield(y));
    assert_eq!(t.life(P0), 23);
    // Two Knights: plus 2.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Knight of Dawn's Light");
    t.battlefield(P0, "Knight of Dawn's Light");
    t.g.gain_life(P0, 3);
    assert_eq!(t.life(P0), 25);
}

#[test]
fn gluttonous_troll_ignores_opponents_who_left() {
    cr!("800.4a", "102.3");
    ruling!(
        "Gluttonous Troll",
        "In a multiplayer game, opponents who have left the game aren't counted by Gluttonous Troll's triggered ability."
    );
    let mut t = TestGame::new(4);
    t.g.perform_action(P3, Action::Concede).unwrap();
    t.settle();
    enter(&mut t, P0, "Gluttonous Troll");
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Food").len(), 2);
}

#[test]
fn energy_in_one_go_triggers_one_or_more_once() {
    cr!("122.1", "603.2c");
    ruling!(
        "Aetherwind Basker",
        "Abilities that trigger \"whenever you get one or more {E},\" such as that of Fabrication Module from the Kaladesh set, trigger only once as Aetherwind Basker's triggered ability resolves."
    );
    ruling!(
        "Azula, Ruthless Firebender",
        "All experience counters are identical, no matter how you got them."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fabrication Module");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    enter(&mut t, P0, "Aetherwind Basker");
    t.resolve();
    assert_eq!(
        t.g.player(P0).counters.get(counters::ENERGY).copied(),
        Some(2)
    );
    assert_eq!(t.stack_len(), 1);
    assert_eq!(triggers_on_stack(&t, "+1/+1 counter"), 1);
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    // Azula counts experience counters from anywhere.
    let mut t = TestGame::new(2);
    let azula = t.battlefield(P0, "Azula, Ruthless Firebender");
    t.g.players[0]
        .counters
        .insert(counters::EXPERIENCE.into(), 2);
    t.lands(P0, "Swamp", 3);
    activate_containing(&mut t, P0, azula, "menace").expect("activate");
    t.resolve_all();
    assert_eq!(t.pt(azula), (5, 5));
}

#[test]
fn elvish_warmaster_triggers_once_each_turn() {
    cr!("603.2c", "603.2");
    ruling!(
        "Elvish Warmaster",
        "It doesn't matter how many Elves enter the battlefield under your control. The ability creates only one Elf Warrior creature token."
    );
    ruling!(
        "Elvish Warmaster",
        "Once the triggered ability has triggered once during a turn, it can't trigger again, even if the triggered ability is still on the stack, has been countered"
    );
    supported("Elvish Warmaster");
    supported("Defabricate");
    // Two Elves at once: one trigger, one token.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Elvish Warmaster");
    cast_spell(&mut t, P0, "Gilt-Leaf Ambush", &[]);
    assert_eq!(tokens(&t, P0).len(), 3);
    // Countered by Defabricate: another Elf this turn doesn't trigger it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Elvish Warmaster");
    enter(&mut t, P0, "Llanowar Elves");
    assert_eq!(t.stack_len(), 1);
    let trig = t.g.stack[0];
    crate::r_s29_common::choose_modes(&mut t, P1, &[1]);
    give_mana_for(&mut t, P1, "Defabricate");
    let d = t.hand(P1, "Defabricate");
    t.cast(P1, d).target(trig).go();
    t.resolve_all();
    assert!(tokens(&t, P0).is_empty());
    enter(&mut t, P0, "Elvish Mystic");
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn a_madness_vampire_enters_before_stromkirk_condemned_resolves() {
    cr!("702.35a", "702.35b", "602.2b");
    ruling!(
        "Stromkirk Condemned",
        "If you discard a Vampire card with madness and cast it, it'll enter the battlefield before Stromkirk Condemned's ability resolves."
    );
    supported("Asylum Visitor");
    let mut t = TestGame::new(2);
    let sc = t.battlefield(P0, "Stromkirk Condemned");
    t.lands(P0, "Swamp", 2);
    let v = t.hand(P0, "Asylum Visitor");
    t.answer_choose(P0, &[obj(v)]);
    t.answer_yes(P0, true);
    activate_containing(&mut t, P0, sc, "Discard").expect("activate");
    t.resolve_all();
    let visitor = t.named_on_battlefield("Asylum Visitor");
    assert_eq!(visitor.len(), 1);
    assert_eq!(t.pt(visitor[0]), (4, 2));
    assert_eq!(t.pt(sc), (3, 3));
}

// --- Void Maw ---------------------------------------------------------------------------

#[test]
fn void_maw_dying_with_another_creature() {
    cr!("614.1a", "704.5g", "603.10a");
    ruling!(
        "Void Maw",
        "If Void Maw and another creature would be put into the graveyard from the battlefield at the same time, Void Maw is put into your graveyard and the other creature is exiled."
    );
    supported("Void Maw");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Void Maw");
    t.battlefield(P1, "Grizzly Bears");
    cast_spell(&mut t, P0, "Wrath of God", &[]);
    assert!(t.in_graveyard(P0, "Void Maw"));
    assert!(t.in_exile("Grizzly Bears"));
}

#[test]
fn void_maw_cards_exiled_again_are_new_objects() {
    cr!("400.7", "602.2b", "614.1a");
    ruling!(
        "Void Maw",
        "Cards exiled with Void Maw that would go to the graveyard but have that replaced by an effect such as Leyline of the Void are considered new objects."
    );
    let mut t = TestGame::new(2);
    let maw = t.battlefield(P0, "Void Maw");
    let bears = t.battlefield(P1, "Grizzly Bears");
    kill(&mut t, bears);
    assert_eq!(t.zone(bears), Zone::Exile);
    t.battlefield(P0, "Leyline of the Void");
    let exiled = t.g.current(bears);
    t.answer_choose(P0, &[obj(exiled)]);
    activate_containing(&mut t, P0, maw, "Put a card exiled").expect("activate");
    t.resolve_all();
    assert_eq!(t.pt(maw), (6, 7));
    // Leyline of the Void exiled it again: a new object, not exiled with Void Maw.
    let again = t.g.current(bears);
    assert_ne!(again, exiled);
    assert_eq!(t.zone(bears), Zone::Exile);
    t.answer_choose(P0, &[obj(again)]);
    assert!(activate_containing(&mut t, P0, maw, "Put a card exiled").is_err());
}

// --- Cycling, entering from the library, dying -------------------------------------------

#[test]
fn yidaro_counts_every_card_named_yidaro_cycled() {
    cr!("702.29a", "201.2");
    ruling!(
        "Yidaro, Wandering Monster",
        "You don't have to cycle the same physical card four times for Yidaro to get tired of being cycled and stomp onto the battlefield. Any cards named Yidaro, Wandering Monster will count."
    );
    supported("Yidaro, Wandering Monster");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 8);
    for i in 0..4 {
        let y = t.hand(P0, "Yidaro, Wandering Monster");
        activate_containing(&mut t, P0, y, "Cycling").expect("cycle");
        t.resolve_all();
        let on = t.named_on_battlefield("Yidaro, Wandering Monster").len();
        assert_eq!(on, if i == 3 { 1 } else { 0 }, "cycle {}", i + 1);
    }
}

#[test]
fn fblthp_draws_two_only_from_the_library() {
    cr!("603.2", "614.15");
    ruling!(
        "Fblthp, the Lost",
        "There's normally no way to cast Fblthp from your library or to have it enter the battlefield from your library."
    );
    supported("Fblthp, the Lost");
    // Cast from hand: one card.
    let mut t = TestGame::new(2);
    give_mana_for(&mut t, P0, "Fblthp, the Lost");
    let f = t.hand(P0, "Fblthp, the Lost");
    t.cast(P0, f).go();
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // Put onto the battlefield from the library by another effect: two.
    let mut t = TestGame::new(2);
    let f = t.library_top(P0, "Fblthp, the Lost");
    let hand = t.hand_size(P0);
    move_to(&mut t, f, Zone::Battlefield);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn anax_checks_the_power_the_creature_had_as_it_died() {
    cr!("603.10a", "603.6c");
    ruling!(
        "Anax, Hardened in the Forge",
        "To determine whether you create two tokens, check the power of the creature that died immediately before it died."
    );
    supported("Anax, Hardened in the Forge");
    for (pump, n) in [(true, 2), (false, 1)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Anax, Hardened in the Forge");
        let bears = t.battlefield(P0, "Grizzly Bears");
        if pump {
            cast_spell(&mut t, P0, "Giant Growth", &[obj(bears)]);
        }
        kill(&mut t, bears);
        t.resolve_all();
        assert_eq!(with_subtype(&t, P0, "Satyr").len(), n, "pumped {pump}");
    }
}

#[test]
fn defabricate_cant_target_mana_abilities_or_special_actions() {
    cr!("605.3b", "116.1", "115.1");
    ruling!(
        "Defabricate",
        "Turn-based actions and special actions like the normal card draw, combat damage, or turning a face-down creature face up can't be targeted. Mana abilities also can't be targeted."
    );
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let elves = t.battlefield(P1, "Llanowar Elves");
    assert!(crate::r_s20_common::tap_for_mana(&mut t, P1, elves, "Add"));
    let land = t.hand(P1, "Forest");
    t.play_land(P1, land).expect("land");
    assert_eq!(t.stack_len(), 0);
    give_mana_for(&mut t, P0, "Defabricate");
    let d = t.hand(P0, "Defabricate");
    crate::r_s29_common::choose_modes(&mut t, P0, &[1]);
    assert!(t.cast(P0, d).try_go().is_err());
}

// --- Choices -------------------------------------------------------------------------------

#[test]
fn vonas_hunger_choices_go_in_turn_order_then_sacrifice_together() {
    cr!("101.4", "701.21a", "702.131a");
    ruling!(
        "Vona's Hunger",
        "first the player whose turn it is (if that player is an opponent) chooses which creature or creatures they will sacrifice, then each other opponent in turn order does the same, then all chosen creatures are sacrificed at the same time."
    );
    supported("Vona's Hunger");
    let mut t = TestGame::new(3);
    t.set_step(P2, Step::PrecombatMain);
    t.g.players[0].has_citys_blessing = true;
    let p1 = [
        t.battlefield(P1, "Grizzly Bears"),
        t.battlefield(P1, "Grizzly Bears"),
        t.battlefield(P1, "Grizzly Bears"),
    ];
    let p2 = [t.battlefield(P2, "Hill Giant"), t.battlefield(P2, "Hill Giant")];
    let from = t.asked().len();
    cast_only(&mut t, P0, "Vona's Hunger", &[]);
    t.resolve_all();
    let choosers: Vec<PlayerId> = asked_since(&t, from)
        .into_iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
        .map(|(p, _)| p)
        .collect();
    assert_eq!(choosers, vec![P2, P1]);
    // Half rounded up: two of P1's three, one of P2's two.
    assert_eq!(p1.iter().filter(|x| t.on_battlefield(**x)).count(), 1);
    assert_eq!(p2.iter().filter(|x| t.on_battlefield(**x)).count(), 1);
    let sacrificed: Vec<usize> = t
        .g
        .turn_events
        .iter()
        .enumerate()
        .filter(|(_, e)| matches!(e, Event::Sacrificed { .. }))
        .map(|(i, _)| i)
        .collect();
    assert_eq!(sacrificed.len(), 3);
}

#[test]
fn secret_of_bloodbending_makes_the_opponents_combat_decisions() {
    cr!("723.1", "723.5");
    ruling!(
        "Secret of Bloodbending",
        "While controlling another player, you make all choices and decisions that player is allowed to make or is told to make."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_spell(&mut t, P0, "Secret of Bloodbending", &[Entity::Player(P1)]);
    // P0 (controlling P1) declares P1's attack; P1's own scripted answer (no attack) is
    // never asked for.
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bears, Entity::Player(P0))]),
    );
    t.advance_to(P1, Step::DeclareAttackers);
    assert!(t.g.is_attacking(bears));
}

#[test]
fn faiths_shield_chooses_one_color_as_it_resolves() {
    cr!("608.2c", "702.16a", "702.16b");
    ruling!(
        "Faith's Shield",
        "You choose only one color (not one color per player and permanent). You choose the color as Faith’s Shield resolves."
    );
    let mut t = TestGame::new(2);
    t.g.players[0].life = 5;
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let from = t.asked().len();
    cast_only(&mut t, P0, "Faith's Shield", &[obj(a)]);
    let colors = |t: &TestGame| {
        asked_since(t, from)
            .into_iter()
            .filter(|(_, d)| matches!(d, Decision::ChooseOption { .. }))
            .count()
    };
    assert_eq!(colors(&t), 0);
    let red = |_: &mtg_engine::game::Game, d: &Decision| match d {
        Decision::ChooseOption { options, .. } => options
            .iter()
            .position(|o| o.eq_ignore_ascii_case("red"))
            .map(Answer::Index),
        _ => None,
    };
    crate::r_s03_common::respond(&mut t, P0, red);
    t.resolve_all();
    assert_eq!(colors(&t), 1);
    // P1's red spell can't target P0 or either creature.
    t.set_step(P1, Step::PrecombatMain);
    t.battlefield(P1, "Raging Goblin");
    t.lands(P1, "Mountain", 1);
    let s = t.hand(P1, "Shock");
    let from = t.asked().len();
    t.cast(P1, s).go();
    let cands: Vec<Vec<Entity>> = asked_since(&t, from)
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseTargets { candidates, .. } => Some(candidates),
            _ => None,
        })
        .collect();
    assert_eq!(cands.len(), 1);
    for x in [Entity::Player(P0), obj(a), obj(b)] {
        assert!(!cands[0].contains(&x), "{x:?}");
    }
    assert!(cands[0].contains(&Entity::Player(P1)));
}

#[test]
fn searches_and_targets_that_may_find_nothing() {
    cr!("701.23b", "115.1", "608.2b", "701.24a");
    ruling!("Primal Growth", "You may choose not to find any basic lands when you search.");
    ruling!(
        "Anoint with Affliction",
        "You may choose to target a creature that has mana value greater than 3 even if no player has three of more poison counters."
    );
    ruling!(
        "Devious Cover-Up",
        "You may target zero cards in your graveyard and still shuffle your library."
    );
    supported("Primal Growth");
    supported("Anoint with Affliction");
    supported("Devious Cover-Up");
    // Primal Growth finds nothing; the library is shuffled.
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Forest"]);
    let lib = t.library_size(P0);
    t.answer_choose(P0, &[]);
    cast_spell(&mut t, P0, "Primal Growth", &[]);
    assert_eq!(t.library_size(P0), lib);
    let shuffled = |t: &TestGame| {
        t.g.turn_events
            .iter()
            .filter(|e| matches!(e, Event::Shuffled { player } if *player == P0))
            .count()
    };
    assert_eq!(shuffled(&t), 1);
    // Anoint with Affliction targets Shivan Dragon (mana value 6) and does nothing.
    let mut t = TestGame::new(2);
    let dragon = t.battlefield(P1, "Shivan Dragon");
    cast_spell(&mut t, P0, "Anoint with Affliction", &[obj(dragon)]);
    assert!(t.on_battlefield(dragon));
    assert!(t.in_graveyard(P0, "Anoint with Affliction"));
    // Devious Cover-Up with no graveyard targets: the library is still shuffled.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    give_mana_for(&mut t, P1, "Grizzly Bears");
    let bears = t.hand(P1, "Grizzly Bears");
    let spell = t.cast(P1, bears).go();
    t.answer_targets(P0, &[obj(spell)]);
    t.answer_targets(P0, &[]);
    cast_only(&mut t, P0, "Devious Cover-Up", &[]);
    t.resolve();
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(shuffled(&t), 1);
}

#[test]
fn haunting_voyage_offers_only_creature_types() {
    cr!("205.3m", "608.2c");
    ruling!(
        "Haunting Voyage",
        "You must choose an existing creature type, such as Ox or Coward. You can't choose card types (e.g., artifact), supertypes (e.g., snow), or noncreature subtypes (e.g., Aura)."
    );
    supported("Haunting Voyage");
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    let from = t.asked().len();
    cast_spell(&mut t, P0, "Haunting Voyage", &[]);
    let opts: Vec<Vec<String>> = asked_since(&t, from)
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseOption { options, .. } => Some(options),
            Decision::NameCard { .. } => Some(vec![]),
            _ => None,
        })
        .collect();
    assert_eq!(opts.len(), 1, "{opts:?}");
    let o = &opts[0];
    assert!(o.iter().any(|x| x == "Ox") && o.iter().any(|x| x == "Coward"));
    for bad in ["Artifact", "Snow", "Aura", "Creature"] {
        assert!(!o.iter().any(|x| x == bad), "{bad}");
    }
    let _ = bears;
}

#[test]
fn form_of_the_dinosaur_must_fight_and_an_illegal_target_spares_you() {
    cr!("603.3d", "608.2b");
    ruling!(
        "Form of the Dinosaur",
        "Form of the Dinosaur’s last ability isn’t optional. You have to pick a fight with an opponent’s creature if any are legal targets. If that creature leaves the battlefield after it becomes the target of Form of the Dinosaur’s ability but before it resolves, you won’t be dealt damage."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Form of the Dinosaur");
    let dragon = t.battlefield(P1, "Shivan Dragon");
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "15 damage"), 1);
    assert_eq!(
        crate::r_s25_common::targets_of(&t, t.g.stack[0]),
        vec![obj(dragon)]
    );
    move_to(&mut t, dragon, Zone::Hand(P1));
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}

#[test]
fn storm_elementals_abilities_exile_as_a_cost() {
    cr!("602.2a", "602.2b");
    ruling!(
        "Storm Elemental",
        "Even though the two activated abilities have the same cost, they’re separate. You choose which one you’re activating before you exile the top card of your library"
    );
    supported("Storm Elemental");
    for (top, pt) in [("Snow-Covered Island", (4, 5)), ("Island", (3, 4))] {
        let mut t = TestGame::new(2);
        let e = t.battlefield(P0, "Storm Elemental");
        let top = t.library_top(P0, top);
        t.lands(P0, "Island", 1);
        activate_containing(&mut t, P0, e, "snow land").expect("activate");
        assert_eq!(t.zone(top), Zone::Exile);
        t.resolve_all();
        assert_eq!(t.pt(e), pt);
    }
}

#[test]
fn talent_of_the_telepath_ignores_type_timing_but_not_other_timing() {
    cr!("608.2g", "601.3", "307.1");
    ruling!(
        "Talent of the Telepath",
        "You cast the instant and/or sorcery card(s) from your opponent’s library as Talent of the Telepath is resolving. Ignore timing restrictions based on the card’s type. Other timing restrictions, such as “Cast [this card] only during combat,” must be followed."
    );
    supported("Talent of the Telepath");
    fn pick(name: &'static str) -> fn(&mtg_engine::game::Game, &Decision) -> Option<Answer> {
        match name {
            "Panic" => |g, d| pick_named(g, d, "Panic"),
            _ => |g, d| pick_named(g, d, "Divination"),
        }
    }
    fn pick_named(g: &mtg_engine::game::Game, d: &Decision, name: &str) -> Option<Answer> {
        match d {
            Decision::ChooseEntities { candidates, .. } => Some(Answer::Entities(
                candidates
                    .iter()
                    .filter(|e| matches!(e, Entity::Object(o) if g.obj(*o).chars.name == name))
                    .copied()
                    .collect(),
            )),
            _ => None,
        }
    }
    // (card picked, cards drawn): Divination (a sorcery) is cast while Talent resolves;
    // Panic ("Cast this spell only during combat before blockers are declared") can't be.
    for (name, drawn) in [("Divination", 2), ("Panic", 0)] {
        let mut t = TestGame::new(2);
        t.library_top(P1, "Divination");
        t.library_top(P1, "Panic");
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.answer_targets(P0, &[obj(bears)]);
        crate::r_s03_common::respond(&mut t, P0, pick(name));
        let hand = t.hand_size(P0);
        cast_spell(&mut t, P0, "Talent of the Telepath", &[Entity::Player(P1)]);
        assert_eq!(t.hand_size(P0), hand + drawn, "{name}");
        assert!(t.in_graveyard(P1, name), "{name}");
        let cast = t
            .g
            .history
            .spells_cast
            .iter()
            .filter(|(p, _)| *p == P0)
            .count();
        assert_eq!(cast, if name == "Panic" { 1 } else { 2 }, "{name}");
    }
}
