//! Rulings batch P058 — firebreathing-style pump abilities: what an effect affects and
//! what a value counts is determined as the ability resolves (CR 608.2h, 611.2c), mana
//! empties between steps (CR 106.4), "activate only once each turn" (CR 602.5b), cast
//! triggers (CR 603.2), dividing damage (CR 601.2d), the legend rule (CR 704.5j), and
//! Flagbearers.

use crate::r_p058_common::*;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s02_common::{can_activate, destroy};
use crate::r_s06_common::{activate_containing, attach_new, damage, give_control};
use crate::r_s24_common::pool;
use crate::r_s25_common::targets_of;
use mtg_engine::ability::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn pump(t: &mut TestGame, id: ObjectId, p: i32, tough: i32) {
    crate::r_s05_common::run_from(
        t,
        P0,
        None,
        Effect::Modify {
            what: Sel::All(Filter::Objects(vec![t.g.current(id)])),
            mods: vec![Modification::ModifyPT(Value::c(p), Value::c(tough))],
            duration: Duration::EndOfTurn,
        },
        &[],
    );
}

#[test]
fn creatures_you_control_get_the_bonus_only_as_it_resolves() {
    cr!("611.2c", "608.2h");
    ruling!(
        "Sunhome Guildmage",
        "Only creatures you control when Sunhome Guildmage’s first ability resolves will get +1/+0. Creatures that come under your control later in the turn will not."
    );
    supported("Sunhome Guildmage");
    let mut t = TestGame::new(2);
    mana(&mut t, P0, 2);
    let mage = t.battlefield(P0, "Sunhome Guildmage");
    let bears = t.battlefield(P0, "Grizzly Bears");
    activate_containing(&mut t, P0, mage, "+1/+0").unwrap();
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 2));
    assert_eq!(t.pt(mage), (3, 2));
    let later = t.enter(P0, "Gray Ogre");
    let stolen = t.battlefield(P1, "Grizzly Bears");
    give_control(&mut t, stolen, P0);
    assert_eq!(t.pt(later), (2, 2));
    assert_eq!(t.pt(stolen), (2, 2));

    ruling!(
        "Artificer's Dragon",
        "The first activated ability affects only artifact creatures you control at the time it resolves. Any artifact creatures that come under your control later in the turn won't be affected."
    );
    supported("Artificer's Dragon");
    let mut t = TestGame::new(2);
    mana(&mut t, P0, 2);
    let dragon = t.battlefield(P0, "Artificer's Dragon");
    let thopter = t.battlefield(P0, "Ornithopter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    activate_containing(&mut t, P0, dragon, "+1/+0").unwrap();
    t.resolve_all();
    assert_eq!(t.pt(thopter), (1, 2));
    assert_eq!(t.pt(dragon), (5, 4));
    assert_eq!(t.pt(bears), (2, 2), "not an artifact");
    let later = t.enter(P0, "Ornithopter");
    assert_eq!(t.pt(later), (0, 2));
}

#[test]
fn dragonrage_mana_empties_between_steps() {
    cr!("106.4", "500.4");
    ruling!(
        "Dragonrage",
        "Remember that unused mana is lost at the end of each step and phase."
    );
    supported("Dragonrage");
    let mut t = TestGame::new(2);
    mana(&mut t, P0, 1);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Gray Ogre");
    attack_with(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    let spell = t.hand(P0, "Dragonrage");
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(pool(&t, P0, ManaType::R), 2);
    t.advance_to(P0, Step::DeclareBlockers);
    assert_eq!(pool(&t, P0, ManaType::R), 0);
    // The attackers have the firebreathing ability until end of turn.
    assert!(t
        .obj_now(a)
        .chars
        .abilities
        .iter()
        .any(|x| x.text.contains("+1/+0")));
}

#[test]
fn akki_avalanchers_once_each_turn_after_a_control_change() {
    cr!("602.5b");
    ruling!(
        "Akki Avalanchers",
        "The \"activate only once each turn\" restriction applies even if the creature changes controllers."
    );
    supported("Akki Avalanchers");
    let mut t = TestGame::new(2);
    let akki = t.battlefield(P0, "Akki Avalanchers");
    let land = t.battlefield(P0, "Mountain");
    t.battlefield(P1, "Mountain");
    t.answer_choose(P0, &[Entity::Object(land)]);
    activate_containing(&mut t, P0, akki, "+2/+0").unwrap();
    t.resolve_all();
    assert_eq!(t.pt(akki), (3, 1));
    give_control(&mut t, akki, P1);
    assert!(!can_activate(&mut t, P1, akki), "already activated this turn");
    // Next turn, its new controller can.
    t.advance_to(P1, Step::PrecombatMain);
    assert!(can_activate(&mut t, P1, akki));
}

#[test]
fn prossh_cast_trigger() {
    cr!("603.2", "601.2h", "603.3");
    ruling!(
        "Prossh, Skyraider of Kher",
        "The first ability triggers when you cast Prossh, not when it enters the battlefield. This means that it will resolve before Prossh, so any Kobold tokens it creates will be put onto the battlefield before Prossh. If Prossh enters the battlefield without being cast, the ability will not trigger."
    );
    supported("Prossh, Skyraider of Kher");
    let mut t = TestGame::new(2);
    mana(&mut t, P0, 1);
    t.lands(P0, "Wastes", 2);
    let card = t.hand(P0, "Prossh, Skyraider of Kher");
    let spell = t.cast(P0, card).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(t.zone(spell), Zone::Stack, "Prossh is still a spell");
    let kobolds = |t: &TestGame| {
        t.g.permanents()
            .filter(|o| o.chars.name == "Kobolds of Kher Keep")
            .count()
    };
    assert_eq!(kobolds(&t), 6);
    t.resolve_all();
    assert!(t.on_battlefield(spell));
    // Entering without being cast: no trigger.
    let mut t = TestGame::new(2);
    t.enter(P0, "Prossh, Skyraider of Kher");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(kobolds(&t), 0);
}

#[test]
fn prossh_counts_the_commander_tax() {
    cr!("903.8", "601.2f", "603.4");
    ruling!(
        "Prossh, Skyraider of Kher",
        "The amount of mana you spent to cast this creature is usually equal to its mana value. However, you also include any additional costs you pay, including the commander tax."
    );
    let mut t = crate::r_s13_common::commander_game();
    let prossh = crate::r_s13_common::commander(&mut t, P0, "Prossh, Skyraider of Kher");
    let key = mtg_engine::kw::partner::commander_key(&t.g, prossh);
    t.g.players[0].commander_casts.insert(key, 1);
    mana(&mut t, P0, 1);
    t.lands(P0, "Wastes", 4);
    t.cast(P0, prossh).go();
    t.resolve_all();
    let kobolds = t
        .g
        .permanents()
        .filter(|o| o.chars.name == "Kobolds of Kher Keep")
        .count();
    assert_eq!(kobolds, 8, "6 + the {{2}} tax");
}

#[test]
fn counts_are_made_as_the_ability_resolves() {
    cr!("608.2h", "611.2c");
    ruling!(
        "Hellkite Igniter",
        "The number of artifacts you control is counted when the ability resolves."
    );
    supported("Hellkite Igniter");
    let mut t = TestGame::new(2);
    mana(&mut t, P0, 2);
    let hellkite = t.battlefield(P0, "Hellkite Igniter");
    t.battlefield(P0, "Sol Ring");
    activate_containing(&mut t, P0, hellkite, "+X/+0").unwrap();
    t.battlefield(P0, "Ornithopter");
    t.resolve_all();
    assert_eq!(t.pt(hellkite), (7, 5));
    // Later artifacts don't change it.
    t.battlefield(P0, "Sol Ring");
    assert_eq!(t.pt(hellkite), (7, 5));

    ruling!(
        "Foundry Champion",
        "The number of creatures you control is counted when Foundry Champion's triggered ability resolves. Foundry Champion will count itself if it's still on the battlefield under your control at that time."
    );
    supported("Foundry Champion");
    for leaves in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Grizzly Bears");
        t.answer_targets(P0, &[Entity::Player(P1)]);
        let champ = t.enter(P0, "Foundry Champion");
        t.settle();
        t.battlefield(P0, "Gray Ogre");
        if leaves {
            destroy(&mut t, champ);
        }
        t.resolve_all();
        assert_eq!(t.life(P1), if leaves { 18 } else { 17 });
    }

    ruling!(
        "Armored Armadillo",
        "The value of X is determined as Armored Armadillo's activated ability resolves. It won't change later in the turn if other effects modify Armored Armadillo's toughness."
    );
    supported("Armored Armadillo");
    let mut t = TestGame::new(2);
    mana(&mut t, P0, 2);
    let dillo = t.battlefield(P0, "Armored Armadillo");
    activate_containing(&mut t, P0, dillo, "+X/+0").unwrap();
    t.resolve_all();
    assert_eq!(t.pt(dillo), (4, 4));
    pump(&mut t, dillo, 0, 3);
    assert_eq!(t.pt(dillo), (4, 7));

    ruling!(
        "Feral Animist",
        "You don't choose the value of X. The bonus Feral Animist gets is based on its power when the ability resolves. For example, if you activate the ability twice, the first ability will give it +2/+0 and the second one will give it +4/+0."
    );
    supported("Feral Animist");
    let mut t = TestGame::new(2);
    mana(&mut t, P0, 2);
    let animist = t.battlefield(P0, "Feral Animist");
    activate_containing(&mut t, P0, animist, "+X/+0").unwrap();
    activate_containing(&mut t, P0, animist, "+X/+0").unwrap();
    t.resolve();
    assert_eq!(t.pt(animist), (4, 1));
    t.resolve_all();
    assert_eq!(t.pt(animist), (8, 1));
}

#[test]
fn demonspine_whip_pumps_the_creature_equipped_as_it_resolves() {
    cr!("608.2h", "301.5");
    ruling!(
        "Demonspine Whip",
        "When the first ability resolves, the +X/+0 bonus is given to the creature that Demonspine Whip is attached to at that time, regardless of what creature (if any) it was attached to when the ability was activated. If Demonspine Whip is on the battlefield but not attached to a creature at this time, no creature gets the bonus."
    );
    supported("Demonspine Whip");
    let mut t = TestGame::new(2);
    mana(&mut t, P0, 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let ogre = t.battlefield(P0, "Gray Ogre");
    let whip = attach_new(&mut t, P0, "Demonspine Whip", bears);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    activate_containing(&mut t, P0, whip, "+X/+0").unwrap();
    assert!(t.g.attach(whip, Entity::Object(ogre)));
    t.resolve_all();
    assert_eq!(t.pt(ogre), (4, 2));
    assert_eq!(t.pt(bears), (2, 2));
    // Unattached: nobody.
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    activate_containing(&mut t, P0, whip, "+X/+0").unwrap();
    t.g.unattach(whip);
    t.resolve_all();
    assert_eq!(t.pt(ogre), (4, 2));
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn drana_x_can_exceed_toughness_and_target_herself() {
    cr!("107.3", "115.1");
    ruling!(
        "Drana, Kalastria Bloodchief",
        "The value of X may exceed the targeted creature's toughness. Drana's bonus is based on the value of X, regardless of what the targeted creature's toughness was."
    );
    ruling!(
        "Drana, Kalastria Bloodchief",
        "You may target Drana itself with its ability. For example, if you do and X is 3, Drana will become 7/1 until end of turn."
    );
    supported("Drana, Kalastria Bloodchief");
    let mut t = TestGame::new(2);
    mana(&mut t, P0, 2);
    let drana = t.battlefield(P0, "Drana, Kalastria Bloodchief");
    let thopter = t.battlefield(P1, "Ornithopter");
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.answer_targets(P0, &[Entity::Object(thopter)]);
    activate_containing(&mut t, P0, drana, "-0/-X").unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Ornithopter"));
    assert_eq!(t.pt(drana), (7, 4));
    // Drana herself.
    let mut t = TestGame::new(2);
    mana(&mut t, P0, 2);
    let drana = t.battlefield(P0, "Drana, Kalastria Bloodchief");
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.answer_targets(P0, &[Entity::Object(drana)]);
    activate_containing(&mut t, P0, drana, "-0/-X").unwrap();
    t.resolve_all();
    assert_eq!(t.pt(drana), (7, 1));
}

#[test]
fn dina_soul_steeper() {
    cr!("119.9", "608.2h", "113.7a");
    ruling!(
        "Dina, Soul Steeper",
        "When Dina's first ability resolves, each opponent loses only 1 life, no matter how much life was gained."
    );
    ruling!(
        "Dina, Soul Steeper",
        "X is the power the creature had when it was on the battlefield, not the power it has in the graveyard."
    );
    supported("Dina, Soul Steeper");
    let mut t = TestGame::new(2);
    mana(&mut t, P0, 1);
    let dina = t.battlefield(P0, "Dina, Soul Steeper");
    crate::r_s05_common::run_from(
        &mut t,
        P0,
        None,
        Effect::GainLife {
            who: PlayerRef::You,
            n: Value::c(5),
        },
        &[],
    );
    t.resolve_all();
    assert_eq!(t.life(P0), 25);
    assert_eq!(t.life(P1), 19);
    // The sacrificed Ogre was 5/2 on the battlefield (2/2 in the graveyard).
    let ogre = t.battlefield(P0, "Gray Ogre");
    pump(&mut t, ogre, 3, 0);
    t.answer_choose(P0, &[Entity::Object(ogre)]);
    activate_containing(&mut t, P0, dina, "+X/+0").unwrap();
    assert!(t.in_graveyard(P0, "Gray Ogre"));
    t.resolve_all();
    assert_eq!(t.pt(dina), (6, 3));
}

#[test]
fn char_rumbler_negative_power_deals_no_damage() {
    cr!("510.1a", "107.1b");
    ruling!(
        "Char-Rumbler",
        "Yes, Char-Rumbler's printed power is -1. While its power is -1 or 0, it simply deals no combat damage."
    );
    supported("Char-Rumbler");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Char-Rumbler");
    assert_eq!(t.pt(c), (-1, 3));
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(c, Entity::Player(P1))]);
    crate::r_s01_common::block_and_finish(&mut t, P1, &[(bears, c)]);
    assert_eq!(t.obj_now(bears).damage, 0);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn icehide_troll_can_activate_while_tapped() {
    cr!("602.1", "701.26a");
    ruling!(
        "Icehide Troll",
        "You can activate Icehide Troll's last ability even if it's already tapped."
    );
    supported("Icehide Troll");
    let mut t = TestGame::new(2);
    t.lands(P0, "Snow-Covered Forest", 2);
    let troll = t.battlefield(P0, "Icehide Troll");
    t.g.objects[troll.0 as usize].tapped = true;
    assert!(can_activate(&mut t, P0, troll));
    activate_containing(&mut t, P0, troll, "indestructible").unwrap();
    t.resolve_all();
    assert_eq!(t.pt(troll), (4, 3));
    assert!(t.obj_now(troll).has_keyword(KeywordKind::Indestructible));
    assert!(t.obj_now(troll).tapped);
}

#[test]
fn chartooth_cougar_mountaincycling() {
    cr!("702.29e", "701.23b");
    ruling!(
        "Chartooth Cougar",
        "You can choose to find any card with the Mountain land type, including nonbasic lands. You can also choose not to find a card, even if there is a Mountain card in your library."
    );
    supported("Chartooth Cougar");
    for pick in [true, false] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Wastes", 2);
        let taiga = t.library_top(P0, "Taiga");
        let cougar = t.hand(P0, "Chartooth Cougar");
        let from = t.asked().len();
        t.answer_choose(
            P0,
            &if pick {
                vec![Entity::Object(taiga)]
            } else {
                vec![]
            },
        );
        crate::r_s04_common::cycle(&mut t, P0, cougar, 0).unwrap();
        t.resolve_all();
        let offered = t.asked()[from..].iter().any(|(_, d)| {
            matches!(d, Decision::ChooseEntities { candidates, .. }
                if candidates.contains(&Entity::Object(taiga)))
        });
        assert!(offered, "Taiga is a Mountain");
        assert_eq!(t.in_hand(P0, "Taiga"), pick);
        assert!(t.in_graveyard(P0, "Chartooth Cougar"));
    }
}

#[test]
fn inferno_titan_divides_as_the_ability_is_put_on_the_stack() {
    cr!("601.2d", "603.3d");
    ruling!(
        "Inferno Titan",
        "You divide the damage as you put Inferno Titan's triggered ability on the stack, not as it resolves. Each target must be assigned at least 1 damage."
    );
    supported("Inferno Titan");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let ogre = t.battlefield(P1, "Gray Ogre");
    t.answer_targets(P0, &[Entity::Object(bears), Entity::Object(ogre)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![2, 1]));
    let from = t.asked().len();
    t.enter(P0, "Inferno Titan");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let divide = t.asked()[from..]
        .iter()
        .find_map(|(_, d)| match d {
            Decision::Divide { min_each, .. } => Some(*min_each),
            _ => None,
        })
        .expect("divided as it was put on the stack");
    assert_eq!(divide, 1);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.obj_now(ogre).damage, 1);
}

#[test]
fn inner_flame_igniter_counts_its_own_resolutions() {
    cr!("608.2h", "707.10", "113.7a");
    ruling!(
        "Inner-Flame Igniter",
        "You get the bonus only the third time the ability resolves. You won’t get the bonus the fourth, fifth, sixth, or any subsequent times."
    );
    ruling!(
        "Inner-Flame Igniter",
        "When the ability resolves, it counts the number of times that same ability from that this creature has already resolved that turn. It doesn’t matter who controlled the creature or the previous abilities when they resolved. A copy of this ability (created by Rings of Brighthearth, for example) will count toward the total. Abilities from other creatures with the same name don’t count towards the total."
    );
    supported("Inner-Flame Igniter");
    let fs = |t: &TestGame, id: ObjectId| t.obj_now(id).has_keyword(KeywordKind::FirstStrike);
    let mut t = TestGame::new(2);
    mana(&mut t, P0, 5);
    let a = t.battlefield(P0, "Inner-Flame Igniter");
    let b = t.battlefield(P0, "Inner-Flame Igniter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    for src in [a, a, b] {
        activate_containing(&mut t, P0, src, "+1/+0").unwrap();
        t.resolve_all();
    }
    assert_eq!(t.pt(bears), (5, 2));
    assert!(!fs(&t, bears), "other Igniters' resolutions don't count");
    // A's third resolution, after a control change: first strike.
    give_control(&mut t, a, P1);
    give_control(&mut t, a, P0);
    activate_containing(&mut t, P0, a, "+1/+0").unwrap();
    t.resolve_all();
    assert!(fs(&t, bears));
    // A fourth: a creature that came later doesn't get first strike.
    let later = t.battlefield(P0, "Gray Ogre");
    activate_containing(&mut t, P0, a, "+1/+0").unwrap();
    t.resolve_all();
    assert_eq!(t.pt(later), (3, 2));
    assert!(!fs(&t, later));
    // Copies count: with Illusionist's Bracers ("Whenever an ability of equipped creature
    // is activated, if it isn't a mana ability, copy that ability."), the second
    // activation's copy is the third resolution.
    let mut t = TestGame::new(2);
    mana(&mut t, P0, 3);
    let a = t.battlefield(P0, "Inner-Flame Igniter");
    attach_new(&mut t, P0, "Illusionist's Bracers", a);
    activate_containing(&mut t, P0, a, "+1/+0").unwrap();
    t.resolve_all();
    assert!(!fs(&t, a));
    assert_eq!(t.pt(a), (4, 2));
    activate_containing(&mut t, P0, a, "+1/+0").unwrap();
    t.resolve_all();
    assert!(fs(&t, a));
}

#[test]
fn ill_tempered_loner_trigger_works_when_it_dies() {
    cr!("603.10a", "603.2");
    ruling!(
        "Ill-Tempered Loner // Howlpack Avenger",
        "The triggered ability on each face still works even if that damage causes this creature to die."
    );
    supported("Ill-Tempered Loner // Howlpack Avenger");
    let mut t = TestGame::new(2);
    let loner = t.battlefield(P0, "Ill-Tempered Loner // Howlpack Avenger");
    let src = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    damage(&mut t, src, 3, loner);
    assert!(!t.on_battlefield(loner), "it died");
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn fire_nation_cadets_needs_the_lesson_as_it_attacks() {
    cr!("603.2", "611.3a");
    ruling!(
        "Fire Nation Cadets",
        "You must have a Lesson card in your graveyard while you attack with Fire Nation Cadets in order for the firebending 2 ability to trigger."
    );
    supported("Fire Nation Cadets");
    for before in [true, false] {
        let mut t = TestGame::new(2);
        let cadets = t.battlefield(P0, "Fire Nation Cadets");
        if before {
            t.graveyard(P0, "Environmental Sciences");
        }
        attack_with(&mut t, &[(cadets, Entity::Player(P1))]);
        if !before {
            t.graveyard(P0, "Environmental Sciences");
            t.g.recompute();
            assert!(t
                .obj_now(cadets)
                .has_keyword(KeywordKind::Firebending));
        }
        t.resolve_all();
        assert_eq!(pool(&t, P0, ManaType::R), if before { 2 } else { 0 });
    }
}

#[test]
fn pia_nalaar_and_pia_and_kiran_nalaar_have_different_names() {
    cr!("704.5j", "201.2");
    ruling!(
        "Pia Nalaar",
        "The “legend rule” cares about legendary permanents with the exact same English name."
    );
    supported("Pia Nalaar");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Pia Nalaar");
    let b = t.battlefield(P0, "Pia and Kiran Nalaar");
    t.settle();
    assert!(t.on_battlefield(a) && t.on_battlefield(b));
}

#[test]
fn flagbearers() {
    cr!("601.2c", "603.3d", "115.1");
    ruling!(
        "Coalition Honor Guard",
        "Triggered abilities (written with “when,” “whenever,” or “at”) don’t have to target a Flagbearer."
    );
    supported("Coalition Honor Guard");
    // Flametongue Kavu's trigger ("When this creature enters, it deals 4 damage to
    // target creature.") may target the Bears; a Shock must target a Flagbearer.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Coalition Honor Guard");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.enter(P0, "Flametongue Kavu");
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    mana(&mut t, P0, 1);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let shock = t.hand(P0, "Shock");
    let shock = t.cast(P0, shock).target(bears).go();
    assert_ne!(targets_of(&t, shock), vec![Entity::Object(bears)]);
    ruling!(
        "Coalition Honor Guard",
        "Any player’s Flagbearer may be targeted. For example, if each player controls a Coalition Honor Guard"
    );
    let mut t = TestGame::new(2);
    mana(&mut t, P0, 1);
    t.battlefield(P1, "Coalition Honor Guard");
    let mine = t.battlefield(P0, "Coalition Honor Guard");
    t.battlefield(P1, "Grizzly Bears");
    let shock = t.hand(P0, "Shock");
    let shock = t.cast(P0, shock).target(mine).go();
    assert_eq!(targets_of(&t, shock), vec![Entity::Object(mine)]);

    ruling!(
        "Coalition Flag",
        "If this card is ever on a creature you don’t control, it is put into the graveyard as a State-Based Action."
    );
    supported("Coalition Flag");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let flag = attach_new(&mut t, P0, "Coalition Flag", bears);
    t.settle();
    assert!(t.on_battlefield(flag));
    give_control(&mut t, bears, P1);
    assert!(t.in_graveyard(P0, "Coalition Flag"));
}
