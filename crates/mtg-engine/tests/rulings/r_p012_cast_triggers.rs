//! Rulings batch P012 — abilities that trigger when a player casts a spell: they go on the
//! stack above the spell and resolve first, even if the spell is countered (CR 601.2i,
//! 603.3); "a player" includes you; a trigger isn't a cost (CR 601.2f-h); and what
//! "cast" does and doesn't cover.

use crate::r_p012_common::*;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s04_common::{add_mana, stack_items};
use crate::r_s08_common::legal_cast_methods;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Ral's −8 has resolved: P0 has the emblem.
fn ral_emblem(t: &mut TestGame) {
    supported("Ral, Izzet Viceroy");
    let ral = t.battlefield(P0, "Ral, Izzet Viceroy");
    t.g.objects[ral.0 as usize]
        .counters
        .insert("loyalty".into(), 8);
    t.activate(P0, ral, 2, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.g.command.len(), 1, "the emblem");
}

/// Puts Curse of Wizardry onto the battlefield for P0 choosing blue.
fn curse_of_wizardry_blue(t: &mut TestGame) -> ObjectId {
    supported("Curse of Wizardry");
    let i = Color::ALL.iter().position(|c| *c == Color::Blue).unwrap();
    t.answer(P0, DecisionKind::Option, Answer::Index(i));
    let c = t.enter(P0, "Curse of Wizardry");
    t.settle();
    c
}

#[test]
fn cast_triggers_resolve_before_the_spell_even_if_it_is_countered() {
    cr!("601.2i", "603.3", "701.6a");
    ruling!(
        "Ral, Izzet Viceroy",
        "The triggered ability of Ral's emblem resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    ruling!(
        "Ash Zealot",
        "Ash Zealot’s triggered ability will resolve before that spell resolves."
    );
    ruling!(
        "Ruric Thar, the Unbowed",
        "Ruric Thar's triggered ability resolves before the spell that caused it to trigger. The ability will resolve even if that spell is countered."
    );
    ruling!(
        "Sentinel Tower",
        "Sentinel Tower’s ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    ruling!(
        "Tokka & Rahzar, Terrible Twos",
        "Tokka & Rahzar's last ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    ruling!(
        "Tokka & Rahzar, Terrible Twos",
        "If the spell is no longer on the stack when Tokka & Rahzar's last ability resolves, use its mana value as it last existed on the stack."
    );
    ruling!(
        "Ammit Eternal",
        "Ammit Eternal's middle ability will resolve before the spell that caused it to trigger. The ability will resolve even if that spell is countered."
    );
    ruling!(
        "Curse of Wizardry",
        "If a player casts a spell of the chosen color, Curse of Wizardry’s second ability triggers and goes on the stack on top of it. The ability will resolve before the spell does."
    );
    /// A case: the permanent's setup (returns objects to check), the caster, how the spell
    /// is cast (returns it, its triggers on the stack), and the check of the trigger's
    /// effect alone.
    struct Case {
        what: &'static str,
        setup: fn(&mut TestGame) -> Vec<ObjectId>,
        cast: fn(&mut TestGame) -> ObjectId,
        check: fn(&TestGame, &[ObjectId]),
    }
    let cases = [
        Case {
            what: "Ral's emblem",
            setup: |t| {
                ral_emblem(t);
                vec![]
            },
            cast: |t| {
                // The emblem's target: P1.
                let s = cast_card(t, P0, "Divination", &[]);
                s
            },
            check: |t, _| {
                assert_eq!(t.life(P1), 16);
                assert_eq!(t.hand_size(P0), 2);
            },
        },
        Case {
            what: "Ash Zealot",
            setup: |t| vec![t.battlefield(P0, "Ash Zealot")],
            cast: |t| {
                // P1 flashes back Think Twice from their graveyard.
                t.lands(P1, "Island", 3);
                let tt = t.graveyard(P1, "Think Twice");
                let s = t
                    .cast(P1, tt)
                    .method(CastMethod::Keyword(KeywordKind::Flashback))
                    .go();
                t.settle();
                s
            },
            check: |t, _| assert_eq!(t.life(P1), 17),
        },
        Case {
            what: "Ruric Thar",
            setup: |t| vec![t.battlefield(P0, "Ruric Thar, the Unbowed")],
            cast: |t| cast_card(t, P1, "Divination", &[]),
            check: |t, _| assert_eq!(t.life(P1), 14),
        },
        Case {
            what: "Sentinel Tower",
            setup: |t| vec![t.battlefield(P0, "Sentinel Tower")],
            cast: |t| cast_card(t, P0, "Divination", &[]),
            check: |t, _| assert_eq!(t.life(P1), 19),
        },
        Case {
            // Helm of Awakening makes Divination (mana value 3) cost {1}{U}.
            what: "Tokka & Rahzar",
            setup: |t| {
                t.battlefield(P1, "Helm of Awakening");
                vec![t.battlefield(P0, "Tokka & Rahzar, Terrible Twos")]
            },
            cast: |t| cast_card(t, P1, "Divination", &[]),
            check: |t, _| assert_eq!(t.life(P1), 17),
        },
        Case {
            what: "Ammit Eternal",
            setup: |t| vec![t.battlefield(P0, "Ammit Eternal")],
            cast: |t| cast_card(t, P1, "Divination", &[]),
            check: |t, o| assert_eq!(t.counters(o[0], "-1/-1"), 1),
        },
        Case {
            what: "Curse of Wizardry",
            setup: |t| vec![curse_of_wizardry_blue(t)],
            cast: |t| cast_card(t, P1, "Divination", &[]),
            check: |t, _| assert_eq!(t.life(P1), 19),
        },
    ];
    for c in cases {
        for countered in [false, true] {
            let mut t = TestGame::new(2);
            let objs = (c.setup)(&mut t);
            t.answer_targets(P0, &[pl(P1)]);
            let spell = (c.cast)(&mut t);
            let caster = t.obj(spell).controller;
            assert!(trigger_on_top(&t), "{}: the trigger is above the spell", c.what);
            if countered {
                assert!(t.g.counter(spell, None));
            }
            t.resolve();
            (c.check)(&t, &objs);
            if !countered {
                // The spell hasn't resolved yet.
                assert_eq!(t.zone(spell), Zone::Stack, "{}", c.what);
                let lib = t.library_size(caster);
                t.resolve_all();
                assert!(t.library_size(caster) < lib, "{}: the spell resolved", c.what);
            }
        }
    }
}

/// P0 controls the permanent and casts Divination themself.
#[test]
fn a_player_casts_a_spell_includes_you() {
    cr!("603.2");
    ruling!(
        "Ruric Thar, the Unbowed",
        "Ruric Thar's triggered ability triggers whenever any player casts a noncreature spell, including you."
    );
    ruling!(
        "Blind Creeper",
        "This ability triggers off anyone casting a spell, including you, not just Blind Creeper’s controller or an opponent."
    );
    ruling!(
        "Desecration Elemental",
        "This ability triggers off anyone casting a spell, including you, not just Desecration Elemental’s controller or an opponent."
    );
    ruling!(
        "Ebon Drake",
        "This ability triggers off anyone casting a spell, including you, not just Ebon Drake’s controller or an opponent."
    );
    ruling!("Curse of Wizardry", "Curse of Wizardry affects all players, including you.");
    ruling!("Spellshock", "It affects all players, including you.");
    type Check = fn(&TestGame, &[ObjectId]);
    let cases: [(&str, Check); 5] = [
        ("Ruric Thar, the Unbowed", |t, _| assert_eq!(t.life(P0), 14)),
        ("Blind Creeper", |t, o| assert_eq!(t.pt(o[0]), (2, 2))),
        ("Desecration Elemental", |t, o| {
            assert!(!t.on_battlefield(o[0]) || !t.on_battlefield(o[1]));
            assert_eq!(crate::r_s01_common::creatures(t, P0).len(), 1);
        }),
        ("Ebon Drake", |t, _| assert_eq!(t.life(P0), 19)),
        ("Spellshock", |t, _| assert_eq!(t.life(P0), 18)),
    ];
    for (name, check) in cases {
        supported(name);
        let mut t = TestGame::new(2);
        let perm = t.battlefield(P0, name);
        let bears = t.battlefield(P0, "Grizzly Bears");
        cast_card(&mut t, P0, "Divination", &[]);
        assert!(trigger_on_top(&t), "{name}");
        t.resolve_all();
        check(&t, &[perm, bears]);
        assert_eq!(t.life(P1), 20, "{name}");
    }
    // Curse of Wizardry (blue chosen): P0 loses 1 life for their own blue spell.
    let mut t = TestGame::new(2);
    curse_of_wizardry_blue(&mut t);
    cast_card(&mut t, P0, "Divination", &[]);
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
}

#[test]
fn sphere_of_resistance_makes_your_own_spells_cost_more_too() {
    cr!("601.2f");
    ruling!("Sphere of Resistance", "It affects all players, including you.");
    supported("Sphere of Resistance");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sphere of Resistance");
    t.lands(P0, "Island", 3);
    let div = t.hand(P0, "Divination");
    assert!(t.cast(P0, div).try_go().is_err(), "{{2}}{{U}} + {{1}} > 3 lands");
    t.lands(P0, "Island", 1);
    t.cast(P0, div).go();
    assert_eq!(crate::r_s04_common::untapped_lands(&t, P0), 0);
}

#[test]
fn desecration_elemental_sacrifices_itself_if_it_is_your_only_creature() {
    cr!("701.21a");
    ruling!(
        "Desecration Elemental",
        "If Desecration Elemental is the only creature you control when its triggered ability resolves, you’ll have to sacrifice it."
    );
    let mut t = TestGame::new(2);
    let de = t.battlefield(P0, "Desecration Elemental");
    t.battlefield(P1, "Grizzly Bears");
    cast_card(&mut t, P1, "Divination", &[]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Desecration Elemental"));
    assert!(!t.g.is_live(de));
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn presence_of_the_master_triggers_on_casting_and_counters_the_enchantment() {
    cr!("601.2i", "603.3", "701.6a");
    ruling!(
        "Presence of the Master",
        "This effect triggers when the spell is announced and prior to allowing responses to it."
    );
    supported("Presence of the Master");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Presence of the Master");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let pac = cast_card(&mut t, P1, "Pacifism", &[obj(bears)]);
    // The trigger is on the stack before any player gets priority with Pacifism on top.
    assert_eq!(t.stack_len(), 2);
    assert!(trigger_on_top(&t));
    t.resolve_all();
    assert_eq!(t.zone(pac), Zone::Graveyard(P1));
    assert!(t.on_battlefield(bears));
    assert!(t.g.battlefield.iter().all(|a| t.g.obj(*a).attached_to.is_none()));
}

#[test]
fn presence_of_the_master_does_not_stop_enchantments_put_onto_the_battlefield() {
    cr!("603.2");
    ruling!(
        "Presence of the Master",
        "Does not stop enchantments from being put onto the battlefield by a spell or ability."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Presence of the Master");
    t.graveyard(P1, "Glorious Anthem");
    // Open the Vaults is a sorcery, not an enchantment spell: no trigger.
    cast_card(&mut t, P1, "Open the Vaults", &[]);
    assert!(!trigger_on_top(&t));
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Glorious Anthem").len(), 1);
}

#[test]
fn nether_void_is_a_trigger_not_an_additional_cost() {
    cr!("601.2f", "603.3", "118.12a");
    ruling!(
        "Nether Void",
        "This is not an additional cost, it is a triggered ability."
    );
    supported("Nether Void");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Nether Void");
    // P1 has exactly Divination's {2}{U}: casting it is legal.
    let div = cast_card(&mut t, P1, "Divination", &[]);
    assert_eq!(t.zone(div), Zone::Stack);
    assert!(trigger_on_top(&t));
    // P1 can't pay {3} as the trigger resolves: Divination is countered.
    t.resolve_all();
    assert_eq!(t.zone(div), Zone::Graveyard(P1));
    assert_eq!(t.hand_size(P1), 0);

    // With {3} more available, P1 pays as the trigger resolves and Divination resolves.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Nether Void");
    let div = cast_card(&mut t, P1, "Divination", &[]);
    t.lands(P1, "Wastes", 3);
    t.answer_yes(P1, true);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 2);
    assert_ne!(t.zone(div), Zone::Stack);
}

#[test]
fn ash_zealot_only_counts_spells_cast_from_a_graveyard() {
    cr!("601.3", "603.2");
    ruling!(
        "Ash Zealot",
        "Ash Zealot’s ability doesn’t allow any player to cast cards from a graveyard. Such cards need another way to be cast, such as the flashback ability."
    );
    ruling!(
        "Ash Zealot",
        "Ash Zealot’s ability doesn’t trigger if you put a card onto the battlefield from a graveyard (using Grave Betrayal’s ability, for example)."
    );
    supported("Ash Zealot");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ash Zealot");
    // A Divination in a graveyard can't be cast.
    t.lands(P0, "Island", 3);
    let div = t.graveyard(P0, "Divination");
    assert!(legal_cast_methods(&mut t, P0, div).is_empty());
    // P0 reanimates a creature card: putting it onto the battlefield isn't casting it.
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 1);
    let rean = t.hand(P0, "Reanimate");
    t.cast(P0, rean).target(bears).go();
    t.settle();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    // Only Reanimate's own life loss (Grizzly Bears has mana value 2).
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn norin_the_wary_is_exiled_if_it_attacks() {
    cr!("603.2", "506.4", "603.7");
    ruling!("Norin the Wary", "Norin the Wary will be exiled if it attacks.");
    supported("Norin the Wary");
    let mut t = TestGame::new(2);
    let norin = t.battlefield(P0, "Norin the Wary");
    attack_with(&mut t, &[(norin, pl(P1))]);
    t.resolve_all();
    assert_eq!(t.zone(norin), Zone::Exile);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Norin the Wary").len(), 1);
    assert_eq!(t.life(P1), 20, "removed from combat");
}

#[test]
fn hearthborn_battlers_target_need_not_be_the_caster() {
    cr!("603.3d");
    ruling!(
        "Hearthborn Battler",
        "The target of Hearthborn Battler’s last ability doesn’t have to be the same player who cast their second spell."
    );
    supported("Hearthborn Battler");
    // P0 casts their second spell; the trigger targets P0's opponent.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hearthborn Battler");
    cast_card(&mut t, P0, "Opt", &[]);
    t.resolve_all();
    assert!(!trigger_on_top(&t));
    t.answer_targets(P0, &[pl(P1)]);
    cast_card(&mut t, P0, "Opt", &[]);
    assert!(trigger_on_top(&t));
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn roiling_vortex_does_not_undo_life_already_gained() {
    cr!("119.3");
    ruling!(
        "Roiling Vortex",
        "Activating Roiling Vortex’s last ability won’t undo any life gained before it resolved."
    );
    supported("Roiling Vortex");
    let mut t = TestGame::new(2);
    let vortex = t.battlefield(P0, "Roiling Vortex");
    t.lands(P1, "Plains", 2);
    let salve = t.hand(P1, "Healing Salve");
    t.cast(P1, salve).modes(&[0]).target(pl(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 23);
    add_mana(&mut t, P0, ManaType::R, 1);
    t.activate(P0, vortex, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 23);
    let salve = t.hand(P1, "Healing Salve");
    t.cast(P1, salve).modes(&[0]).target(pl(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 23, "can't gain life now");
}

#[test]
fn roiling_vortex_triggers_for_each_player_on_a_two_headed_giant_team() {
    cr!("805.4d", "810.9");
    ruling!(
        "Roiling Vortex",
        "In a Two-Headed Giant game, Roiling Vortex’s first ability triggers twice during each team’s upkeep. Each instance deals 1 damage to one of the players on that team."
    );
    let mut t = crate::r_p050_common::two_headed_giant();
    t.battlefield(P0, "Roiling Vortex");
    t.advance_to(P2, Step::Upkeep);
    t.settle();
    let items = stack_items(&t);
    assert_eq!(items.len(), 2, "{items:?}");
    t.resolve_all();
    assert_eq!(t.life(P2), 28);
    assert_eq!(t.life(P0), 30);
}
