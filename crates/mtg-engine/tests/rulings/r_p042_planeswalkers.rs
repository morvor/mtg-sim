//! Rulings batch P042 — planeswalkers with prevention abilities: a Gideon creature an
//! opponent controls, attacked and blocking at once (CR 506.4, 509.1a, 510.1), "attacks
//! Gideon if able" and creatures put onto the battlefield attacking (CR 508.1d, 508.4),
//! sets of creatures fixed as an ability resolves (CR 611.2c), Oko as a copy, loyalty
//! costs and choices made on resolution, and Ajani's and Kiora's other abilities.

use crate::r_p042_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

use crate::r_s06_common::give_control;

const KYTHEON: &str = "Kytheon, Hero of Akros // Gideon, Battle-Forged";

#[test]
fn a_gideon_creature_controlled_by_an_opponent_can_be_attacked_and_block() {
    cr!("506.4", "509.1a", "510.1a", "510.1c", "306.4");
    ruling!(
        "Gideon Jura",
        "Say you activate Gideon Jura's third ability, then an opponent gains control of him before combat. You may have any of your creatures attack Gideon Jura (since he's still a planeswalker). Then Gideon Jura may block (since he's a creature). He may block any eligible attacking creature, including one that's attacking him! During combat, he behaves as an attacked planeswalker and/or a blocking creature, as appropriate. For example, he deals combat damage to any creatures he's blocking, but he doesn't deal combat damage to any unblocked creatures that are attacking him."
    );
    ruling!(
        "Kytheon, Hero of Akros // Gideon, Battle-Forged",
        "Say you activate Gideon's third ability, then an opponent gains control of him before combat. You may have any of your creatures attack Gideon (since he's still a planeswalker). Then Gideon may block (since he's a creature). He may block any eligible attacking creature, including one that's attacking him. During combat, he behaves as an attacked planeswalker and/or a blocking creature, as appropriate. For example, he deals combat damage to any creatures he's blocking, but he doesn't deal combat damage to any unblocked creatures that are attacking him."
    );
    ruling!(
        "Gideon, Champion of Justice",
        "Say you activate Gideon, Champion of Justice's second ability, and then an opponent gains control of him before combat. You may have any of your creatures attack Gideon, Champion of Justice (since he's still a planeswalker). Then Gideon, Champion of Justice may block (since he's a creature). He may block any eligible attacking creature, including one that's attacking him! During combat, he behaves as an attacked planeswalker and/or a blocking creature, as appropriate. For example, he deals combat damage to any creatures he's blocking, but he doesn't deal combat damage to any unblocked creatures that are attacking him."
    );
    supported(KYTHEON);
    supported("Gideon, Champion of Justice");
    for (name, ability) in [
        ("Gideon Jura", 2usize),
        (KYTHEON, 2),
        ("Gideon, Champion of Justice", 1),
    ] {
        let mut t = TestGame::new(2);
        // P1 activates their Gideon's creature ability in their turn ...
        t.set_step(P1, Step::PrecombatMain);
        let gideon = t.battlefield(P1, name);
        let gideon = if name == KYTHEON {
            assert!(mtg_engine::dfc::transform(&mut t.g, gideon));
            let g = t.g.current(gideon);
            set_loyalty(&mut t, g, 3);
            g
        } else {
            gideon
        };
        activate_resolve(&mut t, P1, gideon, ability, &[]);
        assert!(t.obj_now(gideon).is(CardType::Creature), "{name}");
        // ... and P0 gains control of him (the ability lasts until end of turn).
        give_control(&mut t, gideon, P0);
        assert_eq!(t.obj_now(gideon).controller, P0);
        // P1 attacks the Gideon P0 now controls with the Bears and the Giant; Gideon
        // blocks the Bears, which is attacking him.
        let bears = t.battlefield(P1, "Grizzly Bears");
        let giant = t.battlefield(P1, "Hill Giant");
        to_beginning_of_combat(&mut t, P1);
        assert!(crate::r_s09_common::legal_attack(
            &mut t,
            &[(bears, obj(gideon)), (giant, obj(gideon))]
        ), "{name}");
        t.attack(&[(bears, obj(gideon)), (giant, obj(gideon))], &[(gideon, bears)]);
        assert!(!t.on_battlefield(bears), "{name}: he dealt damage to the creature he blocked");
        let dealt = damage_by(&t, gideon);
        assert_eq!(dealt.len(), 1, "{name}");
        assert_eq!((dealt[0].0, dealt[0].2), (obj(bears), true), "{name}");
        assert_eq!(damage_marked(&t, giant), 0, "{name}: none to the unblocked attacker");
        // The unblocked Giant's damage to him (as an attacked planeswalker) and the
        // Bears' (as a blocked creature) were both prevented by his ability (one
        // prevention event for the simultaneous damage, CR 615.13).
        let prevented: u32 = prevented_events(&t)
            .iter()
            .filter(|e| e.1 == obj(gideon))
            .map(|e| e.2)
            .sum();
        assert_eq!(prevented, 5, "{name}");
        assert_eq!(damage_dealt_to(&t, obj(gideon)), 0, "{name}");
        assert!(t.on_battlefield(gideon), "{name}");
    }
}

#[test]
fn gideon_jura_requirement_doesnt_apply_to_creatures_put_onto_the_battlefield_attacking() {
    cr!("508.1d", "508.4");
    ruling!(
        "Gideon Jura",
        "The first ability only affects the declaration of attackers. If a creature is put onto the battlefield attacking (thanks to Hero of Bladehold, Preeminent Captain, or the Ninjutsu ability, for example), that creature's controller may choose the defending player or planeswalker that it will be attacking in the normal way."
    );
    supported("Hero of Bladehold");
    let mut t = TestGame::new(2);
    let gideon = t.battlefield(P0, "Gideon Jura");
    activate_resolve(&mut t, P0, gideon, 0, &[pl(P1)]);
    let hero = t.battlefield(P1, "Hero of Bladehold");
    t.advance_to(P1, Step::BeginningOfCombat);
    // The Hero must attack Gideon.
    assert!(!crate::r_s09_common::legal_attack(&mut t, &[(hero, pl(P0))]));
    // Its tokens attack P0, as P1 chooses.
    t.answer_choose(P1, &[pl(P0)]);
    t.answer_choose(P1, &[pl(P0)]);
    attack_with(&mut t, &[(hero, obj(gideon))]);
    t.resolve_all();
    let attacks = crate::r_s09_common::attacks_now(&t);
    let tokens: Vec<Entity> = t
        .named_on_battlefield("Soldier Token")
        .into_iter()
        .filter_map(|s| attacks.iter().find(|(a, _)| *a == s).map(|(_, x)| *x))
        .collect();
    assert_eq!(tokens, vec![pl(P0), pl(P0)]);
}

#[test]
fn serra_and_gideon_martial_paragon_pump_only_the_creatures_there_as_they_resolve() {
    cr!("611.2c");
    ruling!(
        "Serra the Benevolent",
        "Serra's first ability affects only creatures you control with flying at the time it resolves. Creatures you begin to control later in the turn or that gain flying later in the turn won't get +1/+1."
    );
    ruling!(
        "Gideon, Martial Paragon",
        "The set of creatures affected by Gideon's first and third abilities is determined as the abilities resolve. Creatures you begin to control later in the turn and permanents you control that become creatures later in the turn won't get +1/+1 or +2/+2."
    );
    supported("Serra the Benevolent");
    supported("Gideon, Martial Paragon");
    supported("Jump");
    // Serra's +2: the Serra Angel; not the Storm Crow that arrives later, nor the Bears
    // that gains flying later (Jump).
    let mut t = TestGame::new(2);
    let serra = t.battlefield(P0, "Serra the Benevolent");
    let angel = t.battlefield(P0, "Serra Angel");
    let bears = t.battlefield(P0, "Grizzly Bears");
    activate_resolve(&mut t, P0, serra, 0, &[]);
    assert_eq!(t.pt(angel), (5, 5));
    let crow = t.battlefield(P0, "Storm Crow");
    cast_resolve(&mut t, P0, "Jump", &[obj(bears)]);
    assert_eq!((t.pt(crow), t.pt(bears)), ((1, 2), (2, 2)));
    // Gideon's +2 and −10: the Bears; not a creature P0 gains control of later, nor an
    // artifact that becomes a creature later (Tezzeret's Touch).
    for (ability, bonus) in [(0usize, 1), (2, 2)] {
        let mut t = TestGame::new(2);
        let gideon = t.battlefield(P0, "Gideon, Martial Paragon");
        set_loyalty(&mut t, gideon, 10);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let stone = t.battlefield(P0, "Mind Stone");
        activate_resolve(&mut t, P0, gideon, ability, &[]);
        assert_eq!(t.pt(bears), (2 + bonus, 2 + bonus), "ability {ability}");
        let stolen = t.battlefield(P1, "Hill Giant");
        cast_resolve(&mut t, P0, "Act of Treason", &[obj(stolen)]);
        assert_eq!(t.pt(stolen), (3, 3), "ability {ability}");
        let touch = t.battlefield(P0, "Tezzeret's Touch");
        assert!(t.g.attach(touch, obj(stone)));
        t.g.recompute();
        assert_eq!(t.pt(stone), (5, 5), "ability {ability}");
    }
}

#[test]
fn ajani_steadfast_emblem_ignores_who_controls_the_source() {
    cr!("114.4", "615.1a");
    ruling!(
        "Ajani Steadfast",
        "The emblem’s ability doesn’t consider who controls the source dealing damage to you or a planeswalker you control."
    );
    supported("Ajani Steadfast");
    let mut t = TestGame::new(2);
    let ajani = t.battlefield(P0, "Ajani Steadfast");
    set_loyalty(&mut t, ajani, 7);
    activate_resolve(&mut t, P0, ajani, 2, &[]);
    let gideon = t.battlefield(P0, "Gideon Jura");
    let mine = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Hill Giant");
    for src in [mine, theirs] {
        damage(&mut t, src, pl(P0), 3);
        damage(&mut t, src, obj(gideon), 3);
    }
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.counters(gideon, "loyalty"), 4);
    assert_eq!(prevented_events(&t).len(), 4);
}

#[test]
fn ajani_steadfast_plus_one_with_no_target() {
    cr!("606.4", "601.2c", "115.1");
    ruling!(
        "Ajani Steadfast",
        "You may activate the first ability with no targets. If you do, you simply put a loyalty counter on Ajani to pay the cost of the ability."
    );
    let mut t = TestGame::new(2);
    let ajani = t.battlefield(P0, "Ajani Steadfast");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[]);
    activate_resolve(&mut t, P0, ajani, 0, &[]);
    assert_eq!(t.counters(ajani, "loyalty"), 5);
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn oko_as_a_creature_copy_isnt_a_planeswalker() {
    cr!("707.2", "306.8", "120.3c");
    ruling!(
        "Oko, the Trickster",
        "While Oko is a creature, he isn't a planeswalker. Damage dealt to him won't cause him to lose loyalty."
    );
    supported("Oko, the Trickster");
    let mut t = TestGame::new(2);
    let oko = t.battlefield(P0, "Oko, the Trickster");
    let wurm = t.battlefield(P0, "Craw Wurm");
    activate_resolve(&mut t, P0, oko, 1, &[obj(wurm)]);
    assert!(t.obj_now(oko).is(CardType::Creature));
    assert!(!t.obj_now(oko).is(CardType::Planeswalker));
    assert_eq!(t.counters(oko, "loyalty"), 4);
    // Excruciator's damage can't be prevented: it's marked on him, and he loses no loyalty.
    let ex = t.battlefield(P1, "Excruciator");
    damage(&mut t, ex, obj(oko), 2);
    assert_eq!(damage_marked(&t, oko), 2);
    assert_eq!(t.counters(oko, "loyalty"), 4);
    assert!(t.on_battlefield(oko));
}

#[test]
fn gideon_blackblade_chooses_the_keyword_as_the_ability_resolves() {
    cr!("608.2c", "606.4");
    ruling!(
        "Gideon Blackblade",
        "You choose which ability the target creature gains as Gideon's first loyalty ability resolves, not as you activate it."
    );
    supported("Gideon Blackblade");
    let mut t = TestGame::new(2);
    let gideon = t.battlefield(P0, "Gideon Blackblade");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let from = t.asked().len();
    activate(&mut t, P0, gideon, 0, &[obj(bears)]);
    let asked_now = |t: &TestGame| {
        t.asked()[from..]
            .iter()
            .filter(|(_, d)| matches!(d, Decision::ChooseOption { .. }))
            .count()
    };
    assert_eq!(asked_now(&t), 0, "not chosen on activation");
    // Lifelink is the second option.
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.resolve_all();
    assert_eq!(asked_now(&t), 1, "chosen as it resolved");
    assert!(crate::r_p125_common::has(
        &t,
        bears,
        mtg_engine::keywords::KeywordKind::Lifelink
    ));
}

#[test]
fn gideon_champion_of_justice_counts_creatures_as_his_plus_one_resolves() {
    cr!("606.4", "608.2h");
    ruling!(
        "Gideon, Champion of Justice",
        "You put a loyalty counter on Gideon, Champion of Justice as part of the cost of activating the first ability. When that ability resolves, you count the number of creatures controlled by the target opponent and put that many additional loyalty counters on Gideon."
    );
    let mut t = TestGame::new(2);
    let gideon = t.battlefield(P0, "Gideon, Champion of Justice");
    t.battlefield(P1, "Grizzly Bears");
    activate(&mut t, P0, gideon, 0, &[pl(P1)]);
    assert_eq!(t.counters(gideon, "loyalty"), 5, "the cost");
    // Two more creatures arrive before it resolves.
    t.battlefield(P1, "Hill Giant");
    t.battlefield(P1, "Craw Wurm");
    t.resolve_all();
    assert_eq!(t.counters(gideon, "loyalty"), 8);
}

#[test]
fn kiora_additional_land_is_cumulative() {
    cr!("305.2", "305.2a");
    ruling!(
        "Kiora, the Crashing Wave",
        "The effect of the second ability is cumulative with other effects that let you play additional lands, such as the one from Rites of Flourishing."
    );
    supported("Kiora, the Crashing Wave");
    supported("Rites of Flourishing");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rites of Flourishing");
    let kiora = t.battlefield(P0, "Kiora, the Crashing Wave");
    activate_resolve(&mut t, P0, kiora, 1, &[]);
    for i in 0..3 {
        let land = t.hand(P0, "Forest");
        assert!(t.play_land(P0, land).is_ok(), "land {i}");
    }
    let fourth = t.hand(P0, "Forest");
    assert!(t.play_land(P0, fourth).is_err());
}

