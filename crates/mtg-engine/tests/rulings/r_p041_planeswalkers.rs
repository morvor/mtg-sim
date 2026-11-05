//! Rulings batch P041 — planeswalkers whose abilities prevent damage: Gideons that become
//! creatures (summoning sickness, CR 302.6; not entering, CR 603.6a), "attacks Gideon if
//! able" requirements (CR 508.1d), emblems (CR 114), loyalty costs (CR 606), Oko's copy
//! effects (CR 707.2) and Ajani's counters.

use crate::r_p041_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

use crate::r_s02_common::can_attack;
use crate::r_s09_common::legal_attack;

const KYTHEON: &str = "Kytheon, Hero of Akros // Gideon, Battle-Forged";

/// Puts Kytheon onto the battlefield as Gideon, Battle-Forged with `loyalty`.
fn battle_forged(t: &mut TestGame, p: PlayerId, loyalty: u32) -> ObjectId {
    let k = t.battlefield(p, KYTHEON);
    assert!(mtg_engine::dfc::transform(&mut t.g, k));
    let k = t.g.current(k);
    set_loyalty(t, k, loyalty);
    k
}

#[test]
fn a_gideon_that_entered_this_turn_and_becomes_a_creature_cant_attack() {
    cr!("302.6", "306.5b");
    ruling!(
        "Gideon Jura",
        "Note that summoning sickness cares about when Gideon Jura came under your control, not when he became a creature."
    );
    ruling!(
        "Gideon of the Trials",
        "If Gideon becomes a creature the same turn he enters the battlefield, you can't attack with him or use any of his {T} abilities (if he gains any)."
    );
    ruling!(
        "Gideon, Ally of Zendikar",
        "If Gideon becomes a creature the same turn he enters the battlefield, you can't attack with him or use any of his {T} abilities (if he gains any)."
    );
    ruling!(
        "Gideon, Martial Paragon",
        "If Gideon becomes a creature the same turn he enters the battlefield, you can't attack with him or use any of his {T} abilities (if he gains any)."
    );
    ruling!(
        "Gideon, the Oathsworn",
        "If Gideon becomes a creature the same turn he enters the battlefield, you can't attack with him or use any of his {T} abilities (if he gains any)."
    );
    ruling!(
        "Gideon, Champion of Justice",
        "If Gideon, Champion of Justice becomes a creature the same turn he enters, you can't attack with him or use any of his {T} abilities (if he gains any)."
    );
    ruling!(
        "Oko, the Trickster",
        "If Oko becomes a creature the same turn he enters the battlefield, you can't attack with him or use any of his {T} abilities (if he gains any)."
    );
    ruling!(
        "Gideon Jura",
        "If Gideon Jura becomes a creature, he may be affected by \"summoning sickness.\" You can't attack with him or use any of his {T} abilities (if he gains any) unless he began your most recent turn on the battlefield under your control."
    );
    let cases: [(&str, usize); 7] = [
        ("Gideon Jura", 2),
        ("Gideon of the Trials", 1),
        ("Gideon, Ally of Zendikar", 0),
        ("Gideon, Martial Paragon", 1),
        ("Gideon, the Oathsworn", 0),
        ("Gideon, Champion of Justice", 1),
        ("Oko, the Trickster", 1),
    ];
    for (name, ability) in cases {
        for sick in [true, false] {
            let mut t = TestGame::new(2);
            let pw = if sick {
                t.battlefield_sick(P0, name)
            } else {
                t.battlefield(P0, name)
            };
            let targets = if name.starts_with("Oko") {
                vec![obj(t.battlefield(P0, "Grizzly Bears"))]
            } else {
                vec![]
            };
            activate_resolve(&mut t, P0, pw, ability, &targets);
            assert!(t.obj_now(pw).is_creature(), "{name}");
            t.set_step(P0, Step::BeginningOfCombat);
            assert_eq!(can_attack(&mut t, pw), !sick, "{name}, entered this turn: {sick}");
        }
    }
}

#[test]
fn a_gideon_becoming_a_creature_doesnt_enter_the_battlefield() {
    cr!("603.6a", "613.1d");
    ruling!(
        "Kytheon, Hero of Akros // Gideon, Battle-Forged",
        "If Gideon becomes a creature due to his third ability, that doesn't count as having a creature enter the battlefield."
    );
    ruling!(
        "Gideon, Champion of Justice",
        "If Gideon, Champion of Justice becomes a creature due to his second ability, that doesn't count as having a creature enter."
    );
    supported(KYTHEON);
    supported("Soul Warden");
    for champion in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Soul Warden");
        let gideon = if champion {
            t.battlefield(P0, "Gideon, Champion of Justice")
        } else {
            battle_forged(&mut t, P0, 3)
        };
        activate_resolve(
            &mut t,
            P0,
            gideon,
            if champion { 1 } else { 2 },
            &[],
        );
        assert!(t.obj_now(gideon).is_creature());
        assert_eq!(t.stack_len(), 0, "Soul Warden didn't trigger");
        assert_eq!(t.life(P0), 20);
    }
}

#[test]
fn creatures_told_to_attack_a_gideon_that_left_may_attack_anyone_or_not_at_all() {
    cr!("508.1d");
    ruling!(
        "Gideon Jura",
        "If a creature controlled by the affected player can't attack Gideon Jura (because he's no longer on the battlefield, for example), that player may have it attack you, another one of your planeswalkers, or nothing at all."
    );
    ruling!(
        "Kytheon, Hero of Akros // Gideon, Battle-Forged",
        "If Gideon can't be attacked, perhaps because he has left the battlefield before the creature's controller's next combat, the creature targeted by Gideon's first ability can attack you or another planeswalker you control, or its controller can choose to have it not attack at all."
    );
    for (jura, leaves) in [(true, false), (true, true), (false, false), (false, true)] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let other = t.battlefield(P0, "Gideon, Ally of Zendikar");
        let gideon = if jura {
            let g = t.battlefield(P0, "Gideon Jura");
            activate_resolve(&mut t, P0, g, 0, &[pl(P1)]);
            g
        } else {
            let g = battle_forged(&mut t, P0, 3);
            activate_resolve(&mut t, P0, g, 0, &[obj(bears)]);
            g
        };
        if leaves {
            exile_now(&mut t, gideon);
        }
        t.advance_to(P1, Step::BeginningOfCombat);
        let ctx = format!("Jura: {jura}, left: {leaves}");
        assert_eq!(legal_attack(&mut t, &[]), leaves, "{ctx}");
        assert_eq!(legal_attack(&mut t, &[(bears, pl(P0))]), leaves, "{ctx}");
        assert_eq!(legal_attack(&mut t, &[(bears, obj(other))]), leaves, "{ctx}");
        if !leaves {
            assert!(legal_attack(&mut t, &[(bears, obj(gideon))]), "{ctx}");
        }
    }
}

#[test]
fn gideon_the_oathsworn_with_nine_loyalty_goes_to_the_graveyard() {
    cr!("606.4", "704.5i", "608.2c");
    ruling!(
        "Gideon, the Oathsworn",
        "If Gideon has 9 loyalty as you activate his last ability, he'll be put into your graveyard and won't be exiled."
    );
    let mut t = TestGame::new(2);
    let gideon = t.battlefield(P0, "Gideon, the Oathsworn");
    set_loyalty(&mut t, gideon, 9);
    let bears = t.battlefield(P1, "Grizzly Bears");
    activate_resolve(&mut t, P0, gideon, 1, &[]);
    assert!(matches!(t.zone(gideon), Zone::Graveyard(P0)));
    assert_eq!(t.zone(bears), Zone::Exile);
}

#[test]
fn gideon_of_the_trials_emblem_stays_and_works_with_any_gideon() {
    cr!("114.4", "104.3b", "104.3c", "104.3h");
    ruling!(
        "Gideon of the Trials",
        "If Gideon leaves the battlefield after giving you his emblem, you'll keep the emblem. Its effect will apply again later if another Gideon comes under your control."
    );
    ruling!(
        "Gideon of the Trials",
        "No game effect can cause you to lose the game or cause any opponent to win the game while Gideon's emblem is in effect. It doesn't matter whether you have 0 or less life, you're forced to draw a card while your library is empty, you have ten or more poison counters"
    );
    let mut t = TestGame::new(2);
    let gideon = t.battlefield(P0, "Gideon of the Trials");
    activate_resolve(&mut t, P0, gideon, 2, &[]);
    let emblems = t.g.command.len();
    exile_now(&mut t, gideon);
    assert_eq!(t.g.command.len(), emblems, "the emblem stays");
    t.battlefield(P0, "Gideon Jura");
    t.g.players[0].life = 0;
    t.g.players[0].counters.insert("poison".into(), 10);
    while let Some(c) = t.g.players[0].library.pop() {
        t.g.objects[c.0 as usize].zone = Zone::Nowhere;
    }
    t.g.draw_cards(P0, 1);
    t.g.flush_events();
    t.settle();
    assert!(!t.has_lost(P0));
    // Without a Gideon, the emblem does nothing.
    let mut t = TestGame::new(2);
    let gideon = t.battlefield(P0, "Gideon of the Trials");
    activate_resolve(&mut t, P0, gideon, 2, &[]);
    exile_now(&mut t, gideon);
    t.g.players[0].life = 0;
    t.settle();
    assert!(t.has_lost(P0));
}

#[test]
fn gideon_of_the_trials_emblem_in_two_headed_giant() {
    cr!("114.4", "810.8a", "104.3b");
    ruling!(
        "Gideon of the Trials",
        "If you have Gideon's emblem in a Two-Headed Giant game and control a Gideon planeswalker, your team can't lose the game and the opposing team can't win the game. If your teammate controls a Gideon planeswalker but you do not, your emblem's effect doesn't apply."
    );
    for mine in [true, false] {
        let mut t = crate::r_p108_common::two_headed_giant();
        let gideon = t.battlefield(P0, "Gideon of the Trials");
        activate_resolve(&mut t, P0, gideon, 2, &[]);
        exile_now(&mut t, gideon);
        t.battlefield(if mine { P0 } else { P1 }, "Gideon Jura");
        t.g.players[0].life = 0;
        t.g.players[1].life = 0;
        t.settle();
        assert_eq!(t.has_lost(P0), !mine, "P0 controls the Gideon: {mine}");
        assert_eq!(t.has_lost(P1), !mine, "P0 controls the Gideon: {mine}");
    }
}

#[test]
fn gideon_blackblade_sheds_equipment_on_other_players_turns() {
    cr!("301.5c", "704.5n");
    ruling!(
        "Gideon Blackblade",
        "If an Equipment becomes attached to Gideon while he's a creature, it'll become unattached during the next upkeep that isn't yours."
    );
    supported("Gideon Blackblade");
    let mut t = TestGame::new(2);
    let gideon = t.battlefield(P0, "Gideon Blackblade");
    let blade = t.battlefield(P0, "Bonesplitter");
    assert!(t.g.attach(blade, obj(gideon)));
    t.g.recompute();
    assert_eq!(t.pt(gideon), (6, 4));
    t.advance_to(P1, Step::Upkeep);
    // (State-based actions are checked before P1 receives priority.)
    t.settle();
    assert!(!t.obj_now(gideon).is_creature());
    assert_eq!(t.obj_now(blade).attached_to, None);
    assert!(t.on_battlefield(blade));
}

#[test]
fn gideon_blackblade_losing_all_abilities_is_still_a_creature() {
    cr!("613.1d", "613.1f", "613.8a");
    ruling!(
        "Gideon Blackblade",
        "If an effect causes Gideon to lose all abilities during your turn, he's still a Human Soldier creature and a Gideon planeswalker."
    );
    supported("Dress Down");
    let mut t = TestGame::new(2);
    let gideon = t.battlefield(P0, "Gideon Blackblade");
    t.battlefield(P1, "Dress Down");
    t.g.recompute();
    let o = t.obj_now(gideon);
    assert!(o.is_creature());
    assert!(o.is(mtg_engine::types::CardType::Planeswalker));
    assert!(o.chars.has_subtype("Human") && o.chars.has_subtype("Soldier"));
    assert!(o.chars.has_subtype("Gideon"));
    assert!(o.chars.abilities.is_empty());
}

#[test]
fn gideon_champion_of_justice_whose_plus_one_doesnt_resolve() {
    cr!("606.4", "608.2b");
    ruling!(
        "Gideon, Champion of Justice",
        "If the first ability doesn't resolve (perhaps because the target opponent is an illegal target when the ability tries to resolve), you won't put any additional loyalty counters on Gideon, although the loyalty counter you put on him to activate the ability will remain."
    );
    for hexproof in [false, true] {
        let mut t = TestGame::new(2);
        let gideon = t.battlefield(P0, "Gideon, Champion of Justice");
        t.battlefield(P1, "Grizzly Bears");
        t.battlefield(P1, "Hill Giant");
        activate(&mut t, P0, gideon, 0, &[pl(P1)]);
        if hexproof {
            t.battlefield(P1, "Leyline of Sanctity");
            t.g.recompute();
        }
        t.resolve_all();
        assert_eq!(
            t.counters(gideon, "loyalty"),
            if hexproof { 5 } else { 7 },
            "P1 has hexproof: {hexproof}"
        );
    }
}

#[test]
fn kytheon_counts_attackers_that_died_but_not_creatures_put_onto_the_battlefield_attacking() {
    cr!("506.3", "508.3a", "603.4");
    ruling!(
        "Kytheon, Hero of Akros // Gideon, Battle-Forged",
        "Kytheon's first ability will count creatures that attacked but are no longer on the battlefield (perhaps because they didn't survive combat damage being dealt). It will not count any creatures that were put onto the battlefield attacking"
    );
    supported("Hero of Bladehold");
    // Kytheon, Grizzly Bears and Hill Giant attack; the Bears die blocked by a Wurm.
    let mut t = TestGame::new(2);
    let kytheon = t.battlefield(P0, KYTHEON);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let wurm = t.battlefield(P1, "Craw Wurm");
    fight_it_out(&mut t, &[kytheon, bears, giant], &[(wurm, bears)]);
    assert!(!t.on_battlefield(bears));
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Gideon, Battle-Forged").len(), 1);
    // Kytheon and Hero of Bladehold attack; Hero's two Soldiers enter attacking.
    let mut t = TestGame::new(2);
    let kytheon = t.battlefield(P0, KYTHEON);
    let hero = t.battlefield(P0, "Hero of Bladehold");
    fight_it_out(&mut t, &[kytheon, hero], &[]);
    t.resolve_all();
    assert_eq!(crate::r_p108_common::tokens_with(&t, P0, "Soldier"), 2);
    assert!(t.on_battlefield(kytheon), "not transformed");
    assert!(t.named_on_battlefield("Gideon, Battle-Forged").is_empty());
}

// --- Ajani Steadfast ----------------------------------------------------------------------

#[test]
fn ajani_steadfast_gives_a_creature_planeswalker_both_counters() {
    cr!("122.1", "306.5c");
    ruling!(
        "Ajani Steadfast",
        "If a permanent you control is both a creature and a planeswalker, Ajani’s second ability will put both a +1/+1 counter and a loyalty counter on it."
    );
    supported("Ajani Steadfast");
    let mut t = TestGame::new(2);
    let ajani = t.battlefield(P0, "Ajani Steadfast");
    let gideon = t.battlefield(P0, "Gideon Jura");
    activate_resolve(&mut t, P0, gideon, 2, &[]);
    assert!(t.obj_now(gideon).is_creature());
    activate_resolve(&mut t, P0, ajani, 1, &[]);
    assert_eq!(t.counters(gideon, "+1/+1"), 1);
    assert_eq!(t.counters(gideon, "loyalty"), 7);
    assert_eq!(t.counters(ajani, "loyalty"), 2, "not on Ajani himself");
}

// --- Oko, the Trickster -------------------------------------------------------------------

#[test]
fn oko_copies_what_the_target_is_copying() {
    cr!("707.3", "707.2");
    ruling!(
        "Oko, the Trickster",
        "If the target creature is copying something else, then Oko becomes a copy of whatever that creature copied."
    );
    let mut t = TestGame::new(2);
    let oko = t.battlefield(P0, "Oko, the Trickster");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(giant)]);
    let clone = cast_resolve(&mut t, P0, "Clone", &[]);
    let clone = t.g.current(clone);
    assert_eq!(t.obj_now(clone).name(), "Hill Giant");
    activate_resolve(&mut t, P0, oko, 1, &[obj(clone)]);
    assert_eq!(t.obj_now(oko).name(), "Hill Giant");
    assert_eq!(t.pt(oko), (3, 3));
}

#[test]
fn oko_copies_only_printed_values() {
    cr!("707.2", "613.2");
    ruling!(
        "Oko, the Trickster",
        "Oko copies the printed values of the target creature. It won't copy counters on that creature or effects that have changed its power, toughness, types, color, or so on."
    );
    let mut t = TestGame::new(2);
    let oko = t.battlefield(P0, "Oko, the Trickster");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.add_counters(obj(bears), "+1/+1", 2, None);
    cast_resolve(&mut t, P0, "Giant Growth", &[obj(bears)]);
    assert_eq!(t.pt(bears), (7, 7));
    activate_resolve(&mut t, P0, oko, 1, &[obj(bears)]);
    assert_eq!(t.obj_now(oko).name(), "Grizzly Bears");
    assert_eq!(t.pt(oko), (2, 2));
    assert_eq!(t.counters(oko, "+1/+1"), 0);
}

#[test]
fn okos_ultimate_affects_only_creatures_controlled_as_it_resolves() {
    cr!("611.2c", "613.4b");
    ruling!(
        "Oko, the Trickster",
        "Oko's last ability affects only creatures you control at the time it resolves. Creatures you begin to control later in the turn won't be 10/10."
    );
    let mut t = TestGame::new(2);
    let oko = t.battlefield(P0, "Oko, the Trickster");
    set_loyalty(&mut t, oko, 7);
    let bears = t.battlefield(P0, "Grizzly Bears");
    activate_resolve(&mut t, P0, oko, 2, &[]);
    assert_eq!(t.pt(bears), (10, 10));
    let giant = t.battlefield(P0, "Hill Giant");
    assert_eq!(t.pt(giant), (3, 3));
    let stolen = t.battlefield(P1, "Craw Wurm");
    cast_resolve(&mut t, P0, "Act of Treason", &[obj(stolen)]);
    assert_eq!(t.pt(stolen), (6, 4));
}
