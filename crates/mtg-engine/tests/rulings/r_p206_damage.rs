//! Rulings batch P206 — doubling damage (CR 701.10g): a replacement effect applied after
//! damage is divided or assigned, the doubled damage is still dealt by the original
//! source, several doublers multiply, and the player (or controller of the permanent)
//! being dealt damage orders doubling and prevention (CR 616.1).

use crate::r_p205_common::delirium_graveyard;
use crate::r_p206_common::*;
use crate::r_s01_common::{attack_with, block_and_finish, supported};
use crate::r_s03_common::respond;
use crate::r_s06_common::{activate_containing, attach_new};
use crate::r_s17_common::enter_transformed;
use crate::r_s24_common::choose_creature_type;
use crate::r_s25_common::{cast_new, lands_for_cost};
use crate::r_s26_common::modify_until_eot;
use crate::r_s29_common::{put_counters, replacement_choosers};
use crate::r_s30_common::{damage_events, pick_replacement};
use mtg_engine::ability::{Modification, Value};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Damage dealt this turn by `source` to `to`.
fn dealt(t: &TestGame, source: ObjectId, to: Entity) -> u32 {
    damage_events(t)
        .iter()
        .filter(|(s, r, _, _)| *s == source && *r == to)
        .map(|(_, _, n, _)| *n)
        .sum()
}

/// All damage dealt this turn to `to`, with its sources.
fn sources_of_damage_to(t: &TestGame, to: Entity) -> Vec<ObjectId> {
    damage_events(t)
        .iter()
        .filter(|(_, r, n, _)| *r == to && *n > 0)
        .map(|(s, _, _, _)| *s)
        .collect()
}

/// Craw Giant (a 6/4 Giant with trample) under P0's control.
fn craw_giant(t: &mut TestGame) -> ObjectId {
    t.battlefield(P0, "Craw Giant")
}

/// P0 puts four quest counters on Quest for Pure Flame and activates its second ability.
fn pure_flame(t: &mut TestGame) {
    let q = t.battlefield(P0, "Quest for Pure Flame");
    put_counters(t, q, "quest", 4);
    activate_containing(t, P0, q, "Remove four").expect("Quest for Pure Flame");
    t.resolve_all();
}

/// A damage doubler: `setup` builds the board and returns the trampling attacker.
struct Case {
    card: &'static str,
    setup: fn(&mut TestGame) -> ObjectId,
}

fn trample_cases() -> Vec<Case> {
    vec![
        Case {
            card: "Furnace of Rath",
            setup: |t| {
                t.battlefield(P0, "Furnace of Rath");
                craw_giant(t)
            },
        },
        Case {
            card: "Calamity Bearer",
            setup: |t| {
                t.battlefield(P0, "Calamity Bearer");
                craw_giant(t)
            },
        },
        Case {
            card: "Gisela, Blade of Goldnight",
            setup: |t| {
                t.battlefield(P0, "Gisela, Blade of Goldnight");
                craw_giant(t)
            },
        },
        Case {
            card: "Inquisitor's Flail",
            setup: |t| {
                let g = craw_giant(t);
                attach_new(t, P0, "Inquisitor's Flail", g);
                g
            },
        },
        Case {
            card: "Collective Inferno",
            setup: |t| {
                choose_creature_type(t, P0, "Giant");
                t.enter(P0, "Collective Inferno");
                t.resolve_all();
                craw_giant(t)
            },
        },
        Case {
            card: "Sawhorn Nemesis",
            setup: |t| {
                t.answer_choose(P0, &[Entity::Player(P1)]);
                t.enter(P0, "Sawhorn Nemesis");
                t.resolve_all();
                craw_giant(t)
            },
        },
        Case {
            card: "Uncivil Unrest",
            setup: |t| {
                t.battlefield(P0, "Uncivil Unrest");
                let g = craw_giant(t);
                put_counters(t, g, counters::PLUS1, 1);
                g
            },
        },
        Case {
            card: "Raphael, the Muscle",
            setup: |t| {
                t.battlefield(P0, "Raphael, the Muscle");
                let g = craw_giant(t);
                put_counters(t, g, counters::PLUS1, 1);
                g
            },
        },
        Case {
            card: "Twinflame Tyrant",
            setup: |t| {
                t.battlefield(P0, "Twinflame Tyrant");
                craw_giant(t)
            },
        },
        Case {
            card: "Kuja, Genome Sorcerer // Trance Kuja, Fate Defied",
            setup: |t| {
                enter_transformed(t, P0, "Kuja, Genome Sorcerer // Trance Kuja, Fate Defied");
                // Prodigal Sorcerer, a 1/1 Wizard, made a 5/6 with trample.
                let w = t.battlefield(P0, "Prodigal Sorcerer");
                modify_until_eot(
                    t,
                    w,
                    vec![
                        Modification::ModifyPT(Value::c(4), Value::c(5)),
                        Modification::AddKeyword(Keyword::new(KeywordKind::Trample)),
                    ],
                );
                w
            },
        },
        Case {
            card: "Quest for Pure Flame",
            setup: |t| {
                pure_flame(t);
                craw_giant(t)
            },
        },
    ]
}

/// The trampling attacker attacks P1 and is blocked by Grizzly Bears; P0 assigns 2 to the
/// Bears and the rest to P1. Returns (the attacker's power, damage to the Bears, damage
/// to P1).
fn trample_into_bears(t: &mut TestGame, attacker: ObjectId) -> (i32, u32, u32) {
    let bears = t.battlefield(P1, "Grizzly Bears");
    let power = t.pt(attacker).0;
    attack_with(t, &[(attacker, Entity::Player(P1))]);
    t.answer(
        P0,
        DecisionKind::Damage,
        Answer::Numbers(vec![2, (power - 2) as i64]),
    );
    block_and_finish(t, P1, &[(bears, attacker)]);
    let to_bears = dealt(t, attacker, Entity::Object(bears));
    let to_p1 = dealt(t, attacker, Entity::Player(P1));
    // The doubled damage is all dealt by the attacker itself.
    let srcs = sources_of_damage_to(t, Entity::Player(P1));
    assert!(srcs.iter().all(|s| *s == attacker), "sources {srcs:?}");
    let srcs = sources_of_damage_to(t, Entity::Object(bears));
    assert!(srcs.iter().all(|s| *s == attacker), "sources {srcs:?}");
    (power, to_bears, to_p1)
}

#[test]
fn trample_damage_is_assigned_before_it_is_doubled() {
    cr!("701.10g", "702.19c", "510.1c", "614.1a");
    ruling!(
        "Calamity Bearer",
        "If damage dealt by a Giant source you control is being divided or assigned among multiple permanents and/or players, that damage is divided or assigned before doubling. For example, if you attack with a 5/5 Giant with trample and it’s blocked by a 2/2 creature, you can assign 2 damage to the blocker and 3 damage to the defending player. Those amounts are then doubled to 4 and 6, respectively."
    );
    ruling!(
        "Kuja, Genome Sorcerer // Trance Kuja, Fate Defied",
        "If damage dealt by a Wizard you control is being divided or assigned among multiple permanents and/or players, that damage is divided or assigned before doubling. For example, if you attack with a 4/6 Wizard with trample and it's blocked by a 1/1 creature, you can assign 1 damage to the blocker and 3 damage to the defending player. Those amounts are then doubled to 2 and 6, respectively."
    );
    ruling!(
        "Uncivil Unrest",
        "If damage dealt by a creature you control with a +1/+1 counter on it is being divided or assigned among multiple permanents and/or players, that damage is divided or assigned before doubling. For example, if you attack with a 5/5 creature with trample and it’s blocked by a 2/2 creature, you can assign 2 damage to the blocker and 3 damage to the defending player. Those amounts are then doubled to 4 and 6, respectively."
    );
    ruling!(
        "Gisela, Blade of Goldnight",
        "If damage dealt by a source you control is being divided or assigned among multiple permanents an opponent controls or among an opponent and one or more permanents they control simultaneously, divide the original amount and double the results. For example, if you attack with a 5/5 creature with trample and your opponent blocks with a 2/2 creature, you can assign 2 damage to the blocker and 3 damage to the defending player. These amounts are then doubled to 4 and 6 damage, respectively. You can't double the damage to 10 first and then assign 2 to the creature and 8 to the player."
    );
    ruling!(
        "Collective Inferno",
        "If damage dealt is being divided or assigned among multiple permanents or players, that damage is divided or assigned before any effects modify how much damage would be dealt. For example, say you control Collective Inferno and you attack an opponent with a 5/5 creature of the chosen type with trample and haste. If it's blocked by a 2/2 creature, you can assign 2 damage to the blocker and 3 damage to the defending player. Those amounts are then doubled to 4 and 6, respectively."
    );
    ruling!(
        "Sawhorn Nemesis",
        "If damage dealt is being divided or assigned among multiple permanents or players, that damage is divided or assigned before any effects that modify how much damage will be dealt. For example, if you attack the chosen player with a 5/5 creature with trample and it's blocked by a 2/2 creature they control, you can assign 2 damage to the blocker and 3 damage to the defending player. Those amounts are then doubled to 4 and 6, respectively."
    );
    ruling!(
        "Inquisitor's Flail",
        "If you divide the combat damage dealt by the equipped creature, perhaps because the creature has trample or is dealing combat damage to multiple creatures, you’ll divide the original amount and then double the results. For example, if a 5/5 creature with trample is blocked by a 2/2 creature, you can assign 2 damage to the blocker and 3 damage to the defending player. These amounts are then doubled to 4 and 6 damage, respectively. You can’t double the damage to 10 first and then assign 2 to the creature and 8 to the player."
    );
    ruling!(
        "Furnace of Rath",
        "The trample rules cause damage to be divided before it is doubled."
    );
    for c in trample_cases() {
        supported(c.card);
        let mut t = TestGame::new(2);
        let a = (c.setup)(&mut t);
        let (power, to_bears, to_p1) = trample_into_bears(&mut t, a);
        assert_eq!(to_bears, 4, "{}", c.card);
        assert_eq!(to_p1, 2 * (power as u32 - 2), "{}", c.card);
        assert_eq!(t.life(P1), 20 - to_p1 as i32, "{}", c.card);
    }
}

#[test]
fn the_doubled_damage_is_dealt_by_the_original_source() {
    cr!("701.10g", "120.3", "609.7");
    ruling!(
        "Collective Inferno",
        "The damage is dealt by the same source as the original source of damage. The doubled damage isn't dealt by Collective Inferno unless (in a very unusual case) it was the original source of damage."
    );
    ruling!(
        "Raphael, the Muscle",
        "The damage is dealt by the same source as the original source of damage. The doubled damage isn't dealt by Raphael unless he was the original source of damage."
    );
    ruling!(
        "Sawhorn Nemesis",
        "The damage is dealt by the same source as the original source of damage. The doubled damage isn't dealt by Sawhorn Nemesis unless it was the original source of damage."
    );
    ruling!(
        "Kuja, Genome Sorcerer // Trance Kuja, Fate Defied",
        "The damage is dealt by the same source as the original source of damage. The doubled damage isn't dealt by Trance Kuja unless it was the original source of damage."
    );
    ruling!(
        "Twinflame Tyrant",
        "The damage is dealt by the same source as the original source of damage. The doubled damage isn't dealt by Twinflame Tyrant unless it was the original source of damage."
    );
    ruling!(
        "Calamity Bearer",
        "The damage is dealt by the same source as the original source of damage. The doubled damage isn’t dealt by Calamity Bearer unless it was the original source of damage."
    );
    ruling!(
        "Uncivil Unrest",
        "The damage is dealt by the same source as the original source of damage. The doubled damage isn’t dealt by Uncivil Unrest unless it was somehow the original source of damage."
    );
    // (Checked for each of them by `trample_into_bears`.)
    for c in trample_cases() {
        let mut t = TestGame::new(2);
        let a = (c.setup)(&mut t);
        trample_into_bears(&mut t, a);
    }
}

#[test]
fn noncombat_doubled_damage_is_dealt_by_the_original_source() {
    cr!("701.10g", "120.3");
    ruling!(
        "Lightning, Army of One",
        "The damage is dealt by the same source as the original source of damage. The doubled damage isn't dealt by Lightning unless it was the original source of damage."
    );
    ruling!(
        "The Rollercrusher Ride",
        "The damage is dealt by the same source as the original source of damage. The doubled damage isn't dealt by The Rollercrusher Ride unless it was the original source of damage."
    );
    supported("Lightning, Army of One");
    supported("The Rollercrusher Ride");
    // Lightning: after it deals combat damage to P1, damage to P1 is doubled until P0's
    // next turn.
    let mut t = TestGame::new(2);
    let l = t.battlefield(P0, "Lightning, Army of One");
    attack_with(&mut t, &[(l, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    let shock = cast_new(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 13);
    assert_eq!(sources_of_damage_to(&t, Entity::Player(P1)), vec![l, shock]);
    // The Rollercrusher Ride, with delirium: noncombat damage doubled.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Rollercrusher Ride");
    delirium_graveyard(&mut t, P0);
    let shock = cast_new(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert_eq!(sources_of_damage_to(&t, Entity::Player(P1)), vec![shock]);
    // ...but not combat damage.
    let g = t.battlefield(P0, "Hill Giant");
    attack_with(&mut t, &[(g, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 13);
}

/// P0 casts Forked Bolt dividing 1 damage to P1 and 1 to P1's Hill Giant. Returns the
/// damage dealt to P1 and to the Giant.
fn forked_bolt_1_and_1(t: &mut TestGame) -> (u32, u32) {
    let giant = t.battlefield(P1, "Hill Giant");
    lands_for_cost(t, P0, "Forked Bolt");
    let card = t.hand(P0, "Forked Bolt");
    t.answer_targets(P0, &[Entity::Object(giant), Entity::Player(P1)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![1, 1]));
    let spell = t.cast_with(P0, card, &[]).unwrap();
    t.resolve_all();
    (
        dealt(t, spell, Entity::Player(P1)),
        dealt(t, spell, Entity::Object(giant)),
    )
}

#[test]
fn divided_damage_is_divided_before_it_is_doubled() {
    cr!("701.10g", "601.2d");
    ruling!(
        "Fire Servant",
        "If a red instant or sorcery spell you control divides damage among multiple recipients (such as Fireball does), the damage is divided before Fire Servant’s effect doubles it."
    );
    ruling!(
        "Quest for Pure Flame",
        "If a spell or ability divides damage among multiple recipients (such as Arrow Volley Trap does), the damage is divided before Quest for Pure Fire’s effect doubles it. The same is true for combat damage."
    );
    supported("Fire Servant");
    supported("Quest for Pure Flame");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fire Servant");
    assert_eq!(forked_bolt_1_and_1(&mut t), (2, 2));
    let mut t = TestGame::new(2);
    pure_flame(&mut t);
    assert_eq!(forked_bolt_1_and_1(&mut t), (2, 2));
}

#[test]
fn anthem_of_rakdos_doubles_damage_to_each_recipient() {
    cr!("701.10g");
    ruling!(
        "Anthem of Rakdos",
        "If a source you control would deal damage to multiple creatures and/or players simultaneously, all of that damage is doubled."
    );
    supported("Anthem of Rakdos");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Anthem of Rakdos");
    let mine = t.battlefield(P0, "Indomitable Ancients");
    let theirs = t.battlefield(P1, "Indomitable Ancients");
    // Hellbent: Pyroclasm is the only card in P0's hand.
    assert_eq!(t.hand_size(P0), 0);
    let spell = cast_new(&mut t, P0, "Pyroclasm", &[]);
    t.resolve_all();
    assert_eq!(dealt(&t, spell, Entity::Object(mine)), 4);
    assert_eq!(dealt(&t, spell, Entity::Object(theirs)), 4);
}

#[test]
fn anthem_of_rakdos_doubles_its_own_damage_to_you() {
    cr!("701.10g");
    ruling!(
        "Anthem of Rakdos",
        "The hellbent ability doubles the damage dealt to you by Anthem of Rakdos itself."
    );
    let mut t = TestGame::new(2);
    let anthem = t.battlefield(P0, "Anthem of Rakdos");
    let g = t.battlefield(P0, "Hill Giant");
    assert_eq!(t.hand_size(P0), 0);
    attack_with(&mut t, &[(g, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(dealt(&t, anthem, Entity::Player(P0)), 2);
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.pt(g), (5, 3));
}

#[test]
fn two_dictates_quadruple_damage() {
    cr!("701.10g", "616.1");
    ruling!(
        "Dictate of the Twin Gods",
        "If more than one Dictate of the Twin Gods is on the battlefield, damage dealt will double for each one (two of them will end up multiplying the damage by four, three of them by eight, and four of them by sixteen)."
    );
    supported("Dictate of the Twin Gods");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dictate of the Twin Gods");
    t.battlefield(P1, "Dictate of the Twin Gods");
    cast_new(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 12);
}

#[test]
fn goldnight_castigators_multiply_damage_to_you_but_not_to_each_other() {
    cr!("701.10g", "616.1");
    ruling!(
        "Goldnight Castigator",
        "If you control two Goldnight Castigators and a source would deal damage to you, it deals four times that much damage instead. If you control a third Goldnight Castigator, the source deals eight times that much damage to you, and so on. Damage dealt to a Goldnight Castigator is only doubled, regardless of how many are on the battlefield."
    );
    supported("Goldnight Castigator");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Goldnight Castigator");
    t.battlefield(P0, "Goldnight Castigator");
    t.lands(P1, "Mountain", 1);
    let shock = t.hand(P1, "Shock");
    t.cast_with(P1, shock, &[Entity::Player(P0)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 12);
    let shock = cast_new(&mut t, P1, "Shock", &[Entity::Object(a)]);
    t.resolve_all();
    assert_eq!(dealt(&t, shock, Entity::Object(a)), 4);
}

/// P1 casts Mending Hands on themselves ("Prevent the next 4 damage that would be dealt
/// to any target this turn"), then P0 casts Lava Axe at P1 (5 damage). P1 applies the
/// prevention first or not. Returns the damage P1 took and who ordered the effects.
fn mending_hands_then_lava_axe(
    t: &mut TestGame,
    prevent_first: bool,
    doubler: fn(&mtg_engine::game::Game, &Decision) -> Option<Answer>,
) -> (i32, Vec<PlayerId>) {
    cast_new(t, P1, "Mending Hands", &[Entity::Player(P1)]);
    t.resolve_all();
    if prevent_first {
        respond(t, P1, |g, d| pick_replacement(g, d, "Mending Hands"));
    } else {
        respond(t, P1, doubler);
    }
    let from = t.asked().len();
    cast_new(t, P0, "Lava Axe", &[Entity::Player(P1)]);
    t.resolve_all();
    (20 - t.life(P1), replacement_choosers(t, from))
}

fn not_mending(g: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
    match d {
        Decision::ChooseReplacement { options } => options
            .iter()
            .position(|o| !o.contains("Mending Hands"))
            .map(Answer::Index),
        _ => {
            let _ = g;
            None
        }
    }
}

#[test]
fn the_player_dealt_damage_orders_doubling_and_prevention() {
    cr!("616.1", "615.1a", "701.10g");
    ruling!(
        "Anthem of Rakdos",
        "If damage that would be dealt to a player or creature is affected by both Anthem of Rakdos and a damage prevention effect, that player or that creature’s controller chooses the order to apply the effects. In most cases that player will want to prevent damage, then double what’s left."
    );
    ruling!(
        "Curse of Bloodletting",
        "If multiple effects modify how damage will be dealt to the enchanted player, that player chooses the order to apply the effects. For example, Mending Hands says, “Prevent the next 4 damage that would be dealt to any target this turn.” Suppose a spell would deal 5 damage to enchanted player and that player has cast Mending Hands targeting themselves. The enchanted player can either (a) prevent 4 damage first and then let Curse of Bloodletting’s effect double the remaining 1 damage, taking 2 damage, or (b) double the damage to 10 and then prevent 4 damage, taking 6 damage."
    );
    ruling!(
        "Furnace of Rath",
        "If multiple effects modify how damage will be dealt, the player who would be dealt damage or the controller of the creature that would be dealt damage chooses the order to apply the effects. For example, Mending Hands says, “Prevent the next 4 damage that would be dealt to any target this turn.” Suppose a spell would deal 5 damage to a player who has cast Mending Hands targeting themselves. That player can either (a) prevent 4 damage first and then let Furnace of Rath double the remaining 1 damage, taking 2 damage, or (b) double the damage to 10 and then prevent 4 damage, taking 6 damage."
    );
    ruling!(
        "Quest for Pure Flame",
        "If multiple effects modify how damage will be dealt, the player who would be dealt damage or the controller of the creature that would be dealt damage chooses the order to apply the effects. For example, Mending Hands says, “Prevent the next 4 damage that would be dealt to any target.” Suppose a spell controlled by a player who has activated Quest for Pure Fire’s second ability would deal 5 damage to a player who has cast Mending Hands targeting themselves. The player who would be dealt damage can either (a) prevent 4 damage first and then let Quest for Pure Fire’s effect double the remaining 1 damage, taking 2 damage, or (b) double the damage to 10 and then prevent 4 damage, taking 6 damage."
    );
    ruling!(
        "Fire Servant",
        "If multiple effects modify how damage will be dealt, the player who would be dealt damage or the controller of the permanent that would be dealt damage chooses the order to apply the effects. For example, Mending Hands says, “Prevent the next 4 damage that would be dealt to any target this turn” and Lava Axe is a red sorcery that says “Lava Axe deals 5 damage to target player or planeswalker.” Suppose a Lava Axe controlled by a player who controls Fire Servant would deal 5 damage to a player who has cast Mending Hands targeting themselves. The player who would be dealt damage can either (a) prevent 4 damage first and then let Fire Servant’s effect double the remaining 1 damage, taking 2 damage, or (b) double the damage to 10 and then prevent 4 damage, taking 6 damage."
    );
    let setups: Vec<(&str, fn(&mut TestGame))> = vec![
        ("Furnace of Rath", |t| {
            t.battlefield(P0, "Furnace of Rath");
        }),
        ("Curse of Bloodletting", |t| {
            attach_new(t, P0, "Curse of Bloodletting", Entity::Player(P1));
        }),
        ("Fire Servant", |t| {
            t.battlefield(P0, "Fire Servant");
        }),
        ("Quest for Pure Flame", pure_flame),
        ("Anthem of Rakdos", |t| {
            t.battlefield(P0, "Anthem of Rakdos");
        }),
    ];
    for (card, setup) in setups {
        supported(card);
        for prevent_first in [true, false] {
            let mut t = TestGame::new(2);
            setup(&mut t);
            let (taken, asked) = mending_hands_then_lava_axe(&mut t, prevent_first, not_mending);
            assert_eq!(asked, vec![P1], "{card}");
            assert_eq!(taken, if prevent_first { 2 } else { 6 }, "{card}");
        }
    }
}

#[test]
fn decorated_griffin_and_dictate_of_the_twin_gods_order() {
    cr!("616.1", "615.1a", "701.10g");
    ruling!(
        "Dictate of the Twin Gods",
        "If multiple effects modify how damage will be dealt, the player being dealt damage or the controller of the permanent being dealt damage chooses the order to apply the effects. For example, the ability of Decorated Griffin says “Prevent the next 1 combat damage that would be dealt to you this turn.” Suppose you would be dealt 3 combat damage and you activate the ability of Decorated Griffin. You can either (a) prevent 1 damage first and then let Dictate of the Twin Gods's effect double the remaining 2 damage, for a result of being dealt 4 damage, or (b) double the damage to 6 and then prevent 1 damage, for a result of being dealt 5 damage."
    );
    supported("Decorated Griffin");
    for prevent_first in [true, false] {
        let mut t = TestGame::new(2);
        t.battlefield(P1, "Dictate of the Twin Gods");
        let griffin = t.battlefield(P0, "Decorated Griffin");
        let giant = t.battlefield(P1, "Hill Giant");
        t.set_step(P1, Step::BeginningOfCombat);
        attack_with(&mut t, &[(giant, Entity::Player(P0))]);
        rainbow_pool(&mut t, P0, 2);
        activate_containing(&mut t, P0, griffin, "Prevent").unwrap();
        t.resolve_all();
        if prevent_first {
            respond(&mut t, P0, |g, d| {
                pick_replacement(g, d, "Decorated Griffin")
            });
        } else {
            respond(&mut t, P0, |g, d| pick_replacement(g, d, "Dictate"));
        }
        block_and_finish(&mut t, P0, &[]);
        assert_eq!(t.life(P0), if prevent_first { 16 } else { 15 });
    }
}
