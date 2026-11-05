//! Rulings batch P040 — Gideons and the other cards of the damage prevention group:
//! planeswalkers that become creatures (types, not entering, locked-in power), "attacks
//! if able" requirements and their exceptions (CR 508.1d), non-Gideon creatures, counters
//! on a planeswalker that isn't a creature, Oko's copy and later P/T changes, Noble
//! Heritage's player protection, and Gor Muldrak.

use crate::r_p039_creatureland::activate_text;
use crate::r_p040_common::*;
use crate::r_s01_common::{
    asked_since, attack_with, block_and_finish, supported, triggers_on_stack,
};
use crate::r_s02_common::destroy;
use crate::r_s04_common::{next_upkeep, spell_targets};
use crate::r_s25_common::cast_new;
use mtg_engine::combat::{attack_declaration_legal, attack_options};
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Whether the declaration of attackers `decl` would be legal now (CR 508.1).
fn legal_attack(t: &mut TestGame, decl: &[(ObjectId, Entity)]) -> bool {
    t.g.recompute();
    let options = attack_options(&t.g);
    attack_declaration_legal(&t.g, &options, decl)
}

/// Advances to the beginning of combat of `p`'s next turn.
fn next_combat(t: &mut TestGame, p: PlayerId) {
    if t.g.turn.active == p {
        t.advance_to_step(Step::End);
    }
    t.advance_to(p, Step::BeginningOfCombat);
}

/// Kytheon transformed into Gideon, Battle-Forged under `p`'s control, with `loyalty`.
fn battle_forged(t: &mut TestGame, p: PlayerId, loyalty: u32) -> ObjectId {
    let g = t.battlefield(p, "Kytheon, Hero of Akros // Gideon, Battle-Forged");
    assert!(mtg_engine::dfc::transform(&mut t.g, g));
    t.g.recompute();
    t.g.objects[g.0 as usize]
        .counters
        .insert("loyalty".into(), loyalty);
    g
}

// ---------------------------------------------------------------------------
// Becoming a creature: types, not entering, P/T
// ---------------------------------------------------------------------------

/// (card, the ability, its creature types, its P/T)
const BECOMES: [(&str, &str, &[&str], (i32, i32)); 6] = [
    (
        "Gideon, Ally of Zendikar",
        "+1:",
        &["Human", "Soldier", "Ally"],
        (5, 5),
    ),
    ("Gideon, the Oathsworn", "+2:", &["Soldier"], (5, 5)),
    (
        "Gideon of the Trials",
        "0: Until end of turn",
        &["Human", "Soldier"],
        (4, 4),
    ),
    (
        "Gideon, Martial Paragon",
        "0:",
        &["Human", "Soldier"],
        (5, 5),
    ),
    (
        "Kytheon, Hero of Akros // Gideon, Battle-Forged",
        "0:",
        &["Human", "Soldier"],
        (4, 4),
    ),
    (
        "Gideon, Champion of Justice",
        "0:",
        &["Human", "Soldier"],
        (4, 4),
    ),
];

/// P0's `card` (Kytheon transformed), with loyalty 4.
fn gideon(t: &mut TestGame, card: &str) -> ObjectId {
    if card.starts_with("Kytheon") {
        battle_forged(t, P0, 4)
    } else {
        let g = t.battlefield(P0, card);
        t.g.objects[g.0 as usize]
            .counters
            .insert("loyalty".into(), 4);
        g
    }
}

#[test]
fn gideons_become_creatures_with_creature_types_and_stay_gideon_planeswalkers() {
    cr!("205.1b", "205.3d", "205.3j", "205.3m", "613.1d");
    ruling!(
        "Gideon, Ally of Zendikar",
        "Gideon's first ability causes him to become a creature with the creature types Human, Soldier, and Ally. He remains a planeswalker with the planeswalker type Gideon. (He also retains any other card types or subtypes he may have had.) Each subtype is correlated to the proper card type: planeswalker is only a type (not a creature type), and Human, Soldier, and Ally are just creature types (not planeswalker types)."
    );
    ruling!(
        "Gideon, the Oathsworn",
        "Gideon's first loyalty ability causes him to become a creature with the creature type Soldier. He remains a planeswalker with the planeswalker type Gideon. (He also retains any other card types or subtypes he may have had.) Each subtype is correlated to the proper card type: planeswalker is only a type (not a creature type), and Soldier is just a creature type (not a planeswalker type)."
    );
    ruling!(
        "Gideon of the Trials",
        "Gideon's second ability causes him to become a creature with the creature types Human and Soldier. He remains a planeswalker with the planeswalker type Gideon. (He also retains any other card types or subtypes he may have had.) Each subtype is correlated to the proper card type: planeswalker is only a type (not a creature type), and Human and Soldier are just creature types (not planeswalker types)."
    );
    ruling!(
        "Gideon, Martial Paragon",
        "Gideon's second ability causes him to become a creature with the creature types Human and Soldier. He remains a planeswalker with the planeswalker type Gideon. (He also retains any other card types or subtypes he may have had.) Each subtype is correlated to the proper card type: planeswalker is only a type (not a creature type), and Human and Soldier are just creature types (not planeswalker types)."
    );
    ruling!(
        "Kytheon, Hero of Akros // Gideon, Battle-Forged",
        "Gideon's third ability causes him to become a creature with the creature types Human Soldier. He remains a planeswalker with the planeswalker type Gideon. (He also retains any other card types or subtypes he may have had.) Each subtype is correlated to the proper card type: Gideon is just a planeswalker type (not a creature type), and Human and Soldier are just creature types (not planeswalker types)."
    );
    ruling!(
        "Gideon, Champion of Justice",
        "Gideon, Champion of Justice's second ability causes him to become a creature with the creature types Human and Soldier. He remains a planeswalker with the planeswalker type Gideon. (He also retains any other card types or subtypes he may have had.) Each subtype is correlated to the proper card type: Gideon is just a planeswalker type (not a creature type), and Human and Soldier are just creature types (not planeswalker types)."
    );
    assert!(!is_creature_type("Gideon"));
    assert!(is_planeswalker_type("Gideon"));
    for (card, prefix, types, pt) in BECOMES {
        supported(card);
        let mut t = TestGame::new(2);
        let g = gideon(&mut t, card);
        loyalty(&mut t, P0, g, prefix, &[]);
        assert!(is(&t, g, CardType::Creature), "{card}");
        assert!(is(&t, g, CardType::Planeswalker), "{card}");
        assert!(
            t.obj_now(g).chars.supertypes.contains(Supertype::Legendary),
            "{card}"
        );
        assert!(has_subtype(&t, g, "Gideon"), "{card}");
        for ty in types {
            assert!(has_subtype(&t, g, ty), "{card} {ty}");
            assert!(is_creature_type(ty) && !is_planeswalker_type(ty), "{ty}");
        }
        // (Gideon, Champion of Justice: P/T equal to its loyalty, 4.)
        assert_eq!(t.pt(g), pt, "{card}");
        // As a Soldier creature it gets Field Marshal's bonus ("Other Soldier creatures
        // you control get +1/+1 and have first strike").
        t.battlefield(P0, "Field Marshal");
        t.g.recompute();
        assert_eq!(t.pt(g), (pt.0 + 1, pt.1 + 1), "{card}");
    }
}

#[test]
fn gideons_becoming_creatures_dont_enter_the_battlefield() {
    cr!("603.6a", "611.2a");
    ruling!(
        "Gideon, Ally of Zendikar",
        "Gideon's first ability doesn't count as a creature entering the battlefield. Gideon was already on the battlefield; he only changed his types. Specifically, your rally abilities won't trigger. (Of course, Gideon's second ability will cause rally abilities to trigger.)"
    );
    ruling!(
        "Gideon, the Oathsworn",
        "Gideon's first loyalty ability doesn't count as a creature entering the battlefield. Gideon was already on the battlefield; he only changed his types."
    );
    ruling!(
        "Gideon of the Trials",
        "Gideon's second ability doesn't count as a creature entering the battlefield. Gideon was already on the battlefield; he only changed his types."
    );
    ruling!(
        "Gideon, Martial Paragon",
        "Gideon's second ability doesn't count as a creature entering the battlefield. Gideon was already on the battlefield; he only changed his types."
    );
    supported("Makindi Patrol");
    for (card, prefix, _, _) in &BECOMES[..4] {
        let mut t = TestGame::new(2);
        // Soul Warden: "Whenever another creature enters, you gain 1 life."
        t.battlefield(P0, "Soul Warden");
        let g = gideon(&mut t, card);
        loyalty(&mut t, P0, g, prefix, &[]);
        assert!(is(&t, g, CardType::Creature), "{card}");
        assert_eq!(t.life(P0), 20, "{card}");
    }
    // Rally: Makindi Patrol ("Rally — Whenever this creature or another Ally you control
    // enters, creatures you control gain vigilance until end of turn"). Gideon (an Ally
    // creature now) doesn't trigger it; his 0 (a 2/2 Knight Ally token) does.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Makindi Patrol");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let g = gideon(&mut t, "Gideon, Ally of Zendikar");
    let i = ability(&mut t, g, "+1:");
    t.activate(P0, g, i, &[]).unwrap();
    t.resolve_all();
    assert!(has_subtype(&t, g, "Ally"));
    assert_eq!(triggers_on_stack(&t, "gain vigilance"), 0);
    assert!(!t
        .obj_now(bears)
        .has_keyword(mtg_engine::keywords::KeywordKind::Vigilance));
    // Next turn, his 0.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    loyalty(&mut t, P0, g, "0:", &[]);
    assert!(t
        .obj_now(bears)
        .has_keyword(mtg_engine::keywords::KeywordKind::Vigilance));
}

#[test]
fn gideon_champion_of_justice_locks_in_its_power_and_toughness() {
    cr!("608.2h", "613.4b");
    ruling!(
        "Gideon, Champion of Justice",
        "Gideon, Champion of Justice's power and toughness are set to the number of loyalty counters on him when his second ability resolves. They won't change later in the turn if the number of loyalty counters on him changes."
    );
    let mut t = TestGame::new(2);
    let g = gideon(&mut t, "Gideon, Champion of Justice");
    loyalty(&mut t, P0, g, "0:", &[]);
    assert_eq!(t.pt(g), (4, 4));
    t.g.add_counters(o(g), "loyalty", 3, None);
    t.g.flush_events();
    t.g.recompute();
    assert_eq!(t.counters(g, "loyalty"), 7);
    assert_eq!(t.pt(g), (4, 4));
}

#[test]
fn gideon_of_the_trials_prevention_outlasts_him() {
    cr!("610.3", "615.1a");
    ruling!(
        "Gideon of the Trials",
        "Gideon's first ability will continue to prevent damage dealt by the target permanent even if Gideon leaves the battlefield before your next turn."
    );
    let mut t = TestGame::new(2);
    let g = gideon(&mut t, "Gideon of the Trials");
    let giant = old(&mut t, P1, "Hill Giant");
    loyalty(&mut t, P0, g, "+1:", &[o(giant)]);
    destroy(&mut t, g);
    assert!(!t.on_battlefield(g));
    t.advance_to(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(giant, pl(P0))]);
    block_and_finish(&mut t, P0, &[]);
    assert_eq!(t.life(P0), 20);
    assert_eq!(damage_dealt_by(&t, giant), 0);
}

/// Damage dealt this turn by `source`.
fn damage_dealt_by(t: &TestGame, source: ObjectId) -> u32 {
    crate::r_s30_common::damage_events(t)
        .iter()
        .filter(|(s, _, _, _)| *s == source)
        .map(|(_, _, n, _)| *n)
        .sum()
}

#[test]
fn gideon_blackblade_keeps_counters_while_not_a_creature() {
    cr!("122.1a", "122.2", "306.5c");
    ruling!(
        "Gideon Blackblade",
        "Any counters that are put on Gideon remain on him while he's not a creature, even if they have no effect on a noncreature planeswalker."
    );
    supported("Gideon Blackblade");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Gideon Blackblade");
    assert!(is(&t, g, CardType::Creature));
    t.g.add_counters(o(g), counters::PLUS1, 1, None);
    t.g.flush_events();
    t.g.recompute();
    assert_eq!(t.pt(g), (5, 5));
    let loyalty_now = t.counters(g, "loyalty");
    // P1's turn: not a creature, the counter stays.
    t.advance_to(P1, Step::PrecombatMain);
    assert!(!is(&t, g, CardType::Creature));
    assert_eq!(t.counters(g, counters::PLUS1), 1);
    assert_eq!(t.counters(g, "loyalty"), loyalty_now);
    // P0's next turn: a 5/5 again.
    t.advance_to(P0, Step::PrecombatMain);
    assert!(is(&t, g, CardType::Creature));
    assert_eq!(t.pt(g), (5, 5));
}

#[test]
fn oko_copy_still_gets_counters_and_modifying_effects() {
    cr!("613.4b", "613.4c", "613.7", "707.2");
    ruling!(
        "Oko, the Trickster",
        "Any effects that modify a creature's power and/or toughness without setting them to a specific number or value will apply after its base power and toughness are set, regardless of the order in which those effects were created. The same is true of counters that modify its power and toughness."
    );
    supported("Oko, the Trickster");
    let mut t = TestGame::new(2);
    // Glorious Anthem's effect is older than the copy effect; the counter is put on Oko
    // before he becomes a creature.
    t.battlefield(P0, "Glorious Anthem");
    let oko = t.battlefield(P0, "Oko, the Trickster");
    t.g.add_counters(o(oko), counters::PLUS1, 1, None);
    t.g.flush_events();
    let bears = t.battlefield(P0, "Grizzly Bears");
    loyalty(&mut t, P0, oko, "0:", &[o(bears)]);
    assert!(is(&t, oko, CardType::Creature));
    assert_eq!(t.obj_now(oko).chars.name, "Grizzly Bears");
    // 2/2 + 1/1 (Anthem) + 1/1 (counter).
    assert_eq!(t.pt(oko), (4, 4));
    // A newer modifying effect: Giant Growth.
    cast_new(&mut t, P0, "Giant Growth", &[o(oko)]);
    t.resolve_all();
    assert_eq!(t.pt(oko), (7, 7));
}

#[test]
fn gideon_martial_paragon_pumps_all_creatures_not_just_untapped_ones() {
    cr!("608.2c", "611.2c");
    ruling!(
        "Gideon, Martial Paragon",
        "All creatures you control get +1/+1 from Gideon's first ability, not just the ones that it untapped."
    );
    let mut t = TestGame::new(2);
    let g = gideon(&mut t, "Gideon, Martial Paragon");
    let tapped = t.battlefield(P0, "Grizzly Bears");
    t.g.objects[tapped.0 as usize].tapped = true;
    let untapped = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    loyalty(&mut t, P0, g, "+2:", &[]);
    assert!(!t.obj_now(tapped).tapped);
    assert_eq!(t.pt(tapped), (3, 3));
    assert_eq!(t.pt(untapped), (4, 4));
    assert_eq!(t.pt(theirs), (2, 2));
}

// ---------------------------------------------------------------------------
// Gideon, the Oathsworn: non-Gideon creatures
// ---------------------------------------------------------------------------

#[test]
fn gideon_the_oathsworn_counts_non_gideon_attackers() {
    cr!("508.1m", "205.3j", "603.2");
    ruling!(
        "Gideon, the Oathsworn",
        "A non-Gideon creature is a creature that isn't also a Gideon planeswalker. For example, Gideon's Company is a non-Gideon creature."
    );
    ruling!(
        "Gideon, the Oathsworn",
        "Any attacking Gideon creatures won't get a +1/+1 counter, but as long as two or more attacking creatures aren't Gideons, those attacking creatures each get a +1/+1 counter."
    );
    // Gideon's Company and the Bears: both get a counter.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gideon, the Oathsworn");
    let company = old(&mut t, P0, "Gideon's Company");
    let bears = old(&mut t, P0, "Grizzly Bears");
    to_combat(&mut t, P0);
    attack_with(&mut t, &[(company, pl(P1)), (bears, pl(P1))]);
    t.resolve_all();
    assert_eq!(t.counters(company, counters::PLUS1), 1);
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    // Gideon (a creature) and two others: the others get counters, Gideon doesn't.
    let mut t = TestGame::new(2);
    let g = gideon(&mut t, "Gideon, the Oathsworn");
    loyalty(&mut t, P0, g, "+2:", &[]);
    let a = old(&mut t, P0, "Grizzly Bears");
    let b = old(&mut t, P0, "Hill Giant");
    to_combat(&mut t, P0);
    attack_with(&mut t, &[(g, pl(P1)), (a, pl(P1)), (b, pl(P1))]);
    t.resolve_all();
    assert_eq!(t.counters(g, counters::PLUS1), 0);
    assert_eq!(t.counters(a, counters::PLUS1), 1);
    assert_eq!(t.counters(b, counters::PLUS1), 1);
    // Gideon and one other: no trigger.
    let mut t = TestGame::new(2);
    let g = gideon(&mut t, "Gideon, the Oathsworn");
    loyalty(&mut t, P0, g, "+2:", &[]);
    let a = old(&mut t, P0, "Grizzly Bears");
    to_combat(&mut t, P0);
    attack_with(&mut t, &[(g, pl(P1)), (a, pl(P1))]);
    t.resolve_all();
    assert_eq!(t.counters(a, counters::PLUS1), 0);
}

// ---------------------------------------------------------------------------
// "Attacks ... if able" (CR 508.1d)
// ---------------------------------------------------------------------------

#[test]
fn gideon_jura_requirement_has_the_usual_exceptions() {
    cr!("508.1d", "508.1a", "508.1c");
    ruling!(
        "Gideon Jura",
        "Gideon Jura's first ability causes creatures to attack him if able. If, during the affected player's declare attackers step, a creature they control is tapped, is affected by a spell or ability that says it can't attack, or is affected by \"summoning sickness,\" then that creature doesn't attack. If there's a cost associated with having a creature attack, the player isn't forced to pay that cost, so the creature doesn't have to attack in that case either."
    );
    supported("Gideon Jura");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Gideon Jura");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let tapped = t.battlefield(P1, "Hill Giant");
    let pacified = t.battlefield(P1, "Craw Wurm");
    crate::r_s06_common::attach_new(&mut t, P0, "Pacifism", o(pacified));
    loyalty(&mut t, P0, g, "+2:", &[pl(P1)]);
    next_combat(&mut t, P1);
    t.g.objects[tapped.0 as usize].tapped = true;
    let sick = t.battlefield_sick(P1, "Grizzly Bears");
    // Only the untapped, unrestricted, non-sick Bears must attack him.
    assert!(!legal_attack(&mut t, &[]));
    assert!(!legal_attack(&mut t, &[(bears, pl(P0))]));
    assert!(legal_attack(&mut t, &[(bears, o(g))]));
    for x in [tapped, pacified, sick] {
        assert!(!legal_attack(&mut t, &[(bears, o(g)), (x, o(g))]));
    }
    // With a cost to attack (Baird, Steward of Argive: "Creatures can't attack you or
    // planeswalkers you control unless their controller pays {1} for each of those
    // creatures"), not attacking is legal.
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Gideon Jura");
    let bears = t.battlefield(P1, "Grizzly Bears");
    loyalty(&mut t, P0, g, "+2:", &[pl(P1)]);
    t.battlefield(P0, "Baird, Steward of Argive");
    next_combat(&mut t, P1);
    assert!(legal_attack(&mut t, &[]));
    assert!(legal_attack(&mut t, &[(bears, o(g))]));
}

#[test]
fn gideon_battle_forged_requirement_has_the_usual_exceptions() {
    cr!("508.1d", "508.1a", "508.1c");
    ruling!(
        "Kytheon, Hero of Akros // Gideon, Battle-Forged",
        "Gideon's first ability causes a creature to attack him if able. If, during its controller's declare attackers step, that creature is tapped, is affected by a spell or ability that says it can't attack, or hasn't been under its controller's control continuously since that player's turn began, then that creature doesn't attack. If there's a cost associated with having that creature attack, its controller isn't forced to pay that cost. If they don't, the creature doesn't have to attack."
    );
    // (what happens to the targeted Bears in P1's turn before combat; whether P1 must
    // attack Gideon with it)
    let cases: [(&str, bool); 5] = [
        ("nothing", true),
        ("tapped", false),
        ("pacifism", false),
        ("control change", false),
        ("attack tax", false),
    ];
    for (what, must) in cases {
        let mut t = TestGame::new(2);
        let g = battle_forged(&mut t, P0, 3);
        let bears = t.battlefield(P1, "Grizzly Bears");
        loyalty(&mut t, P0, g, "+2:", &[o(bears)]);
        if what == "attack tax" {
            t.battlefield(P0, "Baird, Steward of Argive");
        }
        next_combat(&mut t, P1);
        match what {
            "tapped" => t.g.objects[bears.0 as usize].tapped = true,
            "pacifism" => {
                crate::r_s06_common::attach_new(&mut t, P0, "Pacifism", o(bears));
            }
            "control change" => {
                // P1 lost control of it and got it back during this turn.
                for p in [P0, P1] {
                    t.g.objects[bears.0 as usize].base_controller = p;
                    t.g.recompute();
                }
                assert_eq!(t.obj_now(bears).controller, P1);
            }
            _ => {}
        }
        assert_eq!(legal_attack(&mut t, &[]), !must, "{what}");
        if what != "tapped" && what != "pacifism" && what != "control change" {
            assert!(legal_attack(&mut t, &[(bears, o(g))]), "{what}");
        }
    }
}

// ---------------------------------------------------------------------------
// Noble Heritage
// ---------------------------------------------------------------------------

/// A Commander game: P0's commander (Grizzly Bears) has Noble Heritage's ability; at
/// P0's upkeep P1 puts two +1/+1 counters on their Hill Giant, so P0 gains protection
/// from P1. Returns (P0's commander, P1's Hill Giant).
fn noble_heritage() -> (TestGame, ObjectId, ObjectId) {
    supported("Noble Heritage");
    let mut t = crate::r_s13_common::commander_game();
    let cmdr = t.battlefield(P0, "Grizzly Bears");
    t.g.objects[cmdr.0 as usize].is_commander = true;
    t.g.players[P0.idx()]
        .commander_names
        .push("Grizzly Bears".into());
    t.battlefield(P0, "Noble Heritage");
    let giant = old(&mut t, P1, "Hill Giant");
    t.g.recompute();
    yes(&mut t, P0, false);
    yes(&mut t, P1, true);
    t.answer_choose(P1, &[o(giant)]);
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.counters(giant, counters::PLUS1), 2);
    (t, cmdr, giant)
}

#[test]
fn noble_heritage_protects_the_player_not_their_permanents() {
    cr!("702.16b", "702.16k");
    ruling!(
        "Noble Heritage",
        "Although you'll have protection from players who take advantage of your generous offer, permanents you control will not."
    );
    let (mut t, cmdr, _) = noble_heritage();
    // P1 can't target P0, but can target P0's commander and damage it.
    t.advance_to(P1, Step::PrecombatMain);
    let targets = spell_targets(&mut t, P1, "Shock");
    assert!(!targets.contains(&pl(P0)));
    assert!(targets.contains(&o(cmdr)));
    cast_new(&mut t, P1, "Shock", &[o(cmdr)]);
    t.resolve_all();
    assert!(!t.on_battlefield(cmdr));
}

#[test]
fn noble_heritage_lets_creatures_attack_but_prevents_their_damage() {
    cr!("702.16e", "508.1");
    ruling!(
        "Noble Heritage",
        "Creatures can still attack you even if you have protection from their controller, although any damage they would deal to you will be prevented."
    );
    let (mut t, _, giant) = noble_heritage();
    let life = t.life(P0);
    t.advance_to(P1, Step::BeginningOfCombat);
    assert!(legal_attack(&mut t, &[(giant, pl(P0))]));
    attack_with(&mut t, &[(giant, pl(P0))]);
    assert!(t.g.is_attacking(giant));
    block_and_finish(&mut t, P0, &[]);
    assert_eq!(t.life(P0), life);
}

// ---------------------------------------------------------------------------
// Gor Muldrak
// ---------------------------------------------------------------------------

#[test]
fn gor_muldrak_has_protection_from_salamanders_itself() {
    cr!("702.16b", "702.16f");
    ruling!(
        "Gor Muldrak, Amphinologist",
        "Gor Muldrak's first ability affects itself."
    );
    supported("Gor Muldrak, Amphinologist");
    let mut t = TestGame::new(2);
    let gor = old(&mut t, P0, "Gor Muldrak, Amphinologist");
    let salamander = old(&mut t, P1, "Pyric Salamander");
    let bears = old(&mut t, P1, "Grizzly Bears");
    to_combat(&mut t, P0);
    attack_with(&mut t, &[(gor, pl(P1))]);
    assert!(!t.g.can_block(salamander, gor));
    assert!(t.g.can_block(bears, gor));
}

#[test]
fn gor_muldrak_counts_creatures_on_resolution_and_you_order_end_step_triggers() {
    cr!("603.3b", "608.2h");
    ruling!(
        "Gor Muldrak, Amphinologist",
        "Gor Muldrak's last ability checks how many creatures each player controls as it resolves. If other abilities trigger at the beginning of your end step (such as the delayed triggered ability of an encore ability) you choose the order in which they resolve."
    );
    supported("Alert Heedbonder");
    let salamanders =
        |t: &TestGame, p: PlayerId| crate::r_s01_common::with_subtype(t, p, "Salamander").len();
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gor Muldrak, Amphinologist");
    // Alert Heedbonder: "At the beginning of your end step, you gain 1 life for each
    // creature you control with vigilance."
    t.battlefield(P0, "Alert Heedbonder");
    t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::PostcombatMain);
    let from = mark(&t);
    t.advance_to(P0, Step::End);
    t.settle();
    // P0 was asked to order the two end step triggers.
    let orders: Vec<Vec<String>> = asked_since(&t, from)
        .into_iter()
        .filter_map(|(p, d)| match d {
            Decision::Order { items, .. } if p == P0 => Some(items),
            _ => None,
        })
        .collect();
    assert_eq!(orders.len(), 1);
    assert_eq!(orders[0].len(), 2);
    // When the triggers went on the stack P1 had the fewest creatures (1 vs 2); before
    // Gor Muldrak's resolves, P1 gets two more.
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(salamanders(&t, P0), 1);
    assert_eq!(salamanders(&t, P1), 0);
    assert_eq!(t.life(P0), 21);
}
