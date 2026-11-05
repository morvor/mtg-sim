//! Rulings batch P003 — artifacts that make themselves creatures: what they keep (CR
//! 205.2a, 205.3g: Treasure is an artifact type), base power and toughness set in layer
//! 7b (CR 613.4), summoning sickness (CR 302.6), a creature that stops being one during
//! combat (CR 506.4), and tapping untapped tokens for a cost (CR 302.6).

use crate::r_p003_common::*;
use crate::r_s01_common::supported;
use crate::r_s02_common::can_attack;
use crate::r_s20_common::tap_for_mana;
use crate::r_s24_common::enter_together;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn mimics_are_still_treasures_and_treasure_is_never_a_creature_type() {
    cr!("205.3g", "205.3m", "613.1d");
    ruling!(
        "Dire Mimic",
        "Dire Mimic is still a Treasure after its last ability resolves, but Treasure is an artifact type, never a creature type."
    );
    ruling!(
        "Mimic",
        "Mimic is still a Treasure after its last ability resolves, but Treasure is an artifact type, never a creature type."
    );
    supported("Dire Mimic");
    supported("Mimic");
    supported("Maskwood Nexus");
    for (name, cost, n) in [("Dire Mimic", "3", 5), ("Mimic", "2", 3)] {
        let mut t = TestGame::new(2);
        // Maskwood Nexus: "Creatures you control are every creature type."
        t.battlefield(P0, "Maskwood Nexus");
        let m = t.battlefield(P0, name);
        activate_resolve(&mut t, P0, m, "Shapeshifter", cost, &[]);
        let o = t.obj_now(m);
        assert!(o.is(CardType::Creature) && o.is(CardType::Artifact), "{name}");
        assert!(o.chars.has_subtype("Treasure") && o.chars.has_subtype("Shapeshifter"));
        assert!(o.chars.has_subtype("Elf"), "{name}: every creature type");
        assert_eq!(t.pt(m), (n, n), "{name}");
        // Still sacrificed for mana as a Treasure.
        assert!(tap_for_mana(&mut t, P0, m, "Add one mana"), "{name}");
        assert!(!t.on_battlefield(m));
    }
}

#[test]
fn an_effect_that_refers_to_a_treasure_means_any_treasure_artifact() {
    cr!("205.3g", "111.10a");
    ruling!(
        "Dire Mimic",
        "If an effect refers to a Treasure, it means any Treasure artifact, not just a Treasure artifact token."
    );
    ruling!(
        "Goldhound",
        "If an effect refers to a Treasure, it means any Treasure artifact, not just a Treasure artifact token."
    );
    ruling!(
        "Mimic",
        "If an effect refers to a Treasure, it means any Treasure artifact, not just a Treasure artifact token."
    );
    ruling!(
        "Minimus Containment",
        "If an effect refers to a Treasure, it means any Treasure artifact, not just a Treasure artifact token."
    );
    supported("Captain Lannery Storm");
    supported("Goldhound");
    supported("Minimus Containment");
    // Captain Lannery Storm (2/2): "Whenever you sacrifice a Treasure, Captain Lannery
    // Storm gets +1/+0 until end of turn."
    for name in ["Dire Mimic", "Goldhound", "Mimic", "Minimus Containment"] {
        let mut t = TestGame::new(2);
        let storm = t.battlefield(P0, "Captain Lannery Storm");
        let treasure = if name == "Minimus Containment" {
            // "Enchanted permanent is a Treasure artifact with "{T}, Sacrifice this
            // artifact: Add one mana of any color," and it loses all other abilities."
            let bear = t.battlefield(P0, "Grizzly Bears");
            cast_resolve(&mut t, P0, name, "2W", &[obj(bear)]);
            assert!(has_subtype(&t, bear, "Treasure"));
            bear
        } else {
            t.battlefield(P0, name)
        };
        assert!(!t.obj_now(treasure).is_token());
        assert!(tap_for_mana(&mut t, P0, treasure, "Add one mana"), "{name}");
        t.resolve_all();
        assert_eq!(t.pt(storm), (3, 2), "{name}");
    }
}

#[test]
fn glint_hawk_idol_sets_its_base_2_2_over_other_setters_and_is_a_bird() {
    cr!("613.4b", "613.4c", "613.7a");
    ruling!(
        "Glint Hawk Idol",
        "For example, say one of Glint Hawk Idol's abilities has resolved and it's a 2/2 creature. You cast Giant Growth targeting it. It's now 5/5. Then Diminish (\"Target creature becomes 1/1 until end of turn\") is cast targeting it. Once Diminish resolves, Glint Hawk Idol would be 4/4. Activating Glint Hawk Idol's last ability at this point would make it 5/5 again until end of turn."
    );
    ruling!(
        "Glint Hawk Idol",
        "If either of Glint Hawk Idol's two abilities resolve while it's a creature, that ability will override any effects that set its power and/or toughness to another number, but effects that modify power and/or toughness without directly setting them will still apply."
    );
    ruling!(
        "Glint Hawk Idol",
        "when either of its abilities resolves, it will become a Bird artifact creature."
    );
    supported("Glint Hawk Idol");
    let mut t = TestGame::new(2);
    let idol = t.battlefield(P0, "Glint Hawk Idol");
    activate_resolve(&mut t, P0, idol, "Bird", "W", &[]);
    assert_eq!(t.pt(idol), (2, 2));
    assert!(has_subtype(&t, idol, "Bird"));
    assert!(t.obj_now(idol).has_keyword(KeywordKind::Flying));
    giant_growth(&mut t, idol);
    assert_eq!(t.pt(idol), (5, 5));
    cast_resolve(&mut t, P0, "Diminish", "U", &[obj(idol)]);
    assert_eq!(t.pt(idol), (4, 4));
    activate_resolve(&mut t, P0, idol, "Bird", "W", &[]);
    assert_eq!(t.pt(idol), (5, 5));
    // The triggered ability does the same.
    cast_resolve(&mut t, P0, "Diminish", "U", &[obj(idol)]);
    assert_eq!(t.pt(idol), (4, 4));
    t.answer_yes(P0, true);
    enter(&mut t, P0, "Ornithopter");
    t.resolve_all();
    assert_eq!(t.pt(idol), (5, 5));
    assert!(has_subtype(&t, idol, "Bird"));
}

#[test]
fn glint_hawk_idol_entering_this_turn_is_summoning_sick() {
    cr!("302.6", "603.6a", "603.10a");
    ruling!(
        "Glint Hawk Idol",
        "If Glint Hawk Idol and another artifact enter under your control at the same time, Glint Hawk Idol's first ability will trigger. You may have it become a creature, but since it will be affected by \"summoning sickness,\" it won't be able to attack that turn."
    );
    ruling!(
        "Glint Hawk Idol",
        "If Glint Hawk Idol becomes a creature before you begin a turn with it under your control, it will be affected by \"summoning sickness.\""
    );
    let mut t = TestGame::new(2);
    t.answer_yes(P0, true);
    let ids = enter_together(&mut t, &[(P0, "Glint Hawk Idol"), (P0, "Ornithopter")]);
    t.resolve_all();
    let idol = ids[0];
    assert!(is_creature(&t, idol));
    t.set_step(P0, Step::DeclareAttackers);
    assert!(!can_attack(&mut t, idol));
    // One that was there since the turn began may attack after its {W} ability.
    let old = t.battlefield(P0, "Glint Hawk Idol");
    t.set_step(P0, Step::PrecombatMain);
    activate_resolve(&mut t, P0, old, "Bird", "W", &[]);
    t.set_step(P0, Step::DeclareAttackers);
    assert!(can_attack(&mut t, old));
}

#[test]
fn rusted_relic_removed_from_combat_doesnt_return_and_its_sickness_is_about_control() {
    cr!("506.4", "509.1h", "302.6");
    ruling!(
        "Rusted Relic",
        "If Rusted Relic stops being a creature during combat (because you lose control of one or more artifacts, perhaps), it is removed from combat and will not deal or be dealt combat damage. Creatures it was blocking remain blocked. It won't re-enter that combat, even if you wind up controlling three or more artifacts again during that combat phase."
    );
    ruling!(
        "Rusted Relic",
        "Whether Rusted Relic has \"summoning sickness\" is based on how long you have continuously controlled it, not on how long it has continuously been a creature."
    );
    supported("Rusted Relic");
    let mut t = TestGame::new(2);
    let relic = t.battlefield(P0, "Rusted Relic");
    let stone = t.battlefield(P0, "Mind Stone");
    assert!(!is_creature(&t, relic));
    // A third artifact: it's a 5/5 Golem now and may attack (controlled all turn).
    t.battlefield(P0, "Ornithopter");
    assert!(is_creature(&t, relic));
    t.set_step(P0, Step::DeclareAttackers);
    assert!(can_attack(&mut t, relic));
    // On P1's turn, it blocks P1's Hill Giant. Then P0 loses Mind Stone: removed from
    // combat. A Memnite enters before damage: a creature again, not in combat.
    let giant = t.battlefield(P1, "Hill Giant");
    crate::r_s20_common::to_beginning_of_combat(&mut t, P1);
    t.answer(
        P1,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(giant, pl(P0))]),
    );
    t.answer(P0, DecisionKind::Blockers, Answer::Blockers(vec![(relic, giant)]));
    t.advance_to(P1, Step::DeclareBlockers);
    kill(&mut t, stone);
    assert!(!is_creature(&t, relic));
    enter(&mut t, P0, "Memnite");
    assert!(is_creature(&t, relic));
    t.advance_to(P1, Step::EndOfCombat);
    assert_eq!(t.life(P0), 20, "the Giant stayed blocked");
    assert!(t.on_battlefield(giant) && t.on_battlefield(relic));
    assert_eq!(dmg(&t, giant), 0);
    assert_eq!(dmg(&t, relic), 0);
}

#[test]
fn gruul_war_plow_is_a_juggernaut_with_trample_but_no_juggernaut_abilities() {
    cr!("205.3m", "613.1d");
    ruling!(
        "Gruul War Plow",
        "While it's a creature, Gruul War Plow gives itself trample."
    );
    ruling!(
        "Gruul War Plow",
        "While it's a creature, Gruul War Plow has creature type Juggernaut. It doesn't have any abilities associated with the card named Juggernaut."
    );
    supported("Gruul War Plow");
    let mut t = TestGame::new(2);
    let plow = t.battlefield(P0, "Gruul War Plow");
    activate_resolve(&mut t, P0, plow, "Juggernaut", "1RG", &[]);
    assert!(has_subtype(&t, plow, "Juggernaut"));
    assert!(t.obj_now(plow).has_keyword(KeywordKind::Trample));
    assert_eq!(t.pt(plow), (4, 4));
    // Juggernaut's "attacks each combat if able" doesn't apply: declaring no attackers
    // is legal. (A real Juggernaut is made to attack.)
    let jugg = t.battlefield(P0, "Juggernaut");
    t.attack(&[], &[]);
    assert!(crate::r_p015_common::attacking(&t, plow).is_none());
    assert!(crate::r_p015_common::attacking(&t, jugg).is_some());
}

#[test]
fn tangle_tumbler_may_tap_summoning_sick_tokens() {
    cr!("302.6", "118.3");
    ruling!(
        "Tangle Tumbler",
        "You may tap any untapped tokens you control, including creature tokens you control that you haven't controlled continuously since the beginning of your most recent turn, to pay the cost of Tangle Tumbler's last ability."
    );
    supported("Tangle Tumbler");
    supported("Raise the Alarm");
    let mut t = TestGame::new(2);
    let tumbler = t.battlefield(P0, "Tangle Tumbler");
    // Raise the Alarm: two 1/1 Soldier tokens, this turn.
    cast_resolve(&mut t, P0, "Raise the Alarm", "1W", &[]);
    let toks = crate::r_s01_common::with_subtype(&t, P0, "Soldier");
    assert_eq!(toks.len(), 2);
    assert!(toks.iter().all(|x| t.obj_now(*x).summoning_sick));
    let es: Vec<Entity> = toks.iter().map(|x| obj(*x)).collect();
    t.answer_choose(P0, &es);
    activate_resolve(&mut t, P0, tumbler, "Tap two untapped tokens", "", &[]);
    assert!(is_creature(&t, tumbler));
    assert!(toks.iter().all(|x| t.obj_now(*x).tapped));
}
