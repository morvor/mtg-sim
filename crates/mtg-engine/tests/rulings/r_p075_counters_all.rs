//! Rulings batch P075 — "put a +1/+1 counter on each creature you control" abilities: what
//! "each" covers and when it's determined, intervening-if conditions, triggers that resolve
//! before the spell that caused them, replacement choices between two exile effects, last
//! known information, and choices made as the ability resolves.

use crate::r_p075_common::*;
use crate::r_s02_common::create_token;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn set_loyalty(t: &mut TestGame, id: ObjectId, n: u32) {
    t.g.objects[id.0 as usize]
        .counters
        .insert("loyalty".into(), n);
    t.g.recompute();
}

#[test]
fn simple_group_counter_rulings() {
    cr!("122.1", "608.2c");
    ruling!(
        "Steel Overseer",
        "Steel Overseer's ability puts a +1/+1 counter on each artifact creature you control, including Steel Overseer."
    );
    ruling!(
        "Cathars' Crusade",
        "The creature that entered and caused the ability to trigger will also get a +1/+1 counter"
    );
    ruling!(
        "Kazuul Warlord",
        "Kazuul Warlord's ability lets you put +1/+1 counters on all your Ally creatures, not just itself."
    );
    ruling!(
        "Vodalian Wave-Knight",
        "If a creature you control is both a Merfolk and a Knight, it gets one +1/+1 counter."
    );
    ruling!(
        "Filigree Vector",
        "You may choose an artifact creature as a target for both parts of the triggered ability."
    );
    supported("Steel Overseer");
    supported("Cathars' Crusade");
    supported("Kazuul Warlord");
    supported("Vodalian Wave-Knight");
    supported("Filigree Vector");
    // Steel Overseer.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Steel Overseer");
    let o = t.battlefield(P0, "Ornithopter");
    t.activate(P0, s, 0, &[]).expect("activates");
    t.resolve_all();
    assert_eq!((plus(&t, s), plus(&t, o)), (1, 1));
    // Cathars' Crusade.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cathars' Crusade");
    let old = t.battlefield(P0, "Hill Giant");
    let new = t.enter(P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!((plus(&t, old), plus(&t, new)), (1, 1));
    // Kazuul Warlord.
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Kazuul Warlord");
    t.answer_yes(P0, true);
    let d = t.enter(P0, "Drana, Liberator of Malakir");
    t.resolve_all();
    assert_eq!((plus(&t, k), plus(&t, d)), (1, 1));
    // Vodalian Wave-Knight.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vodalian Wave-Knight");
    let gk = t.battlefield(P0, "Galina's Knight");
    draw(&mut t, P0);
    t.resolve_all();
    assert_eq!(plus(&t, gk), 1);
    // Filigree Vector.
    let mut t = TestGame::new(2);
    let o = t.battlefield(P0, "Ornithopter");
    t.answer_targets(P0, &[obj(o)]);
    t.answer_targets(P0, &[obj(o)]);
    t.enter(P0, "Filigree Vector");
    t.resolve_all();
    assert_eq!((plus(&t, o), t.counters(o, "charge")), (1, 1));
}

#[test]
fn brokers_ascendancy_creature_planeswalker_gets_both() {
    cr!("122.1");
    ruling!(
        "Brokers Ascendancy",
        "A permanent that is somehow both a creature and a planeswalker as Brokers Ascendancy's ability resolves will get both a +1/+1 counter and a loyalty counter."
    );
    supported("Brokers Ascendancy");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Brokers Ascendancy");
    let g = t.battlefield(P0, "Gideon Blackblade");
    let loyalty = t.counters(g, "loyalty");
    end_step(&mut t, P0);
    assert_eq!(plus(&t, g), 1);
    assert_eq!(t.counters(g, "loyalty"), loyalty + 1);
}

#[test]
fn leyline_of_abundance_only_for_the_creatures_own_tap_mana_ability() {
    cr!("106.12", "605.1a");
    ruling!(
        "Leyline of Abundance",
        "triggers only if you activate a mana ability of a creature including {T} in its cost"
    );
    supported("Leyline of Abundance");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Leyline of Abundance");
    let elves = t.battlefield(P0, "Llanowar Elves");
    assert!(tap_for_mana(&mut t, P0, elves, "Add"));
    assert_eq!(pool_total(&t, P0), 2, "an additional {{G}}");
    // Tapping a creature for Springleaf Drum's mana ability: no extra mana.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Leyline of Abundance");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let drum = t.battlefield(P0, "Springleaf Drum");
    t.answer_choose(P0, &[obj(elves)]);
    assert!(tap_for_mana(&mut t, P0, drum, "Add"));
    assert!(tapped(&t, elves));
    assert_eq!(pool_total(&t, P0), 1);
}

#[test]
fn belladonna_took_does_nothing_beyond_the_third_time() {
    cr!("608.2c");
    ruling!(
        "Belladonna Took",
        "Belladonna Took's ability has no effect each time beyond the third it resolves in a turn."
    );
    supported("Belladonna Took");
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Belladonna Took");
    let hand = t.hand_size(P0);
    for _ in 0..3 {
        create_token(&mut t, P0, "Treasure");
        t.resolve_all();
    }
    assert_eq!(t.life(P0), 21);
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(plus(&t, b), 1);
    create_token(&mut t, P0, "Treasure");
    t.settle();
    assert_eq!(t.stack_len(), 1, "it still triggers");
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(plus(&t, b), 1);
}

#[test]
fn cosmogrand_zenith_counts_itself_and_resolves_before_the_spell() {
    cr!("603.3", "601.2i", "700.2");
    ruling!(
        "Cosmogrand Zenith",
        "Notably, if your second spell is a creature spell and you choose the second mode for Cosmogrand Zenith’s ability, the resulting creature from that spell won’t get a +1/+1 counter."
    );
    ruling!(
        "Cosmogrand Zenith",
        "Cosmogrand Zenith’s ability will count any spells you’ve cast this turn, which may include Cosmogrand Zenith itself."
    );
    supported("Cosmogrand Zenith");
    let mut t = TestGame::new(2);
    cast_card(&mut t, P0, "Cosmogrand Zenith", &[]);
    t.resolve_all();
    let z = t.named_on_battlefield("Cosmogrand Zenith")[0];
    choose_mode(&mut t, P0, 1);
    cast_card(&mut t, P0, "Grizzly Bears", &[]);
    t.settle();
    assert_eq!(t.stack_len(), 2, "the second spell triggered it");
    t.resolve_all();
    let bears = t.named_on_battlefield("Grizzly Bears")[0];
    assert_eq!(plus(&t, z), 1);
    assert_eq!(
        plus(&t, bears),
        0,
        "still a spell when the counters were put"
    );
}

#[test]
fn dawnstrike_vanguard_checks_as_the_end_step_starts_and_on_resolution() {
    cr!("603.4", "513.1");
    ruling!(
        "Dawnstrike Vanguard",
        "If you don’t control two or more tapped creatures when the ability resolves, the ability won’t do anything."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dawnstrike Vanguard");
    let a = t.battlefield(P0, "Grizzly Bears");
    t.g.tap(a);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0, "only one tapped creature: no trigger");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dawnstrike Vanguard");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.g.tap(a);
    t.g.tap(b);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.g.untap(b);
    t.resolve_all();
    assert_eq!((plus(&t, a), plus(&t, b)), (0, 0));
}

#[test]
fn domri_affects_only_creatures_controlled_as_it_resolves() {
    cr!("611.2c");
    ruling!(
        "Domri, City Smasher",
        "Domri's first and third abilities affect only creatures you control at the time they resolve."
    );
    supported("Domri, City Smasher");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Domri, City Smasher");
    let old = t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, d, 0, &[]).expect("+2");
    t.resolve_all();
    let late = t.battlefield_sick(P0, "Grizzly Bears");
    assert_eq!(t.pt(old), (3, 3));
    assert_eq!(t.pt(late), (2, 2));
    assert!(!has(&t, late, KeywordKind::Haste));
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Domri, City Smasher");
    set_loyalty(&mut t, d, 8);
    let old = t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, d, 2, &[]).expect("-8");
    t.resolve_all();
    let late = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(plus(&t, old), 3);
    assert_eq!(plus(&t, late), 0);
    assert!(!has(&t, late, KeywordKind::Trample));
}

#[test]
fn valiant_veteran_exiles_itself_as_a_cost() {
    cr!("602.2b", "601.2h");
    ruling!(
        "Valiant Veteran",
        "Exiling Valiant Veteran from your graveyard is part of the cost to activate its last ability."
    );
    supported("Valiant Veteran");
    let mut t = TestGame::new(2);
    let v = t.graveyard(P0, "Valiant Veteran");
    let soldier = t.battlefield(P0, "Elite Vanguard");
    give_mana_for(&mut t, P0, "Serra Angel");
    t.lands(P0, "Plains", 1);
    act(&mut t, P0, v, "Exile", &[]).expect("activates from the graveyard");
    assert_eq!(t.stack_len(), 1);
    assert!(
        t.in_exile("Valiant Veteran"),
        "already exiled while on the stack"
    );
    t.resolve_all();
    assert_eq!(plus(&t, soldier), 1);
}

#[test]
fn valentin_and_another_exile_effect_the_opponent_chooses() {
    cr!("616.1", "603.12");
    ruling!(
        "Valentin, Dean of the Vein // Lisette, Dean of the Root",
        "If Valentin’s effect ends up not applying because of this choice, the triggered ability that allows you to pay {2} won’t trigger, even if the creature card ends up in exile."
    );
    for choose_valentin in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(
            P0,
            "Valentin, Dean of the Vein // Lisette, Dean of the Root",
        );
        t.battlefield(P0, "Rest in Peace");
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.lands(P0, "Swamp", 2);
        let from = t.asked().len();
        let i = if choose_valentin { 0 } else { 1 };
        t.answer(P1, DecisionKind::Replacement, Answer::Index(i));
        t.answer_yes(P0, true);
        destroy(&mut t, bears);
        t.resolve_all();
        let options = asked_since(&t, from)
            .into_iter()
            .find_map(|(p, d)| match d {
                Decision::ChooseReplacement { options } => Some((p, options)),
                _ => None,
            });
        let (who, options) = options.expect("a replacement choice");
        assert_eq!(who, P1, "the affected creature's controller chooses");
        let valentin_chosen = options[i].contains("Valentin");
        assert!(t.in_exile("Grizzly Bears"));
        let pests = t.named_on_battlefield("Pest Token").len();
        assert_eq!(pests, valentin_chosen as usize, "{options:?} chose {i}");
    }
}

#[test]
fn ajani_unyielding_life_equals_last_known_power() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Ajani Unyielding",
        "The amount of life gained from Ajani's second ability is equal to power of the creature as it last existed on the battlefield."
    );
    supported("Ajani Unyielding");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Ajani Unyielding");
    let bears = t.battlefield(P1, "Grizzly Bears");
    boost(&mut t, bears, 3, 0);
    t.activate(P0, a, 1, &[obj(bears)]).expect("-2");
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(t.life(P1), 25);
}

#[test]
fn mayaels_aria_parts_happen_sequentially() {
    cr!("608.2c", "503.1a");
    ruling!(
        "Mayael's Aria",
        "if you control a creature with 9 power when the ability starts to resolve, you'll put a +1/+1 counter on each creature you control — and now, since you control a creature with 10 power, you'll gain 10 life."
    );
    ruling!(
        "Mayael's Aria",
        "Whether you control any applicable creatures is checked only when the ability resolves."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mayael's Aria");
    let w = t.battlefield(P0, "Craw Wurm");
    put_counters(&mut t, w, "+1/+1", 3);
    next_upkeep(&mut t);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.pt(w).0, 10);
    assert_eq!(t.life(P0), 30);
    // It triggers with no creatures at all, and does nothing.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mayael's Aria");
    next_upkeep(&mut t);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}

#[test]
fn celes_may_discard_no_cards() {
    cr!("608.2d");
    ruling!(
        "Celes, Rune Knight",
        "You may choose to discard no cards and just draw a card."
    );
    let mut t = TestGame::new(2);
    t.hand(P0, "Grizzly Bears");
    let hand = t.hand_size(P0);
    t.answer_choose(P0, &[]);
    t.enter(P0, "Celes, Rune Knight");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn sidequest_catch_a_fish_may_leave_the_card_on_top() {
    cr!("701.20a", "603.5");
    ruling!(
        "Sidequest: Catch a Fish // Cooking Campsite",
        "You may choose not to reveal it and instead leave it on top of your library."
    );
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Sidequest: Catch a Fish // Cooking Campsite");
    t.answer_yes(P0, false);
    next_upkeep(&mut t);
    let top = t.library_top(P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.g.player(P0).library.last(), Some(&top));
    assert!(tokens(&t, P0).is_empty());
    assert!(t.on_battlefield(s));
}

#[test]
fn gideons_battle_cry_without_creatures_still_searches() {
    cr!("608.2c", "701.23a");
    ruling!(
        "Gideon's Battle Cry",
        "You may cast Gideon’s Battle Cry while you control no creatures. You’ll still search for Gideon."
    );
    supported("Gideon's Battle Cry");
    let mut t = TestGame::new(2);
    let gid = t.library_top(P0, "Gideon, the Oathsworn");
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(gid)]);
    cast_card(&mut t, P0, "Gideon's Battle Cry", &[]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Gideon, the Oathsworn"));
}
