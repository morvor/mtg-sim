//! Rulings batch P041 — damage that can't be prevented (CR 615.12) meeting prevention
//! effects with additional effects (CR 615.5), damage dealt to several recipients at once
//! (CR 120.2, 704.3), and the source of damage (CR 120.1, 609.7).

use crate::r_p041_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;
use mtg_engine::object::Zone;

/// A game where P1 controls Leyline of Punishment ("Damage can't be prevented.") and a
/// Hill Giant (the usual source of damage).
fn unpreventable() -> (TestGame, ObjectId) {
    supported("Leyline of Punishment");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Leyline of Punishment");
    let giant = t.battlefield(P1, "Hill Giant");
    (t, giant)
}

// --- Gideons ----------------------------------------------------------------------------------

#[test]
fn unpreventable_damage_to_an_animated_gideon_is_marked_and_removes_loyalty() {
    cr!("615.12", "120.3c", "120.3e", "704.5i", "702.12b");
    ruling!(
        "Gideon, Ally of Zendikar",
        "If damage that can't be prevented is dealt to Gideon after his first ability has resolved, that damage will have all applicable results"
    );
    ruling!(
        "Gideon, the Oathsworn",
        "If damage that can't be prevented is dealt to Gideon after his first loyalty ability has resolved, that damage will have all applicable results"
    );
    ruling!(
        "Gideon of the Trials",
        "If damage that can't be prevented is dealt to Gideon after his second ability has resolved, that damage will have all applicable results"
    );
    ruling!(
        "Gideon, Martial Paragon",
        "If damage that can't be prevented is dealt to Gideon after his second ability has resolved, that damage will have all applicable results"
    );
    ruling!(
        "Gideon Blackblade",
        "If damage that can't be prevented is dealt to Gideon during your turn, that damage will have all applicable results"
    );
    ruling!(
        "Gideon, Champion of Justice",
        "If damage that can't be prevented is dealt to Gideon, Champion of Justice after his second ability has resolved, that damage will have all applicable results"
    );
    ruling!(
        "Gideon Jura",
        "If you activate Gideon Jura's third ability and then unpreventable damage is dealt to him (due to Unstable Footing, for example), that damage has all applicable results"
    );
    ruling!(
        "Kytheon, Hero of Akros // Gideon, Battle-Forged",
        "If you activate Gideon's third ability and then damage is dealt to him that can't be prevented, that damage has all applicable results"
    );
    let kytheon = "Kytheon, Hero of Akros // Gideon, Battle-Forged";
    // (card, its "becomes a creature" ability, if activated)
    let cases: [(&str, Option<usize>); 8] = [
        ("Gideon, Ally of Zendikar", Some(0)),
        ("Gideon, the Oathsworn", Some(0)),
        ("Gideon of the Trials", Some(1)),
        ("Gideon, Martial Paragon", Some(1)),
        ("Gideon Blackblade", None),
        ("Gideon, Champion of Justice", Some(1)),
        ("Gideon Jura", Some(2)),
        (kytheon, Some(2)),
    ];
    for (name, ability) in cases {
        supported(name);
        let (mut t, giant) = unpreventable();
        let gideon = t.battlefield(P0, name);
        if name == kytheon {
            assert!(mtg_engine::dfc::transform(&mut t.g, gideon));
            t.g.recompute();
        }
        let gideon = t.g.current(gideon);
        set_loyalty(&mut t, gideon, 5);
        if let Some(i) = ability {
            activate_resolve(&mut t, P0, gideon, i, &[]);
        }
        assert!(t.obj_now(gideon).is_creature(), "{name} is a creature");
        set_loyalty(&mut t, gideon, 3);
        damage(&mut t, giant, obj(gideon), 2);
        assert_eq!(damage_marked(&t, gideon), 2, "{name}: the damage is marked");
        assert_eq!(t.counters(gideon, "loyalty"), 1, "{name}: and removes loyalty");
        assert!(t.on_battlefield(gideon), "{name}");
        damage(&mut t, giant, obj(gideon), 1);
        assert!(!t.on_battlefield(gideon), "{name}: no loyalty");
        assert!(matches!(t.zone(gideon), Zone::Graveyard(P0)), "{name}");
    }
}

#[test]
fn gideon_jura_dealt_lethal_unpreventable_damage_is_destroyed() {
    cr!("615.12", "704.5g", "120.3e");
    ruling!(
        "Gideon Jura",
        "If the total amount of damage marked on Gideon Jura is lethal damage, he's destroyed as a state-based action."
    );
    let (mut t, giant) = unpreventable();
    let gideon = t.battlefield(P0, "Gideon Jura");
    activate_resolve(&mut t, P0, gideon, 2, &[]);
    set_loyalty(&mut t, gideon, 10);
    damage(&mut t, giant, obj(gideon), 6);
    assert!(t.in_graveyard(P0, "Gideon Jura"), "6 damage on a 6/6 with 4 loyalty");
}

// --- Prevent and put counters / exile cards ---------------------------------------------------

#[test]
fn unpreventable_damage_still_has_the_shields_additional_effect() {
    cr!("615.12", "615.5", "704.3");
    ruling!(
        "Jared Carthalion, True Heir",
        "If damage that can't be prevented is dealt to Jared Carthalion while you're the monarch, the damage is dealt and you'll simultaneously put that many +1/+1 counters on it, all before the game checks for lethal damage."
    );
    ruling!(
        "Gatta and Luzzu",
        "If damage that can't be prevented is dealt to the target creature after Gatta and Luzzu's last ability resolves, you still put that many +1/+1 counters on it."
    );
    ruling!(
        "Ironscale Hydra",
        "If damage that can’t be prevented is dealt to Ironscale Hydra, the damage is dealt and you’ll simultaneously put a +1/+1 counter on it, before the game checks for lethal damage."
    );
    ruling!(
        "Nine Lives",
        "If damage that a source would deal to you can't be prevented, you still put an incarnation counter on Nine Lives."
    );
    supported("Jared Carthalion, True Heir");
    supported("Gatta and Luzzu");
    supported("Ironscale Hydra");
    supported("Nine Lives");
    // Jared (3/3), P0 the monarch: 3 damage, 3 counters.
    let (mut t, giant) = unpreventable();
    let jared = t.battlefield(P0, "Jared Carthalion, True Heir");
    mtg_engine::designations::become_monarch(&mut t.g, P0);
    damage(&mut t, giant, obj(jared), 3);
    assert!(t.on_battlefield(jared));
    assert_eq!(damage_marked(&t, jared), 3);
    assert_eq!(t.counters(jared, "+1/+1"), 3);
    // Gatta and Luzzu's shield on Grizzly Bears: 3 damage, 3 counters.
    let (mut t, giant) = unpreventable();
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    t.enter(P0, "Gatta and Luzzu");
    t.g.flush_events();
    t.resolve_all();
    damage(&mut t, giant, obj(bears), 3);
    assert!(t.on_battlefield(bears));
    assert_eq!(damage_marked(&t, bears), 3);
    assert_eq!(t.counters(bears, "+1/+1"), 3);
    // Ironscale Hydra (5/5): 5 combat damage and a counter.
    let (mut t, giant) = unpreventable();
    let hydra = t.battlefield(P0, "Ironscale Hydra");
    damage_at_once(&mut t, &[(giant, obj(hydra), 5)], true);
    assert!(t.on_battlefield(hydra));
    assert_eq!(damage_marked(&t, hydra), 5);
    assert_eq!(t.counters(hydra, "+1/+1"), 1);
    // Nine Lives: P0 loses life and still gets a counter.
    let (mut t, giant) = unpreventable();
    let lives = t.battlefield(P0, "Nine Lives");
    damage(&mut t, giant, pl(P0), 3);
    assert_eq!(t.life(P0), 17);
    assert_eq!(t.counters(lives, "incarnation"), 1);
}

#[test]
fn gloom_surgeon_exiles_cards_for_unpreventable_combat_damage() {
    cr!("615.12", "615.5");
    ruling!(
        "Gloom Surgeon",
        "If damage that can’t be prevented (such as the damage Malignus deals) is dealt to Gloom Surgeon, you’ll still exile cards from your library."
    );
    supported("Gloom Surgeon");
    let (mut t, _) = unpreventable();
    let surgeon = t.battlefield(P1, "Gloom Surgeon");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let library = t.library_size(P1);
    fight_it_out(&mut t, &[bears], &[(surgeon, bears)]);
    assert!(!t.on_battlefield(surgeon), "the 2 damage was dealt");
    assert_eq!(t.library_size(P1), library - 2);
}

#[test]
fn phyrexian_hydra_dealt_unpreventable_damage_gets_no_counters() {
    cr!("615.12", "615.5");
    ruling!(
        "Phyrexian Hydra",
        "If damage dealt to Phyrexian Hydra can’t be prevented (due to Leyline of Punishment, for example), it will be dealt as normal and won’t cause Phyrexian Hydra to put -1/-1 counters on itself."
    );
    let (mut t, giant) = unpreventable();
    let hydra = t.battlefield(P0, "Phyrexian Hydra");
    damage(&mut t, giant, obj(hydra), 3);
    assert_eq!(damage_marked(&t, hydra), 3);
    assert_eq!(t.counters(hydra, "-1/-1"), 0);
}

#[test]
fn stormwild_capridor_gets_counters_before_lethal_damage_is_checked() {
    cr!("615.5", "615.12", "704.3");
    ruling!(
        "Stormwild Capridor",
        "If damage that can be prevented is dealt to Stormwild Capridor at the same time as damage that can’t be prevented, it gets +1/+1 counters for the prevented damage before checking to see if it dies from the damage that couldn’t be prevented."
    );
    supported("Stormwild Capridor");
    supported("Malignus");
    // Malignus: "Damage that would be dealt by Malignus can't be prevented." 3 from
    // Malignus and 2 from the Bears to the 1/3 Capridor at once.
    let mut t = TestGame::new(2);
    let capridor = t.battlefield(P0, "Stormwild Capridor");
    let malignus = t.battlefield(P1, "Malignus");
    let bears = t.battlefield(P1, "Grizzly Bears");
    damage_at_once(
        &mut t,
        &[(malignus, obj(capridor), 3), (bears, obj(capridor), 2)],
        false,
    );
    assert!(t.on_battlefield(capridor));
    assert_eq!(damage_marked(&t, capridor), 3);
    assert_eq!(t.counters(capridor, "+1/+1"), 2);
}

// --- Damage dealt at once to several recipients ------------------------------------------------

#[test]
fn prevention_for_others_applies_even_if_its_source_is_dealt_lethal_damage_at_once() {
    cr!("704.3", "615.1a");
    ruling!(
        "Crystal Barricade",
        "If Crystal Barricade and other creatures you control would be dealt noncombat damage at the same time, the damage that would be dealt to your other creatures is prevented even if the damage being dealt to Crystal Barricade is lethal damage."
    );
    ruling!(
        "Emmara Tandris",
        "If Emmara Tandris is dealt lethal damage at the same creature tokens you control would be dealt damage, the damage that would be dealt to those tokens is prevented."
    );
    ruling!(
        "Tajic, Legion's Edge",
        "If Tajic and other creatures you control would be dealt lethal noncombat damage at the same time, the damage that would be dealt to your other creatures is prevented."
    );
    ruling!(
        "The Wanderer",
        "If The Wanderer would be dealt lethal damage at the same time as you and/or other permanents you control would be dealt noncombat damage, the damage that would be dealt to you and your other permanents is prevented."
    );
    supported("Crystal Barricade");
    supported("Emmara Tandris");
    supported("Tajic, Legion's Edge");
    supported("The Wanderer");
    for (name, lethal) in [
        ("Crystal Barricade", 4),
        ("Emmara Tandris", 7),
        ("Tajic, Legion's Edge", 2),
        ("The Wanderer", 5),
    ] {
        let mut t = TestGame::new(2);
        let protector = t.battlefield(P0, name);
        let other = if name == "Emmara Tandris" {
            crate::r_s02_common::create_token(&mut t, P0, "Soldier")
        } else {
            t.battlefield(P0, "Grizzly Bears")
        };
        let src = t.battlefield(P1, "Hill Giant");
        let mut events = vec![(src, obj(protector), lethal), (src, obj(other), 2)];
        if name == "The Wanderer" {
            events.push((src, pl(P0), 3));
        }
        damage_at_once(&mut t, &events, false);
        assert!(!t.on_battlefield(protector), "{name} was dealt lethal damage");
        assert!(t.on_battlefield(other), "{name}");
        assert_eq!(damage_marked(&t, other), 0, "{name}");
        assert_eq!(t.life(P0), 20, "{name}");
    }
}

#[test]
fn elderscale_wurm_and_damage_dealt_at_once_to_you_and_it() {
    cr!("704.3", "614.1a");
    ruling!(
        "Elderscale Wurm",
        "For example, if you have 10 life and a spell deals 10 damage to you and 10 damage to Elderscale Wurm, you’ll end up at 7 life and Elderscale Wurm will be destroyed."
    );
    ruling!(
        "Elderscale Wurm",
        "If you have less than 7 life, damage dealt to you reduces your life total as normal."
    );
    supported("Elderscale Wurm");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Elderscale Wurm");
    let src = t.battlefield(P1, "Hill Giant");
    t.g.players[0].life = 10;
    damage_at_once(&mut t, &[(src, pl(P0), 10), (src, obj(wurm), 10)], false);
    assert_eq!(t.life(P0), 7);
    assert!(!t.on_battlefield(wurm));
    // Below 7 life.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Elderscale Wurm");
    let src = t.battlefield(P1, "Hill Giant");
    t.g.players[0].life = 5;
    damage(&mut t, src, pl(P0), 3);
    assert_eq!(t.life(P0), 2);
}

#[test]
fn jared_prevents_damage_before_the_monarch_trigger_resolves() {
    cr!("725.2", "510.2", "615.5");
    ruling!(
        "Jared Carthalion, True Heir",
        "If you're the monarch and damage would be dealt to Jared Carthalion at the same time combat damage is dealt to you, Jared Carthalion's last ability will prevent damage and put counters on it before the triggered ability that makes another player the monarch resolves."
    );
    let mut t = TestGame::new(2);
    let jared = t.battlefield(P0, "Jared Carthalion, True Heir");
    mtg_engine::designations::become_monarch(&mut t.g, P0);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    to_beginning_of_combat(&mut t, P1);
    attack_with(&mut t, &[(giant, pl(P0)), (bears, pl(P0))]);
    block_and_finish(&mut t, P0, &[(jared, bears)]);
    assert_eq!(t.life(P0), 17);
    assert_eq!(t.g.monarch, Some(P1));
    assert_eq!(damage_marked(&t, jared), 0);
    assert_eq!(t.counters(jared, "+1/+1"), 2);
    // P0 is no longer the monarch: damage to Jared is dealt.
    damage(&mut t, giant, obj(jared), 1);
    assert_eq!(damage_marked(&t, jared), 1);
    assert_eq!(t.counters(jared, "+1/+1"), 2);
}

// --- Swans of Bryn Argoll: the source of the damage --------------------------------------------

#[test]
fn swans_of_bryn_argoll_and_the_source_of_damage() {
    cr!("120.1", "615.5", "615.6");
    ruling!(
        "Swans of Bryn Argoll",
        "If a creature with wither would deal damage to Swans of Bryn Argoll, that damage is treated just like any other damage. It’s prevented, and the creature’s controller draws cards. Swans of Bryn Argoll doesn’t get any -1/-1 counters."
    );
    ruling!(
        "Swans of Bryn Argoll",
        "If a spell is causing damage to be dealt, that spell will always identify the source of the damage. In most cases, the source is the spell itself."
    );
    ruling!(
        "Swans of Bryn Argoll",
        "If an ability is causing damage to be dealt, that ability will always identify the source of the damage. The ability itself is never the source. However, the source of the ability is often the source of the damage."
    );
    supported("Swans of Bryn Argoll");
    supported("Boggart Ram-Gang");
    supported("Prodigal Pyromancer");
    // Wither: 3 damage from P1's Ram-Gang.
    let mut t = TestGame::new(2);
    let swans = t.battlefield(P0, "Swans of Bryn Argoll");
    let gang = t.battlefield(P1, "Boggart Ram-Gang");
    let hand = t.hand_size(P1);
    damage(&mut t, gang, obj(swans), 3);
    assert_eq!(t.hand_size(P1), hand + 3);
    assert_eq!(t.counters(swans, "-1/-1"), 0);
    assert_eq!(damage_marked(&t, swans), 0);
    // A spell: P1's Lightning Bolt; its controller draws.
    let mut t = TestGame::new(2);
    let swans = t.battlefield(P0, "Swans of Bryn Argoll");
    t.set_step(P1, Step::PrecombatMain);
    cast_new(&mut t, P1, "Lightning Bolt", &[obj(swans)]);
    let hand = t.hand_size(P1);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), hand + 3);
    assert_eq!(t.hand_size(P0), 0);
    // An ability: P1's Prodigal Pyromancer is the source; P1 draws 1.
    let mut t = TestGame::new(2);
    let swans = t.battlefield(P0, "Swans of Bryn Argoll");
    let pyromancer = t.battlefield(P1, "Prodigal Pyromancer");
    t.set_step(P1, Step::PrecombatMain);
    let hand = t.hand_size(P1);
    activate_resolve(&mut t, P1, pyromancer, 0, &[obj(swans)]);
    assert_eq!(t.hand_size(P1), hand + 1);
    assert_eq!(t.hand_size(P0), 0);
}
