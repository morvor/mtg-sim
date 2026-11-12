//! Rulings batch P182 — search spells and abilities: modes and their targets (CR 700.2,
//! 608.2b), costs paid as the spell is cast or the ability activated (CR 601.2h, 602.2),
//! effects that happen whether or not the search finds anything, and Maze's End's win
//! check (CR 104.2b).

use crate::r_p182_common::*;
use crate::r_s01_common::{tokens, watch};
use mtg_engine::decision::Decision;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// `p` casts `name` with the given modes and targets (one per target slot), paying from
/// its pool; the spell stays on the stack (or the cast is refused).
fn cast_modes(
    t: &mut TestGame,
    p: PlayerId,
    name: &str,
    cost: &str,
    modes: &[usize],
    targets: &[Entity],
) -> Result<ObjectId, mtg_engine::casting::Illegal> {
    let c = t.hand(p, name);
    mana(t, p, cost);
    for e in targets {
        t.answer_targets(p, &[*e]);
    }
    t.g.turn.priority = Some(p);
    t.cast(p, c).modes(modes).try_go()
}

#[test]
fn archdruids_charm_second_mode_with_an_illegal_target() {
    cr!("608.2b");
    ruling!(
        "Archdruid's Charm",
        "If you choose the second mode and the creature you don't control is not a legal target as Archdruid's Charm resolves, you'll still put a +1/+1 counter on your creature."
    );
    ruling!(
        "Archdruid's Charm",
        "If you choose the second mode and your creature is not a legal target as Archdruid's Charm resolves, you won't put a +1/+1 counter on it, and no damage will be dealt."
    );
    // Their creature is gone: the counter is still put on P0's.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    cast_modes(
        &mut t,
        P0,
        "Archdruid's Charm",
        "GGG",
        &[1],
        &[obj(mine), obj(theirs)],
    )
    .unwrap();
    destroy(&mut t, theirs);
    t.resolve_all();
    assert_eq!(t.counters(mine, counters::PLUS1), 1);
    // P0's creature is gone: no counter, no damage.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    let spell = cast_modes(
        &mut t,
        P0,
        "Archdruid's Charm",
        "GGG",
        &[1],
        &[obj(mine), obj(theirs)],
    )
    .unwrap();
    crate::r_p152_common::bounce(&mut t, mine);
    t.resolve_all();
    assert!(crate::r_p108_common::resolved(&t, spell));
    assert_eq!(t.g.obj(theirs).damage, 0);
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn archdruids_charm_second_mode_needs_both_targets() {
    cr!("700.2a", "601.2c");
    ruling!(
        "Archdruid's Charm",
        "You can't choose the second mode for Archdruid's Charm unless you and another player have creatures to target."
    );
    // The modes P0 may choose as Archdruid's Charm is cast with these creatures.
    let available = |mine: bool, theirs: bool| {
        let mut t = TestGame::new(2);
        if mine {
            t.battlefield(P0, "Grizzly Bears");
        }
        if theirs {
            t.battlefield(P1, "Grizzly Bears");
        }
        let from = t.asked().len();
        let _ = cast_modes(&mut t, P0, "Archdruid's Charm", "GGG", &[0], &[]);
        t.asked()[from..]
            .iter()
            .find_map(|(_, d)| match d {
                Decision::ChooseModes { available, .. } => Some(available.clone()),
                _ => None,
            })
            .expect("no mode choice")
    };
    assert!(!available(true, false).contains(&1));
    assert!(!available(false, true).contains(&1));
    assert!(available(true, true).contains(&1));
}

#[test]
fn titanias_command_with_its_only_target_illegal_does_nothing() {
    cr!("608.2b", "702.11c");
    ruling!(
        "Titania's Command",
        "If you choose the first mode and that player is an illegal target by the time Titania's Command would resolve, the spell is removed from the stack and none of its effects happen."
    );
    let mut t = TestGame::new(2);
    t.graveyard(P1, "Grizzly Bears");
    set_library(&mut t, P0, &["Forest", "Forest"]);
    cast_modes(&mut t, P0, "Titania's Command", "4GG", &[0, 1], &[pl(P1)]).unwrap();
    // P1 gains hexproof.
    t.battlefield(P1, "Leyline of Sanctity");
    t.resolve_all();
    assert_eq!(t.graveyard_size(P1), 1);
    assert_eq!(t.life(P0), 20);
    assert_eq!(searches(&t, P0), 0);
    assert_eq!(t.library_size(P0), 2);
}

#[test]
fn cosmium_confluence_repeats_modes_in_the_order_chosen_and_picks_its_cave_on_resolution() {
    cr!("700.2d", "608.2c", "608.2d");
    ruling!(
        "Cosmium Confluence",
        "If the same mode is chosen more than once, you choose their relative order as you cast the spell. For example, if you choose the last mode of Cosmium Confluence more than once, you choose the relative order to destroy the target enchantments."
    );
    ruling!(
        "Cosmium Confluence",
        "Notably, the second mode doesn't require any targets. You choose the Cave you control that will be affected as Cosmium Confluence is resolving. You may choose a Cave you just put onto the battlefield with the first mode, if applicable."
    );
    // Three enchantments destroyed in the order chosen.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Endless Horizons");
    let b = t.battlefield(P1, "Land Tax");
    let c = t.battlefield(P1, "Wild Pair");
    cast_modes(
        &mut t,
        P0,
        "Cosmium Confluence",
        "4G",
        &[2, 2, 2],
        &[obj(c), obj(a), obj(b)],
    )
    .unwrap();
    t.resolve_all();
    let gy: Vec<String> =
        t.g.player(P1)
            .graveyard
            .iter()
            .map(|o| t.g.obj(*o).chars.name.to_string())
            .collect();
    assert_eq!(gy, vec!["Wild Pair", "Endless Horizons", "Land Tax"]);
    // Find a Cave, then animate that one (no target).
    let mut t = TestGame::new(2);
    let lib = set_library(&mut t, P0, &["Captivating Cave"]);
    t.battlefield(P0, "Volatile Fault");
    t.answer_choose(P0, &[obj(lib[0])]);
    t.answer_choose_moved(P0, &[obj(lib[0])]);
    t.answer_choose_moved(P0, &[obj(lib[0])]);
    let spell = cast_modes(&mut t, P0, "Cosmium Confluence", "4G", &[0, 1, 1], &[]).unwrap();
    assert!(crate::r_s25_common::targets_of(&t, spell).is_empty());
    t.resolve_all();
    let cave = t.g.current(lib[0]);
    assert!(t.on_battlefield(cave));
    assert_eq!(t.counters(cave, counters::PLUS1), 6);
    let o = t.g.obj(cave);
    assert!(o.is(CardType::Creature) && o.is(CardType::Land));
    assert_eq!(t.pt(cave), (6, 6));
}

#[test]
fn scapeshift_sacrifices_on_resolution() {
    cr!("608.2c", "701.6a");
    ruling!(
        "Scapeshift",
        "You sacrifice the lands as part of the resolution of Scapeshift. It isn’t an additional cost. If Scapeshift is countered, you won’t sacrifice any lands."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    let spell = cast(&mut t, P0, "Scapeshift", "2GG", &[]);
    assert_eq!(lands_of(&t, P0), 2, "nothing sacrificed as it's cast");
    crate::r_p153_common::counter(&mut t, P1, spell);
    assert_eq!(lands_of(&t, P0), 2);
    // It resolves: two lands become two others.
    let mut t = TestGame::new(2);
    let lib = set_library(&mut t, P0, &["Mountain", "Plains"]);
    let forests = t.lands(P0, "Forest", 2);
    t.answer_choose(P0, &[obj(forests[0]), obj(forests[1])]);
    t.answer_choose(P0, &[obj(lib[0]), obj(lib[1])]);
    cast(&mut t, P0, "Scapeshift", "2GG", &[]);
    t.resolve_all();
    assert_eq!(permanents_of(&t, P0), names(&["Mountain", "Plains"]));
}

#[test]
fn crop_rotation_needs_exactly_one_land_sacrificed() {
    cr!("601.2b", "601.2h", "118.3");
    ruling!(
        "Crop Rotation",
        "You can’t cast Crop Rotation without sacrificing a land, and you can’t sacrifice additional lands."
    );
    let mut t = TestGame::new(2);
    let c = t.hand(P0, "Crop Rotation");
    mana(&mut t, P0, "G");
    assert!(!crate::r_s02_common::can_cast(
        &mut t,
        P0,
        c,
        CastMethod::Normal
    ));
    let mut t = TestGame::new(2);
    let lib = set_library(&mut t, P0, &["Maze's End"]);
    let forests = t.lands(P0, "Forest", 3);
    t.answer_choose(P0, &[obj(forests[0]), obj(forests[1])]);
    t.answer_choose(P0, &[obj(lib[0])]);
    let c = t.hand(P0, "Crop Rotation");
    mana(&mut t, P0, "G");
    t.cast(P0, c).go();
    assert_eq!(lands_of(&t, P0), 2, "one land is sacrificed");
    t.resolve_all();
    assert_eq!(lands_of(&t, P0), 3);
}

#[test]
fn growth_spasm_makes_its_token_whether_or_not_it_finds_a_land() {
    cr!("608.2c");
    ruling!(
        "Growth Spasm",
        "You put an Eldrazi Spawn token onto the battlefield whether or not you put a basic land card onto the battlefield."
    );
    let mut t = TestGame::new(2);
    set_library(&mut t, P0, &["Maze's End"]);
    cast(&mut t, P0, "Growth Spasm", "2G", &[]);
    t.resolve_all();
    let tk = tokens(&t, P0);
    assert_eq!(tk.len(), 1);
    assert!(t.g.obj(tk[0]).chars.has_subtype("Spawn"));
    assert_eq!(t.library_size(P0), 1);
}

#[test]
fn hour_of_promise_counts_the_deserts_it_found() {
    cr!("608.2c");
    ruling!(
        "Hour of Promise",
        "When Hour of Promise checks the number of Deserts you control, it includes any Deserts you found earlier with Hour of Promise."
    );
    let mut t = TestGame::new(2);
    let lib = set_library(&mut t, P0, &["Desert", "Ipnu Rivulet"]);
    t.battlefield(P0, "Desert");
    t.answer_choose(P0, &[obj(lib[0]), obj(lib[1])]);
    cast(&mut t, P0, "Hour of Promise", "4G", &[]);
    t.resolve_all();
    let zombies = tokens(&t, P0)
        .into_iter()
        .filter(|o| t.g.obj(*o).chars.has_subtype("Zombie"))
        .count();
    assert_eq!(zombies, 2);
}

#[test]
fn flourish_affects_only_the_creatures_there_as_it_resolves() {
    cr!("611.2c");
    ruling!(
        "Flower // Flourish",
        "Flourish affects only creatures you control at the time it resolves. Creatures that come under your control later in the turn won't get +2/+2."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let card = t.hand(P0, "Flower // Flourish");
    mana(&mut t, P0, "4GW");
    t.cast(P0, card).method(CastMethod::Half(1)).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 4));
    let later = t.battlefield(P0, "Grizzly Bears");
    t.g.recompute();
    assert_eq!(t.pt(later), (2, 2));
}

#[test]
fn disorienting_choice_opponents_decide_in_turn_order_then_exile_together() {
    cr!("101.4", "608.2e");
    ruling!(
        "Disorienting Choice",
        "When Disorienting Choice resolves, the next opponent in turn order that controls a permanent chosen with Disorienting Choice decides whether or not to exile that permanent. Then each other opponent in turn order does the same. Then all appropriate permanents are exiled simultaneously. Opponents will know what choices opponents earlier in the turn order made."
    );
    let mut t = TestGame::new(3);
    let lib = set_library(&mut t, P0, &["Forest", "Forest"]);
    let stone = t.battlefield(P1, "Mind Stone");
    let ring = t.battlefield(P2, "Sol Ring");
    // Whether Mind Stone is still on the battlefield as P2 decides.
    let seen = watch(
        &mut t,
        P2,
        |d| matches!(d, Decision::YesNo { .. }),
        |g| g.find_in_zone(Zone::Battlefield, "Mind Stone").len(),
    );
    t.answer_yes(P1, true);
    t.answer_yes(P2, false);
    t.answer_choose(P0, &[obj(lib[0])]);
    let from = t.asked().len();
    cast(
        &mut t,
        P0,
        "Disorienting Choice",
        "3G",
        &[obj(stone), obj(ring)],
    );
    t.resolve_all();
    let order: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::YesNo { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(order, vec![P1, P2]);
    assert_eq!(*seen.lock().unwrap(), vec![1], "exiled after every choice");
    assert!(t.in_exile("Mind Stone"));
    assert!(t.on_battlefield(ring));
    // One chosen permanent is still there: up to one land.
    assert_eq!(offer(&t, P0, from).max, 1);
    assert!(tapped(&t, lib[0]));
}

#[test]
fn dichotomancy_auras_cant_attach_to_permanents_entering_with_them() {
    cr!("303.4g", "303.4f", "701.23a");
    ruling!(
        "Dichotomancy",
        "The cards all enter simultaneously. If one of the cards that's entering is an Aura, it must enter attached to a permanent already on the battlefield. It can't enter attached to another permanent entering via Dichotomancy. If an Aura can't enter this way, it remains in the opponent's library."
    );
    // P1's tapped Bears with a tapped Aura on it; the same two cards in P1's library.
    // Holy Strength (enchant creature) can join P1's Bears already there, not the new
    // Bears; Spirit Loop (enchant creature you control) has no creature of P0's to join.
    for (aura_name, attaches) in [("Holy Strength", true), ("Spirit Loop", false)] {
        let mut t = TestGame::new(2);
        let lib = set_library(&mut t, P1, &["Grizzly Bears", aura_name]);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let aura = crate::r_s06_common::attach_new(&mut t, P1, aura_name, obj(bears));
        for id in [bears, aura] {
            t.g.objects[id.0 as usize].tapped = true;
        }
        let from = t.asked().len();
        cast(&mut t, P0, "Dichotomancy", "7UU", &[pl(P1)]);
        t.resolve_all();
        let new_bears = t.g.current(lib[0]);
        assert!(t.on_battlefield(new_bears) && t.g.obj(new_bears).controller == P0);
        // What the Aura may enchant: only what was already on the battlefield.
        let attach_choices: Vec<Vec<Entity>> = t.asked()[from..]
            .iter()
            .filter_map(|(_, d)| match d {
                Decision::ChooseEntities {
                    prompt, candidates, ..
                } if prompt.contains("enchant") => Some(candidates.clone()),
                _ => None,
            })
            .collect();
        assert!(
            attach_choices.iter().all(|c| !c.contains(&obj(new_bears))),
            "{aura_name}: {attach_choices:?}"
        );
        let found = t.g.current(lib[1]);
        if attaches {
            assert!(t.on_battlefield(found), "{aura_name}");
            assert_eq!(t.g.obj(found).attached_to, Some(obj(bears)), "{aura_name}");
        } else {
            assert_eq!(t.zone(found), Zone::Library(P1), "{aura_name}");
        }
    }
}

#[test]
fn flourishing_bloom_kin_with_one_forest_puts_it_onto_the_battlefield() {
    cr!("701.23a", "608.2c");
    ruling!(
        "Flourishing Bloom-Kin",
        "If you find only one Forest card with Flourishing Bloom-Kin's last ability, you'll put it onto the battlefield tapped. You won't have the option to put it into your hand."
    );
    let mut t = TestGame::new(2);
    let lib = set_library(&mut t, P0, &["Forest", "Island"]);
    let id = crate::r_p180_common::disguise(&mut t, P0, "Flourishing Bloom-Kin");
    crate::r_p180_common::turn_up(&mut t, P0, id, &[("Forest", 1), ("Wastes", 4)]);
    t.answer_choose(P0, &[obj(lib[0])]);
    t.resolve_all();
    assert!(tapped(&t, lib[0]));
    assert_eq!(t.hand_size(P0), 0);
}

#[test]
fn scholar_of_new_horizons_may_put_the_plains_into_hand() {
    cr!("608.2d");
    ruling!(
        "Scholar of New Horizons",
        "You may choose to put the card into your hand rather than onto the battlefield even if an opponent controls more lands than you."
    );
    // An opponent controls more lands: P0 is offered the battlefield and may decline it.
    for battlefield in [true, false] {
        let mut t = TestGame::new(2);
        let lib = set_library(&mut t, P0, &["Plains"]);
        t.lands(P1, "Forest", 2);
        let s = t.battlefield(P0, "Scholar of New Horizons");
        crate::r_p003_common::put(&mut t, s, counters::PLUS1, 1);
        t.answer_choose(P0, &[obj(s)]);
        t.answer_choose(P0, &[obj(lib[0])]);
        t.answer_yes(P0, battlefield);
        let from = t.asked().len();
        activate_resolve(&mut t, P0, s, "Search", "", &[]);
        assert!(crate::r_p152_common::asked_yes_no(&t, P0, from));
        assert_eq!(tapped(&t, lib[0]), battlefield);
        assert_eq!(t.in_hand(P0, "Plains"), !battlefield);
    }
}

const GATES: [&str; 10] = [
    "Azorius Guildgate",
    "Boros Guildgate",
    "Dimir Guildgate",
    "Golgari Guildgate",
    "Gruul Guildgate",
    "Izzet Guildgate",
    "Orzhov Guildgate",
    "Rakdos Guildgate",
    "Selesnya Guildgate",
    "Simic Guildgate",
];

/// P0 controls Maze's End and the Gates named; P0's library is `library`. Returns Maze's
/// End.
fn maze(t: &mut TestGame, gates: &[&str], library: &[&str]) -> (ObjectId, Vec<ObjectId>) {
    let lib = set_library(t, P0, library);
    for g in gates {
        t.battlefield(P0, g);
    }
    (t.battlefield(P0, "Maze's End"), lib)
}

#[test]
fn mazes_end_returns_itself_as_a_cost_and_isnt_a_land_play() {
    cr!("602.2b", "305.2", "601.2h");
    ruling!(
        "Maze's End",
        "Returning Maze's End to its owner's hand is part of the cost to activate its last ability. Once that ability is announced, players can't respond to it until after you've paid its activation cost and returned Maze's End to hand."
    );
    ruling!(
        "Maze's End",
        "Putting a Gate onto the battlefield with Maze's End doesn't count as the one land you can play during your turn. If it's your turn, you can play Maze's End or a different land card from your hand after its ability has resolved."
    );
    let mut t = TestGame::new(2);
    let (m, lib) = maze(&mut t, &[], &["Azorius Guildgate"]);
    t.answer_choose(P0, &[obj(lib[0])]);
    activate(&mut t, P0, m, "Gate card", "3", &[]);
    assert!(
        t.in_hand(P0, "Maze's End"),
        "returned as the ability is activated"
    );
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(lib[0])));
    let m = t.g.current(m);
    assert!(crate::r_s02_common::can_play_land(&mut t, P0, m));
    t.play_land(P0, m).unwrap();
}

#[test]
fn mazes_end_checks_ten_differently_named_gates_as_it_resolves() {
    cr!("104.2b", "608.2c");
    ruling!(
        "Maze's End",
        "Controlling multiple Gates with the same name has no effect on your ability to win the game with Maze's End. The excess Gates are simply ignored."
    );
    ruling!(
        "Maze's End",
        "When the last ability of Maze's End resolves, you'll search for a Gate and put it onto the battlefield before checking to see if you win the game. This check will happen even if you don't put a Gate onto the battlefield this way. This check will happen only as the ability resolves, not at other times."
    );
    // Nine names and a duplicate found: no win.
    let mut t = TestGame::new(2);
    let (m, lib) = maze(
        &mut t,
        &GATES[..9],
        &["Azorius Guildgate", "Simic Guildgate"],
    );
    t.battlefield(P0, "Boros Guildgate");
    t.answer_choose(P0, &[obj(lib[0])]);
    activate_resolve(&mut t, P0, m, "Gate card", "3", &[]);
    assert!(!t.has_lost(P1));
    // The tenth name found: P0 wins.
    let mut t = TestGame::new(2);
    let (m, lib) = maze(
        &mut t,
        &GATES[..9],
        &["Azorius Guildgate", "Simic Guildgate"],
    );
    t.answer_choose(P0, &[obj(lib[1])]);
    activate_resolve(&mut t, P0, m, "Gate card", "3", &[]);
    assert!(t.has_lost(P1));
    // Ten names already: no win until the ability resolves, even finding nothing.
    let mut t = TestGame::new(2);
    let (m, _) = maze(&mut t, &GATES, &["Forest"]);
    t.settle();
    t.advance_to(P0, Step::End);
    assert!(!t.has_lost(P1), "no win at other times");
    activate_resolve(&mut t, P0, m, "Gate card", "3", &[]);
    assert!(t.has_lost(P1));
}
