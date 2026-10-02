//! Rulings batch P057 — firebreathing-style creatures and their neighbors: what a pump
//! ability does and doesn't change once blocks are declared (CR 509.1h, 506.4), negative
//! power and first-strike damage steps (CR 510.1a, 510.4), last known information for
//! abilities whose source has left (CR 608.2h, 113.7a), "can't be countered" (CR
//! 101.2), illegal targets (CR 608.2b), Auras (CR 303.4), mana-producing spells (CR 605),
//! and attack requirements in additional combats (CR 508.1d).

use crate::r_p057_common::*;
use crate::r_s01_common::{attack_with, supported, triggers_on_stack, with_subtype};
use crate::r_s02_common::{can_activate, destroy};
use crate::r_s03_common::to_blockers;
use crate::r_s04_common::add_mana;
use crate::r_s05_common::{enter, run_from};
use crate::r_s06_common::{activate_containing, attach_new, damage, has_kw};
use crate::r_s09_common::legal_attack;
use crate::r_s10_common::{attacking, blocking};
use crate::r_s12_common::MORPH;
use crate::r_s20_common::to_beginning_of_combat;
use crate::r_s21_common::legal_blocks;
use crate::r_s25_common::cast_new;
use mtg_engine::ability::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const SMALL: &str = "Grizzly Bears";

fn at_p1(id: ObjectId) -> (ObjectId, Entity) {
    (id, Entity::Player(P1))
}

#[test]
fn char_rumbler_negative_power_and_first_strike_steps() {
    cr!("510.4", "510.1a", "702.4b");
    ruling!(
        "Char-Rumbler",
        "A first strike damage step will be created if Char-Rumbler is in combat, even if its power is 0 or less."
    );
    ruling!(
        "Char-Rumbler",
        "Because Char-Rumber's power is normally negative, you'll have to give it at least +2/+0 before it'll deal damage."
    );
    ruling!(
        "Char-Rumbler",
        "If Char-Rumbler's power changes between the assignment of first-strike combat damage and the assignment of normal combat damage"
    );
    supported("Char-Rumbler");
    // Char-Rumbler (-1/3, double strike, "{R}: This creature gets +1/+0 until end of
    // turn.") attacks alone.
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Char-Rumbler");
    attack_with(&mut t, &[at_p1(c)]);
    t.advance_to(P0, Step::FirstStrikeDamage);
    assert_eq!(t.life(P1), 20, "no negative damage");
    // +2/+0 between the steps: 1 damage in the regular step.
    add_mana(&mut t, P0, ManaType::R, 2);
    for _ in 0..2 {
        activate_containing(&mut t, P0, c, "+1/+0").expect("activated");
        t.resolve_all();
    }
    assert_eq!(t.pt(c), (1, 3));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 19);
    // With +1/+0 only (power 0): no damage at all.
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Char-Rumbler");
    pump(&mut t, c, 1, 0);
    attack_with(&mut t, &[at_p1(c)]);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn akroma_cant_be_countered_but_extra_effects_happen_and_face_down_she_can() {
    cr!("101.2", "702.37c", "708.4");
    ruling!(
        "Akroma, Angel of Fury",
        "A spell or ability that counters spells can still target Akroma. When that spell or ability resolves, Akroma won't be countered, but any additional effects of the countering spell or ability will still happen."
    );
    ruling!(
        "Akroma, Angel of Fury",
        "If Akroma, Angel of Fury is cast face down, it can be countered."
    );
    supported("Akroma, Angel of Fury");
    supported("Dismiss");
    // Dismiss: "Counter target spell. Draw a card."
    let mut t = TestGame::new(2);
    let akroma = cast_new(&mut t, P0, "Akroma, Angel of Fury", &[]);
    let hand = t.hand_size(P1);
    cast_new(&mut t, P1, "Dismiss", &[Entity::Object(akroma)]);
    t.resolve();
    assert_eq!(t.hand_size(P1), hand + 1, "P1 drew a card");
    assert_eq!(t.stack_len(), 1, "Akroma is still on the stack");
    t.resolve_all();
    assert!(t.on_battlefield(akroma));
    // Face down (a 2/2 with no abilities), she can be countered.
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 3);
    let card = t.hand(P0, "Akroma, Angel of Fury");
    let spell = t.cast(P0, card).method(MORPH).go();
    cast_new(&mut t, P1, "Dismiss", &[Entity::Object(spell)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Akroma, Angel of Fury"));
}

#[test]
fn akroma_as_a_face_down_commander_pays_the_tax_and_deals_commander_damage() {
    cr!("903.8", "903.10a", "702.37c");
    ruling!(
        "Akroma, Angel of Fury",
        "In a Commander game where Akroma is your commander, you may cast it face down from the command zone by paying {3} plus {2} for each time it has been cast from your command zone."
    );
    let mut t = crate::r_s13_common::commander_game();
    let akroma = crate::r_s13_common::commander(&mut t, P0, "Akroma, Angel of Fury");
    t.g.players[0]
        .commander_casts
        .insert("Akroma, Angel of Fury".into(), 1);
    t.lands(P0, "Wastes", 4);
    assert!(t.cast(P0, akroma).method(MORPH).try_go().is_err(), "{{3}} plus {{2}}");
    t.lands(P0, "Wastes", 1);
    let spell = t.cast(P0, akroma).method(MORPH).go();
    t.resolve_all();
    let fd = t.g.current(spell);
    assert!(t.obj(fd).face_down && t.on_battlefield(fd));
    t.g.objects[fd.0 as usize].summoning_sick = false;
    // Combat damage dealt face down counts as commander damage.
    attack_with(&mut t, &[at_p1(fd)]);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 38);
    let dealt: u32 = t.g.player(P1).commander_damage.values().sum();
    assert_eq!(dealt, 2);
    // The cost to turn it face up is unaffected: {3}{R}{R}{R}.
    t.set_step(P0, Step::PostcombatMain);
    t.lands(P0, "Mountain", 3);
    t.lands(P0, "Wastes", 3);
    assert!(crate::r_s11_common::turn_face_up(&mut t, P0, fd));
    assert!(!t.obj_now(fd).face_down);
}

#[test]
fn gaining_menace_after_being_blocked_by_one_creature_doesnt_unblock() {
    cr!("509.1h", "702.111b", "506.4");
    ruling!(
        "Grim Draugr",
        "Activating Grim Draugr's ability after it's been blocked by a single creature won't cause Grim Draugr to become unblocked."
    );
    ruling!(
        "Kozilek's Shrieker",
        "If Kozilek's Shrieker gains menace after it's been legally blocked by one creature, it will remain blocked."
    );
    supported("Grim Draugr");
    supported("Kozilek's Shrieker");
    for (name, mana) in [
        ("Grim Draugr", "Snow-Covered Swamp"),
        ("Kozilek's Shrieker", "Wastes"),
    ] {
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, name);
        t.lands(P0, mana, 2);
        let b = t.battlefield(P1, "Hill Giant");
        to_blockers(&mut t, &[at_p1(c)], &[(b, c)]);
        activate_containing(&mut t, P0, c, "menace").expect("activated");
        t.resolve_all();
        assert!(has_kw(&t, c, KeywordKind::Menace));
        assert!(blocking(&t, b), "{name}: still blocked");
        t.advance_to(P0, Step::EndOfCombat);
        assert_eq!(t.life(P1), 20, "{name}: no damage to the player");
    }
}

#[test]
fn greater_stone_spirits_granted_ability_belongs_to_the_creatures_controller() {
    cr!("602.2");
    ruling!(
        "Greater Stone Spirit",
        "After Greater Stone Spirit’s activated ability is activated, the affected creature gains an activated ability that only that creature’s controller can activate."
    );
    supported("Greater Stone Spirit");
    // "{2}{R}: Until end of turn, target creature gets +0/+2 and gains "{R}: This creature
    // gets +1/+0 until end of turn.""
    let mut t = TestGame::new(2);
    let spirit = t.battlefield(P0, "Greater Stone Spirit");
    let bears = t.battlefield(P1, SMALL);
    t.lands(P0, "Mountain", 4);
    t.lands(P1, "Mountain", 1);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    activate_containing(&mut t, P0, spirit, "+0/+2").expect("activated");
    t.resolve_all();
    assert_eq!(t.pt(bears), (2, 4));
    assert!(!can_activate(&mut t, P0, bears));
    assert!(can_activate(&mut t, P1, bears));
    activate_containing(&mut t, P1, bears, "+1/+0").expect("P1 activates");
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 4));
}

#[test]
fn brood_keeper_triggers_on_entering_and_moving_auras_for_its_controller() {
    cr!("303.4", "603.2", "603.3a");
    ruling!(
        "Brood Keeper",
        "Brood Keeper’s ability triggers both whenever an Aura enters the battlefield attached to Brood Keeper and whenever an Aura on the battlefield attached to a different object becomes attached to Brood Keeper."
    );
    ruling!(
        "Brood Keeper",
        "If an Aura becomes attached to Brood Keeper that causes another player to gain control of it, that player will control the triggered ability that creates a Dragon."
    );
    supported("Brood Keeper");
    supported("Holy Strength");
    supported("Control Magic");
    // "Whenever an Aura becomes attached to this creature, create a 2/2 red Dragon creature
    // token with flying. ..."
    let mut t = TestGame::new(2);
    let keeper = t.battlefield(P0, "Brood Keeper");
    let bears = t.battlefield(P0, SMALL);
    cast_new(&mut t, P0, "Holy Strength", &[Entity::Object(keeper)]);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Dragon").len(), 1);
    // An Aura moving from another creature onto Brood Keeper.
    let aura = attach_new(&mut t, P0, "Holy Strength", bears);
    assert!(t.g.attach(aura, Entity::Object(keeper)));
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Dragon").len(), 2);
    // Control Magic: P1 gains control of it, and P1 gets the Dragon.
    let mut t = TestGame::new(2);
    let keeper = t.battlefield(P0, "Brood Keeper");
    t.set_step(P1, Step::PrecombatMain);
    cast_new(&mut t, P1, "Control Magic", &[Entity::Object(keeper)]);
    t.resolve_all();
    assert_eq!(t.obj_now(keeper).controller, P1);
    assert_eq!(with_subtype(&t, P1, "Dragon").len(), 1);
    assert_eq!(with_subtype(&t, P0, "Dragon").len(), 0);
}

#[test]
fn dina_triggers_once_per_life_gain_event() {
    cr!("119.9", "603.2");
    ruling!(
        "Dina, Soul Steeper",
        "Dina's first ability triggers only once for each life gain event, no matter how much life was gained."
    );
    supported("Dina, Soul Steeper");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dina, Soul Steeper");
    run_from(
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
    assert_eq!((t.life(P0), t.life(P1)), (25, 19));
}

#[test]
fn dragonrage_uses_the_stack_and_counts_attackers_as_it_resolves() {
    cr!("605.1a", "605.5a", "608.2h");
    ruling!(
        "Dragonrage",
        "Even though it can generate mana, Dragonrage isn't a mana ability. It uses the stack and can be responded to."
    );
    ruling!(
        "Dragonrage",
        "Only creatures that are attacking as Dragonrage resolves will count toward how much mana is generated"
    );
    supported("Dragonrage");
    // "Add {R} for each attacking creature you control. Until end of turn, attacking
    // creatures you control gain "{R}: This creature gets +1/+0 until end of turn.""
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, SMALL);
    let b = t.battlefield(P0, SMALL);
    let home = t.battlefield(P0, SMALL);
    attack_with(&mut t, &[at_p1(a), at_p1(b)]);
    cast_new(&mut t, P0, "Dragonrage", &[]);
    assert_eq!(t.stack_len(), 1, "on the stack");
    t.resolve();
    assert_eq!(t.g.player(P0).mana_pool.total(), 2);
    assert!(activate_containing(&mut t, P0, a, "+1/+0").is_ok());
    t.resolve_all();
    assert_eq!(t.pt(a), (3, 2));
    assert!(can_activate(&mut t, P0, b));
    assert!(!can_activate(&mut t, P0, home));
    // Cast before attackers are declared: no mana, and later attackers get nothing.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, SMALL);
    cast_new(&mut t, P0, "Dragonrage", &[]);
    t.resolve_all();
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    attack_with(&mut t, &[at_p1(a)]);
    add_mana(&mut t, P0, ManaType::R, 1);
    assert!(!can_activate(&mut t, P0, a));
}

#[test]
fn demonspine_whip_uses_last_known_attachment_and_its_bonus_stays() {
    cr!("608.2h", "611.2c", "113.7a");
    ruling!(
        "Demonspine Whip",
        "If Demonspine Whip is no longer on the battlefield by the time its first ability resolves, the game checks whether it was attached to a creature at the time it left the battlefield."
    );
    ruling!(
        "Demonspine Whip",
        "Once Demonspine Whip’s ability resolves and gives the +X/+0 bonus to a creature, that bonus stays with that creature for the rest of the turn"
    );
    supported("Demonspine Whip");
    // "{X}: Equipped creature gets +X/+0 until end of turn."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, SMALL);
    t.battlefield(P0, SMALL);
    let whip = attach_new(&mut t, P0, "Demonspine Whip", bears);
    t.lands(P0, "Wastes", 3);
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    activate_containing(&mut t, P0, whip, "+X/+0").expect("activated");
    destroy(&mut t, whip);
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 2), "attached as it left");
    // Unattached as it left: no creature gets it.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, SMALL);
    let whip = attach_new(&mut t, P0, "Demonspine Whip", bears);
    t.lands(P0, "Wastes", 3);
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    activate_containing(&mut t, P0, whip, "+X/+0").expect("activated");
    t.g.unattach(whip);
    destroy(&mut t, whip);
    t.resolve_all();
    assert_eq!(t.pt(bears), (2, 2));
    // Resolved: the bonus stays when the Whip moves and when it leaves.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, SMALL);
    let whip = attach_new(&mut t, P0, "Demonspine Whip", bears);
    t.lands(P0, "Wastes", 2);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    activate_containing(&mut t, P0, whip, "+X/+0").expect("activated");
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 2));
    let other2 = t.battlefield(P0, SMALL);
    assert!(t.g.attach(whip, Entity::Object(other2)));
    t.g.recompute();
    assert_eq!(t.pt(bears), (4, 2));
    assert_eq!(t.pt(other2), (2, 2));
    destroy(&mut t, whip);
    assert_eq!(t.pt(bears), (4, 2));
}

#[test]
fn goblin_fire_fiend_must_be_blocked_if_able() {
    cr!("509.1c");
    ruling!(
        "Goblin Fire Fiend",
        "If Goblin Fire Fiend is attacking, the defending player must assign at least one blocker to it"
    );
    supported("Goblin Fire Fiend");
    let mut t = TestGame::new(2);
    let fiend = t.battlefield(P0, "Goblin Fire Fiend");
    let other = t.battlefield(P0, SMALL);
    let b = t.battlefield(P1, SMALL);
    attack_with(&mut t, &[at_p1(fiend), at_p1(other)]);
    t.set_step(P0, Step::DeclareBlockers);
    assert!(!legal_blocks(&mut t, P1, &[]));
    assert!(!legal_blocks(&mut t, P1, &[(b, other)]));
    assert!(legal_blocks(&mut t, P1, &[(b, fiend)]));
}

#[test]
fn lathliss_triggers_for_each_dragon_entering_with_it() {
    cr!("603.6a");
    ruling!(
        "Lathliss, Dragon Queen",
        "If Lathliss enters at the same time as one or more other nontoken Dragons you control, its second ability will trigger once for each of those other Dragons."
    );
    supported("Lathliss, Dragon Queen");
    // "Whenever another nontoken Dragon you control enters, create a 5/5 red Dragon
    // creature token with flying."
    let mut t = TestGame::new(2);
    let cards = vec![
        t.hand(P0, "Lathliss, Dragon Queen"),
        t.hand(P0, "Shivan Dragon"),
        t.hand(P0, "Shivan Dragon"),
        t.hand(P0, SMALL),
    ];
    run_from(
        &mut t,
        P0,
        None,
        Effect::Move {
            what: Sel::AllTargets,
            to: Destination::battlefield().under_your_control(),
        },
        &cards.iter().map(|c| Entity::Object(*c)).collect::<Vec<_>>(),
    );
    assert_eq!(t.named_on_battlefield("Shivan Dragon").len(), 2);
    assert_eq!(triggers_on_stack(&t, "create a 5/5"), 2);
}

#[test]
fn scourge_of_valkas_damage_comes_from_the_dragon_even_after_it_left() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Scourge of Valkas",
        "If a Dragon entering the battlefield causes Scourge of Valkas's ability to trigger but leaves the battlefield before that ability resolves, that Dragon still deals damage."
    );
    supported("Scourge of Valkas");
    // "Whenever this creature or another Dragon you control enters, it deals X damage to
    // any target, where X is the number of Dragons you control."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Scourge of Valkas");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let shivan = enter(&mut t, P0, "Shivan Dragon");
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, shivan);
    t.resolve_all();
    assert_eq!(t.life(P1), 19, "X = 1 Dragon now; the Shivan Dragon deals it");
}

#[test]
fn incendiary_oracle_exiles_only_while_it_is_on_the_battlefield() {
    cr!("614.6", "614.12", "704.5g");
    ruling!(
        "Incendiary Oracle",
        "If a creature dealt damage by Incendiary Oracle would die at the same time as Incendiary Oracle, that creature is exiled instead."
    );
    ruling!(
        "Incendiary Oracle",
        "If a creature dealt damage by Incendiary Oracle would die after Incendiary Oracle has left the battlefield, that creature dies and isn't exiled instead."
    );
    supported("Incendiary Oracle");
    // "If a creature dealt damage by this creature this turn would die, exile it instead."
    // (2/2) It blocks a Grizzly Bears: both die at once; the Bears are exiled.
    let mut t = TestGame::new(2);
    let oracle = t.battlefield(P1, "Incendiary Oracle");
    let bears = t.battlefield(P0, SMALL);
    to_blockers(&mut t, &[at_p1(bears)], &[(oracle, bears)]);
    t.advance_to(P0, Step::EndOfCombat);
    assert!(t.in_graveyard(P1, "Incendiary Oracle"));
    assert!(t.in_exile(SMALL));
    // It damages a Hill Giant, then leaves; the Giant then dies: graveyard.
    let mut t = TestGame::new(2);
    let oracle = t.battlefield(P1, "Incendiary Oracle");
    let giant = t.battlefield(P0, "Hill Giant");
    damage(&mut t, oracle, 2, giant);
    destroy(&mut t, oracle);
    destroy(&mut t, giant);
    assert!(t.in_graveyard(P0, "Hill Giant"));
}

#[test]
fn ill_tempered_loner_uses_last_known_lifelink() {
    cr!("608.2h", "702.15b", "113.7a");
    ruling!(
        "Ill-Tempered Loner // Howlpack Avenger",
        "If the damage causes this creature to die before the triggered ability resolves, use last known information"
    );
    supported("Ill-Tempered Loner // Howlpack Avenger");
    // "Whenever this creature is dealt damage, it deals that much damage to any target."
    // (3/3) With lifelink, it's dealt 3 damage and dies.
    let mut t = TestGame::new(2);
    let loner = t.battlefield(P0, "Ill-Tempered Loner // Howlpack Avenger");
    let src = t.battlefield(P1, "Hill Giant");
    run_from(
        &mut t,
        P0,
        None,
        Effect::Modify {
            what: Sel::All(Filter::Objects(vec![loner])),
            mods: vec![Modification::AddKeyword(Keyword::new(KeywordKind::Lifelink))],
            duration: Duration::EndOfTurn,
        },
        &[],
    );
    t.answer_targets(P0, &[Entity::Player(P1)]);
    damage(&mut t, src, 3, loner);
    assert!(!t.on_battlefield(loner));
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (23, 17));
}

#[test]
fn an_illegal_target_stops_dragon_mantle_and_dranas_ability() {
    cr!("608.2b", "608.3b");
    ruling!(
        "Dragon Mantle",
        "If the target creature is an illegal target by the time Dragon Mantle tries to resolve, it doesn't resolve."
    );
    ruling!(
        "Drana, Kalastria Bloodchief",
        "If the targeted creature is an illegal target by the time the ability resolves, the ability doesn't resolve. Drana won't get the bonus."
    );
    supported("Dragon Mantle");
    supported("Drana, Kalastria Bloodchief");
    // Dragon Mantle: "Enchant creature / When this Aura enters, draw a card."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, SMALL);
    cast_new(&mut t, P0, "Dragon Mantle", &[Entity::Object(bears)]);
    destroy(&mut t, bears);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Dragon Mantle"));
    assert_eq!(t.hand_size(P0), hand);
    // Drana (4/4): "{X}{B}{B}: Target creature gets -0/-X until end of turn and Drana gets
    // +X/+0 until end of turn."
    let mut t = TestGame::new(2);
    let drana = t.battlefield(P0, "Drana, Kalastria Bloodchief");
    let bears = t.battlefield(P1, SMALL);
    t.lands(P0, "Swamp", 3);
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    t.answer_targets(P0, &[Entity::Object(bears)]);
    activate_containing(&mut t, P0, drana, "-0/-X").expect("activated");
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.pt(drana), (4, 4));
}

#[test]
fn warmonger_hellkite_requirements_and_bonus() {
    cr!("508.1d", "506.1", "611.2c");
    ruling!(
        "Warmonger Hellkite",
        "If there are multiple combat phases in a turn, creatures must attack during each combat phase in which they're able to."
    );
    ruling!(
        "Warmonger Hellkite",
        "Only creatures that are attacking when the last ability resolves will get +1/+0."
    );
    supported("Warmonger Hellkite");
    // "All creatures attack each combat if able. / {1}{R}: Attacking creatures get +1/+0
    // until end of turn."
    let mut t = TestGame::new(2);
    let kite = t.battlefield(P1, "Warmonger Hellkite");
    let angel = t.battlefield(P0, "Serra Angel"); // 4/4 flying, vigilance
    let bears = t.battlefield(P0, SMALL);
    t.lands(P1, "Mountain", 4);
    // Activated before attackers: nobody gets the bonus, including later attackers.
    activate_containing(&mut t, P1, kite, "Attacking creatures").expect("activated");
    t.resolve_all();
    to_beginning_of_combat(&mut t, P0);
    assert!(!legal_attack(&mut t, &[]));
    assert!(!legal_attack(&mut t, &[at_p1(angel)]));
    attack_with(&mut t, &[at_p1(angel), at_p1(bears)]);
    assert_eq!(t.pt(angel), (4, 4));
    // Activated now: attacking creatures get +1/+0, nonattacking ones don't.
    activate_containing(&mut t, P1, kite, "Attacking creatures").expect("activated");
    t.resolve_all();
    assert_eq!((t.pt(angel), t.pt(bears)), ((5, 4), (3, 2)));
    assert_eq!(t.pt(kite), (5, 5));
    t.advance_to(P0, Step::EndOfCombat);
    // A second combat phase: the Angel (vigilance, untapped) must attack again.
    to_beginning_of_combat(&mut t, P0);
    assert!(!legal_attack(&mut t, &[]));
    assert!(legal_attack(&mut t, &[at_p1(angel)]));
}

#[test]
fn betrothed_of_fire_on_an_opponents_creature_cant_sacrifice_it() {
    cr!("602.2", "118.3", "701.21a");
    ruling!(
        "Betrothed of Fire",
        "If you control Betrothed of Fire, but it’s attached to a creature you don’t control, no one can activate the last ability."
    );
    supported("Betrothed of Fire");
    // "Sacrifice enchanted creature: Creatures you control get +2/+0 until end of turn."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, SMALL);
    let aura = attach_new(&mut t, P0, "Betrothed of Fire", bears);
    assert!(activate_containing(&mut t, P0, aura, "Sacrifice enchanted creature").is_err());
    assert!(activate_containing(&mut t, P1, aura, "Sacrifice enchanted creature").is_err());
    assert!(t.on_battlefield(bears));
    // On P0's own creature, P0 can.
    let mine = t.battlefield(P0, SMALL);
    let aura = attach_new(&mut t, P0, "Betrothed of Fire", mine);
    assert!(activate_containing(&mut t, P0, aura, "Sacrifice enchanted creature").is_ok());
}

#[test]
fn pia_nalaars_cant_block_doesnt_undo_a_block() {
    cr!("509.1h", "506.4");
    ruling!(
        "Pia Nalaar",
        "Once a creature has blocked an attacking creature, activating Pia Nalaar's last ability won't cause the attacking creature to become unblocked."
    );
    supported("Pia Nalaar");
    // "{1}, Sacrifice an artifact: Target creature can't block this turn."
    let mut t = TestGame::new(2);
    let pia = t.battlefield(P0, "Pia Nalaar");
    let art = t.battlefield(P0, "Ornithopter");
    let bears = t.battlefield(P0, SMALL);
    let blocker = t.battlefield(P1, SMALL);
    t.lands(P0, "Wastes", 1);
    to_blockers(&mut t, &[at_p1(bears)], &[(blocker, bears)]);
    t.answer_choose(P0, &[Entity::Object(art)]);
    t.answer_targets(P0, &[Entity::Object(blocker)]);
    activate_containing(&mut t, P0, pia, "can't block").expect("activated");
    t.resolve_all();
    assert!(blocking(&t, blocker));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn the_first_iroan_games_gold_token() {
    cr!("111.10c", "714.2b");
    ruling!(
        "The First Iroan Games",
        "A Gold token is a colorless Gold artifact token with \"Sacrifice this artifact: Add one mana of any color.\""
    );
    supported("The First Iroan Games");
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P0, "The First Iroan Games");
    crate::r_s19_common::add_lore(&mut t, saga, 4);
    t.resolve_all();
    let gold = with_subtype(&t, P0, "Gold");
    assert_eq!(gold.len(), 1);
    let o = t.obj_now(gold[0]);
    assert!(o.is_token() && o.is(CardType::Artifact) && o.chars.colors == ColorSet::NONE);
    assert!(crate::r_s20_common::tap_for_mana(
        &mut t,
        P0,
        gold[0],
        "Sacrifice this"
    ));
    assert!(!t.on_battlefield(gold[0]));
}
