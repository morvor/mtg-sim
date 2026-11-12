//! Rulings batch P182 — what a search can find (CR 701.23): land types rather than names,
//! nonbasic lands, multicolored cards, mana values with {X} counted as 0 (CR 202.3e), and
//! the cards with a stated quality a searcher needn't find (CR 701.23b).

use crate::r_p182_common::*;
use crate::r_s04_common::add_mana;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

/// A search P0 makes from a scenario: the library (top first) is set up, `run` makes the
/// search happen (P0 finds nothing, so the library stays as it was), and the search
/// choices P0 was offered are returned.
fn offered(library: &[&str], run: impl FnOnce(&mut TestGame)) -> (TestGame, Vec<Offer>) {
    let mut t = TestGame::new(2);
    set_library(&mut t, P0, library);
    let from = t.asked().len();
    run(&mut t);
    t.resolve_all();
    let o = offers(&t, P0, from);
    (t, o)
}

/// P0 activates the ability of the permanent `name` (on their battlefield) whose text
/// contains `needle`, finding nothing.
fn activated(
    name: &'static str,
    needle: &'static str,
    cost: &'static str,
) -> impl FnOnce(&mut TestGame) {
    move |t: &mut TestGame| {
        let id = t.battlefield(P0, name);
        t.answer_choose(P0, &[]);
        activate(t, P0, id, needle, cost, &[]);
    }
}

/// `name` enters under P0's control; P0 searches and finds nothing.
fn enters(name: &'static str) -> impl FnOnce(&mut TestGame) {
    move |t: &mut TestGame| {
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[]);
        t.enter(P0, name);
    }
}

#[test]
fn searches_for_a_land_type_find_nonbasic_lands_of_that_type() {
    cr!("701.23a", "205.3i");
    ruling!(
        "Wirewood Guardian",
        "You can choose to find any card with the Forest land type, including nonbasic lands."
    );
    ruling!(
        "Farseek",
        "Farseek can find any land with any of the listed land types, including nonbasic ones, even if that land is a Forest in addition to one or more of those types."
    );
    ruling!(
        "Karametra, God of Harvests",
        "You can use the last ability to put any land card with the subtype Forest or Plains onto the battlefield, not just ones named Forest or Plains."
    );
    ruling!(
        "Perilous Forays",
        "The land doesn’t need to be a basic land, so you can find a land such as Overgrown Tomb or Tropical Island."
    );
    ruling!(
        "Slimefoot's Survey",
        "you may find cards that are not basic lands as long as they have one or more of the basic land types"
    );
    ruling!(
        "Primeval Titan",
        "You may find any land cards, not just basic land cards."
    );
    let lib = &[
        "Breeding Pool",
        "Stomping Ground",
        "Temple Garden",
        "Overgrown Tomb",
        "Forest",
        "Island",
        "Maze's End",
        "Grizzly Bears",
    ];
    // Forestcycling.
    let (_, o) = offered(lib, |t| {
        let c = t.hand(P0, "Wirewood Guardian");
        mana(t, P0, "2");
        t.answer_choose(P0, &[]);
        t.activate(P0, c, 0, &[]).unwrap();
    });
    assert_eq!(
        o[0].names,
        names(&[
            "Breeding Pool",
            "Stomping Ground",
            "Temple Garden",
            "Overgrown Tomb",
            "Forest"
        ])
    );
    // Farseek: Plains, Island, Swamp or Mountain (a Forest too is fine; a plain Forest isn't).
    let (_, o) = offered(lib, |t| {
        t.answer_choose(P0, &[]);
        cast(t, P0, "Farseek", "1G", &[]);
    });
    assert_eq!(
        o[0].names,
        names(&[
            "Breeding Pool",
            "Stomping Ground",
            "Temple Garden",
            "Overgrown Tomb",
            "Island"
        ])
    );
    // Karametra: a creature spell is cast; any Forest or Plains card.
    let (_, o) = offered(lib, |t| {
        t.battlefield(P0, "Karametra, God of Harvests");
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[]);
        cast(t, P0, "Grizzly Bears", "1G", &[]);
    });
    assert_eq!(
        o[0].names,
        names(&[
            "Breeding Pool",
            "Stomping Ground",
            "Temple Garden",
            "Overgrown Tomb",
            "Forest"
        ])
    );
    // Perilous Forays and Slimefoot's Survey: a land card with a basic land type.
    let with_type = names(&[
        "Breeding Pool",
        "Stomping Ground",
        "Temple Garden",
        "Overgrown Tomb",
        "Forest",
        "Island",
    ]);
    let (_, o) = offered(lib, |t| {
        t.battlefield(P0, "Grizzly Bears");
        t.answer_choose(P0, &[]);
        let f = t.battlefield(P0, "Perilous Forays");
        activate(t, P0, f, "Search", "1", &[]);
    });
    assert_eq!(o[0].names, with_type);
    let (_, o) = offered(lib, |t| {
        t.answer_choose(P0, &[]);
        cast(t, P0, "Slimefoot's Survey", "4G", &[]);
    });
    assert_eq!(o[0].names, with_type);
    // Primeval Titan: any land cards.
    let (_, o) = offered(lib, enters("Primeval Titan"));
    let mut all = with_type.clone();
    all.push("Maze's End".into());
    all.sort();
    assert_eq!(o[0].names, all);
}

#[test]
fn slimefoots_survey_finds_two_of_one_type_and_counts_them_for_domain() {
    cr!("701.23a", "608.2c");
    ruling!(
        "Slimefoot's Survey",
        "You may search for two cards with the same basic land type or different basic land types."
    );
    ruling!(
        "Slimefoot's Survey",
        "The value of X is determined after you put the lands you searched for onto the battlefield."
    );
    let mut t = TestGame::new(2);
    let lib = set_library(
        &mut t,
        P0,
        &["Forest", "Forest", "Island", "Swamp", "Plains"],
    );
    t.battlefield(P0, "Plains");
    t.answer_choose(P0, &[obj(lib[0]), obj(lib[1])]);
    let from = t.asked().len();
    cast(&mut t, P0, "Slimefoot's Survey", "4G", &[]);
    t.resolve_all();
    assert!(
        tapped(&t, lib[0]) && tapped(&t, lib[1]),
        "both Forests found"
    );
    // Plains and Forest (the Forests just found): the top two cards are looked at.
    let looked: Vec<_> = t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            mtg_engine::decision::Decision::ChooseEntities {
                prompt, candidates, ..
            } if *p == P0 && !prompt.starts_with("Search") => Some(candidates.len()),
            _ => None,
        })
        .collect();
    assert_eq!(looked, vec![2], "X counts the Forests found");
}

#[test]
fn shard_convergence_finds_subtypes_and_needs_not_find_them_all() {
    cr!("701.23a", "701.23b", "205.3i");
    ruling!(
        "Shard Convergence",
        "Shard Convergence checks for cards with the subtypes Plains, Island, Swamp, and Mountain, not for cards with those names."
    );
    ruling!(
        "Shard Convergence",
        "You don’t have to find all four cards."
    );
    ruling!(
        "Shard Convergence",
        "you may find a Hallowed Fountain (which has subtypes Plains and Island) as the Plains card, and another Hallowed Fountain as the Island card"
    );
    let mut t = TestGame::new(2);
    let lib = set_library(
        &mut t,
        P0,
        &["Hallowed Fountain", "Hallowed Fountain", "Swamp", "Forest"],
    );
    t.answer_choose(P0, &[obj(lib[0])]);
    t.answer_choose(P0, &[obj(lib[1])]);
    t.answer_choose(P0, &[]);
    let from = t.asked().len();
    cast(&mut t, P0, "Shard Convergence", "3G", &[]);
    t.resolve_all();
    let o = offers(&t, P0, from);
    // Plains, Island, Swamp (there's no Mountain card: nothing to choose).
    assert_eq!(o.len(), 3, "{o:?}");
    assert_eq!(
        o[0].names,
        names(&["Hallowed Fountain", "Hallowed Fountain"])
    );
    assert_eq!(o[1].names, names(&["Hallowed Fountain"]));
    assert_eq!(o[2].names, names(&["Swamp"]));
    assert!(o.iter().all(|x| x.min == 0));
    assert_eq!(
        hand_of(&t, P0),
        names(&["Hallowed Fountain", "Hallowed Fountain"])
    );
    assert!(
        t.g.player(P0).library.contains(&lib[2]),
        "the Swamp wasn't found"
    );
}

#[test]
fn yasharn_can_find_just_one_basic() {
    cr!("701.23b");
    ruling!(
        "Yasharn, Implacable Earth",
        "Yasharn’s first ability can find just one basic Forest or one basic Plains card if you don’t have both in your library or if you don’t wish to find both."
    );
    // No Plains in the library.
    let mut t = TestGame::new(2);
    set_library(&mut t, P0, &["Forest", "Island"]);
    t.enter(P0, "Yasharn, Implacable Earth");
    t.resolve_all();
    assert_eq!(hand_of(&t, P0), names(&["Forest"]));
    // Both are there; P0 finds only the Plains.
    let mut t = TestGame::new(2);
    let lib = set_library(&mut t, P0, &["Forest", "Plains"]);
    t.answer_choose(P0, &[]);
    t.answer_choose(P0, &[obj(lib[1])]);
    t.enter(P0, "Yasharn, Implacable Earth");
    t.resolve_all();
    assert_eq!(hand_of(&t, P0), names(&["Plains"]));
}

#[test]
fn cards_with_a_stated_quality_neednt_be_found() {
    cr!("701.23b");
    ruling!(
        "Mountain Valley",
        "You do not have to find a mountain or forest card if you do not want to."
    );
    ruling!(
        "Grasslands",
        "You do not have to find a plains or forest card if you do not want to."
    );
    ruling!(
        "Wirewood Guardian",
        "You can also choose not to find a card, even if there is a Forest card in your library."
    );
    ruling!(
        "Welkin Hawk",
        "You do not have to find a Welkin Hawk card if you do not want to."
    );
    ruling!(
        "Skyshroud Poacher",
        "You do not have to find an Elf card if you do not want to."
    );
    ruling!(
        "Academy Rector",
        "You do not have to find an enchantment card if you do not want to, even if you have one in your library."
    );
    ruling!(
        "Firemind's Foresight",
        "You don't have to find any instant cards in your library, even if they are there."
    );
    let lands = &["Forest", "Mountain", "Plains", "Breeding Pool"];
    // (card, library, scenario, the cards offered)
    type Run = Box<dyn FnOnce(&mut TestGame)>;
    let cases: Vec<(&str, &[&str], Run, Vec<String>)> = vec![
        (
            "Mountain Valley",
            lands,
            Box::new(activated("Mountain Valley", "Search", "")),
            names(&["Forest", "Mountain", "Breeding Pool"]),
        ),
        (
            "Grasslands",
            lands,
            Box::new(activated("Grasslands", "Search", "")),
            names(&["Forest", "Plains", "Breeding Pool"]),
        ),
        (
            "Wirewood Guardian",
            lands,
            Box::new(|t: &mut TestGame| {
                let c = t.hand(P0, "Wirewood Guardian");
                mana(t, P0, "2");
                t.answer_choose(P0, &[]);
                t.activate(P0, c, 0, &[]).unwrap();
            }),
            names(&["Forest", "Breeding Pool"]),
        ),
        (
            "Welkin Hawk",
            &["Welkin Hawk", "Forest"],
            Box::new(|t: &mut TestGame| {
                let h = t.battlefield(P0, "Welkin Hawk");
                t.answer_yes(P0, true);
                t.answer_choose(P0, &[]);
                destroy(t, h);
            }),
            names(&["Welkin Hawk"]),
        ),
        (
            "Skyshroud Poacher",
            &["Llanowar Sentinel", "Forest"],
            Box::new(activated("Skyshroud Poacher", "Search", "3")),
            names(&["Llanowar Sentinel"]),
        ),
        (
            "Academy Rector",
            &["Endless Horizons", "Forest"],
            Box::new(|t: &mut TestGame| {
                let r = t.battlefield(P0, "Academy Rector");
                t.answer_yes(P0, true);
                t.answer_choose(P0, &[]);
                destroy(t, r);
            }),
            names(&["Endless Horizons"]),
        ),
        (
            "Firemind's Foresight",
            &["Cancel", "Counterspell", "Shock"],
            Box::new(|t: &mut TestGame| {
                for _ in 0..3 {
                    t.answer_choose(P0, &[]);
                }
                cast(t, P0, "Firemind's Foresight", "5UR", &[]);
            }),
            names(&["Cancel"]),
        ),
    ];
    for (name, lib, run, want) in cases {
        supported(name);
        let (t, o) = offered(lib, run);
        assert!(!o.is_empty(), "{name}: no search choice");
        assert_eq!(o[0].names, want, "{name}");
        assert_eq!(o[0].min, 0, "{name}: must find a card");
        assert_eq!(t.library_size(P0), lib.len(), "{name}: a card was found");
        assert_eq!(shuffles(&t, P0), 1, "{name}: the library is shuffled");
    }
}

#[test]
fn pattern_of_rebirth_search_is_optional() {
    cr!("603.5", "701.23a");
    ruling!(
        "Pattern of Rebirth",
        "Searching for a creature card is optional."
    );
    // The enchanted creature dies: its controller is asked, and searches only on a yes.
    for search in [true, false] {
        let mut t = TestGame::new(2);
        let lib = set_library(&mut t, P0, &["Grizzly Bears"]);
        let sentinel = t.battlefield(P0, "Llanowar Sentinel");
        crate::r_s06_common::attach_new(&mut t, P0, "Pattern of Rebirth", obj(sentinel));
        t.answer_yes(P0, search);
        t.answer_choose(P0, &[obj(lib[0])]);
        let from = t.asked().len();
        destroy(&mut t, sentinel);
        t.resolve_all();
        assert!(
            crate::r_p152_common::asked_yes_no(&t, P0, from),
            "the search is offered"
        );
        let n = usize::from(search);
        assert_eq!(
            (searches(&t, P0), shuffles(&t, P0)),
            (n, n),
            "search: {search}"
        );
        assert_eq!(
            t.on_battlefield(t.g.current(lib[0])),
            search,
            "search: {search}"
        );
    }
}

#[test]
fn searches_by_subtype_or_name_find_exactly_those_cards() {
    cr!("701.23a", "205.3m");
    ruling!(
        "Self-Assembler",
        "Self-Assembler's ability can find any creature card with the Assembly-Worker subtype, not only creature cards named Assembly-Worker. Notably, it can't find Mishra's Factory."
    );
    ruling!(
        "Fugitive of the Judoon",
        "The last ability can find any card with the Doctor creature type, not just a card with \"Doctor\" in the name."
    );
    ruling!(
        "Bogbrew Witch",
        "You don’t have to declare which card you’re searching for when you activate Bogbrew Witch’s ability. You may find either while you’re searching."
    );
    ruling!(
        "Academy Rector",
        "An enchantment card is any Enchantment card, including Auras."
    );
    let (_, o) = offered(
        &[
            "Mishra's Self-Replicator",
            "Mishra's Factory",
            "Self-Assembler",
        ],
        enters("Self-Assembler"),
    );
    assert_eq!(
        o[0].names,
        names(&["Mishra's Self-Replicator", "Self-Assembler"])
    );
    let (_, o) = offered(
        &["Festering Newt", "Bubbling Cauldron", "Grizzly Bears"],
        activated("Bogbrew Witch", "Search", "2"),
    );
    assert_eq!(o[0].names, names(&["Festering Newt", "Bubbling Cauldron"]));
    // Fugitive of the Judoon's chapter III: a Human and an artifact are exiled. Cult Healer
    // (Human Doctor) is a Doctor card without "Doctor" in its name; Doctor Spectrum (Human
    // Hero) has it in its name but isn't a Doctor.
    let (_, o) = offered(
        &["The Tenth Doctor", "Cult Healer", "Doctor Spectrum"],
        |t| {
            t.battlefield(P0, "Elite Vanguard");
            t.battlefield(P0, "Mind Stone");
            let saga = t.battlefield(P0, "Fugitive of the Judoon");
            t.g.objects[saga.0 as usize]
                .counters
                .insert(counters::LORE.into(), 2);
            t.answer_yes(P0, true);
            t.answer_choose(P0, &[]);
            crate::r_p003_common::put(t, saga, counters::LORE, 1);
        },
    );
    assert_eq!(
        o.last().unwrap().names,
        names(&["Cult Healer", "The Tenth Doctor"])
    );
    // Academy Rector finds an Aura, which enters attached to a creature.
    let mut t = TestGame::new(2);
    let lib = set_library(&mut t, P0, &["Holy Strength"]);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let rector = t.battlefield(P0, "Academy Rector");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(lib[0])]);
    destroy(&mut t, rector);
    t.resolve_all();
    let aura = t.g.current(lib[0]);
    assert!(t.on_battlefield(aura));
    assert_eq!(t.g.obj(aura).attached_to, Some(obj(bears)));
}

#[test]
fn mask_of_the_mimic_looks_for_the_name_whatever_the_type() {
    cr!("701.23a", "201.2a");
    ruling!(
        "Mask of the Mimic",
        "You look for a card of the same name, the current type of the card does not matter."
    );
    let mut t = TestGame::new(2);
    let lib = set_library(&mut t, P0, &["Mishra's Factory", "Grizzly Bears"]);
    // P1's Mishra's Factory is a 2/2 Assembly-Worker creature until end of turn.
    let factory = t.battlefield(P1, "Mishra's Factory");
    add_mana(&mut t, P1, ManaType::C, 1);
    activate_containing(&mut t, P1, factory, "becomes").unwrap();
    t.resolve_all();
    let bears = t.battlefield(P0, "Grizzly Bears");
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(bears)]);
    t.answer_choose(P0, &[obj(lib[0])]);
    cast(&mut t, P0, "Mask of the Mimic", "U", &[obj(factory)]);
    t.resolve_all();
    assert_eq!(offer(&t, P0, from).names, names(&["Mishra's Factory"]));
    let found = t.g.current(lib[0]);
    assert!(t.on_battlefield(found) && t.g.obj(found).controller == P0);
}

#[test]
fn lotuslight_dancers_finds_multicolored_cards_and_needs_not_find_all() {
    cr!("701.23a", "701.23b", "105.2");
    ruling!(
        "Lotuslight Dancers",
        "You can find multicolored cards with Lotuslight Dancers’s last ability."
    );
    ruling!(
        "Lotuslight Dancers",
        "You don’t have to find all three cards. You can find two, one, or (if you really want) none."
    );
    let mut t = TestGame::new(2);
    let lib = set_library(
        &mut t,
        P0,
        &["Lotuslight Dancers", "Grizzly Bears", "Merfolk Looter"],
    );
    // The black-green-blue card is the black card, the Bears the green; no blue card.
    t.answer_choose(P0, &[obj(lib[0])]);
    t.answer_choose(P0, &[obj(lib[1])]);
    t.answer_choose(P0, &[]);
    let from = t.asked().len();
    t.enter(P0, "Lotuslight Dancers");
    t.resolve_all();
    let o = offers(&t, P0, from);
    assert_eq!(o.len(), 3, "{o:?}");
    assert_eq!(o[0].names, names(&["Lotuslight Dancers"]));
    assert_eq!(o[1].names, names(&["Grizzly Bears"]));
    assert_eq!(o[2].names, names(&["Merfolk Looter"]));
    assert!(o.iter().all(|x| x.min == 0));
    assert!(t.in_graveyard(P0, "Lotuslight Dancers") && t.in_graveyard(P0, "Grizzly Bears"));
    assert!(!t.in_graveyard(P0, "Merfolk Looter"));
}

#[test]
fn x_is_zero_for_cards_in_a_library() {
    cr!("202.3e", "701.23a");
    ruling!(
        "Firemind's Foresight",
        "If a card in your library has {X} in its mana cost, X is 0. For example, a card with mana cost {X}{R} has a mana value of 1, and so on."
    );
    ruling!(
        "The Hunger Tide Rises",
        "If a card in a player's library has {X} in its mana cost, X is 0 when determining its mana value."
    );
    ruling!(
        "Reshape",
        "If a card in a player's library has {X} in its mana cost, X is considered to be 0 for that card."
    );
    ruling!(
        "Tribute Mage",
        "If a card in your library has X in its mana cost, X is considered to be 0."
    );
    // Firemind's Foresight: Power Sink ({X}{U}) is the instant with mana value 1.
    let (_, o) = offered(&["Power Sink", "Cancel", "Fireball"], |t| {
        for _ in 0..3 {
            t.answer_choose(P0, &[]);
        }
        cast(t, P0, "Firemind's Foresight", "5UR", &[]);
    });
    assert_eq!(o.last().unwrap().names, names(&["Power Sink"]));
    // Reshape with X = 0: Chalice of the Void ({X}{X}) has mana value 0.
    let (_, o) = offered(&["Chalice of the Void", "Mind Stone"], |t| {
        let a = t.battlefield(P0, "Darksteel Relic");
        t.answer_choose(P0, &[obj(a)]);
        t.answer_choose(P0, &[]);
        let r = t.hand(P0, "Reshape");
        mana(t, P0, "UU");
        t.cast(P0, r).x(0).go();
    });
    assert_eq!(o[0].names, names(&["Chalice of the Void"]));
    // Tribute Mage: mana value 2 exactly, so not Chalice of the Void.
    let (_, o) = offered(
        &["Chalice of the Void", "Mind Stone"],
        enters("Tribute Mage"),
    );
    assert_eq!(o[0].names, names(&["Mind Stone"]));
    // The Hunger Tide Rises' chapter IV with no creatures sacrificed: mana value 0 or less.
    let (_, o) = offered(&["Marketback Walker", "Grizzly Bears"], |t| {
        let saga = t.battlefield(P0, "The Hunger Tide Rises");
        t.g.objects[saga.0 as usize]
            .counters
            .insert(counters::LORE.into(), 3);
        t.answer_choose(P0, &[]);
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[]);
        crate::r_p003_common::put(t, saga, counters::LORE, 1);
    });
    assert_eq!(o.last().unwrap().names, names(&["Marketback Walker"]));
}

#[test]
fn wild_pair_compares_total_power_and_toughness_as_the_creature_is_on_the_battlefield() {
    cr!("701.23a", "608.2h");
    ruling!(
        "Wild Pair",
        "If you cast a 2/2 Grizzly Bears, for example, you may then search for any creature with total power and toughness 4. It can be 0/4, 1/3, 2/2, or 3/1."
    );
    ruling!(
        "Wild Pair",
        "a Triskelion on the battlefield is 4/4, but a Triskelion in your library is 1/1"
    );
    ruling!(
        "Wild Pair",
        "The power and toughness might also have been affected while the triggered ability was on the stack (such as with Giant Growth, Sudden Death, or Ovinize)."
    );
    ruling!(
        "Wild Pair",
        "If the creature that entered has left the battlefield by the time the ability resolves, use the power and toughness of that creature as it last appeared before leaving the battlefield."
    );
    let lib = &[
        "Yoked Ox",
        "Maritime Guard",
        "Grizzly Bears",
        "Oreskos Swiftclaw",
        "Hill Giant",
        "Triskelion",
        "Serra Angel",
        "Craw Wurm",
    ];
    let cast_creature = |name: &'static str, cost: &'static str| {
        move |t: &mut TestGame| {
            t.battlefield(P0, "Wild Pair");
            t.answer_yes(P0, true);
            t.answer_choose(P0, &[]);
            cast(t, P0, name, cost, &[]);
            t.resolve();
        }
    };
    let (_, o) = offered(lib, cast_creature("Grizzly Bears", "1G"));
    assert_eq!(
        o[0].names,
        names(&[
            "Yoked Ox",
            "Maritime Guard",
            "Grizzly Bears",
            "Oreskos Swiftclaw"
        ])
    );
    // Triskelion enters with three +1/+1 counters: total 8.
    let (_, o) = offered(lib, cast_creature("Triskelion", "6"));
    assert_eq!(o[0].names, names(&["Serra Angel"]));
    // Bears pumped by Giant Growth while the trigger waits (5/5), then destroyed: 10.
    for dies in [false, true] {
        let (_, o) = offered(lib, |t| {
            t.battlefield(P0, "Wild Pair");
            t.answer_yes(P0, true);
            t.answer_choose(P0, &[]);
            cast(t, P0, "Grizzly Bears", "1G", &[]);
            t.resolve();
            t.settle();
            let bears = t.named_on_battlefield("Grizzly Bears")[0];
            cast(t, P0, "Giant Growth", "G", &[obj(bears)]);
            t.g.resolve_top();
            if dies {
                destroy(t, bears);
            }
        });
        assert_eq!(o[0].names, names(&["Craw Wurm"]), "dies: {dies}");
    }
}

#[test]
fn boreas_charger_finds_up_to_the_difference() {
    cr!("701.23a", "701.23b");
    ruling!(
        "Boreas Charger",
        "You can find fewer Plains cards than the difference, perhaps because you don't have that many Plains cards left in your library."
    );
    ruling!(
        "Boreas Charger",
        "If no opponent controls more lands than you, you'll search your library and shuffle it, but you won't be able to find any Plains cards."
    );
    let mut t = TestGame::new(2);
    let lib = set_library(&mut t, P0, &["Plains", "Forest"]);
    t.lands(P1, "Mountain", 3);
    let charger = t.battlefield(P0, "Boreas Charger");
    destroy(&mut t, charger);
    t.resolve_all();
    assert!(
        tapped(&t, lib[0]),
        "the one Plains is put onto the battlefield"
    );
    assert_eq!(shuffles(&t, P0), 1);
    // No opponent controls more lands.
    let mut t = TestGame::new(2);
    set_library(&mut t, P0, &["Plains", "Forest"]);
    let charger = t.battlefield(P0, "Boreas Charger");
    let from = t.asked().len();
    destroy(&mut t, charger);
    t.resolve_all();
    assert!(offers(&t, P0, from).is_empty());
    assert_eq!(t.library_size(P0), 2);
    assert_eq!(searches(&t, P0), 1);
    assert_eq!(shuffles(&t, P0), 1);
}

#[test]
fn searching_and_shuffling_happen_even_when_nothing_is_found() {
    cr!("701.23a", "701.23b", "701.24a");
    ruling!(
        "Collective Voyage",
        "You may find any number of basic lands up to X, including zero. However, searching and shuffling your library is not optional."
    );
    ruling!(
        "Land Tax",
        "You shuffle your library if you search, even if you don't put any basic land cards into your hand."
    );
    ruling!(
        "Nissa Revane",
        "You may activate Nissa’s first ability even if you know you have no cards named Nissa’s Chosen in your library. You must still shuffle your library in that situation."
    );
    ruling!(
        "Screaming Seahawk",
        "The \"If you do\" means \"If you search\"."
    );
    // Collective Voyage: P0 pays two, P1 nothing.
    let mut t = TestGame::new(2);
    set_library(&mut t, P0, &["Forest", "Plains", "Island"]);
    t.answer(P0, DecisionKind::X, mtg_engine::decision::Answer::Number(2));
    t.answer(P1, DecisionKind::X, mtg_engine::decision::Answer::Number(0));
    mana(&mut t, P0, "2");
    let from = t.asked().len();
    t.answer_choose(P0, &[]);
    cast(&mut t, P0, "Collective Voyage", "G", &[]);
    t.resolve_all();
    let o = offer(&t, P0, from);
    assert_eq!((o.min, o.max), (0, 2));
    assert!(
        !t.asked()[from..]
            .iter()
            .any(|(_, d)| matches!(d, mtg_engine::decision::Decision::YesNo { .. })),
        "searching isn't optional"
    );
    assert_eq!((searches(&t, P0), shuffles(&t, P0)), (1, 1));
    assert_eq!((searches(&t, P1), shuffles(&t, P1)), (1, 1));
    // Land Tax: the search finds nothing; the library is still shuffled.
    let mut t = TestGame::new(2);
    set_library(&mut t, P0, &["Forest", "Plains"]);
    t.battlefield(P0, "Land Tax");
    t.lands(P1, "Mountain", 2);
    t.set_step(P0, mtg_engine::turn::Step::Untap);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[]);
    t.advance_to(P0, mtg_engine::turn::Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!((searches(&t, P0), shuffles(&t, P0)), (1, 1));
    // Nissa Revane's first ability with no Nissa's Chosen.
    let mut t = TestGame::new(2);
    set_library(&mut t, P0, &["Forest"]);
    let nissa = t.battlefield(P0, "Nissa Revane");
    activate_resolve(&mut t, P0, nissa, "Nissa's Chosen", "", &[]);
    assert_eq!(t.library_size(P0), 1);
    assert_eq!((searches(&t, P0), shuffles(&t, P0)), (1, 1));
    // Screaming Seahawk: searching (finding nothing) shuffles; declining doesn't search.
    for search in [true, false] {
        let mut t = TestGame::new(2);
        set_library(&mut t, P0, &["Screaming Seahawk"]);
        t.answer_yes(P0, search);
        t.answer_choose(P0, &[]);
        t.enter(P0, "Screaming Seahawk");
        t.resolve_all();
        let n = usize::from(search);
        assert_eq!(
            (searches(&t, P0), shuffles(&t, P0)),
            (n, n),
            "search: {search}"
        );
    }
}

#[test]
fn nissa_revanes_second_ability_with_no_elves() {
    cr!("606.3", "119.3");
    ruling!(
        "Nissa Revane",
        "You may activate Nissa’s second ability even if you control no Elves."
    );
    let mut t = TestGame::new(2);
    let nissa = t.battlefield(P0, "Nissa Revane");
    activate_resolve(&mut t, P0, nissa, "gain 2 life", "", &[]);
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.counters(nissa, counters::LOYALTY), 3);
}

#[test]
fn dark_supplicant_chooses_the_zones_and_shuffles_only_if_it_searched_the_library() {
    cr!("701.23a", "701.24a");
    ruling!(
        "Dark Supplicant",
        "You can choose which and how many of the three zones (graveyard, hand, and library) you want to search."
    );
    ruling!(
        "Dark Supplicant",
        "You only shuffle your library if you choose to search it."
    );
    // (where Scion of Darkness is, whether P0 searches the library too)
    for (zone, library) in [
        ("hand", false),
        ("hand", true),
        ("graveyard", false),
        ("library", true),
    ] {
        let mut t = TestGame::new(2);
        let lib = set_library(&mut t, P0, &["Forest"]);
        let scion = match zone {
            "hand" => t.hand(P0, "Scion of Darkness"),
            "graveyard" => t.graveyard(P0, "Scion of Darkness"),
            _ => t.library_top(P0, "Scion of Darkness"),
        };
        let s = t.battlefield(P0, "Dark Supplicant");
        let a = t.battlefield(P0, "Dark Supplicant");
        let b = t.battlefield(P0, "Dark Supplicant");
        t.answer_choose(P0, &[obj(s), obj(a), obj(b)]);
        t.answer_yes(P0, library);
        t.answer_choose(P0, &[obj(scion)]);
        let from = t.asked().len();
        activate_resolve(&mut t, P0, s, "Search", "", &[]);
        assert!(
            crate::r_p152_common::asked_yes_no(&t, P0, from),
            "the library is a choice"
        );
        let scion = t.g.current(scion);
        assert!(
            t.on_battlefield(scion),
            "found in the {zone} (library: {library})"
        );
        let n = usize::from(library);
        assert_eq!(
            (searches(&t, P0), shuffles(&t, P0)),
            (n, n),
            "{zone} {library}"
        );
        assert_eq!(t.zone(lib[0]), Zone::Library(P0), "the Forest stays");
    }
}
