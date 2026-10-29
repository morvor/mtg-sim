//! Rulings batch P010 — burn whose targets become illegal (CR 608.2b): a spell whose only
//! target is illegal doesn't resolve at all (none of its other effects happen, including
//! damage to "that creature's controller"); with several targets it does what it can, and
//! damage divided among targets keeps its division. Also the targets these spells
//! require as they're cast (CR 601.2c).

use crate::r_p009_common::*;
use crate::r_s01_common::{supported, tokens};
use crate::r_s25_common::{cast_new, lands_for_cost};
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn cast_x(t: &mut TestGame, p: PlayerId, name: &str, x: i64, targets: &[Entity]) -> ObjectId {
    lands_for_cost(t, p, name);
    t.lands(p, "Wastes", x as usize);
    let card = t.hand(p, name);
    t.cast(p, card).x(x).targets(targets).go()
}

fn divide(t: &mut TestGame, p: PlayerId, amounts: &[i64]) {
    t.answer(p, DecisionKind::Divide, Answer::Numbers(amounts.to_vec()));
}

/// One spell whose single target creature is destroyed in response.
struct Fizzle {
    card: &'static str,
    /// Sets up what makes the spell's other effects apply (returns nothing).
    setup: fn(&mut TestGame),
    /// The creature to target (P1's).
    creature: &'static str,
    x: Option<i64>,
}

#[test]
fn single_target_spells_with_an_illegal_target_do_nothing() {
    cr!("608.2b");
    ruling!(
        "Draconic Roar",
        "Draconic Roar targets only the creature, not its controller. If that creature becomes an illegal target before Draconic Roar resolves, the spell doesn’t resolve and none of its effects will occur. No damage is dealt."
    );
    ruling!(
        "Ravaging Blaze",
        "If the creature becomes an illegal target by the time Ravaging Blaze tries to resolve, Ravaging Blaze won’t resolve and none of its effects will happen. No damage will be dealt to the creature or its controller."
    );
    ruling!(
        "Burn the Impure",
        "If the creature is an illegal target when Burn the Impure resolves, the spell won’t resolve and none of its effects will happen, even if that creature has infect."
    );
    ruling!(
        "Sonic Assault",
        "If the target creature becomes an illegal target for Sonic Assault, the spell doesn't resolve. No player is dealt damage."
    );
    ruling!(
        "Searing Barrage",
        "If the target creature is an illegal target by the time Searing Barrage tries to resolve, the spell won't resolve. No player will be dealt damage, even if you spent three red mana."
    );
    ruling!(
        "Synchronized Spellcraft",
        "If the target creature is an illegal target by the time Synchronized Spellcraft tries to resolve, the spell doesn’t resolve. No player is dealt damage."
    );
    ruling!(
        "Blur of Blades",
        "If the targeted creature is an illegal target by the time Blur of Blades resolves, it won’t resolve and none of its effects will happen. Blur of Blades won’t deal damage to any player."
    );
    let cases = [
        Fizzle {
            card: "Draconic Roar",
            setup: |t| {
                t.battlefield(P0, "Shivan Dragon");
            },
            creature: "Craw Wurm",
            x: None,
        },
        Fizzle {
            card: "Ravaging Blaze",
            setup: |t| {
                t.graveyard(P0, "Divination");
                t.graveyard(P0, "Lightning Bolt");
            },
            creature: "Craw Wurm",
            x: Some(2),
        },
        Fizzle {
            card: "Burn the Impure",
            setup: |_| {},
            creature: "Glistener Elf",
            x: None,
        },
        Fizzle {
            card: "Sonic Assault",
            setup: |_| {},
            creature: "Craw Wurm",
            x: None,
        },
        Fizzle {
            card: "Searing Barrage",
            // Three red mana: Mountains for the generic part too.
            setup: |t| {
                t.lands(P0, "Mountain", 4);
            },
            creature: "Craw Wurm",
            x: None,
        },
        Fizzle {
            card: "Synchronized Spellcraft",
            setup: |t| {
                t.battlefield(P0, "Prodigal Sorcerer");
            },
            creature: "Craw Wurm",
            x: None,
        },
        Fizzle {
            card: "Blur of Blades",
            setup: |_| {},
            creature: "Craw Wurm",
            x: None,
        },
    ];
    for c in &cases {
        supported(c.card);
        for respond in [false, true] {
            let mut t = TestGame::new(2);
            (c.setup)(&mut t);
            let target = t.battlefield(P1, c.creature);
            match c.x {
                Some(x) => {
                    cast_x(&mut t, P0, c.card, x, &[obj(target)]);
                }
                None => {
                    cast_new(&mut t, P0, c.card, &[obj(target)]);
                }
            }
            if respond {
                kill(&mut t, target);
            }
            t.resolve_all();
            let life = t.life(P1) + t.g.player(P1).poison() as i32;
            if respond {
                assert_eq!(life, 20, "{}: the controller was dealt damage", c.card);
            } else {
                assert!(life < 20, "{}: the controller wasn't dealt damage", c.card);
            }
        }
    }
}

#[test]
fn illegal_target_means_no_search_and_no_energy() {
    cr!("608.2b");
    ruling!(
        "Liberating Combustion",
        "If the target creature becomes an illegal target for Liberating Combustion, the spell won't resolve and none of its effects will happen. You won't search for Chandra, Pyrogenius."
    );
    ruling!(
        "Harnessed Lightning",
        "If the target creature becomes an illegal target, Harnessed Lightning doesn't resolve and none of its effects happen. You won't get {E}{E}{E} or be able to pay any {E}."
    );
    supported("Liberating Combustion");
    supported("Harnessed Lightning");
    for respond in [false, true] {
        let mut t = TestGame::new(2);
        let wurm = t.battlefield(P1, "Craw Wurm");
        t.library_top(P0, "Chandra, Pyrogenius");
        t.answer_yes(P0, true);
        cast_new(&mut t, P0, "Liberating Combustion", &[obj(wurm)]);
        if respond {
            kill(&mut t, wurm);
        }
        t.resolve_all();
        assert_eq!(t.in_hand(P0, "Chandra, Pyrogenius"), !respond);

        let mut t = TestGame::new(2);
        let wurm = t.battlefield(P1, "Craw Wurm");
        t.answer(P0, DecisionKind::Number, Answer::Number(1));
        cast_new(&mut t, P0, "Harnessed Lightning", &[obj(wurm)]);
        if respond {
            kill(&mut t, wurm);
        }
        t.resolve_all();
        let energy = t.g.player(P0).counter(counters::ENERGY);
        if respond {
            assert_eq!(energy, 0);
        } else {
            assert_eq!(energy + dmg(&t, wurm), 3);
        }
    }
}

#[test]
fn chandras_flame_wave_targets_the_player_not_the_creatures() {
    cr!("608.2b", "702.11b");
    ruling!(
        "Chandra's Flame Wave",
        "Chandra’s Flame Wave targets the player, not their creatures. A creature with hexproof may be dealt damage this way."
    );
    ruling!(
        "Chandra's Flame Wave",
        "If the target player is an illegal target by the time Chandra’s Flame Wave resolves, the spell doesn’t resolve. No creatures or players are dealt damage, and you won’t search for Chandra, Flame’s Fury."
    );
    supported("Chandra's Flame Wave");
    for respond in [false, true] {
        let mut t = TestGame::new(2);
        // A creature with hexproof.
        let bogle = t.battlefield(P1, "Slippery Bogle");
        t.library_top(P0, "Chandra, Flame's Fury");
        cast_new(&mut t, P0, "Chandra's Flame Wave", &[pl(P1)]);
        if respond {
            // P1 gains hexproof in response.
            add(&mut t, P1, "Leyline of Sanctity");
        }
        t.resolve_all();
        if respond {
            assert_eq!(t.life(P1), 20);
            assert_eq!(dmg(&t, bogle), 0);
            assert!(!t.in_hand(P0, "Chandra, Flame's Fury"));
        } else {
            assert_eq!(t.life(P1), 18);
            assert!(!t.on_battlefield(bogle), "the hexproof creature is dealt damage");
            assert!(t.in_hand(P0, "Chandra, Flame's Fury"));
        }
    }
}

#[test]
fn gravitic_punch_needs_both_targets() {
    cr!("608.2b");
    ruling!(
        "Gravitic Punch",
        "If the target creature or the target player is an illegal target as Gravitic Punch tries to resolve, the creature won't deal damage."
    );
    supported("Gravitic Punch");
    for illegal in ["none", "creature", "player"] {
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P0, "Hill Giant");
        cast_new(&mut t, P0, "Gravitic Punch", &[obj(giant), pl(P1)]);
        match illegal {
            "creature" => {
                // Another creature with the same power is no substitute.
                t.battlefield(P0, "Hill Giant");
                kill(&mut t, giant);
            }
            "player" => {
                add(&mut t, P1, "Leyline of Sanctity");
            }
            _ => {}
        }
        t.resolve_all();
        assert_eq!(t.life(P1), if illegal == "none" { 17 } else { 20 }, "{illegal}");
    }
}

#[test]
fn souls_fire_deals_no_damage_if_either_target_is_illegal() {
    cr!("608.2b");
    ruling!(
        "Soul's Fire",
        "If either target is an illegal target as Soul's Fire resolves, no damage is dealt."
    );
    supported("Soul's Fire");
    for illegal in ["none", "source", "recipient"] {
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P0, "Hill Giant");
        let wurm = t.battlefield(P1, "Craw Wurm");
        let to = if illegal == "recipient" { pl(P1) } else { obj(wurm) };
        cast_new(&mut t, P0, "Soul's Fire", &[obj(giant), to]);
        match illegal {
            // No longer a creature P0 controls.
            "source" => crate::r_s06_common::give_control(&mut t, giant, P1),
            "recipient" => {
                add(&mut t, P1, "Leyline of Sanctity");
            }
            _ => {}
        }
        t.resolve_all();
        let expected = if illegal == "none" { 3 } else { 0 };
        assert_eq!(dmg(&t, wurm), expected, "{illegal}");
        assert_eq!(t.life(P1), 20, "{illegal}");
    }
}

#[test]
fn reckless_rage_still_damages_the_legal_target() {
    cr!("608.2b", "601.2c");
    ruling!(
        "Reckless Rage",
        "If either target is an illegal target as Reckless Rage resolves, the other will still be dealt damage."
    );
    ruling!(
        "Reckless Rage",
        "You can’t cast Reckless Rage unless you choose both a creature you control and a creature you don’t control as targets."
    );
    supported("Reckless Rage");
    for illegal in ["none", "theirs", "mine"] {
        let mut t = TestGame::new(2);
        let wurm = t.battlefield(P1, "Colossal Dreadmaw");
        let mine = t.battlefield(P0, "Craw Wurm");
        cast_new(&mut t, P0, "Reckless Rage", &[obj(wurm), obj(mine)]);
        match illegal {
            "theirs" => kill(&mut t, wurm),
            "mine" => kill(&mut t, mine),
            _ => {}
        }
        t.resolve_all();
        if illegal != "theirs" {
            assert_eq!(dmg(&t, wurm), 4, "{illegal}");
        }
        if illegal != "mine" {
            assert_eq!(dmg(&t, mine), 2, "{illegal}");
        }
    }
    // Without a creature of its own to target, P0 can't cast it.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Mountain", 1);
    let rage = t.hand(P0, "Reckless Rage");
    let r = t.cast(P0, rage).target(obj(wurm)).try_go();
    assert!(r.is_err());
    assert!(t.in_hand(P0, "Reckless Rage"));
    assert_eq!(dmg(&t, wurm), 0);
}

#[test]
fn flameshot_needs_at_least_one_target() {
    cr!("601.2c");
    ruling!(
        "Flameshot",
        "You can’t choose zero targets. You must choose at least one target."
    );
    supported("Flameshot");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    let shot = t.hand(P0, "Flameshot");
    assert!(t.cast(P0, shot).targets(&[]).try_go().is_err());
    assert!(t.in_hand(P0, "Flameshot"));
    // With a creature to target, an empty answer still gets a target.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Mountain", 4);
    let shot = t.hand(P0, "Flameshot");
    if let Ok(s) = t.cast(P0, shot).targets(&[]).try_go() {
        assert_eq!(crate::r_s25_common::targets_of(&t, s), vec![obj(wurm)]);
    }
}

#[test]
fn divided_damage_to_an_illegal_target_is_lost() {
    cr!("608.2b", "601.2d");
    ruling!(
        "Ghired's Belligerence",
        "If a target creature becomes an illegal target in response to Ghired's Belligerence, the damage that would have been dealt to that creature is lost. It can't be dealt instead to another legal target."
    );
    ruling!(
        "Fight with Fire",
        "If some of the targets are illegal targets as Fight with Fire tries to resolve, the original division of damage still applies and the damage that would have been dealt to the illegal targets is lost."
    );
    supported("Ghired's Belligerence");
    supported("Fight with Fire");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Craw Wurm");
    let b = t.battlefield(P1, "Craw Wurm");
    divide(&mut t, P0, &[3, 1]);
    cast_x(&mut t, P0, "Ghired's Belligerence", 4, &[obj(a), obj(b)]);
    kill(&mut t, a);
    t.resolve_all();
    assert_eq!(dmg(&t, b), 1);

    // Kicked Fight with Fire: 6 to P1, 4 to the Wurm that leaves.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Wastes", 7);
    let fwf = t.hand(P0, "Fight with Fire");
    divide(&mut t, P0, &[4, 6]);
    t.cast(P0, fwf)
        .kicked(true)
        .targets(&[obj(a), pl(P1)])
        .go();
    kill(&mut t, a);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
}

#[test]
fn ghireds_belligerence_populates_when_a_damaged_creature_dies_any_way() {
    cr!("603.7c");
    ruling!(
        "Ghired's Belligerence",
        "The delayed triggered ability will trigger if a creature dealt damage by Ghired's Belligerence dies for any reason this turn, not just due to lethal damage."
    );
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    // A creature token for populate.
    cast_new(&mut t, P0, "Raise the Alarm", &[]);
    t.resolve_all();
    let before = tokens(&t, P0).len();
    cast_x(&mut t, P0, "Ghired's Belligerence", 1, &[obj(wurm)]);
    t.resolve_all();
    assert!(t.on_battlefield(wurm));
    // Destroyed later in the turn, not by the damage.
    kill(&mut t, wurm);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), before + 1);
}

#[test]
fn blood_token_spells_with_illegal_targets_make_no_blood() {
    cr!("608.2b");
    ruling!(
        "Grisly Ritual",
        "if all of those targets become illegal targets, the spell won't resolve and you won't create any Blood tokens"
    );
    ruling!(
        "Bloody Betrayal",
        "if all of those targets become illegal targets, the spell won't resolve and you won't create any Blood tokens"
    );
    ruling!(
        "Lacerate Flesh",
        "if all of those targets become illegal targets, the spell won't resolve and you won't create any Blood tokens"
    );
    ruling!(
        "Vampire's Kiss",
        "if all of those targets become illegal targets, the spell won't resolve and you won't create any Blood tokens"
    );
    ruling!(
        "Syphon Essence",
        "if all of those targets become illegal targets, the spell won't resolve and you won't create any Blood tokens"
    );
    for name in [
        "Grisly Ritual",
        "Bloody Betrayal",
        "Lacerate Flesh",
        "Vampire's Kiss",
    ] {
        supported(name);
        for respond in [false, true] {
            let mut t = TestGame::new(2);
            // Lacerate Flesh makes Blood for excess damage: a 1/1.
            let elf = t.battlefield(P1, "Llanowar Elves");
            let target = if name == "Vampire's Kiss" {
                pl(P1)
            } else {
                obj(elf)
            };
            cast_new(&mut t, P0, name, &[target]);
            if respond {
                if name == "Vampire's Kiss" {
                    add(&mut t, P1, "Leyline of Sanctity");
                } else {
                    kill(&mut t, elf);
                }
            }
            t.resolve_all();
            let blood = tokens(&t, P0)
                .into_iter()
                .filter(|b| t.obj_now(*b).chars.has_subtype("Blood"))
                .count();
            assert_eq!(blood > 0, !respond, "{name}");
        }
    }
    // Syphon Essence: the creature spell is countered by something else first.
    supported("Syphon Essence");
    for respond in [false, true] {
        let mut t = TestGame::new(2);
        let bears = cast_new(&mut t, P0, "Grizzly Bears", &[]);
        cast_new(&mut t, P1, "Syphon Essence", &[obj(bears)]);
        if respond {
            cast_new(&mut t, P1, "Counterspell", &[obj(bears)]);
        }
        t.resolve_all();
        assert_eq!(tokens(&t, P1).len(), if respond { 0 } else { 1 });
        assert!(t.in_graveyard(P0, "Grizzly Bears"));
    }
}

#[test]
fn reclusive_artificer_may_have_to_target_your_own_creature() {
    cr!("603.3d", "603.5");
    ruling!(
        "Reclusive Artificer",
        "If you are the only player who controls a creature when Reclusive Artificer’s second ability triggers, you must choose one of those creatures as the target, although you can choose to not deal it damage."
    );
    supported("Reclusive Artificer");
    for deal in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Mind Stone");
        t.answer_yes(P0, deal);
        let artificer = t.enter(P0, "Reclusive Artificer");
        t.settle();
        assert_eq!(t.stack_len(), 1, "it triggers, targeting the Artificer");
        let top = t.stack[0];
        assert_eq!(
            crate::r_s25_common::targets_of(&t, top),
            vec![obj(artificer)]
        );
        t.resolve_all();
        assert_eq!(dmg(&t, artificer), if deal { 1 } else { 0 });
    }
}
