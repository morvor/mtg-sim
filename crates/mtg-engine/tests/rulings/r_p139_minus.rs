//! Rulings batch P139 — "-N/-N until end of turn" effects: which creatures they affect
//! (those on the battlefield as they resolve, CR 611.2c), targets (illegal targets,
//! mandatory targets, the source itself), and state-based actions afterwards (CR 704.5f,
//! 704.5g).

use crate::r_p137_common::obj;
use crate::r_s01_common::supported;
use crate::r_s02_common::destroy;
use crate::r_s04_common::add_mana;
use crate::r_s06_common::damage;
use crate::r_s25_common::cast_new;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Mishra's Factory becomes a 2/2 creature (paying {1} from the pool).
fn animate_factory(t: &mut TestGame, factory: ObjectId) {
    add_mana(t, P0, ManaType::C, 1);
    t.activate(P0, factory, 1, &[]).unwrap();
    t.resolve_all();
}

#[test]
fn global_shrinks_affect_only_creatures_there_as_they_resolve() {
    cr!("611.2c");
    // (card, how its effect is started, the -N/-N, ruling cited)
    type Start = fn(&mut TestGame);
    fn dies(t: &mut TestGame, name: &str) {
        let c = t.battlefield(P0, name);
        destroy(t, c);
        t.resolve_all();
    }
    fn cast(t: &mut TestGame, name: &str) {
        cast_new(t, P0, name, &[]);
        t.resolve_all();
    }
    let cases: [(&str, Start, i32, bool); 5] = [
        ("Havoc Demon", |t| dies(t, "Havoc Demon"), 5, true),
        ("Nausea", |t| cast(t, "Nausea"), 1, true),
        ("Feast of Succession", |t| cast(t, "Feast of Succession"), 4, false),
        ("Shrivel", |t| cast(t, "Shrivel"), 1, false),
        ("Death's-Head Buzzard", |t| dies(t, "Death's-Head Buzzard"), 1, false),
    ];
    ruling!(
        "Havoc Demon",
        "All creatures on the battlefield when Havoc Demon’s triggered ability resolves are affected. Ones that enter the battlefield or become creatures later in the turn are not."
    );
    ruling!(
        "Nausea",
        "All creatures on the battlefield when Nausea resolves are affected. Ones that enter the battlefield or become creatures later in the turn are not."
    );
    ruling!(
        "Feast of Succession",
        "Feast of Succession affects only creatures on the battlefield at the time it resolves. Creatures that enter the battlefield later in the turn won't get -4/-4."
    );
    ruling!(
        "Shrivel",
        "Only creatures on the battlefield as Shrivel resolves are affected."
    );
    ruling!(
        "Death's-Head Buzzard",
        "The last ability of Death's-Head Buzzard affects only creatures on the battlefield at the time it resolves. Creatures that enter the battlefield later in the turn won't get -1/-1."
    );
    for (card, start, n, animate) in cases {
        supported(card);
        let mut t = TestGame::new(2);
        let wurm = t.battlefield(P1, "Colossal Dreadmaw");
        let factory = t.battlefield(P0, "Mishra's Factory");
        start(&mut t);
        assert_eq!(t.pt(wurm), (6 - n, 6 - n), "{card}");
        let late = t.battlefield(P1, "Colossal Dreadmaw");
        t.g.recompute();
        assert_eq!(t.pt(late), (6, 6), "{card}: entered later");
        if animate {
            animate_factory(&mut t, factory);
            assert_eq!(t.pt(factory), (2, 2), "{card}: became a creature later");
        }
    }
}

#[test]
fn feast_of_succession_with_no_creatures_still_makes_you_the_monarch() {
    cr!("724.1");
    ruling!(
        "Feast of Succession",
        "You can cast Feast of Succession to become the monarch while no creatures are on the battlefield."
    );
    supported("Feast of Succession");
    let mut t = TestGame::new(2);
    cast_new(&mut t, P0, "Feast of Succession", &[]);
    t.resolve_all();
    assert_eq!(t.g.monarch, Some(P0));
}

#[test]
fn hagra_sharpshooter_minus_one_isnt_a_counter() {
    cr!("611.2c", "122.3");
    ruling!(
        "Hagra Sharpshooter",
        "An effect that gives a creature -1/-1 is different from putting a -1/-1 counter on that creature. Notably, if the target of Hagra Sharpshooter’s ability has any +1/+1 counters on it, those counters will remain."
    );
    supported("Hagra Sharpshooter");
    let mut t = TestGame::new(2);
    let hagra = t.battlefield(P0, "Hagra Sharpshooter");
    let bears = t.battlefield(P1, "Grizzly Bears");
    crate::r_s29_common::put_counters(&mut t, bears, "+1/+1", 2);
    add_mana(&mut t, P0, ManaType::B, 1);
    add_mana(&mut t, P0, ManaType::C, 4);
    t.activate(P0, hagra, 0, &[obj(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 2);
    assert_eq!(t.counters(bears, "-1/-1"), 0);
    assert_eq!(t.pt(bears), (3, 3));
}

#[test]
fn kaervek_makes_marked_damage_lethal() {
    cr!("704.5g", "120.6");
    ruling!(
        "Kaervek, the Spiteful",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to creatures may become lethal if Kaervek enters the battlefield during that turn."
    );
    supported("Kaervek, the Spiteful");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    damage(&mut t, pyro, 1, bears);
    assert!(t.on_battlefield(bears));
    t.enter(P0, "Kaervek, the Spiteful");
    t.settle();
    assert!(!t.on_battlefield(bears));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn bile_blight_has_one_target_and_needs_it_legal() {
    cr!("115.1", "608.2b", "702.11b");
    ruling!(
        "Bile Blight",
        "Bile Blight has only one target. Other creatures with that name are not targeted. For example, a creature with hexproof will still get -3/-3 if it has the same name as the target creature."
    );
    supported("Bile Blight");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Gladecover Scout");
    let theirs = t.battlefield(P1, "Gladecover Scout");
    cast_new(&mut t, P0, "Bile Blight", &[obj(mine)]);
    t.resolve_all();
    assert!(!t.on_battlefield(mine) && !t.on_battlefield(theirs));
    ruling!(
        "Bile Blight",
        "If the target creature is an illegal target as Bile Blight tries to resolve, it won't resolve and none of its effects will happen. No creature will get -3/-3."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Bile Blight", &[obj(a)]);
    destroy(&mut t, a);
    t.resolve_all();
    assert!(t.on_battlefield(b));
    assert_eq!(t.pt(b), (2, 2));
}

#[test]
fn illegal_targets_stop_the_whole_spell() {
    cr!("608.2b");
    ruling!(
        "Moment of Craving",
        "If the target creature is an illegal target by the time Moment of Craving tries to resolve, it won't resolve and none of its effects will happen. You won't gain 2 life."
    );
    supported("Moment of Craving");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Moment of Craving", &[obj(bears)]);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);

    ruling!(
        "Blight Grenade",
        "If Blight Grenade's target is no longer legal as it tries to resolve, it will be removed from the stack and no creatures will get -3/-3."
    );
    supported("Blight Grenade");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    cast_new(&mut t, P0, "Blight Grenade", &[obj(bears)]);
    destroy(&mut t, bears);
    t.resolve_all();
    assert!(t.on_battlefield(giant));
    assert_eq!(t.pt(giant), (3, 3));

    ruling!(
        "Rat Out",
        "You can cast Rat Out without a target just to create a Rat token. However, if you do choose a target, and that target is illegal at the time Rat Out tries to resolve, the spell won't resolve and none of its effects will happen. You won't create a Rat token."
    );
    supported("Rat Out");
    let mut t = TestGame::new(2);
    cast_new(&mut t, P0, "Rat Out", &[]);
    t.resolve_all();
    assert_eq!(crate::r_s01_common::tokens(&t, P0).len(), 1);
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Rat Out", &[obj(bears)]);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(crate::r_s01_common::tokens(&t, P0).len(), 1, "no second Rat");
}

#[test]
fn moment_of_craving_lifegain_triggers_see_the_creature_before_it_dies() {
    cr!("608.3", "704.5f", "603.3");
    ruling!(
        "Moment of Craving",
        "The target creature will still be on the battlefield when you gain life, even if its toughness has been reduced to 0 or less. Any abilities it has that interact with gaining life do so as appropriate. If any abilities trigger on you gaining life, the creature will be put into its owner's graveyard after that ability triggers but before it resolves."
    );
    supported("Moment of Craving");
    let mut t = TestGame::new(2);
    let pridemate = t.battlefield(P0, "Ajani's Pridemate");
    cast_new(&mut t, P0, "Moment of Craving", &[obj(pridemate)]);
    t.resolve();
    assert_eq!(t.life(P0), 22);
    assert!(!t.on_battlefield(pridemate));
    assert!(t.in_graveyard(P0, "Ajani's Pridemate"));
    assert_eq!(
        crate::r_s01_common::triggers_on_stack(&t, "Whenever you gain life"),
        1,
        "its own trigger triggered while it was still there"
    );
}

#[test]
fn shrink_triggers_with_forced_targets() {
    cr!("603.3d", "115.1");
    ruling!(
        "Fourth Bridge Prowler",
        "Fourth Bridge Prowler can be the target of its own ability."
    );
    supported("Fourth Bridge Prowler");
    let mut t = TestGame::new(2);
    t.answer_yes(P0, true);
    let prowler = t.enter(P0, "Fourth Bridge Prowler");
    t.answer_targets(P0, &[obj(prowler)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Fourth Bridge Prowler"));

    ruling!(
        "Blister Beetle",
        "If there are no other creatures on the battlefield when Blister Beetle enters, it must target itself."
    );
    supported("Blister Beetle");
    let mut t = TestGame::new(2);
    let beetle = t.enter(P0, "Blister Beetle");
    t.resolve_all();
    assert!(!t.on_battlefield(beetle));
    assert!(t.in_graveyard(P0, "Blister Beetle"));

    ruling!(
        "Hornet Harasser",
        "Hornet Harasser’s ability is mandatory. If you’re the only player who controls any creatures, you must give one of your own creatures -2/-2."
    );
    supported("Hornet Harasser");
    let mut t = TestGame::new(2);
    let harasser = t.battlefield(P0, "Hornet Harasser");
    let bears = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, harasser);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));

    ruling!(
        "Weed-Pruner Poplar",
        "The ability is mandatory. If you’re the only player who controls any other creatures, you must target one of your own creatures with the ability."
    );
    supported("Weed-Pruner Poplar");
    let mut t = TestGame::new(2);
    let poplar = t.battlefield(P0, "Weed-Pruner Poplar");
    let elf = t.battlefield(P0, "Llanowar Elves");
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(!t.on_battlefield(elf));
    assert_eq!(t.pt(poplar), (3, 3));
}

#[test]
fn skulking_killer_triggers_anyway_and_checks_on_resolution() {
    cr!("603.4", "608.2c");
    ruling!(
        "Skulking Killer",
        "Skulking Killer's ability triggers even if all your opponents control more than one creature. If the targeted creature's controller controls any other creatures as the ability resolves, the ability won't do anything. Notably, that player could cast a creature spell with flash or create a creature token in response to avoid the -2/-2 effect."
    );
    supported("Skulking Killer");
    // Two creatures: it triggers, and does nothing.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(bears)]);
    t.enter(P0, "Skulking Killer");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.pt(bears), (2, 2));
    // One creature, and a second arrives in response.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    t.enter(P0, "Skulking Killer");
    t.settle();
    crate::r_s02_common::create_token(&mut t, P1, "Grizzly Bears");
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    // One creature: it gets -2/-2.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    t.enter(P0, "Skulking Killer");
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
}

#[test]
fn stabbing_pain_shrinks_then_taps_then_state_based_actions() {
    cr!("704.5f", "704.5g", "608.2c", "603.2");
    ruling!(
        "Stabbing Pain",
        "If Stabbing Pain targets an untapped creature with 1 toughness, that creature gets -1/-1, then it becomes tapped, then (after Stabbing Pain resolves) it’s put into the graveyard as a state-based action. Similarly, if Stabbing Pain targets an untapped creature with 1 less than lethal damage marked on it (such as a 3/3 creature with 2 damage marked on it), that creature gets -1/-1, then it becomes tapped, then (after Stabbing Pain resolves) it’s destroyed as a state-based action. In each case, any abilities that trigger when that creature becomes tapped will trigger."
    );
    supported("Stabbing Pain");
    // Judge of Currents: "Whenever a Merfolk you control becomes tapped, you may gain 1
    // life."
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Judge of Currents");
    let merfolk = t.battlefield(P1, "Merfolk of the Pearl Trident");
    t.answer_yes(P1, true);
    cast_new(&mut t, P0, "Stabbing Pain", &[obj(merfolk)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Merfolk of the Pearl Trident"));
    assert_eq!(t.life(P1), 21);
    // A 2/3 Merfolk with 2 damage marked on it.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Judge of Currents");
    let merfolk = t.battlefield(P1, "Streambed Aquitects");
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    damage(&mut t, pyro, 2, merfolk);
    t.answer_yes(P1, true);
    cast_new(&mut t, P0, "Stabbing Pain", &[obj(merfolk)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Streambed Aquitects"));
    assert_eq!(t.life(P1), 21);

    ruling!(
        "Stabbing Pain",
        "You may target any creature with Stabbing Pain. It’s okay if it’s already tapped."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.objects[giant.0 as usize].tapped = true;
    cast_new(&mut t, P0, "Stabbing Pain", &[obj(giant)]);
    t.resolve_all();
    assert_eq!(t.pt(giant), (2, 2));
    assert!(t.obj_now(giant).tapped);
}

#[test]
fn night_of_souls_betrayal_legend_rule_and_shrunken_creatures_together() {
    cr!("704.3", "704.5j", "704.5f");
    ruling!(
        "Night of Souls' Betrayal",
        "If a second Night of Souls’ Betrayal comes under your control, you’ll put one into its owner’s graveyard due to the “legend rule” at the same time that any creatures getting -2/-2 are put into their owner’s graveyard for having 0 or less toughness."
    );
    supported("Night of Souls' Betrayal");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Night of Souls' Betrayal");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.recompute();
    assert_eq!(t.pt(bears), (1, 1));
    t.enter(P0, "Night of Souls' Betrayal");
    t.settle();
    assert_eq!(t.named_on_battlefield("Night of Souls' Betrayal").len(), 1);
    assert!(!t.on_battlefield(bears));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}
