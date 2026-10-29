//! Rulings batch P185 — typal lords: marked damage that becomes lethal when a lord leaves
//! (CR 704.5g, 120.6), a creature with two of the named types gets the bonus once (CR
//! 205.3, 613), "each other" lords and Sliver lords that also pump themselves, counts of
//! creature types made as an ability resolves (CR 608.2h), and type-adding effects that
//! apply only on the battlefield (CR 611.3a).

use crate::r_p185_common::*;
use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s17_common::{face, transform};
use mtg_engine::object::{FaceState, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn lord_leaving_makes_marked_damage_lethal() {
    cr!("704.5g", "120.6", "611.3a");
    ruling!(
        "Valiant Knight",
        "Because damage remains marked on a creature until it's removed as the turn ends, nonlethal damage dealt to a Knight you control may become lethal if Valiant Knight leaves the battlefield during that turn."
    );
    ruling!(
        "Nightpack Ambusher",
        "Because damage remains marked on a creature until it’s removed as the turn ends, nonlethal damage dealt to other Wolves and Werewolves you control may become lethal if Nightpack Ambusher leaves the battlefield during that turn."
    );
    ruling!(
        "Merfolk Mistbinder",
        "Because damage remains marked on a creature until it's removed as the turn ends, nonlethal damage dealt to another Merfolk you control may become lethal if Merfolk Mistbinder leaves the battlefield during that turn."
    );
    ruling!(
        "Master of the Pearl Trident",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to a Merfolk you control may become lethal if Master of the Pearl Trident leaves the battlefield during that turn."
    );
    ruling!(
        "Might Sliver",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to a Sliver may become lethal if Might Sliver leaves the battlefield during that turn."
    );
    ruling!(
        "Sinew Sliver",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to a Sliver may become lethal if Sinew Sliver leaves the battlefield during that turn."
    );
    ruling!(
        "Muscle Sliver",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to a Sliver may become lethal if Muscle Sliver leaves the battlefield during that turn."
    );
    ruling!(
        "Watcher Sliver",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to another Sliver may become lethal if Watcher Sliver leaves the battlefield during that turn."
    );
    ruling!(
        "Supreme Phantom",
        "Because damage remains marked on a creature until it's removed as the turn ends, nonlethal damage dealt to a Spirit you control may become lethal if Supreme Phantom leaves the battlefield during that turn."
    );
    ruling!(
        "Legion Lieutenant",
        "Because damage remains marked on a creature until it's removed as the turn ends, nonlethal damage dealt to another Vampire you control may become lethal if Legion Lieutenant leaves the battlefield during that turn."
    );
    ruling!(
        "Thirsting Bloodlord",
        "Because damage remains marked on a creature until it's removed as the turn ends, nonlethal damage dealt to other Vampires you control may become lethal if Thirsting Bloodlord leaves the battlefield during that turn."
    );
    // (lord, a creature it pumps, that creature's size with the lord)
    let cases: &[(&str, &str, (i32, i32))] = &[
        ("Valiant Knight", "Craven Knight", (3, 3)),
        ("Nightpack Ambusher", "Watchwolf", (4, 4)),
        ("Merfolk Mistbinder", "Maritime Guard", (2, 4)),
        ("Master of the Pearl Trident", "Maritime Guard", (2, 4)),
        ("Might Sliver", "Venser's Sliver", (5, 5)),
        ("Sinew Sliver", "Venser's Sliver", (4, 4)),
        ("Muscle Sliver", "Venser's Sliver", (4, 4)),
        ("Watcher Sliver", "Venser's Sliver", (3, 5)),
        ("Supreme Phantom", "Gnarled Mass", (4, 4)),
        ("Legion Lieutenant", "Highborn Vampire", (5, 4)),
        ("Thirsting Bloodlord", "Highborn Vampire", (5, 4)),
    ];
    for (lord, member, pt) in cases {
        supported(lord);
        let mut t = TestGame::new(2);
        let l = t.battlefield(P0, lord);
        let m = t.battlefield(P0, member);
        assert_eq!(t.pt(m), *pt, "{lord}");
        lethal_when_it_leaves(&mut t, m, l, lord);
    }
}

#[test]
fn tendershoot_dryad_leaving_makes_marked_damage_on_saprolings_lethal() {
    cr!("704.5g", "702.131b");
    ruling!(
        "Tendershoot Dryad",
        "Because damage remains marked on a creature until it's removed as the turn ends, nonlethal damage dealt to a Saproling you control may become lethal if Tendershoot Dryad leaves the battlefield during that turn."
    );
    supported("Tendershoot Dryad");
    let mut t = TestGame::new(2);
    let dryad = t.battlefield(P0, "Tendershoot Dryad");
    t.lands(P0, "Forest", 8);
    let sap = vanilla(&mut t, P0, "Creature — Saproling", (1, 1));
    t.settle();
    assert!(t.g.players[0].has_citys_blessing);
    assert_eq!(t.pt(sap), (3, 3));
    lethal_when_it_leaves(&mut t, sap, dryad, "Tendershoot Dryad");
}

#[test]
fn sliver_legion_and_squirrel_mob_lose_size_when_another_of_their_type_leaves() {
    cr!("704.5g", "611.3a");
    ruling!(
        "Sliver Legion",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to a Sliver may become lethal if another Sliver leaves the battlefield during that turn."
    );
    ruling!(
        "Squirrel Mob",
        "Because damage remains marked on a creature until the cleanup step or an effect removes that damage, nonlethal damage dealt to Squirrel Mob may become lethal if the number of other Squirrels on the battlefield decreases."
    );
    ruling!(
        "Sliver Legion",
        "If Sliver Legion and two other Slivers are on the battlefield, for example, each of them gets +2/+2."
    );
    supported("Sliver Legion");
    supported("Squirrel Mob");
    let mut t = TestGame::new(2);
    let legion = t.battlefield(P0, "Sliver Legion");
    let a = t.battlefield(P0, "Venser's Sliver");
    // An opponent's Sliver is a Sliver on the battlefield too.
    let b = t.battlefield(P1, "Venser's Sliver");
    assert_eq!(t.pt(legion), (9, 9));
    assert_eq!(t.pt(a), (5, 5));
    assert_eq!(t.pt(b), (5, 5));
    // Sliver Legion isn't the one leaving: another Sliver is.
    lethal_when_it_leaves(&mut t, a, b, "Sliver Legion");
    assert!(t.on_battlefield(legion));

    let mut t = TestGame::new(2);
    let mob = t.battlefield(P0, "Squirrel Mob");
    let other = t.battlefield(P1, "Squirrelanoids");
    assert_eq!(t.pt(mob), (3, 3));
    lethal_when_it_leaves(&mut t, mob, other, "Squirrel Mob");
}

#[test]
fn wolf_and_werewolf_is_counted_and_pumped_once() {
    cr!("613.4c", "608.2h");
    ruling!(
        "Nightpack Ambusher",
        "A creature that’s both a Wolf and a Werewolf gets only +1/+1 from Nightpack Ambusher’s ability."
    );
    ruling!(
        "Immerwolf",
        "A creature that’s both a Wolf and a Werewolf gets the bonus only once."
    );
    ruling!(
        "Runebound Wolf",
        "If you control a creature that is both a Wolf and a Werewolf, it's counted only once for Runebound Wolf's ability."
    );
    ruling!(
        "Runebound Wolf",
        "Runebound Wolf's activated ability counts the number of Wolves and Werewolves you control at the time it resolves, not at the time it was activated."
    );
    for lord in ["Nightpack Ambusher", "Immerwolf"] {
        supported(lord);
        let mut t = TestGame::new(2);
        t.battlefield(P0, lord);
        let both = vanilla(&mut t, P0, "Creature — Wolf Werewolf", (2, 2));
        assert_eq!(t.pt(both), (3, 3), "{lord}");
    }
    supported("Runebound Wolf");
    let mut t = TestGame::new(2);
    let rw = t.battlefield(P0, "Runebound Wolf");
    vanilla(&mut t, P0, "Creature — Wolf Werewolf", (2, 2));
    // Not counted: an opponent's Wolf.
    t.battlefield(P1, "Watchwolf");
    t.lands(P0, "Mountain", 4);
    t.activate(P0, rw, 0, &[Entity::Player(P1)]).unwrap();
    // Another Wolf arrives before the ability resolves: it's counted.
    t.battlefield(P0, "Watchwolf");
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn immerwolf_never_pumps_itself_and_stops_only_non_human_werewolves() {
    cr!("611.3a", "701.27a");
    ruling!(
        "Immerwolf",
        "Immerwolf’s bonus never applies to itself, even if it somehow becomes a Werewolf creature."
    );
    ruling!(
        "Immerwolf",
        "A “non-Human Werewolf” is a creature that is a Werewolf but is not a Human."
    );
    supported("Immerwolf");
    let mut t = TestGame::new(2);
    let imm = t.battlefield(P0, "Immerwolf");
    add_types(&mut t, imm, &["Werewolf"]);
    assert!(t.obj_now(imm).chars.has_subtype("Werewolf"));
    assert_eq!(t.pt(imm), (2, 2));
    // A Human Werewolf (front face) can transform; the non-Human Werewolf on its back
    // can't transform back.
    let shepherd = t.battlefield(P0, "Gatstaf Shepherd // Gatstaf Howler");
    assert_eq!(t.pt(shepherd), (3, 3), "a Human Werewolf is a Werewolf");
    transform(&mut t, shepherd);
    assert_eq!(face(&t, shepherd), FaceState::Back);
    assert!(!t.obj_now(shepherd).chars.has_subtype("Human"));
    transform(&mut t, shepherd);
    assert_eq!(face(&t, shepherd), FaceState::Back);
    // Without Immerwolf it would.
    destroy(&mut t, imm);
    transform(&mut t, shepherd);
    assert_eq!(face(&t, shepherd), FaceState::Front);
}

#[test]
fn aven_brigadier_gives_a_bird_soldier_plus_two() {
    cr!("613.4c");
    ruling!(
        "Aven Brigadier",
        "It gives +2/+2 to creatures that are both Birds and Soldiers."
    );
    supported("Aven Brigadier");
    let mut t = TestGame::new(2);
    let brig = t.battlefield(P0, "Aven Brigadier");
    let watcher = t.battlefield(P1, "Coast Watcher");
    assert_eq!(t.pt(watcher), (3, 3));
    assert_eq!(t.pt(brig), (3, 5));
}

#[test]
fn lords_that_pump_their_own_type_pump_themselves() {
    cr!("611.3a", "613.4c");
    ruling!("Sinew Sliver", "Sinew Sliver's ability does give itself +1/+1.");
    ruling!("Muscle Sliver", "Muscle Sliver's ability does give itself +1/+1.");
    ruling!(
        "Anaba Spirit Crafter",
        "Anaba Spirit Crafter’s ability affects all Minotaurs, including Anaba Spirit Crafter itself."
    );
    for (name, pt) in [
        ("Sinew Sliver", (2, 2)),
        ("Muscle Sliver", (2, 2)),
        ("Anaba Spirit Crafter", (2, 3)),
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        let s = t.battlefield(P0, name);
        assert_eq!(t.pt(s), pt, "{name}");
    }
    // Anaba pumps an opponent's Minotaur too.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Anaba Spirit Crafter");
    let mino = vanilla(&mut t, P1, "Creature — Minotaur", (2, 2));
    assert_eq!(t.pt(mino), (3, 2));
}

#[test]
fn deranged_hermits_squirrels_are_bigger_per_hermit_until_it_leaves() {
    cr!("611.3a", "613.4c");
    ruling!(
        "Deranged Hermit",
        "The Squirrel tokens get the bonus, so they are effectively 2/2 creatures until the Hermit leaves the battlefield. And they get bigger if multiple Hermits are on the battlefield."
    );
    supported("Deranged Hermit");
    let mut t = TestGame::new(2);
    let hermit = crate::r_p185_common::enter_with(&mut t, P0, "Deranged Hermit", &[]);
    t.resolve_all();
    let squirrels = with_subtype(&t, P0, "Squirrel");
    assert_eq!(squirrels.len(), 4);
    assert!(squirrels.iter().all(|s| t.pt(*s) == (2, 2)));
    let second = t.battlefield(P0, "Deranged Hermit");
    assert_eq!(t.pt(squirrels[0]), (3, 3));
    destroy(&mut t, hermit);
    destroy(&mut t, second);
    assert_eq!(t.pt(squirrels[0]), (1, 1));
}

#[test]
fn hivestone_affects_only_creatures_on_the_battlefield_and_not_slivers_twice() {
    cr!("611.3a", "613.1d");
    ruling!(
        "Hivestone",
        "Hivestone doesn't affect creature spells or creature cards anywhere other than the battlefield."
    );
    ruling!(
        "Hivestone",
        "Slivers are unaffected by Hivestone; a creature can't have the same creature type twice."
    );
    supported("Hivestone");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hivestone");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(t.obj_now(bears).chars.has_subtype("Sliver"));
    let in_hand = t.hand(P0, "Grizzly Bears");
    let in_yard = t.graveyard(P0, "Grizzly Bears");
    t.g.recompute();
    assert!(!t.obj_now(in_hand).chars.has_subtype("Sliver"));
    assert!(!t.obj_now(in_yard).chars.has_subtype("Sliver"));
    t.lands(P0, "Forest", 2);
    let spell = t.cast(P0, in_hand).go();
    assert_eq!(t.zone(spell), Zone::Stack);
    assert!(!t.obj_now(spell).chars.has_subtype("Sliver"));
    // A Sliver is a Sliver once: Sliver Legion counts the other Slivers, one each (the
    // Venser's Sliver and the Bears), for +2/+2.
    let legion = t.battlefield(P0, "Sliver Legion");
    let vs = t.battlefield(P0, "Venser's Sliver");
    let n = t
        .obj_now(vs)
        .chars
        .subtypes
        .iter()
        .filter(|s| s.as_str() == "Sliver")
        .count();
    assert_eq!(n, 1);
    assert_eq!(t.pt(legion), (9, 9));
}

#[test]
fn nightpack_ambusher_looks_at_every_spell_cast_this_turn() {
    cr!("603.4");
    ruling!(
        "Nightpack Ambusher",
        "If you cast a spell that was countered, Nightpack Ambusher’s last ability doesn’t trigger."
    );
    ruling!(
        "Nightpack Ambusher",
        "Nightpack Ambusher looks at the entire turn to see if you have cast a spell, even if Nightpack Ambusher wasn’t on the battlefield when that spell was cast. Notably, you won’t get a Wolf token during your end step if you cast Nightpack Ambusher during your turn."
    );
    supported("Nightpack Ambusher");
    // 0: no spell (a token); 1: a countered spell; 2: Nightpack Ambusher itself cast.
    for case in 0..3 {
        let mut t = TestGame::new(2);
        if case != 2 {
            t.battlefield(P0, "Nightpack Ambusher");
        }
        if case == 1 {
            let spell =
                crate::r_s25_common::cast_new(&mut t, P0, "Shock", &[Entity::Player(P1)]);
            t.g.counter(spell, None);
            t.g.flush_events();
        }
        if case == 2 {
            crate::r_s25_common::cast_new(&mut t, P0, "Nightpack Ambusher", &[]);
            t.resolve_all();
            assert_eq!(t.named_on_battlefield("Nightpack Ambusher").len(), 1);
        }
        t.advance_to(P0, mtg_engine::turn::Step::End);
        t.resolve_all();
        let wolves = tokens(&t, P0).len();
        assert_eq!(wolves, (case == 0) as usize, "case {case}");
    }
}
