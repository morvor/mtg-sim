//! Rulings batch P182 — the other abilities of the batch's tutors: Bilbo's life-gain
//! replacement (CR 614.1a, 119.10), Zilortha's damage assignment (CR 510.1c), Samut's
//! divided damage (CR 601.2d, 608.2b), sacrifice costs (CR 602.2b), Shrines, combat
//! requirements and restrictions (CR 508.1c, 509.1b), leaving a multiplayer game
//! (CR 800.4a), and timing permissions checked as a spell is proposed (CR 601.3).

use crate::r_p182_common::*;
use crate::r_s11_common::triggered_from;
use mtg_engine::ability::Modification;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const BILBO: &str = "Bilbo, Birthday Celebrant";

/// P0's `attackers` attack P1 with the given blocks; combat damage is dealt.
fn fight(t: &mut TestGame, attackers: &[ObjectId], blocks: &[(ObjectId, ObjectId)]) {
    t.set_step(P0, Step::BeginningOfCombat);
    let a: Vec<_> = attackers.iter().map(|a| (*a, pl(P1))).collect();
    t.attack(&a, blocks);
}

fn damage(t: &TestGame, id: ObjectId) -> u32 {
    t.g.obj(t.g.current(id)).damage
}

#[test]
fn bilbo_adds_one_to_each_life_gain_event() {
    cr!("614.1a", "119.10", "702.15b");
    ruling!(
        "Bilbo, Birthday Celebrant",
        "Bilbo, Birthday Celebrant's first ability applies just once to each life-gaining event, no matter how much life is gained."
    );
    ruling!(
        "Bilbo, Birthday Celebrant",
        "Each creature with lifelink dealing combat damage causes a separate life-gaining event."
    );
    // "For each": one event. Nissa Revane gains 2 life for each of two Elves: 4 + 1.
    let mut t = TestGame::new(2);
    t.battlefield(P0, BILBO);
    t.battlefield(P0, "Elvish Mystic");
    t.battlefield(P0, "Elvish Mystic");
    let nissa = t.battlefield(P0, "Nissa Revane");
    activate_resolve(&mut t, P0, nissa, "gain 2 life", "", &[]);
    assert_eq!(t.life(P0), 25);
    // Two lifelinkers unblocked: two events.
    let mut t = TestGame::new(2);
    t.battlefield(P0, BILBO);
    let a = t.battlefield(P0, "Vampire Nighthawk");
    let b = t.battlefield(P0, "Vampire Nighthawk");
    fight(&mut t, &[a, b], &[]);
    assert_eq!(t.life(P0), 26);
    // One lifelinker with trample dealing damage to a blocker and the player: one event.
    let mut t = TestGame::new(2);
    t.battlefield(P0, BILBO);
    let a = t.battlefield(P0, "Vampire Nighthawk");
    crate::r_s06_common::attach_new(&mut t, P0, "Rancor", obj(a));
    let blocker = t.battlefield(P1, "Giant Spider");
    t.answer(
        P0,
        DecisionKind::Damage,
        mtg_engine::decision::Answer::Numbers(vec![1, 3]),
    );
    fight(&mut t, &[a], &[(blocker, a)]);
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P0), 25);
}

#[test]
fn two_bilbos_add_two() {
    cr!("614.1a", "616.1");
    ruling!(
        "Bilbo, Birthday Celebrant",
        "If you somehow control two Bilbo, Birthday Celebrants and you would gain life, you gain that much life plus 2."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mirror Gallery");
    t.battlefield(P0, BILBO);
    t.battlefield(P0, BILBO);
    t.settle();
    assert_eq!(t.named_on_battlefield(BILBO).len(), 2);
    crate::r_p152_common::gain(&mut t, P0, 3);
    assert_eq!(t.life(P0), 25);
    t.battlefield(P0, BILBO);
    crate::r_p152_common::gain(&mut t, P0, 3);
    assert_eq!(t.life(P0), 31);
}

#[test]
fn bilbo_ignores_a_teammates_life_gain() {
    cr!("810.9", "614.1a");
    ruling!(
        "Bilbo, Birthday Celebrant",
        "In a Two-Headed Giant game, life gained by your teammate won't cause Bilbo, Birthday Celebrant's first ability to apply, even though it will cause your team's life total to increase."
    );
    let mut t = crate::r_p108_common::two_headed_giant();
    t.battlefield(P0, BILBO);
    let before = t.life(P0);
    crate::r_p152_common::gain(&mut t, P1, 3);
    assert_eq!(t.life(P0), before + 3);
    crate::r_p152_common::gain(&mut t, P0, 3);
    assert_eq!(t.life(P0), before + 7);
}

#[test]
fn zilortha_lets_each_non_human_assign_all_its_damage_as_though_unblocked() {
    cr!("510.1c", "510.1a", "510.2");
    ruling!(
        "Invasion of Ikoria // Zilortha, Apex of Ikoria",
        "Assigning a creature’s damage as though it weren’t blocked is all or nothing."
    );
    ruling!(
        "Invasion of Ikoria // Zilortha, Apex of Ikoria",
        "Even if a creature assigns damage as though it were not blocked, it will still receive damage from creatures that are blocking it."
    );
    ruling!(
        "Invasion of Ikoria // Zilortha, Apex of Ikoria",
        "You decide whether to assign a non-Human creature’s combat damage as though it weren’t blocked just before it assigns that damage. You may make a different choice for each non-Human creature you control"
    );
    let mut t = TestGame::new(2);
    let z = t.battlefield(P0, "Invasion of Ikoria // Zilortha, Apex of Ikoria");
    mtg_engine::dfc::transform(&mut t.g, z);
    t.settle();
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wall = t.battlefield(P1, "Wall of Wood");
    let blocker = t.battlefield(P1, "Grizzly Bears");
    // Yes for the Giant, no for the Bears.
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    fight(&mut t, &[giant, bears], &[(wall, giant), (blocker, bears)]);
    assert_eq!(t.life(P1), 17, "all of the Giant's damage, none of the Bears'");
    assert_eq!(damage(&t, wall), 0);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    // The Wall dealt its 0 damage; the Bears blocker dealt 2 to the Bears.
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    // The Giant still takes the damage of a blocker that deals some.
    let mut t = TestGame::new(2);
    let z = t.battlefield(P0, "Invasion of Ikoria // Zilortha, Apex of Ikoria");
    mtg_engine::dfc::transform(&mut t.g, z);
    t.settle();
    let giant = t.battlefield(P0, "Hill Giant");
    let blocker = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, true);
    fight(&mut t, &[giant], &[(blocker, giant)]);
    assert_eq!(t.life(P1), 17);
    assert_eq!(damage(&t, blocker), 0);
    assert_eq!(damage(&t, giant), 2);
}

#[test]
fn samut_divides_on_activation_and_a_lone_legal_target_gets_its_share() {
    cr!("601.2d", "608.2b", "115.7");
    ruling!(
        "Samut, the Tested",
        "You divide the damage as you activate Samut's second ability, not as it resolves. Each target must be assigned at least 1 damage."
    );
    ruling!(
        "Samut, the Tested",
        "If Samut's second ability has two targets and one becomes an illegal target, the remaining target is dealt 1 damage, not 2."
    );
    let mut t = TestGame::new(2);
    let samut = t.battlefield(P0, "Samut, the Tested");
    let a = t.battlefield(P1, "Hill Giant");
    let b = t.battlefield(P1, "Hill Giant");
    let from = t.asked().len();
    crate::r_p003_common::divide(&mut t, P0, &[1, 1]);
    t.answer_targets(P0, &[obj(a), obj(b)]);
    crate::r_s06_common::activate_containing(&mut t, P0, samut, "divided").unwrap();
    let divided: Vec<u32> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::Divide { min_each, .. } => Some(*min_each),
            _ => None,
        })
        .collect();
    assert_eq!(divided, vec![1], "divided as it's activated, at least 1 each");
    destroy(&mut t, a);
    t.resolve_all();
    assert_eq!(damage(&t, b), 1);
}

#[test]
fn samuts_ultimate_puts_creatures_onto_the_battlefield_together() {
    cr!("603.6a", "701.23a");
    ruling!(
        "Samut, the Tested",
        "While resolving Samut's last ability, all of the creatures and planeswalkers put onto the battlefield this way enter at the same time. If any have triggered abilities that trigger on something else entering the battlefield, they'll see each other."
    );
    let mut t = TestGame::new(2);
    let lib = set_library(&mut t, P0, &["Soul Warden", "Soul Warden"]);
    let samut = t.battlefield(P0, "Samut, the Tested");
    crate::r_p003_common::put(&mut t, samut, counters::LOYALTY, 3);
    t.answer_choose(P0, &[obj(lib[0]), obj(lib[1])]);
    activate_resolve(&mut t, P0, samut, "creature and/or planeswalker", "", &[]);
    assert_eq!(t.named_on_battlefield("Soul Warden").len(), 2);
    assert_eq!(t.life(P0), 22, "each Soul Warden saw the other");
}

#[test]
fn shadowborn_apostle_ignores_only_the_four_of_rule() {
    cr!("100.2a", "100.6");
    ruling!(
        "Shadowborn Apostle",
        "Shadowborn Apostle's first ability lets you ignore only the \"four-of\" rule. It doesn't let you ignore format legality."
    );
    use mtg_engine::deck::{check_constructed, check_format_legality};
    let deck: Vec<_> = std::iter::repeat_n(mtg_engine::card::card("Shadowborn Apostle"), 30)
        .chain(std::iter::repeat_n(mtg_engine::card::card("Swamp"), 30))
        .collect();
    assert!(check_constructed(&deck).is_empty());
    assert!(check_format_legality(&deck, &[], "modern").is_empty());
    assert!(!check_format_legality(&deck, &[], "standard").is_empty());
}

#[test]
fn shadowborn_apostle_sacrifices_six_as_a_cost_not_necessarily_itself() {
    cr!("602.2b", "601.2h");
    ruling!(
        "Shadowborn Apostle",
        "You don't have to sacrifice the Shadowborn Apostle whose ability you activated. It can be six other Shadowborn Apostles."
    );
    ruling!(
        "Shadowborn Apostle",
        "You sacrifice the six creatures when you activate the ability. You can't respond to that ability by activating the ability of one of the other Shadowborn Apostles you just sacrificed."
    );
    let mut t = TestGame::new(2);
    set_library(&mut t, P0, &["Lord of the Pit"]);
    let ids: Vec<_> = (0..7).map(|_| t.battlefield(P0, "Shadowborn Apostle")).collect();
    let others: Vec<Entity> = ids[1..].iter().map(|o| obj(*o)).collect();
    t.answer_choose(P0, &others);
    activate(&mut t, P0, ids[0], "Demon", "B", &[]);
    assert!(t.on_battlefield(ids[0]), "the activated one stays");
    assert_eq!(t.named_on_battlefield("Shadowborn Apostle").len(), 1);
    for o in &ids[1..] {
        assert!(!crate::r_s02_common::can_activate(&mut t, P0, *o));
    }
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Lord of the Pit").len(), 1);
}

#[test]
fn kuldotha_forgemaster_can_sacrifice_itself() {
    cr!("602.2b", "118.3");
    ruling!(
        "Kuldotha Forgemaster",
        "Kuldotha Forgemaster can be one of the three artifacts you sacrifice to activate the ability."
    );
    let mut t = TestGame::new(2);
    let lib = set_library(&mut t, P0, &["Darksteel Relic"]);
    let k = t.battlefield(P0, "Kuldotha Forgemaster");
    let a = t.battlefield(P0, "Mind Stone");
    let b = t.battlefield(P0, "Sol Ring");
    t.answer_choose(P0, &[obj(k), obj(a), obj(b)]);
    t.answer_choose(P0, &[obj(lib[0])]);
    activate_resolve(&mut t, P0, k, "Search", "", &[]);
    assert!(t.in_graveyard(P0, "Kuldotha Forgemaster"));
    assert!(t.on_battlefield(t.g.current(lib[0])));
}

#[test]
fn rocksteady_with_menace_cant_be_blocked() {
    cr!("509.1b", "702.111b");
    ruling!(
        "Rocksteady, Crash Courser",
        "A creature with menace and \"This creature can't be blocked by more than one creature\" can't be blocked at all."
    );
    for blockers in [1, 2] {
        let mut t = TestGame::new(2);
        let r = t.battlefield(P0, "Rocksteady, Crash Courser");
        crate::r_p117_common::modify(
            &mut t,
            r,
            vec![Modification::AddKeyword(Keyword::new(KeywordKind::Menace))],
        );
        let bs: Vec<_> = (0..blockers)
            .map(|_| t.battlefield(P1, "Grizzly Bears"))
            .collect();
        let blocks: Vec<_> = bs.iter().map(|b| (*b, r)).collect();
        fight(&mut t, &[r], &blocks);
        assert_eq!(t.life(P1), 13, "{blockers} blockers");
    }
}

#[test]
fn alpine_guide_attacks_if_able_but_not_when_unable_or_at_a_cost() {
    cr!("508.1d", "508.1h", "302.6");
    ruling!(
        "Alpine Guide",
        "If Alpine Guide can't attack for any reason (such as being tapped or having come under a player's control that turn), then it doesn't attack. If there's a cost associated with having it attack, its controller isn't forced to pay that cost, so it doesn't have to attack in that case either."
    );
    // (tapped, summoning sick, Ghostly Prison) -> attacks
    for (tap, sick, prison, attacks) in [
        (false, false, false, true),
        (true, false, false, false),
        (false, true, false, false),
        (false, false, true, false),
    ] {
        let mut t = TestGame::new(2);
        let g = if sick {
            t.battlefield_sick(P0, "Alpine Guide")
        } else {
            t.battlefield(P0, "Alpine Guide")
        };
        t.g.objects[g.0 as usize].tapped = tap;
        if prison {
            t.battlefield(P1, "Ghostly Prison");
            t.lands(P0, "Wastes", 2);
        }
        attack_with(&mut t, &[]);
        assert_eq!(attacking(&t, g), attacks, "{tap} {sick} {prison}");
        if prison {
            assert_eq!(crate::r_s04_common::untapped_lands(&t, P0), 2, "nothing paid");
        }
    }
}

#[test]
fn yavimaya_dryad_can_target_you_and_its_forest_follows_multiplayer_rules() {
    cr!("800.4a", "115.1");
    ruling!(
        "Yavimaya Dryad",
        "You can be the target of Yavimaya Dryad's triggered ability."
    );
    ruling!(
        "Yavimaya Dryad",
        "In a multiplayer game, if you leave the game, the Forest you own leaves as well. If instead another player who received your Forest from Yavimaya Dryad leaves the game, that land is exiled."
    );
    let mut t = TestGame::new(2);
    let lib = set_library(&mut t, P0, &["Forest"]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[pl(P0)]);
    t.answer_choose(P0, &[obj(lib[0])]);
    t.enter(P0, "Yavimaya Dryad");
    t.resolve_all();
    let f = t.g.current(lib[0]);
    assert!(tapped(&t, f) && t.g.obj(f).controller == P0);
    for leaver in [P0, P1] {
        let mut t = TestGame::new(3);
        let lib = set_library(&mut t, P0, &["Forest"]);
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[pl(P1)]);
        t.answer_choose(P0, &[obj(lib[0])]);
        t.enter(P0, "Yavimaya Dryad");
        t.resolve_all();
        let f = t.g.current(lib[0]);
        assert_eq!(t.g.obj(f).controller, P1);
        t.g.lose_game(leaver);
        let f = t.g.current(f);
        assert!(!t.on_battlefield(f), "{leaver} left");
        if leaver == P1 {
            assert_eq!(t.zone(f), Zone::Exile);
        } else {
            assert_ne!(t.zone(f), Zone::Exile);
        }
    }
}

#[test]
fn sanctum_of_all_shrine_found_in_upkeep_misses_that_upkeep_and_only_triggers_double() {
    cr!("603.2", "603.2g", "603.6a");
    ruling!(
        "Sanctum of All",
        "If the Shrine you put onto the battlefield with Sanctum of All's first ability during your upkeep has an ability that triggers at the beginning of your upkeep, it won't trigger during that upkeep."
    );
    ruling!(
        "Sanctum of All",
        "Triggered abilities use the word “when,” “whenever,” or “at.” They're often written as “[Trigger condition], [effect].” Replacement effects, using “instead” or “as,” aren't affected. Activated abilities (written as “Cost: Effect”) of Shrines you control are also not affected."
    );
    let mut t = TestGame::new(2);
    let lib = set_library(&mut t, P0, &["Honden of Cleansing Fire"]);
    t.battlefield(P0, "Sanctum of All");
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(lib[0])]);
    crate::r_s04_common::next_upkeep(&mut t, P0);
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(lib[0])));
    assert_eq!(t.life(P0), 20, "the Honden didn't trigger");
    // Six Shrines: the Honden's upkeep trigger triggers twice; an activated ability once.
    let mut t = TestGame::new(2);
    let honden = t.battlefield(P0, "Honden of Cleansing Fire");
    for s in [
        "Sanctum of All",
        "Sanctum of Tranquil Light",
        "Sanctum of Shattered Heights",
        "Sanctum of Calm Waters",
        "Sanctum of Stone Fangs",
    ] {
        t.battlefield(P0, s);
    }
    t.answer_yes(P0, false);
    crate::r_s04_common::next_upkeep(&mut t, P0);
    assert_eq!(triggered_from(&t, honden), 2);
    t.resolve_all();
    assert_eq!(t.life(P0), 44);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    let light = t.named_on_battlefield("Sanctum of Tranquil Light")[0];
    let bears = t.battlefield(P1, "Grizzly Bears");
    let n = t.stack_len();
    activate(&mut t, P0, light, "Tap target", "5W", &[obj(bears)]);
    assert_eq!(t.stack_len(), n + 1);
}

#[test]
fn vizier_draws_after_eternalize_costs_are_paid() {
    cr!("602.2b", "603.3", "702.129a");
    ruling!(
        "Vizier of the Anointed",
        "If you activate an eternalize or embalm ability, you’ll draw a card before that ability resolves, but after you’ve paid all of the costs for that ability. If an eternalize ability requires a discard to activate, you’ll need to have another card available to discard."
    );
    // An empty hand: Sinuous Striker's eternalize can't be activated.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vizier of the Anointed");
    let s = t.graveyard(P0, "Sinuous Striker");
    mana(&mut t, P0, "3UU");
    assert!(!crate::r_s02_common::can_activate(&mut t, P0, s));
    // One card to discard: the draw comes after the cost, before the token.
    let other = t.hand(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(other)]);
    activate(&mut t, P0, s, "Eternalize", "", &[]);
    t.settle();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.hand_size(P0), 0);
    t.resolve();
    assert_eq!(t.hand_size(P0), 1, "the draw resolves first");
    assert!(crate::r_s01_common::tokens(&t, P0).is_empty());
    t.resolve_all();
    assert_eq!(crate::r_s01_common::tokens(&t, P0).len(), 1);
}

#[test]
fn colossal_rattlewurm_keeps_flash_after_its_desert_is_sacrificed_for_mana() {
    cr!("601.3", "601.2g", "702.8a");
    ruling!(
        "Colossal Rattlewurm",
        "The game checks whether it is legal to cast a spell before costs are determined or mana abilities are activated while casting it. If you no longer control a Desert after that point, perhaps because you sacrificed all of your Deserts while activating mana abilities, this won't affect whether you can legally cast Colossal Rattlewurm."
    );
    // On P1's turn: P0's only land is a tapped Desert, which Rain of Filth lets P0
    // sacrifice for the last mana ("Sacrifice this land: Add {B}").
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let desert = t.battlefield(P0, "Desert");
    t.g.objects[desert.0 as usize].tapped = true;
    cast(&mut t, P0, "Rain of Filth", "B", &[]);
    t.resolve_all();
    let w = t.hand(P0, "Colossal Rattlewurm");
    mana(&mut t, P0, "GGG");
    t.g.turn.priority = Some(P0);
    let spell = t.cast(P0, w).method(CastMethod::Normal).try_go();
    let spell = spell.unwrap_or_else(|e| panic!("{e:?}\n{}", t.dump_log()));
    assert!(t.in_graveyard(P0, "Desert"));
    assert_eq!(t.zone(spell), Zone::Stack);
}
