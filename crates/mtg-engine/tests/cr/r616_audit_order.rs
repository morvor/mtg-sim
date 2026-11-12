//! CR 616 audit: the order replacement and prevention effects are applied in. The affected
//! player (or the affected object's controller) chooses one effect at a time, whoever
//! controls the effects (616.1), among only those that still apply to the event as it has
//! been modified (616.1f); effects that change under whose control or as a copy of what a
//! permanent enters come first (616.1b, 616.1c), and a permanent's own "as this enters"
//! choice is not a self-replacement effect (614.15) that would jump ahead of them.

use crate::r614_audit_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Builds the scenario `build` with the choices among replacement effects made by `p` in
/// the order `names` gives (each the text of an option to pick), finding each choice's
/// index from a run that stops short of it. Returns the finished game.
fn choose_in_order(p: PlayerId, names: &[&str], build: impl Fn(&[usize]) -> TestGame) -> TestGame {
    let mut picks = vec![];
    for (k, n) in names.iter().enumerate() {
        let t = build(&picks);
        let choices = replacement_choices(&t);
        let (who, opts) = choices
            .get(k)
            .unwrap_or_else(|| panic!("choice {k} not asked: {choices:?}"))
            .clone();
        assert_eq!(who, p, "the affected player chooses: {opts:?}");
        picks.push(option_index(&opts, n));
    }
    build(&picks)
}

/// P0 controls Furnace of Rath and Torbran; P1 controls Urza's Armor. P0 casts Lightning
/// Bolt at P1, whose answers to the choices among replacement effects are `picks`.
fn bolt_through_three_effects(picks: &[usize]) -> TestGame {
    supported("Furnace of Rath");
    supported("Torbran, Thane of Red Fell");
    supported("Urza's Armor");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Furnace of Rath");
    t.battlefield(P0, "Torbran, Thane of Red Fell");
    t.battlefield(P1, "Urza's Armor");
    pick_replacements(&mut t, P1, picks);
    cast_resolve(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    t
}

#[test]
fn the_damaged_player_orders_all_three_effects_one_at_a_time() {
    cr!("616.1", "616.1e", "616.1f", "615.10");
    // Prevent 1, then plus 2, then double: (3 - 1 + 2) * 2 = 8.
    let t = choose_in_order(P1, &["Urza's Armor", "Torbran"], bolt_through_three_effects);
    assert_eq!(t.life(P1), 12);
    let choices = replacement_choices(&t);
    // Two choices: among three effects, then among the two left; the last one applies
    // without a choice. P0, who controls two of them, never chooses.
    assert_eq!(
        choices.iter().map(|c| (c.0, c.1.len())).collect::<Vec<_>>(),
        vec![(P1, 3), (P1, 2)]
    );
    assert!(
        !choices[1].1.iter().any(|o| o.contains("Urza's Armor")),
        "an applied effect isn't offered again (CR 614.5): {:?}",
        choices[1].1
    );
}

#[test]
fn each_order_of_the_three_damage_effects_gives_its_own_total() {
    cr!("616.1e", "616.1f");
    for (order, dealt) in [
        // Double, plus 2, prevent 1: 6, 8, 7.
        (["Furnace of Rath", "Torbran", "Urza's Armor"], 7),
        // Plus 2, double, prevent 1: 5, 10, 9.
        (["Torbran", "Furnace of Rath", "Urza's Armor"], 9),
        // Prevent 1, double, plus 2: 2, 4, 6.
        (["Urza's Armor", "Furnace of Rath", "Torbran"], 6),
    ] {
        // The third effect applies without a choice.
        let t = choose_in_order(P1, &order[..2], bolt_through_three_effects);
        assert_eq!(t.life(P1), 20 - dealt, "order {order:?}");
    }
}

/// P1 owns Daunting Defender (a 3/3 Cleric: "If a source would deal damage to a Cleric
/// creature you control, prevent 1 of that damage") and Furnace of Rath; P0 has taken the
/// Defender with Control Magic. P1 Shocks the Defender; P0's answers are `picks`.
fn shock_stolen_defender(picks: &[usize]) -> TestGame {
    supported("Daunting Defender");
    supported("Control Magic");
    let mut t = TestGame::new(2);
    let defender = t.battlefield(P1, "Daunting Defender");
    t.battlefield(P1, "Furnace of Rath");
    cast_resolve(&mut t, P0, "Control Magic", &[obj(defender)]);
    assert_eq!(controller(&mut t, defender), P0);
    pick_replacements(&mut t, P0, picks);
    cast_resolve(&mut t, P1, "Shock", &[obj(defender)]);
    t
}

#[test]
fn a_stolen_creatures_controller_not_its_owner_orders_damage_effects() {
    cr!("616.1", "616.1e");
    // The Defender's own prevention now protects P0's Clerics; P0 controls the creature,
    // so P0 chooses: prevent 1 first, then double, is 2 damage, and the 3/3 survives.
    let t = choose_in_order(P0, &["Daunting Defender"], shock_stolen_defender);
    let d = t.named_on_battlefield("Daunting Defender");
    assert_eq!(d.len(), 1, "the Defender survives (1 * 2 = 2 damage)");
    assert_eq!(damage_on(&t, d[0]), 2);
    // Doubling first is 4 - 1 = 3: lethal.
    let t = choose_in_order(P0, &["Furnace of Rath"], shock_stolen_defender);
    assert!(t.in_graveyard(P1, "Daunting Defender"));
    for t in [
        choose_in_order(P0, &["Daunting Defender"], shock_stolen_defender),
        choose_in_order(P0, &["Furnace of Rath"], shock_stolen_defender),
    ] {
        assert!(
            replacement_choices(&t).iter().all(|c| c.0 == P0),
            "the owner (P1), who controls both sources, isn't asked"
        );
    }
}

/// P0's Grizzly Bears and Hardened Scales; P1's Vorinclex, Monstrous Raider. P0 casts
/// Dromoka's Gift (bolster 4) on the Bears; P0's answers are `picks`.
fn bolster_through_scales_and_vorinclex(picks: &[usize]) -> TestGame {
    supported("Hardened Scales");
    supported("Vorinclex, Monstrous Raider");
    supported("Dromoka's Gift");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hardened Scales");
    t.battlefield(P1, "Vorinclex, Monstrous Raider");
    pick_replacements(&mut t, P0, picks);
    cast_resolve(&mut t, P0, "Dromoka's Gift", &[]);
    t
}

#[test]
fn counters_on_your_creature_are_ordered_by_you_with_an_opponents_effect_among_them() {
    cr!("616.1", "616.1e", "616.1f");
    ruling!(
        "Hardened Scales",
        "If two or more effects attempt to modify how many counters would be put on a creature you control, you choose the order to apply those effects, no matter who controls the sources of those effects."
    );
    let bears = |t: &TestGame| t.named_on_battlefield("Grizzly Bears")[0];
    // Halved first, then one more: 4 / 2 + 1 = 3.
    let t = choose_in_order(P0, &["Vorinclex"], bolster_through_scales_and_vorinclex);
    assert_eq!(plus_counters(&t, bears(&t)), 3);
    // One more first, then halved: (4 + 1) / 2 = 2.
    let t = choose_in_order(
        P0,
        &["Hardened Scales"],
        bolster_through_scales_and_vorinclex,
    );
    assert_eq!(plus_counters(&t, bears(&t)), 2);
    assert!(replacement_choices(&t).iter().all(|c| c.0 == P0));
}

/// P0 controls Kalitas and Rest in Peace and Murders P1's Grizzly Bears; P1's answers are
/// `picks`.
fn murder_under_kalitas_and_rest_in_peace(picks: &[usize]) -> TestGame {
    supported("Kalitas, Traitor of Ghet");
    supported("Rest in Peace");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kalitas, Traitor of Ghet");
    t.battlefield(P0, "Rest in Peace");
    let bears = t.battlefield(P1, "Grizzly Bears");
    pick_replacements(&mut t, P1, picks);
    cast_resolve(&mut t, P0, "Murder", &[obj(bears)]);
    t
}

#[test]
fn the_dying_creatures_controller_picks_rest_in_peace_over_kalitas() {
    cr!("616.1", "616.1e", "616.1f", "614.6");
    let zombies = |t: &TestGame| t.named_on_battlefield("Zombie Token").len();
    // P1 applies Rest in Peace: the Bears are exiled and Kalitas no longer applies.
    let t = choose_in_order(
        P1,
        &["Rest in Peace"],
        murder_under_kalitas_and_rest_in_peace,
    );
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(zombies(&t), 0);
    // P1 applies Kalitas's: exiled too, and P0 gets the Zombie.
    let t = choose_in_order(P1, &["Kalitas"], murder_under_kalitas_and_rest_in_peace);
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(zombies(&t), 1);
    assert!(!t.in_graveyard(P1, "Grizzly Bears"));
    assert!(replacement_choices(&t).iter().all(|c| c.0 == P1));
}

/// It's P1's main phase; P0 casts Gather Specimens and it resolves.
fn gather_specimens_on_p1s_turn() -> TestGame {
    supported("Gather Specimens");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    cast_resolve(&mut t, P0, "Gather Specimens", &[]);
    t.set_step(P1, Step::PrecombatMain);
    t
}

#[test]
fn voice_of_all_taken_by_gather_specimens_has_its_color_chosen_by_the_new_controller() {
    cr!("616.1b", "614.12a", "614.15");
    ruling!(
        "Gather Specimens",
        "If Voice of All would enter under an opponent's control, you choose a color as it enters under your control."
    );
    supported("Voice of All");
    let mut t = gather_specimens_on_p1s_turn();
    let before = t.asked().len();
    // P0 picks blue (the second color); P1's answers, if asked, would pick red.
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.answer(P1, DecisionKind::Option, Answer::Index(3));
    let voice = cast_resolve(&mut t, P1, "Voice of All", &[]);
    let voice = t.g.current(voice);
    assert!(t.on_battlefield(voice));
    assert_eq!(controller(&mut t, voice), P0);
    assert_eq!(t.obj_now(voice).choices.color, Some(Color::Blue));
    let asked: Vec<PlayerId> = t.asked()[before..]
        .iter()
        .filter(|(_, d)| is_option(d))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(asked, vec![P0], "only the player it enters under chooses");
}

#[test]
fn nyxathid_taken_by_gather_specimens_chooses_an_opponent_of_its_new_controller() {
    cr!("616.1b", "614.12a", "614.15");
    supported("Nyxathid");
    let mut t = gather_specimens_on_p1s_turn();
    // P1 holds three cards, P0 none: Nyxathid gets -1/-1 for each card in the chosen
    // player's hand.
    for _ in 0..3 {
        t.hand(P1, "Grizzly Bears");
    }
    let nyx = cast_resolve(&mut t, P1, "Nyxathid", &[]);
    let nyx = t.g.current(nyx);
    assert_eq!(controller(&mut t, nyx), P0);
    // P0 chooses an opponent of P0's: P1.
    assert_eq!(t.obj_now(nyx).choices.player, Some(P1));
    assert_eq!(t.pt(nyx), (4, 4));
}

#[test]
fn gather_specimens_applies_before_its_casters_essence_of_the_wild_which_then_applies() {
    cr!("616.1b", "616.1c", "616.1f", "616.2");
    supported("Essence of the Wild");
    let mut t = gather_specimens_on_p1s_turn();
    t.battlefield(P0, "Essence of the Wild");
    // Under P1's control the Bears wouldn't be affected by P0's Essence; under P0's they
    // are, so they enter as a copy of it.
    let bears = cast_resolve(&mut t, P1, "Grizzly Bears", &[]);
    let bears = t.g.current(bears);
    assert_eq!(controller(&mut t, bears), P0);
    assert_eq!(t.obj_now(bears).chars.name.as_str(), "Essence of the Wild");
}

#[test]
fn gather_specimens_applies_before_the_casters_opponents_essence_which_then_doesnt() {
    cr!("616.1b", "616.1c", "616.1f");
    supported("Essence of the Wild");
    let mut t = gather_specimens_on_p1s_turn();
    t.battlefield(P1, "Essence of the Wild");
    // The control change must be applied first; then the Bears are no longer a creature
    // P1 controls, so P1's Essence doesn't apply.
    let bears = cast_resolve(&mut t, P1, "Grizzly Bears", &[]);
    let bears = t.g.current(bears);
    assert_eq!(controller(&mut t, bears), P0);
    assert_eq!(t.obj_now(bears).chars.name.as_str(), "Grizzly Bears");
}

#[test]
fn voice_of_all_entering_under_essence_of_the_wild_never_chooses_a_color() {
    cr!("616.1c", "616.1f", "614.15");
    ruling!(
        "Essence of the Wild",
        "Other \"enters\" replacement abilities printed on the creature entering won't be applied because the creature will already be Essence of the Wild at that point"
    );
    supported("Essence of the Wild");
    supported("Voice of All");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Essence of the Wild");
    let before = t.asked().len();
    let voice = cast_resolve(&mut t, P0, "Voice of All", &[]);
    let voice = t.g.current(voice);
    assert_eq!(t.obj_now(voice).chars.name.as_str(), "Essence of the Wild");
    // The copy effect is applied first; then it no longer has Voice of All's "as this
    // enters, choose a color", so no color is chosen.
    assert_eq!(
        t.asked()[before..]
            .iter()
            .filter(|(_, d)| is_option(d))
            .count(),
        0,
        "no color choice: {:?}",
        &t.asked()[before..]
    );
    assert_eq!(t.obj_now(voice).choices.color, None);
}
