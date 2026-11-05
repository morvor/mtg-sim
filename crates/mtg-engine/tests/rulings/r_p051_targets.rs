//! Rulings batch P051 — targets of drain spells and abilities, and of "drain strength"
//! effects: a spell or ability with an illegal target still affects the legal ones, and
//! one whose targets are all illegal does nothing (CR 608.2b); a spell can't be cast and
//! a triggered ability isn't put on the stack without its required targets (CR 601.2c,
//! 603.3d); any player may be targeted (CR 115.1); reflexive triggered abilities choose
//! their targets as they're put on the stack (CR 603.12); state-based actions wait until
//! a spell or ability has resolved (CR 704.3).

use crate::r_p046_common::fill_pool;
use crate::r_p051_common::*;
use crate::r_s01_common::*;
use crate::r_s05_common::move_to;
use mtg_engine::object::{StackKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const BEARS: &str = "Grizzly Bears";

/// The targets of the stack object `id`, flattened.
fn targets_of(t: &TestGame, id: ObjectId) -> Vec<Entity> {
    t.g.obj(id)
        .stack
        .as_ref()
        .map(|si| {
            si.chosen
                .iter()
                .flat_map(|c| c.targets.iter().flatten().copied())
                .collect()
        })
        .unwrap_or_default()
}

/// The triggered abilities on the stack (top last).
fn triggers(t: &TestGame) -> Vec<ObjectId> {
    t.g.stack
        .iter()
        .copied()
        .filter(|id| {
            t.g.obj(*id)
                .stack
                .as_ref()
                .is_some_and(|si| matches!(si.kind, StackKind::Triggered { .. }))
        })
        .collect()
}

#[test]
fn skulduggery_needs_both_targets_and_affects_the_one_still_legal() {
    cr!("601.2c", "608.2b");
    ruling!(
        "Skulduggery",
        "You can't cast Skulduggery unless you choose both a creature you control and a creature you don't control as targets."
    );
    ruling!(
        "Skulduggery",
        "If either target becomes illegal after you cast Skulduggery but before it resolves, the other is still affected as appropriate."
    );
    supported("Skulduggery");
    // No creature of P1's: it can't be cast.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, BEARS);
    fill_pool(&mut t, P0, 1);
    let s = t.hand(P0, "Skulduggery");
    t.answer_targets(P0, &[obj(mine)]);
    assert!(t.cast(P0, s).try_go().is_err());
    // Each target becomes illegal in turn.
    for gone in [0, 1] {
        let mut t = TestGame::new(2);
        let mine = t.battlefield(P0, BEARS);
        let theirs = t.battlefield(P1, "Craw Wurm");
        cast_pool(&mut t, P0, "Skulduggery", &[obj(mine), obj(theirs)]);
        let (left, kept, pt) = if gone == 0 {
            (mine, theirs, (5, 3))
        } else {
            (theirs, mine, (3, 3))
        };
        move_to(&mut t, left, Zone::Exile);
        t.resolve_all();
        assert_eq!(t.pt(kept), pt, "target {gone} left");
    }
}

#[test]
fn monoist_circuit_feeder_and_gurmag_rakshasa_need_both_targets() {
    cr!("603.3d", "608.2b");
    ruling!(
        "Monoist Circuit-Feeder",
        "If you can’t choose both a creature you control and a creature an opponent controls as targets at the time Monoist Circuit-Feeder’s last ability is put onto the stack, the ability will be removed from the stack and none of its effects will occur."
    );
    ruling!(
        "Gurmag Rakshasa",
        "If you can’t target a creature an opponent controls and a creature you control when Gurmag Rakshasa’s last ability triggers, the ability won’t be put onto the stack."
    );
    supported("Monoist Circuit-Feeder");
    supported("Gurmag Rakshasa");
    for name in ["Monoist Circuit-Feeder", "Gurmag Rakshasa"] {
        // The entering creature is its controller's only creature (a target for "target
        // creature you control"), and P1 has none.
        let mut t = TestGame::new(2);
        let c = t.enter(P0, name);
        t.settle();
        assert!(t.g.stack.is_empty(), "{name}");
        let base = t.pt(c);
        t.resolve_all();
        assert_eq!(t.pt(c), base, "{name}");
    }
    ruling!(
        "Monoist Circuit-Feeder",
        "If one target becomes illegal before Monoist Circuit-Feeder’s last ability resolves, the remaining legal target will still get +X/+0 or -0/-X, as appropriate."
    );
    for gone in [0, 1] {
        let mut t = TestGame::new(2);
        let mine = t.battlefield(P0, BEARS);
        let theirs = t.battlefield(P1, "Craw Wurm");
        t.answer_targets(P0, &[obj(mine)]);
        t.answer_targets(P0, &[obj(theirs)]);
        t.enter(P0, "Monoist Circuit-Feeder");
        t.settle();
        let (left, kept, pt) = if gone == 0 {
            (mine, theirs, (6, 3))
        } else {
            (theirs, mine, (3, 2))
        };
        move_to(&mut t, left, Zone::Exile);
        t.resolve_all();
        assert_eq!(t.pt(kept), pt, "target {gone} left");
    }
}

#[test]
fn consumptive_goo_targeting_itself_survives_the_resolution() {
    cr!("704.3", "608.2c");
    ruling!(
        "Consumptive Goo",
        "If it targets itself, it gets both -1/-1 and the +1/+1 counter during resolution. Since state-based actions (like being at zero toughness) are not checked until resolution is done, it does not die."
    );
    supported("Consumptive Goo");
    let mut t = TestGame::new(2);
    let goo = t.battlefield(P0, "Consumptive Goo");
    assert!(activate_text(&mut t, P0, goo, "-1/-1", &[obj(goo)]));
    t.resolve_all();
    assert!(t.on_battlefield(goo));
    assert_eq!(t.counters(goo, "+1/+1"), 1);
    assert_eq!(t.pt(goo), (1, 1));
}

#[test]
fn path_to_the_world_tree_with_all_targets_illegal_does_nothing() {
    cr!("608.2b");
    ruling!(
        "Path to the World Tree",
        "The last ability of Path to the World Tree always targets an opponent and may target a creature. As the ability tries to resolve, if all its targets are illegal, the ability won't resolve and none of its effects will happen. You won't do any of the cool stuff that ability tells you to do."
    );
    supported("Path to the World Tree");
    for with_creature in [false, true] {
        let mut t = TestGame::new(2);
        let path = t.battlefield(P0, "Path to the World Tree");
        let bears = t.battlefield(P1, BEARS);
        let mut targets = vec![pl(P1)];
        if with_creature {
            targets.push(obj(bears));
        }
        let hand = t.hand_size(P0);
        assert!(activate_text(&mut t, P0, path, "Bear", &targets));
        // P1 gains hexproof (Leyline of Sanctity) and the Bears leave.
        t.battlefield(P1, "Leyline of Sanctity");
        move_to(&mut t, bears, Zone::Exile);
        t.resolve_all();
        assert_eq!(lives(&t, 2), vec![20, 20], "{with_creature}");
        assert_eq!(t.hand_size(P0), hand, "{with_creature}");
        assert!(with_subtype(&t, P0, "Bear").is_empty(), "{with_creature}");
    }
}

#[test]
fn hazels_nocturne_with_no_targets_drains_and_with_illegal_targets_does_nothing() {
    cr!("608.2b", "115.1");
    ruling!(
        "Hazel's Nocturne",
        "You may cast Hazel's Nocturne with no targets just to have each opponent lose 2 life and you gain 2 life. However, if you do choose any targets, and all those targets are illegal as Hazel's Nocturne tries to resolve, it won't resolve and none of its effects will happen. No player will lose or gain life."
    );
    supported("Hazel's Nocturne");
    let mut t = TestGame::new(2);
    cast_pool(&mut t, P0, "Hazel's Nocturne", &[]);
    t.resolve_all();
    assert_eq!(lives(&t, 2), vec![22, 18]);
    let mut t = TestGame::new(2);
    let c = t.graveyard(P0, BEARS);
    t.answer_targets(P0, &[obj(c)]);
    cast_pool(&mut t, P0, "Hazel's Nocturne", &[]);
    move_to(&mut t, c, Zone::Exile);
    t.resolve_all();
    assert_eq!(lives(&t, 2), vec![20, 20]);
}

#[test]
fn punish_ignorance_drains_even_if_the_spell_cant_be_countered() {
    cr!("101.1", "608.2c");
    ruling!(
        "Punish Ignorance",
        "You may target a spell that “can’t be countered” with Punish Ignorance. If you do, the first part of Punish Ignorance’s effect won’t do anything, but the life-loss and life-gain effects will still work."
    );
    supported("Punish Ignorance");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, BEARS);
    t.set_step(P1, Step::PrecombatMain);
    // Abrupt Decay: "This spell can't be countered. Destroy target nonland permanent with
    // mana value 3 or less."
    let decay = cast_pool(&mut t, P1, "Abrupt Decay", &[obj(bears)]);
    cast_pool(&mut t, P0, "Punish Ignorance", &[obj(decay)]);
    t.resolve_all();
    assert_eq!(lives(&t, 2), vec![23, 17]);
    assert!(t.in_graveyard(P0, BEARS), "Abrupt Decay resolved");
}

#[test]
fn any_player_can_be_the_target() {
    cr!("115.1", "115.2");
    ruling!(
        "Inspired Ultimatum",
        "You can target yourself as the player to gain 5 life."
    );
    let mut t = TestGame::new(2);
    cast_pool(&mut t, P0, "Inspired Ultimatum", &[pl(P0), pl(P1)]);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(lives(&t, 2), vec![25, 15]);
    assert_eq!(t.hand_size(P0), hand + 5);

    // Circle of Affliction: "Whenever a source of the chosen color deals damage to you,
    // you may pay {1}. If you do, target player loses 1 life and you gain 1 life."
    ruling!(
        "Circle of Affliction",
        "The player you target doesn’t have to be the controller of the source that dealt damage to you."
    );
    supported("Circle of Affliction");
    let mut t = TestGame::new(3);
    t.answer(
        P0,
        DecisionKind::Option,
        mtg_engine::decision::Answer::Index(3),
    );
    t.enter(P0, "Circle of Affliction");
    t.resolve_all();
    let red = t.battlefield(P1, "Raging Goblin");
    t.answer_targets(P0, &[pl(P2)]);
    t.answer_yes(P0, true);
    fill_pool(&mut t, P0, 1);
    t.g.deal_damage(red, pl(P0), 2, false);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(lives(&t, 3), vec![19, 20, 19]);

    // Axelrod Gunnarson: "Whenever a creature dealt damage by Axelrod Gunnarson this turn
    // dies, you gain 1 life and Axelrod Gunnarson deals 1 damage to target player or
    // planeswalker."
    ruling!(
        "Axelrod Gunnarson",
        "The player you target with Axelrod Gunnarson's ability doesn't have to be the player that controlled the creature that was put into a graveyard."
    );
    supported("Axelrod Gunnarson");
    let mut t = TestGame::new(3);
    let ax = t.battlefield(P0, "Axelrod Gunnarson");
    let bears = t.battlefield(P1, BEARS);
    t.answer_targets(P0, &[pl(P2)]);
    t.g.deal_damage(ax, obj(bears), 2, false);
    t.g.flush_events();
    t.resolve_all();
    assert!(t.in_graveyard(P1, BEARS));
    assert_eq!(lives(&t, 3), vec![21, 20, 19]);
}

#[test]
fn lorehold_command_follows_its_modes_in_order() {
    cr!("608.2c", "700.2");
    ruling!(
        "Lorehold Command",
        "Spell instructions are followed in order. If you choose the first and second modes, the Spirit token Lorehold Command creates will have +1/+0, indestructible, and haste. If you choose the first and fourth modes, you can sacrifice the Spirit token it creates."
    );
    supported("Lorehold Command");
    use mtg_engine::keywords::KeywordKind;
    let mut t = TestGame::new(2);
    fill_pool(&mut t, P0, 10);
    let lc = t.hand(P0, "Lorehold Command");
    t.cast(P0, lc).modes(&[0, 1]).go();
    t.resolve_all();
    let spirit = with_subtype(&t, P0, "Spirit")[0];
    assert_eq!(t.pt(spirit), (4, 2));
    assert!(t.obj_now(spirit).has_keyword(KeywordKind::Indestructible));
    assert!(t.obj_now(spirit).has_keyword(KeywordKind::Haste));
    // First and fourth: the Spirit can be sacrificed (P0 has no other permanent).
    let mut t = TestGame::new(2);
    fill_pool(&mut t, P0, 10);
    let lc = t.hand(P0, "Lorehold Command");
    let hand = t.hand_size(P0);
    t.cast(P0, lc).modes(&[0, 3]).go();
    t.resolve_all();
    assert!(with_subtype(&t, P0, "Spirit").is_empty());
    assert_eq!(t.g.history.sacrificed.len(), 1);
    assert_eq!(t.hand_size(P0), hand - 1 + 2);
}

#[test]
fn ajani_vengeant_targets_any_permanent_and_tracks_it_not_its_controller() {
    cr!("115.1", "611.2c", "502.3");
    ruling!(
        "Ajani Vengeant",
        "You may target any permanent with Ajani Vengeant's first ability. It doesn't have to be tapped."
    );
    ruling!(
        "Ajani Vengeant",
        "The first ability tracks the permanent, but not its controller. If the permanent changes controllers before its first controller's next untap step has come around, then it won't untap during its new controller's next untap step."
    );
    supported("Ajani Vengeant");
    let mut t = TestGame::new(2);
    let ajani = t.battlefield(P0, "Ajani Vengeant");
    let bears = t.battlefield(P1, BEARS);
    assert!(!t.obj_now(bears).tapped);
    t.activate(P0, ajani, 0, &[obj(bears)]).unwrap();
    t.resolve_all();
    // P0 gains control of the untapped Bears, which then become tapped.
    crate::r_s06_common::give_control(&mut t, bears, P0);
    t.g.tap(bears);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert!(t.obj_now(bears).tapped, "doesn't untap in P0's untap step");
    t.advance_to(P0, Step::End);
    t.advance_to(P0, Step::Upkeep);
    assert!(!t.obj_now(bears).tapped, "only the next one");
}

#[test]
fn reflexive_triggers_choose_targets_as_they_are_put_on_the_stack() {
    cr!("603.12", "603.3d");
    ruling!(
        "Dimension X Pizzasaur",
        "You don't choose a second target for Dimension X Pizzasaur's first ability at the time it triggers. Rather, a second \"reflexive\" ability triggers when you put two +1/+1 counters on the target creature this way. You choose a target for that ability as it goes on the stack. Each player may respond to this triggered ability as normal."
    );
    supported("Dimension X Pizzasaur");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, BEARS);
    t.answer_targets(P0, &[obj(mine)]);
    t.enter(P0, "Dimension X Pizzasaur");
    t.settle();
    let first = triggers(&t);
    assert_eq!(first.len(), 1);
    assert_eq!(targets_of(&t, first[0]), vec![obj(mine)]);
    // The creature to destroy arrives only now.
    let theirs = t.battlefield(P1, BEARS);
    t.answer_targets(P0, &[obj(theirs)]);
    t.resolve();
    let second = triggers(&t);
    assert_eq!(second.len(), 1);
    assert_eq!(targets_of(&t, second[0]), vec![obj(theirs)]);
    // P1 may respond: the Bears leave, so the reflexive ability does nothing.
    move_to(&mut t, theirs, Zone::Hand(P1));
    t.resolve_all();
    assert!(t.in_hand(P1, BEARS));
    assert_eq!(t.counters(mine, "+1/+1"), 2);

    ruling!(
        "Sorin, Imperious Bloodlord",
        "You don’t choose a target for Sorin’s second ability at the time you activate it. Rather, a second “reflexive” ability triggers when you sacrifice a Vampire this way. You choose a target for this ability as it goes on the stack. Each player may respond to this triggered ability as normal."
    );
    supported("Sorin, Imperious Bloodlord");
    let mut t = TestGame::new(2);
    let sorin = t.battlefield(P0, "Sorin, Imperious Bloodlord");
    let vamp = t.battlefield(P0, "Vampire Interloper");
    let ab = t.activate(P0, sorin, 1, &[]).unwrap().unwrap();
    assert!(targets_of(&t, ab).is_empty());
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(vamp)]);
    t.answer_targets(P0, &[pl(P1)]);
    t.resolve();
    assert!(t.in_graveyard(P0, "Vampire Interloper"));
    let reflexive = triggers(&t);
    assert_eq!(reflexive.len(), 1);
    assert_eq!(targets_of(&t, reflexive[0]), vec![pl(P1)]);
    t.resolve_all();
    assert_eq!(lives(&t, 2), vec![23, 17]);

    ruling!(
        "Craig Boone, Novac Guard",
        "You don't choose a target for Craig Boone's last ability at the time it triggers. Rather, a second \"reflexive\" ability triggers when you put two quest counters on Craig Boone this way. You choose up to one target for that ability as it goes on the stack. Each player may respond to this triggered ability as normal."
    );
    supported("Craig Boone, Novac Guard");
    let mut t = TestGame::new(2);
    let boone = t.battlefield(P0, "Craig Boone, Novac Guard");
    let bears = t.battlefield(P0, BEARS);
    attack_with(&mut t, &[(boone, pl(P1)), (bears, pl(P1))]);
    let first = triggers(&t);
    assert_eq!(first.len(), 1);
    assert!(targets_of(&t, first[0]).is_empty());
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.answer_targets(P0, &[obj(wurm)]);
    t.resolve();
    assert_eq!(t.counters(boone, "quest"), 2);
    let second = triggers(&t);
    assert_eq!(second.len(), 1);
    assert_eq!(targets_of(&t, second[0]), vec![obj(wurm)]);
}
