//! Rulings batch P149 — costs and casting: cost reductions (Hierophant Bio-Titan, Purple
//! Worm, Tolarian Terror), values determined on resolution (Loot, Pummeler for Hire,
//! Summon: Yojimbo), characteristic-defining abilities in every zone, casting from the top
//! of the library (Mikey & Don, Fblthp), Irencrag Feat's limit, Pyretic Ritual, the
//! planar die, and Swinging Ship's extra combat at odd times.

use crate::r_p149_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// The mana `p` spends casting `spell` (from a full pool), or None if it can't be cast.
fn spent_casting(t: &mut TestGame, p: PlayerId, spell: ObjectId) -> Option<usize> {
    crate::r_p046_common::fill_pool(t, p, 15);
    let before = crate::r_p046_common::pool(t, p);
    let r = t.cast(p, spell).try_go();
    let spent = before - crate::r_p046_common::pool(t, p);
    r.ok().map(|_| spent)
}

// --- Cost reductions -------------------------------------------------------------------------

#[test]
fn hierophant_bio_titan_costs_at_least_gg_and_any_number_of_counters_can_be_removed() {
    cr!("601.2f", "118.7a");
    ruling!(
        "Hierophant Bio-Titan",
        "Heirophant Bio-Titan's first ability can't reduce its cost below {G}{G}. You can remove any number of counters, even if they won't reduce its cost any further."
    );
    supported("Hierophant Bio-Titan");
    // Seven counters removed (fourteen less, for ten generic): {G}{G}.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    put(&mut t, bears, counters::PLUS1, 7);
    let titan = t.hand(P0, "Hierophant Bio-Titan");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(7));
    assert_eq!(spent_casting(&mut t, P0, titan), Some(2));
    assert_eq!(t.counters(bears, counters::PLUS1), 0);
}

#[test]
fn purple_worm_reduces_its_cost_by_at_most_two() {
    cr!("601.2f");
    ruling!(
        "Purple Worm",
        "Purple Worm's first ability will never reduce its cost more than {2}, no matter how many creatures died this turn."
    );
    supported("Purple Worm");
    for died in [0, 1, 3] {
        let mut t = TestGame::new(2);
        for _ in 0..died {
            let b = t.battlefield(P1, "Grizzly Bears");
            kill(&mut t, b);
        }
        let worm = t.hand(P0, "Purple Worm");
        let cost = if died == 0 { 7 } else { 5 };
        assert_eq!(spent_casting(&mut t, P0, worm), Some(cost), "{died} died");
    }
}

#[test]
fn tolarian_terror_costs_at_least_u() {
    cr!("601.2f", "118.7a");
    ruling!(
        "Tolarian Terror",
        "Tolarian Terror's first ability can't reduce its cost below {U}."
    );
    supported("Tolarian Terror");
    let mut t = TestGame::new(2);
    for _ in 0..10 {
        t.graveyard(P0, "Opt");
    }
    let terror = t.hand(P0, "Tolarian Terror");
    assert_eq!(spent_casting(&mut t, P0, terror), Some(1));
}

#[test]
fn tolarian_terror_increases_apply_before_reductions_and_mana_value_is_its_mana_cost() {
    cr!("601.2f", "202.3");
    ruling!(
        "Tolarian Terror",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying, add any cost increases, then apply any cost reductions (such as that of Tolarian Terror). The mana value of the spell is determined by only its mana cost, no matter what the total cost to cast that spell was."
    );
    supported("Sphere of Resistance");
    // {6}{U}, {1} more (Sphere of Resistance), seven less: {U}. (Reductions before the
    // increase would leave {1}{U}.)
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Sphere of Resistance");
    for _ in 0..7 {
        t.graveyard(P0, "Opt");
    }
    let terror = t.hand(P0, "Tolarian Terror");
    assert_eq!(spent_casting(&mut t, P0, terror), Some(1));
    assert_eq!(t.obj_now(terror).zone, Zone::Stack);
    assert_eq!(crate::r_p153_common::mv(&t, terror), 7);
}

#[test]
fn harmonious_grovestriders_power_and_toughness_apply_in_every_zone() {
    cr!("604.3", "208.2a");
    ruling!(
        "Harmonious Grovestrider",
        "The ability that defines Harmonious Grovestrider’s power and toughness works in all zones, not just the battlefield."
    );
    supported("Harmonious Grovestrider");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 3);
    let in_hand = t.hand(P0, "Harmonious Grovestrider");
    let in_yard = t.graveyard(P0, "Harmonious Grovestrider");
    let in_library = t.library_top(P0, "Harmonious Grovestrider");
    t.g.recompute();
    for id in [in_hand, in_yard, in_library] {
        assert_eq!(t.pt(id), (3, 3));
    }
}

// --- Values determined on resolution ---------------------------------------------------------

/// From P1's end step to P0's upkeep, with Loot's trigger on the stack.
fn to_loot_upkeep(t: &mut TestGame) {
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
}

#[test]
fn loot_counts_the_permanent_card_types_including_battle_and_kindred() {
    cr!("205.2a", "110.4", "300.1");
    ruling!(
        "Loot, the Key to Everything",
        "The card types that could appear among your nonland permanents are artifact, battle, creature, enchantment, kindred, and planeswalker."
    );
    supported("Loot, the Key to Everything");
    supported("Bitterblossom");
    supported("Invasion of Zendikar // Awakened Skyclave");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Loot, the Key to Everything");
    for name in [
        "Ornithopter",
        "Bitterblossom",
        "Gideon Jura",
        "Invasion of Zendikar // Awakened Skyclave",
        "Forest",
    ] {
        t.battlefield(P0, name);
    }
    to_loot_upkeep(&mut t);
    t.resolve_all();
    // Artifact, creature, kindred, enchantment, planeswalker, battle: six.
    assert_eq!(t.g.exile.len(), 6);
}

#[test]
fn loot_x_is_determined_as_its_ability_resolves() {
    cr!("608.2h");
    ruling!(
        "Loot, the Key to Everything",
        "The value of X is determined as Loot's last ability resolves."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Loot, the Key to Everything");
    t.battlefield(P0, "Grizzly Bears");
    to_loot_upkeep(&mut t);
    assert_eq!(t.stack_len(), 1);
    // An enchantment arrives before it resolves: X is 2.
    t.battlefield(P0, "Glorious Anthem");
    t.resolve_all();
    assert_eq!(t.g.exile.len(), 2);
}

#[test]
fn loot_exiled_cards_follow_the_normal_timing_rules() {
    cr!("305.1", "117.1a", "601.2f", "601.2h");
    ruling!(
        "Loot, the Key to Everything",
        "You pay all costs and follow all normal timing rules for cards played from exile with Loot's last ability. For example, if the exiled card is a land card, you may play it only during your main phase while the stack is empty."
    );
    use crate::r_s21_common::castable;
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Loot, the Key to Everything");
    t.battlefield(P0, "Grizzly Bears");
    stack_library(&mut t, P0, &["Forest"]);
    to_loot_upkeep(&mut t);
    t.resolve_all();
    let forest = t.g.exile[0];
    assert_eq!(t.g.obj(forest).chars.name, "Forest");
    // In the upkeep: no.
    assert!(t.play_land(P0, forest).is_err());
    // In the main phase with a spell on the stack: no.
    t.advance_to(P0, Step::PrecombatMain);
    cast_at(&mut t, P0, "Shock", &[pl(P1)]);
    assert!(t.play_land(P0, forest).is_err());
    t.resolve_all();
    // With the stack empty: yes.
    assert!(t.play_land(P0, forest).is_ok());
    assert_eq!(t.named_on_battlefield("Forest").len(), 1);
    // A spell is cast with its costs paid, at its normal time: an exiled Grizzly Bears
    // ({1}{G}) not in the upkeep, even with the mana, and in the main phase only with
    // the mana, which pays for it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Loot, the Key to Everything");
    t.battlefield(P0, "Hill Giant");
    stack_library(&mut t, P0, &["Grizzly Bears"]);
    to_loot_upkeep(&mut t);
    t.resolve_all();
    let bears = t.g.exile[0];
    assert_eq!(t.g.obj(bears).chars.name, "Grizzly Bears");
    let forests = t.lands(P0, "Forest", 2);
    assert!(!castable(&mut t, P0, bears), "not in the upkeep");
    t.advance_to(P0, Step::PrecombatMain);
    for f in &forests {
        t.g.tap(*f);
    }
    assert!(!castable(&mut t, P0, bears), "not without the mana");
    for f in &forests {
        t.g.untap(*f);
    }
    assert!(castable(&mut t, P0, bears));
    t.cast(P0, bears).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(crate::r_s04_common::untapped_lands(&t, P0), 0);
}

#[test]
fn pummeler_for_hire_x_is_calculated_once_on_resolution() {
    cr!("608.2h");
    ruling!(
        "Pummeler for Hire",
        "The value of X is calculated only once, as Pummeler for Hire's last ability resolves."
    );
    supported("Pummeler for Hire");
    let mut t = TestGame::new(2);
    let pummeler = enter_targeting(&mut t, P0, "Pummeler for Hire", &[]);
    assert_eq!(t.stack_len(), 1);
    // Pumped in response: 7.
    cast_at(&mut t, P0, "Giant Growth", &[obj(pummeler)]);
    t.resolve();
    t.resolve_all();
    assert_eq!(t.life(P0), 27);
    // Pumped afterwards: no more life.
    cast_at(&mut t, P0, "Giant Growth", &[obj(pummeler)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 27);
}

#[test]
fn summon_yojimbo_chapter_four_x_is_determined_once() {
    cr!("608.2h", "714.2b");
    ruling!(
        "Summon: Yojimbo",
        "The value of X is determined only once, as Summon: Yojimbo's fourth chapter ability resolves."
    );
    let mut t = TestGame::new(3);
    let yojimbo = t.battlefield(P0, "Summon: Yojimbo");
    t.g.objects[yojimbo.0 as usize]
        .counters
        .insert(counters::LORE.into(), 3);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let bears = t.battlefield(P2, "Grizzly Bears");
    put(&mut t, yojimbo, counters::LORE, 1);
    assert_eq!(t.stack_len(), 1);
    // P2's Bears get +3/+3 before it resolves: two opponents count.
    cast_at(&mut t, P0, "Giant Growth", &[obj(bears)]);
    t.resolve();
    t.resolve();
    let treasures = crate::r_s01_common::with_subtype(&t, P0, "Treasure").len();
    assert_eq!(treasures, 2);
    // Later changes don't matter.
    kill(&mut t, wurm);
    t.resolve_all();
    assert_eq!(
        crate::r_s01_common::with_subtype(&t, P0, "Treasure").len(),
        2
    );
}

// --- The top of the library ------------------------------------------------------------------

#[test]
fn mikey_and_don_and_fblthp_let_you_look_at_the_top_card_any_time() {
    cr!("401.5");
    ruling!(
        "Mikey & Don, Party Planners",
        "Mikey & Don's second ability lets you look at the top card of your library whenever you want (with one restriction—see below), even if you don't have priority. This action doesn't use the stack. Knowing what that card is becomes part of the information you have access to, just like you can look at the cards in your hand."
    );
    ruling!(
        "Fblthp, Lost on the Range",
        "While you control Fblthp, you can look at the top card of your library whenever you want (with one restriction; see below), even if you don't have priority. This action doesn't use the stack. Knowing what that card is becomes part of the information you have access to, just like you can look at the cards in your hand."
    );
    use mtg_engine::facedown::can_look_at;
    for name in ["Mikey & Don, Party Planners", "Fblthp, Lost on the Range"] {
        supported(name);
        let mut t = TestGame::new(2);
        let top = t.library_top(P0, "Grizzly Bears");
        assert!(!can_look_at(&t.g, P0, top));
        t.battlefield(P0, name);
        t.g.recompute();
        // In P1's turn, with P1 holding priority.
        t.set_step(P1, Step::DeclareAttackers);
        assert!(can_look_at(&t.g, P0, top), "{name}");
        assert!(!can_look_at(&t.g, P1, top), "{name}");
        assert_eq!(t.stack_len(), 0);
    }
}

#[test]
fn mikey_and_don_cast_mutant_ninja_turtle_spells_from_the_top_with_normal_timing() {
    cr!("601.2", "601.3", "702.8a");
    ruling!(
        "Mikey & Don, Party Planners",
        "Mikey & Don's last ability allows you to cast the top card of your library if the resulting spell is a Mutant, Ninja, or Turtle spell. You pay all required costs, and timing restrictions apply. If that spell is an instant (most likely a kindred instant) or has flash, you'll be able to cast it any time you could cast an instant, even on an opponent's turn."
    );
    supported("Ninja of the Deep Hours");
    supported("Riptide Turtle");
    use crate::r_s21_common::castable;
    // A Ninja creature: in your main phase, not on an opponent's turn.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mikey & Don, Party Planners");
    t.lands(P0, "Island", 4);
    let ninja = t.library_top(P0, "Ninja of the Deep Hours");
    assert!(castable(&mut t, P0, ninja));
    t.set_step(P1, Step::PrecombatMain);
    assert!(!castable(&mut t, P0, ninja));
    // A Turtle with flash: on an opponent's turn too.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mikey & Don, Party Planners");
    t.lands(P0, "Island", 2);
    let turtle = t.library_top(P0, "Riptide Turtle");
    t.set_step(P1, Step::PrecombatMain);
    assert!(castable(&mut t, P0, turtle));
    // Costs are paid: with no mana, not castable.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mikey & Don, Party Planners");
    let turtle = t.library_top(P0, "Riptide Turtle");
    assert!(!castable(&mut t, P0, turtle));
    // Not a Mutant, Ninja or Turtle spell: no.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mikey & Don, Party Planners");
    t.lands(P0, "Forest", 2);
    let bears = t.library_top(P0, "Grizzly Bears");
    assert!(!castable(&mut t, P0, bears));
}

// --- Irencrag Feat and Pyretic Ritual --------------------------------------------------------

#[test]
fn irencrag_feat_restriction_beats_a_permission_to_cast_another_spell() {
    cr!("101.2", "702.85a");
    ruling!(
        "Irencrag Feat",
        "If an effect allows or instructs you to cast another spell after you cast one more, you can't do so. The restriction of Irencrag Feat takes precedence over that permission."
    );
    supported("Irencrag Feat");
    supported("Bloodbraid Elf");
    let mut t = TestGame::new(2);
    cast_at(&mut t, P0, "Irencrag Feat", &[]);
    t.resolve_all();
    // Bloodbraid Elf is the one more spell; its cascade can't cast Shock.
    stack_library(&mut t, P0, &["Shock"]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[pl(P1)]);
    cast_at(&mut t, P0, "Bloodbraid Elf", &[]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Bloodbraid Elf").len(), 1);
    assert_eq!(t.life(P1), 20);
    assert!(!t.in_graveyard(P0, "Shock"));
}

#[test]
fn irencrag_feat_doesnt_let_you_cast_a_spell_you_otherwise_couldnt() {
    cr!("101.2");
    ruling!(
        "Irencrag Feat",
        "Irencrag Feat's effect doesn't allow you to cast any spell you couldn't otherwise cast. For example, if an effect says you can't cast more than one spell each turn, Irencrag Feat won't let you cast a second."
    );
    supported("Rule of Law");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Rule of Law");
    cast_at(&mut t, P0, "Irencrag Feat", &[]);
    t.resolve_all();
    assert!(t.g.players[0].mana_pool.mana.len() >= 7);
    let shock = t.hand(P0, "Shock");
    assert!(!crate::r_s21_common::castable(&mut t, P0, shock));
}

#[test]
fn pyretic_ritual_cant_be_cast_in_the_midst_of_casting_a_spell() {
    cr!("605.1a", "601.2g", "117.1a");
    ruling!(
        "Pyretic Ritual",
        "In the same vein, you can cast Pyretic Ritual only at a time when you could cast any other instant, not at a time when you could activate a mana ability. For example, you can’t cast Pyretic Ritual in the midst of casting another spell; you’d need to cast Pyretic Ritual first."
    );
    supported("Pyretic Ritual");
    supported("Fireball");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    t.hand(P0, "Pyretic Ritual");
    // Fireball with X = 2 ({2}{R}) can't be paid for with two Mountains.
    let fireball = t.hand(P0, "Fireball");
    assert!(t.cast(P0, fireball).x(2).target(pl(P1)).try_go().is_err());
    assert!(t.in_hand(P0, "Pyretic Ritual"));
    // Cast first, it can.
    let ritual = t.g.find_in_zone(Zone::Hand(P0), "Pyretic Ritual")[0];
    t.cast(P0, ritual).go();
    t.resolve_all();
    let fireball = t.g.current(fireball);
    t.cast(P0, fireball).x(2).target(pl(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn pyretic_ritual_uses_the_stack_and_can_be_countered() {
    cr!("605.1a", "117.1a", "701.6a");
    ruling!(
        "Pyretic Ritual",
        "Pyretic Ritual is not a mana ability and doesn’t behave like one. You cast it as a spell. It uses the stack. It can be responded to, and it can be countered."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let ritual = t.hand(P0, "Pyretic Ritual");
    let spell = t.cast(P0, ritual).go();
    assert_eq!(t.stack_len(), 1);
    assert!(t.g.players[0].mana_pool.mana.is_empty());
    cast_at(&mut t, P1, "Counterspell", &[obj(spell)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Pyretic Ritual"));
    assert!(t.g.players[0].mana_pool.mana.is_empty());
}

// --- Dice ------------------------------------------------------------------------------------

#[test]
fn rolling_the_planar_die_triggers_whenever_you_roll_a_die_with_no_result() {
    cr!("706.7", "901.9d", "603.2");
    ruling!(
        "Atomwheel Acrobats",
        "If an ability triggers \"whenever you roll a die,\" it will trigger whenever you roll any die, including the planar die. This is a change from previous Un- rules. Some abilities use the result to determine part of the effect. If you get a non-numerical result (currently just the planar die, but the future is long), that part of the effect won't do anything."
    );
    supported("The Space Family Goblinson");
    supported("As Luck Would Have It");
    supported("Atomwheel Acrobats");
    let mut t = crate::r_s19_common::planechase_game(2);
    crate::r_s19_common::start_planar_deck(&mut t, P0, &["Goldmeadow", "Krosa"]);
    // The Space Family Goblinson ("Whenever you roll a die, put a +1/+1 counter on ~."),
    // As Luck Would Have It ("Whenever you roll a die, put a number of luck counters on
    // this enchantment equal to the result."), Atomwheel Acrobats ("Whenever you roll a 1
    // or 2, ...").
    let goblinson = t.battlefield(P0, "The Space Family Goblinson");
    let luck = t.battlefield(P0, "As Luck Would Have It");
    let acrobats = t.battlefield(P0, "Atomwheel Acrobats");
    // A blank face.
    t.g.dice.loaded.push_back(3);
    mtg_engine::planechase::roll_planar_die(&mut t.g, P0);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "roll a die"), 2);
    t.resolve_all();
    assert_eq!(t.counters(goblinson, counters::PLUS1), 1);
    assert_eq!(t.counters(luck, "luck"), 0);
    assert_eq!(t.counters(acrobats, counters::PLUS1), 0);
}

/// `p` rolls to visit their Attractions with the given die result.
fn roll_to_visit(t: &mut TestGame, p: PlayerId, result: u32) {
    t.g.dice.loaded.push_back(result);
    mtg_engine::variants::roll_to_visit(&mut t.g, p);
    t.g.flush_events();
    t.settle();
}

/// Swinging Ship's lit-up number.
fn ship_light(t: &TestGame, ship: ObjectId) -> u32 {
    t.obj_now(ship).card.as_ref().unwrap().attraction_lights[0] as u32
}

/// The combat phases that have begun this turn.
fn combats_this_turn(t: &TestGame) -> u32 {
    t.g.turn.combat_phases
}

#[test]
fn swinging_ship_visited_after_the_first_combat_does_nothing() {
    cr!("500.8", "701.52a");
    ruling!(
        "Swinging Ship",
        "If you somehow visit Swinging Ship after the first combat phase of a turn has ended, it won’t have any effect. No additional combat phase will be created."
    );
    supported("Swinging Ship");
    let mut t = TestGame::new(2);
    let ship = t.battlefield(P0, "Swinging Ship");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, pl(P1))]);
    t.advance_to(P0, Step::PostcombatMain);
    assert_eq!(combats_this_turn(&t), 1);
    let n = ship_light(&t, ship);
    roll_to_visit(&mut t, P0, n);
    t.resolve_all();
    t.advance_to(P0, Step::End);
    assert_eq!(combats_this_turn(&t), 1);
}

#[test]
fn swinging_ship_visited_on_an_opponents_turn_gives_them_the_extra_combat() {
    cr!("500.8", "506.1", "701.52a");
    ruling!(
        "Swinging Ship",
        "If you somehow visit Swinging Ship during an opponent’s turn before their first combat phase ends, they will get an additional combat phase. Their creatures that attacked will untap."
    );
    let mut t = TestGame::new(2);
    let ship = t.battlefield(P0, "Swinging Ship");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    let n = ship_light(&t, ship);
    roll_to_visit(&mut t, P0, n);
    t.resolve_all();
    // P1's first combat: the Bears attack.
    t.answer(
        P1,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bears, pl(P0))]),
    );
    crate::r_s21_common::go_to(&mut t, Step::BeginningOfCombat);
    crate::r_s21_common::go_to(&mut t, Step::EndOfCombat);
    assert!(t.obj_now(bears).tapped);
    // The additional combat, this turn: untapped at its beginning.
    let turn = t.g.turn.number;
    t.advance_to(P1, Step::BeginningOfCombat);
    assert_eq!(t.g.turn.number, turn);
    t.resolve_all();
    assert_eq!(combats_this_turn(&t), 2);
    assert!(!t.obj_now(bears).tapped);
    assert_eq!(t.life(P0), 18);
}
