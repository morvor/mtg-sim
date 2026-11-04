//! Rulings batch P096 — effects that change how much life you gain (Angel of Vitality,
//! Honor Troll, Boon Reflection, Alhammarret's Archive: CR 614.1a, 616.1) and abilities
//! that look at your life total (Honor Troll, Angel of Vitality, Path of Bravery, Felidar
//! Sovereign, Angel of Destiny), including in Two-Headed Giant games (CR 810.9) and with
//! life gained at the same time combat damage is dealt (CR 510.2, 704.3).

use crate::r_p096_common::*;
use crate::r_s01_common::{supported, triggers_on_stack};
use crate::r_s25_common::cast_new;
use crate::r_s29_common::cast_and_resolve;
use crate::r_s33_common::{set_life, two_headed_giant};
use mtg_engine::game::GameConfig;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

// ---------------------------------------------------------------------------------------
// Life gained at the same time as combat damage
// ---------------------------------------------------------------------------------------

/// P0 (at `life`) controls `name` and Vampire Nighthawk (2/3 flying, deathtouch, lifelink);
/// both attack (with `nighthawk`) and P1's Hill Giant (3/3) blocks `name`. Returns the
/// permanent named `name`.
fn honor_troll_blocked(life: i32, nighthawk: bool) -> (TestGame, ObjectId) {
    let mut t = TestGame::new(2);
    set_life(&mut t, P0, life);
    let troll = t.battlefield(P0, "Honor Troll");
    let giant = t.battlefield(P1, "Hill Giant");
    let mut attackers = vec![(troll, Entity::Player(P1))];
    if nighthawk {
        let hawk = t.battlefield(P0, "Vampire Nighthawk");
        attackers.push((hawk, Entity::Player(P1)));
    }
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&attackers, &[(giant, troll)]);
    t.resolve_all();
    (t, troll)
}

#[test]
fn honor_troll_life_gained_with_combat_damage_applies_before_lethal_damage_is_checked() {
    cr!("510.2", "704.3", "704.5g", "613.4c");
    ruling!(
        "Honor Troll",
        "Honor Troll's last ability applies before checking whether it's destroyed for having lethal damage marked on it."
    );
    supported("Honor Troll");
    // "If you would gain life, you gain that much life plus 1 instead. This creature gets
    // +2/+1 as long as you have 25 or more life." At 24 life, the 2/3 Troll is dealt 3
    // damage by the Giant while the Nighthawk's 2 lifelink damage gains P0 2 + 1 life:
    // P0 is at 27 when state-based actions are checked, so the Troll is a 4/4 and lives.
    let (t, troll) = honor_troll_blocked(24, true);
    assert_eq!(t.life(P0), 27);
    assert!(t.on_battlefield(troll));
    assert_eq!(t.pt(troll), (4, 4));
    assert_eq!(t.obj_now(troll).damage, 3);
    // Without the life gain, the 3 damage is lethal.
    let (t, troll) = honor_troll_blocked(24, false);
    assert_eq!(t.life(P0), 24);
    assert!(!t.on_battlefield(troll));
}

/// P0 (at `life`) controls Path of Bravery and attacks P1 with Grizzly Bears enchanted
/// with Lifelink ("Enchanted creature has lifelink."), which P1's Grizzly Bears block.
/// Returns P0's Bears.
fn path_of_bravery_blocked(life: i32) -> (TestGame, ObjectId) {
    let mut t = TestGame::new(2);
    set_life(&mut t, P0, life);
    t.battlefield(P0, "Path of Bravery");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let aura = t.battlefield(P0, "Lifelink");
    t.g.attach(aura, Entity::Object(bears));
    let blocker = t.battlefield(P1, "Grizzly Bears");
    t.g.recompute();
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P1))], &[(blocker, bears)]);
    t.resolve_all();
    (t, bears)
}

#[test]
fn path_of_bravery_life_gained_with_combat_damage_applies_before_lethal_damage_is_checked() {
    cr!("510.2", "704.3", "704.5g", "613.4c");
    ruling!(
        "Path of Bravery",
        "If a creature you control is dealt damage at the same time that you gain life (most likely because a source you control with lifelink deals damage at the same time), and your life total becomes greater than or equal to your starting life total, apply Path of Bravery’s effect before considering whether or not the damage dealt to that creature is lethal."
    );
    supported("Path of Bravery");
    // "As long as your life total is greater than or equal to your starting life total,
    // creatures you control get +1/+1. Whenever one or more creatures you control attack,
    // you gain life equal to the number of attacking creatures." At 17 life, the attack
    // trigger brings P0 to 18. P0's Bears (2/2) are dealt 2 damage by the blocker as their
    // 2 lifelink damage brings P0 to 20: they're a 3/3 when the damage is checked.
    let (t, bears) = path_of_bravery_blocked(17);
    assert_eq!(t.life(P0), 20);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.pt(bears), (3, 3));
    assert_eq!(t.obj_now(bears).damage, 2);
    // One life short (19): the 2/2 dies.
    let (t, bears) = path_of_bravery_blocked(16);
    assert_eq!(t.life(P0), 19);
    assert!(!t.on_battlefield(bears));
}

#[test]
fn path_of_bravery_counts_creatures_put_onto_the_battlefield_attacking_before_it_resolves() {
    cr!("508.4", "603.2", "608.2h");
    ruling!(
        "Path of Bravery",
        "If a creature enters the battlefield attacking after attacking creatures have been declared, the ability won’t trigger again because you didn’t declare that creature as an attacking creature. However, if such a creature enters the battlefield before the ability resolves, it will be counted when determining how much life is gained."
    );
    supported("Hero of Bladehold");
    // Hero of Bladehold attacks: "Whenever this creature attacks, create two 1/1 white
    // Soldier creature tokens that are tapped and attacking." P0 orders that trigger
    // above Path of Bravery's, so the tokens are attacking when Path's resolves.
    let mut t = TestGame::new(2);
    let path = t.battlefield(P0, "Path of Bravery");
    let hero = t.battlefield(P0, "Hero of Bladehold");
    crate::r_s03_common::respond(&mut t, P0, tokens_first);
    declare(&mut t, &[hero]);
    assert_eq!(triggers_from(&t, path), 1);
    let from = t.g.turn_events.len();
    t.resolve_all();
    // Path didn't trigger again for the tokens; its one trigger counted three attackers.
    assert_eq!(crate::r_s25_common::creature_tokens(&t, P0), 2);
    assert_eq!(gains_since(&t, from, P0), vec![3]);
    assert_eq!(triggers_from(&t, path), 0);
}

#[test]
fn path_of_bravery_triggers_once_for_all_attackers() {
    cr!("508.1m", "603.2c", "119.9");
    ruling!(
        "Path of Bravery",
        "Path of Bravery’s triggered ability triggers only once, after attacking creatures are declared. For example, if you attack with five creatures, you’ll gain 5 life as a single event."
    );
    let mut t = TestGame::new(2);
    let path = t.battlefield(P0, "Path of Bravery");
    let bears: Vec<ObjectId> = (0..5).map(|_| t.battlefield(P0, "Grizzly Bears")).collect();
    declare(&mut t, &bears);
    assert_eq!(triggers_from(&t, path), 1);
    let from = t.g.turn_events.len();
    t.resolve_all();
    assert_eq!(gains_since(&t, from, P0), vec![5]);
}

// ---------------------------------------------------------------------------------------
// Angel of Destiny
// ---------------------------------------------------------------------------------------

#[test]
fn angel_of_destiny_combat_damage_redirected_to_you_gains_twice_as_one_event() {
    cr!("510.2", "119.9", "614.1a");
    ruling!(
        "Angel of Destiny",
        "If a creature you control somehow deals combat damage to you (most likely because its combat damage was redirected), you gain twice that much life as a single life-gaining event."
    );
    supported("Angel of Destiny");
    supported("Harm's Way");
    // "Whenever a creature you control deals combat damage to a player, you and that
    // player each gain that much life." P0's Grizzly Bears attack P1, who casts Harm's Way
    // ("The next 2 damage that a source of your choice would deal to you and/or
    // permanents you control this turn is dealt to any target instead.") choosing the
    // Bears and targeting P0: the Bears' 2 combat damage is dealt to P0.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Angel of Destiny");
    let bears = t.battlefield(P0, "Grizzly Bears");
    declare(&mut t, &[bears]);
    t.answer_choose(P1, &[Entity::Object(bears)]);
    cast_new(&mut t, P1, "Harm's Way", &[Entity::Player(P0)]);
    t.resolve();
    let from = t.g.turn_events.len();
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    // P0 was dealt 2 damage, then gained 4 life as one event.
    assert_eq!(gains_since(&t, from, P0), vec![4]);
    assert_eq!(gains_since(&t, from, P1), Vec::<u32>::new());
    assert_eq!(t.life(P0), 22);
    // Damage dealt to the opponent: each of the two players gains 2.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Angel of Destiny");
    let bears = t.battlefield(P0, "Grizzly Bears");
    declare(&mut t, &[bears]);
    let from = t.g.turn_events.len();
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(gains_since(&t, from, P0), vec![2]);
    assert_eq!(gains_since(&t, from, P1), vec![2]);
    assert_eq!((t.life(P0), t.life(P1)), (22, 20));
}

#[test]
fn angel_of_destiny_two_headed_giant_compares_the_team_life_total() {
    cr!("810.9", "810.4", "119.1a");
    ruling!(
        "Angel of Destiny",
        "In a Two-Headed Giant game, Angel of Destiny's last ability can trigger as long as your team's life total is greater than your team's starting life total by 15 or more life."
    );
    // "At the beginning of your end step, if you have at least 15 life more than your
    // starting life total, each player this creature attacked this turn loses the game."
    // The team's starting life total is 30: 44 isn't enough, 45 is.
    for (life, triggers) in [(44, 0), (45, 1)] {
        let mut t = two_headed_giant();
        let angel = t.battlefield(P0, "Angel of Destiny");
        set_team_life(&mut t, P0, life);
        to_end_step(&mut t, P0);
        assert_eq!(triggers_from(&t, angel), triggers, "at {life} life");
    }
}

// ---------------------------------------------------------------------------------------
// "If you would gain life, you gain that much life plus 1 instead."
// ---------------------------------------------------------------------------------------

#[test]
fn angel_of_vitality_and_honor_troll_apply_once_to_each_life_gaining_event() {
    cr!("614.1a", "119.9", "616.1");
    ruling!(
        "Angel of Vitality",
        "If you control two Angels of Vitality and you would gain life, you gain that much life plus 2. A third Angel of Vitality has you gain that much life plus 3, and so on."
    );
    ruling!(
        "Honor Troll",
        "If you control two Honor Trolls and you would gain life, you gain that much life plus 2. A third Honor Troll has you gain that much life plus 3, and so on."
    );
    ruling!(
        "Angel of Vitality",
        "The middle ability of Angel of Vitality applies just once to each life-gaining event, whether it's 1 life from Dazzling Angel or 4 life from Apothecary Stomper."
    );
    ruling!(
        "Honor Troll",
        "The middle ability of Honor Troll applies just once to each life-gaining event, whether it's 1 life from Impassioned Orator or 4 life from Dawning Angel. If you gain an amount of life \"for each\" of something or \"equal to the number\" of something, that life is gained as one event and the ability of Honor Troll applies only once."
    );
    ruling!(
        "Angel of Vitality",
        "If you gain an amount of life \"for each\" of something or \"equal to the number\" of something, that life is gained as one event and the second ability of Angel of Vitality applies only once."
    );
    for name in ["Angel of Vitality", "Honor Troll"] {
        // One, two and three of them: Sacred Nectar ("You gain 4 life.") gains 4 + n; Soul
        // Warden's 1 life gains 1 + n.
        for n in 1..=3 {
            let mut t = TestGame::new(2);
            for _ in 0..n {
                t.battlefield(P0, name);
            }
            let from = t.g.turn_events.len();
            cast_and_resolve(&mut t, P0, "Sacred Nectar", &[]);
            t.battlefield(P0, "Soul Warden");
            crate::r_s05_common::enter(&mut t, P0, "Grizzly Bears");
            t.resolve_all();
            assert_eq!(gains_since(&t, from, P0), vec![4 + n, 1 + n], "{name} x{n}");
        }
        // Life gained "for each": Peach Garden Oath ("You gain 2 life for each creature
        // you control."), with the creature and three Bears: 8 + 1.
        let mut t = TestGame::new(2);
        t.battlefield(P0, name);
        for _ in 0..3 {
            t.battlefield(P0, "Grizzly Bears");
        }
        let from = t.g.turn_events.len();
        cast_and_resolve(&mut t, P0, "Peach Garden Oath", &[]);
        assert_eq!(gains_since(&t, from, P0), vec![9], "{name}");
        // "Equal to the number": Ondu Cleric ("Whenever this creature or another Ally you
        // control enters, you may gain life equal to the number of Allies you control.")
        // entering with two other Allies: 3 + 1.
        let mut t = TestGame::new(2);
        t.battlefield(P0, name);
        t.battlefield(P0, "Ondu Cleric");
        t.battlefield(P0, "Ondu Cleric");
        let from = t.g.turn_events.len();
        let cleric = crate::r_s05_common::enter(&mut t, P0, "Ondu Cleric");
        // (Only the entering one triggers; the others trigger for it too.)
        let _ = cleric;
        t.answer_yes(P0, true);
        t.answer_yes(P0, true);
        t.answer_yes(P0, true);
        t.resolve_all();
        assert_eq!(gains_since(&t, from, P0), vec![4, 4, 4], "{name}");
    }
}

#[test]
fn angel_of_vitality_and_honor_troll_need_25_life_whatever_the_starting_life_total() {
    cr!("119.1c", "613.4c");
    ruling!(
        "Angel of Vitality",
        "In formats where your life total doesn't start at 20, Angel of Vitality's last ability still applies if your life total is 25 or more."
    );
    ruling!(
        "Honor Troll",
        "In multiplayer formats where your life total doesn't start at 20, Honor Troll's last ability still applies if your life total is 25 or more."
    );
    // A four-player Commander game: everyone starts at 40.
    for (name, base, boosted) in [
        ("Angel of Vitality", (2, 2), (4, 4)),
        ("Honor Troll", (2, 3), (4, 4)),
    ] {
        let mut t = TestGame::with_config(4, GameConfig::commander_game());
        let c = t.battlefield(P0, name);
        assert_eq!(t.life(P0), 40);
        assert_eq!(t.pt(c), boosted, "{name} at 40");
        set_life(&mut t, P0, 24);
        t.g.recompute();
        assert_eq!(t.pt(c), base, "{name} at 24");
        set_life(&mut t, P0, 25);
        t.g.recompute();
        assert_eq!(t.pt(c), boosted, "{name} at 25");
    }
}

#[test]
fn angel_of_vitality_and_honor_troll_two_headed_giant() {
    cr!("810.9", "119.9", "614.1a", "613.4c");
    ruling!(
        "Angel of Vitality",
        "In a Two-Headed Giant game, Angel of Vitality's last ability applies as long as your team's life total is 25 or more."
    );
    ruling!(
        "Honor Troll",
        "In a Two-Headed Giant game, Honor Troll's last ability applies as long as your team's life total is 25 or more."
    );
    ruling!(
        "Angel of Vitality",
        "In a Two-Headed Giant game, life gained by your teammate won't cause Angel of Vitality's second ability to apply, even though it will cause your team's life total to increase."
    );
    ruling!(
        "Honor Troll",
        "In a Two-Headed Giant game, life gained by your teammate won't cause Honor Troll's second ability to apply, even though it will cause your team's life total to increase."
    );
    for (name, base, boosted) in [
        ("Angel of Vitality", (2, 2), (4, 4)),
        ("Honor Troll", (2, 3), (4, 4)),
    ] {
        let mut t = two_headed_giant();
        let c = t.battlefield(P0, name);
        set_team_life(&mut t, P0, 21);
        t.g.recompute();
        assert_eq!(t.pt(c), base);
        // P0's teammate gains 4 life: the team's life total goes up by exactly 4, to 25,
        // which is enough for the static ability.
        let from = t.g.turn_events.len();
        cast_and_resolve(&mut t, P1, "Sacred Nectar", &[]);
        assert_eq!(gains_since(&t, from, P1), vec![4], "{name}");
        assert_eq!((t.life(P0), t.life(P1)), (25, 25));
        assert_eq!(t.pt(c), boosted, "{name}");
        // P0 gains 4 life: 4 + 1.
        let from = t.g.turn_events.len();
        cast_and_resolve(&mut t, P0, "Sacred Nectar", &[]);
        assert_eq!(gains_since(&t, from, P0), vec![5], "{name}");
        assert_eq!(t.life(P1), 30);
    }
}

// ---------------------------------------------------------------------------------------
// "If you would gain life, you gain twice that much life instead."
// ---------------------------------------------------------------------------------------

#[test]
fn doubling_life_gain_from_a_life_total_becoming_a_number() {
    cr!("119.5", "614.1a");
    ruling!(
        "Boon Reflection",
        "If an effect would set your life total to a specific number that's higher than your current life total, that effect would cause you to gain life equal to the difference. Boon Reflection will then double the amount of life that effect would cause you to gain. For example, if you have 3 life and an effect says that your life total “becomes 10,” your life total will actually become 17."
    );
    ruling!(
        "Alhammarret's Archive",
        "If an effect would set your life total to a specific number that’s higher than your current life total, that effect would cause you to gain life equal to the difference. Alhammarret’s Archive will then double the amount of life that effect would cause you to gain. For example, if you have 3 life and an effect says that your life total “becomes 10,” your life total will actually become 17."
    );
    supported("Blessed Wind");
    // Blessed Wind: "Target player's life total becomes 20." At 3 life: 3 + 2 * 17 = 37.
    for name in ["Boon Reflection", "Alhammarret's Archive"] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, name);
        set_life(&mut t, P0, 3);
        let from = t.g.turn_events.len();
        cast_and_resolve(&mut t, P0, "Blessed Wind", &[Entity::Player(P0)]);
        assert_eq!(t.life(P0), 37, "{name}");
        assert_eq!(gains_since(&t, from, P0), vec![34], "{name}");
    }
}

#[test]
fn boon_reflections_are_cumulative() {
    cr!("614.1a", "616.1");
    ruling!(
        "Boon Reflection",
        "The effects of multiple Boon Reflections are cumulative. For example, if you control two Boon Reflections, you'll gain four times the original amount of life. If you control three Boon Reflections, you'll gain eight times the original amount, and so on."
    );
    // Sacred Nectar: "You gain 4 life."
    for (n, gained) in [(1, 8), (2, 16), (3, 32)] {
        let mut t = TestGame::new(2);
        for _ in 0..n {
            t.battlefield(P0, "Boon Reflection");
        }
        cast_and_resolve(&mut t, P0, "Sacred Nectar", &[]);
        assert_eq!(t.life(P0), 20 + gained, "{n} Boon Reflections");
    }
}

#[test]
fn life_doublers_dont_apply_to_life_gained_by_a_teammate() {
    cr!("810.9", "614.1a");
    ruling!(
        "Boon Reflection",
        "In a Two-Headed Giant game, only Boon Reflection's controller is affected by it. If that player's teammate gains life, Boon Reflection will have no effect, even when that life gain is applied to the shared team life total."
    );
    ruling!(
        "Alhammarret's Archive",
        "In a Two-Headed Giant game, only the controller of Alhammarret’s Archive is affected by it. If that player’s teammate gains life, Alhammarret’s Archive will have no effect, even when that life gain is applied to the team’s shared life total."
    );
    for name in ["Boon Reflection", "Alhammarret's Archive"] {
        let mut t = two_headed_giant();
        t.battlefield(P0, name);
        cast_and_resolve(&mut t, P1, "Sacred Nectar", &[]);
        assert_eq!((t.life(P0), t.life(P1)), (34, 34), "{name}");
        cast_and_resolve(&mut t, P0, "Sacred Nectar", &[]);
        assert_eq!((t.life(P0), t.life(P1)), (42, 42), "{name}");
    }
}

// ---------------------------------------------------------------------------------------
// Felidar Sovereign
// ---------------------------------------------------------------------------------------

#[test]
fn felidar_sovereign_two_headed_giant_checks_the_team_life_total() {
    cr!("810.9", "603.4", "104.3h");
    ruling!(
        "Felidar Sovereign",
        "In a Two-Headed Giant game, your life total is the same as your team's life total. As such, the ability triggers if your team has 40 or more life."
    );
    for (life, triggers) in [(39, 0), (40, 1)] {
        let mut t = two_headed_giant();
        let cat = t.battlefield(P0, "Felidar Sovereign");
        set_team_life(&mut t, P0, life);
        t.set_step(P0, Step::Untap);
        t.advance_to(P0, Step::Upkeep);
        t.settle();
        assert_eq!(triggers_from(&t, cat), triggers, "at {life}");
        assert_eq!(triggers_on_stack(&t, "win the game"), triggers);
        t.resolve_all();
        let won = t.g.players.iter().filter(|p| p.has_won).count();
        assert_eq!(won, 2 * triggers, "at {life}");
    }
}
