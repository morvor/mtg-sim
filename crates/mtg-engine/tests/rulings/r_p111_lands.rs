//! Rulings batch P111 — nonbasic lands with basic land types (CR 205.3i, 205.4, 305.6–305.8):
//! they're affected by what names their land types but not by what asks for basic lands;
//! land type changes replace the old types (CR 305.7) while text changes edit one word
//! (CR 612.1); a land creature keeps its creature type and color; a land with flash
//! (Dryad Arbor under Teferi) ignores the timing rules for land plays only.

use crate::r_s01_common::supported;
use crate::r_s02_common::{can_play_land, target_candidates};
use crate::r_s20_common::tap_for_mana;
use crate::r_s24_common::pool;
use crate::r_s25_common::{cast_new, lands_for_cost};
use crate::r_s32_common::{basic_now, subtypes_now};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// The candidates of every choice of objects asked since decision `from`.
fn all_candidates(t: &TestGame, from: usize) -> Vec<Entity> {
    t.asked()[from..]
        .iter()
        .flat_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => candidates.clone(),
            _ => vec![],
        })
        .collect()
}

/// The mana `p` gets by tapping `land` with its ability whose text contains `needle`, or
/// nothing if that ability can't be activated.
fn mana_from(t: &mut TestGame, p: PlayerId, land: ObjectId, needle: &str) -> Vec<ManaType> {
    t.g.players[p.idx()].mana_pool = Default::default();
    let land = t.g.current(land);
    t.g.objects[land.0 as usize].tapped = false;
    if !tap_for_mana(t, p, land, needle) {
        return vec![];
    }
    ManaType::ALL
        .iter()
        .flat_map(|ty| std::iter::repeat_n(*ty, pool(t, p, *ty) as usize))
        .collect()
}

/// Every kind of mana the land's mana abilities can make now.
fn mana_kinds(t: &mut TestGame, p: PlayerId, land: ObjectId) -> Vec<ManaType> {
    let mut out = vec![];
    for needle in ["{W}", "{U}", "{B}", "{R}", "{G}"] {
        let has = t.obj_now(land).chars.abilities.iter().any(|a| {
            matches!(&a.kind, mtg_engine::ability::AbilityKind::Activated(x) if x.is_mana_ability)
                && a.text.contains(needle)
        });
        if has {
            for m in mana_from(t, p, land, needle) {
                if !out.contains(&m) {
                    out.push(m);
                }
            }
        }
    }
    out.sort_by_key(|m| *m as u8);
    out
}

#[test]
fn a_battle_land_is_a_forest_for_destroy_target_forest_but_not_a_basic_land() {
    cr!("205.3i", "205.4c", "305.8", "115.1");
    ruling!(
        "Canopy Vista",
        "However, because these cards have basic land types, effects that specify a basic land type without also specifying that the land be basic can affect them."
    );
    supported("Canopy Vista");
    supported("Nissa's Defeat");
    supported("Tend the Sprigs");
    // Nissa's Defeat: "Destroy target Forest, green enchantment, or green planeswalker."
    let mut t = TestGame::new(2);
    let vista = t.battlefield(P1, "Canopy Vista");
    let from = t.asked().len();
    cast_new(&mut t, P0, "Nissa's Defeat", &[Entity::Object(vista)]);
    assert!(target_candidates(&t, P0, from)
        .iter()
        .any(|c| c.contains(&Entity::Object(vista))));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Canopy Vista"));
    // Tend the Sprigs: "Search your library for a basic land card": it can't find it.
    let mut t = TestGame::new(2);
    let vista = t.library_top(P0, "Canopy Vista");
    let forest = t.library_top(P0, "Forest");
    let from = t.asked().len();
    cast_new(&mut t, P0, "Tend the Sprigs", &[]);
    t.resolve_all();
    let offered = all_candidates(&t, from);
    assert!(offered.contains(&Entity::Object(forest)), "positive control");
    assert!(!offered.contains(&Entity::Object(vista)));
}

#[test]
fn dual_lands_with_two_basic_land_types_arent_basic_but_have_the_types() {
    cr!("205.3i", "205.4c", "305.8", "701.23a", "614.1c");
    ruling!(
        "Temple Garden",
        "It's not basic, so cards such as Tend the Sprigs can't find it, but it does have the appropriate land types for effects such as that of Great Arashin City"
    );
    ruling!(
        "Coastal Peak",
        "It's not basic, so effects that search for basic lands can't find it, but it does have the appropriate land types for effects such as that of Frostboil Snarl."
    );
    ruling!(
        "Eclipsed Steppe",
        "It's not basic, so effects that search for basic lands can't find it, but it does have the appropriate land types for effects such as that of Shineshadow Snarl"
    );
    ruling!(
        "Festering Thicket",
        "It’s not basic, so effects that search for basic lands (such as the ability of a Lander token) can’t find it, but it does have the appropriate land types for effects such as that of Wastewood Verge"
    );
    ruling!(
        "Radiant Summit",
        "It’s not basic, so effects that search for basic lands (such as the ability of a Lander token) can’t find it, but it does have the appropriate land types for effects such as that of Sunbillow Verge"
    );
    ruling!(
        "Scorched Geyser",
        "It's not basic, so effects that search for basic lands can't find it, but it does have the appropriate land types for effects such as that of Frostboil Snarl"
    );
    ruling!(
        "Umbral Expanse",
        "It's not basic, so effects that search for basic lands can't find it, but it does have the appropriate land types for effects such as that of Shineshadow Snarl"
    );
    ruling!(
        "Vernal Fen",
        "It’s not basic, so effects that search for basic lands (such as the ability of a Lander token) can’t find it, but it does have the appropriate land types for effects such as that of Wastewood Verge"
    );
    enum Check {
        /// A land that enters tapped unless you control a land of its types.
        EntersUntapped(&'static str),
        /// A Snarl: reveal a land card of its types from hand to have it enter untapped.
        Reveal(&'static str),
        /// A Verge: its second mana ability needs a land of its types.
        Verge(&'static str, &'static str),
    }
    let cases = [
        ("Temple Garden", ["Forest", "Plains"], Check::EntersUntapped("Great Arashin City")),
        ("Coastal Peak", ["Island", "Mountain"], Check::Reveal("Frostboil Snarl")),
        ("Eclipsed Steppe", ["Plains", "Swamp"], Check::Reveal("Shineshadow Snarl")),
        ("Festering Thicket", ["Swamp", "Forest"], Check::Verge("Wastewood Verge", "{B}")),
        ("Radiant Summit", ["Mountain", "Plains"], Check::Verge("Sunbillow Verge", "{R}")),
        ("Scorched Geyser", ["Island", "Mountain"], Check::Reveal("Frostboil Snarl")),
        ("Umbral Expanse", ["Plains", "Swamp"], Check::Reveal("Shineshadow Snarl")),
        ("Vernal Fen", ["Swamp", "Forest"], Check::Verge("Wastewood Verge", "{B}")),
    ];
    supported("Tend the Sprigs");
    for (land, types, check) in cases {
        supported(land);
        // Not basic: a search for a basic land card can't find it.
        let mut t = TestGame::new(2);
        let it = t.library_top(P0, land);
        let forest = t.library_top(P0, "Forest");
        let from = t.asked().len();
        cast_new(&mut t, P0, "Tend the Sprigs", &[]);
        t.resolve_all();
        let offered = all_candidates(&t, from);
        assert!(offered.contains(&Entity::Object(forest)), "{land}: control");
        assert!(!offered.contains(&Entity::Object(it)), "{land} was offered");
        assert!(!basic_now(&t, it));
        let mut sub = subtypes_now(&t, it);
        sub.sort();
        let mut want = types.map(String::from).to_vec();
        want.sort();
        assert_eq!(sub, want, "{land}");
        // But it has the land types.
        let mut t = TestGame::new(2);
        match check {
            Check::EntersUntapped(other) => {
                supported(other);
                let x = t.enter(P0, other);
                assert!(t.obj_now(x).tapped, "{other}: control");
                t.battlefield(P0, land);
                let x = t.enter(P0, other);
                assert!(!t.obj_now(x).tapped, "{other} with {land}");
            }
            Check::Reveal(snarl) => {
                supported(snarl);
                let c = t.hand(P0, land);
                t.answer_yes(P0, true);
                t.answer_choose(P0, &[Entity::Object(c)]);
                let x = t.enter(P0, snarl);
                assert!(!t.obj_now(x).tapped, "{snarl} revealing {land}");
                let x = t.enter(P0, snarl);
                assert!(t.obj_now(x).tapped, "{snarl}: control (nothing to reveal)");
            }
            Check::Verge(verge, needle) => {
                supported(verge);
                let v = t.battlefield(P0, verge);
                assert!(mana_from(&mut t, P0, v, needle).is_empty(), "{verge}: control");
                t.battlefield(P0, land);
                assert_eq!(mana_from(&mut t, P0, v, needle).len(), 1, "{verge} with {land}");
            }
        }
    }
}

/// Sea's Claim ("Enchanted land is an Island") cast on `land`, resolved.
fn seas_claim(t: &mut TestGame, land: ObjectId) {
    supported("Sea's Claim");
    let c = t.hand(P0, "Sea's Claim");
    t.g.players[0].mana_pool.add_type(ManaType::U, 1);
    t.cast(P0, c).target(land).go();
    t.resolve_all();
    assert!(t.on_battlefield(c), "Sea's Claim didn't resolve");
}

#[test]
fn dryad_arbor_changed_into_an_island_is_still_a_green_dryad_creature() {
    cr!("305.7", "205.3i", "613.1d", "105.2");
    ruling!(
        "Dryad Arbor",
        "If Dryad Arbor is changed into another basic land type (such as by Sea's Claim), it continues to be a creature and a Dryad."
    );
    ruling!(
        "Dryad Arbor",
        "If Dryad Arbor is changed into another basic land type, it continues to be a green Dryad creature."
    );
    supported("Dryad Arbor");
    let mut t = TestGame::new(2);
    let arbor = t.battlefield(P0, "Dryad Arbor");
    assert_eq!(mana_kinds(&mut t, P0, arbor), vec![ManaType::G]);
    seas_claim(&mut t, arbor);
    let o = t.obj_now(arbor);
    assert!(o.chars.has_subtype("Island"));
    assert!(!o.chars.has_subtype("Forest"));
    assert!(o.chars.has_subtype("Dryad"));
    assert!(o.is(CardType::Creature) && o.is(CardType::Land));
    assert_eq!(o.chars.colors, ColorSet::single(Color::Green));
    assert_eq!(t.pt(arbor), (1, 1));
    assert_eq!(mana_kinds(&mut t, P0, arbor), vec![ManaType::U]);
}

#[test]
fn a_land_type_change_replaces_both_types_but_a_text_change_edits_one() {
    cr!("305.7", "612.1", "612.2");
    ruling!(
        "Badlands",
        "Land type changing effects that change a dual land's land type will remove the old land types completely. Text-changing effects that just change one of the two land types will leave the other type unaffected."
    );
    supported("Badlands");
    supported("Magical Hack");
    let mut t = TestGame::new(2);
    let badlands = t.battlefield(P0, "Badlands");
    assert_eq!(mana_kinds(&mut t, P0, badlands), vec![ManaType::B, ManaType::R]);
    seas_claim(&mut t, badlands);
    assert_eq!(subtypes_now(&t, badlands), vec!["Island"]);
    assert_eq!(mana_kinds(&mut t, P0, badlands), vec![ManaType::U]);
    // Magical Hack: Swamp → Island (the options: the five basic land types, then the
    // four others).
    let mut t = TestGame::new(2);
    let badlands = t.battlefield(P0, "Badlands");
    let hack = t.hand(P0, "Magical Hack");
    t.g.players[0].mana_pool.add_type(ManaType::U, 1);
    t.answer(P0, DecisionKind::Option, Answer::Index(2));
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.cast(P0, hack).target(badlands).go();
    t.resolve_all();
    let mut sub = subtypes_now(&t, badlands);
    sub.sort();
    assert_eq!(sub, vec!["Island", "Mountain"]);
    assert_eq!(mana_kinds(&mut t, P0, badlands), vec![ManaType::U, ManaType::R]);
}

#[test]
fn a_dual_land_has_its_land_types_in_every_zone() {
    cr!("205.3i", "109.2", "614.1c");
    ruling!(
        "Badlands",
        "This card is a Mountain and a Swamp even while in the graveyard, library, or any other zone."
    );
    let mut t = TestGame::new(2);
    let ids = [
        t.graveyard(P0, "Badlands"),
        t.library_top(P0, "Badlands"),
        t.exile(P0, "Badlands"),
        t.hand(P0, "Badlands"),
    ];
    for id in ids {
        let mut sub = subtypes_now(&t, id);
        sub.sort();
        assert_eq!(sub, vec!["Mountain", "Swamp"], "in {:?}", t.zone(id));
    }
    // Frostboil Snarl: "As this land enters, you may reveal an Island or Mountain card
    // from your hand." Badlands in hand is a Mountain card.
    let hand = ids[3];
    assert_eq!(t.zone(hand), Zone::Hand(P0));
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(hand)]);
    let snarl = t.enter(P0, "Frostboil Snarl");
    assert!(!t.obj_now(snarl).tapped);
}

#[test]
fn dryad_arbor_with_flash_ignores_only_the_timing_of_land_plays() {
    cr!("702.8a", "305.1", "305.2b", "116.2a");
    ruling!(
        "Dryad Arbor",
        "you can ignore the normal timing rules for when during your turn you can play a land, but not any other restrictions. You can't play Dryad Arbor during another player's turn, and you can't play Dryad Arbor if you don't have any land plays remaining."
    );
    supported("Teferi, Mage of Zhalfir");
    // Without flash: not in combat.
    let mut t = TestGame::new(2);
    let arbor = t.hand(P0, "Dryad Arbor");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_play_land(&mut t, P0, arbor));
    assert!(t.play_land(P0, arbor).is_err());
    // Teferi: "Creature cards you own that aren't on the battlefield have flash."
    t.battlefield(P0, "Teferi, Mage of Zhalfir");
    t.g.recompute();
    assert!(t.obj_now(arbor).has_keyword(mtg_engine::keywords::KeywordKind::Flash));
    assert!(can_play_land(&mut t, P0, arbor));
    t.play_land(P0, arbor).expect("Dryad Arbor with flash in combat");
    assert!(t.on_battlefield(arbor));
    assert_eq!(t.g.player(P0).lands_played_this_turn, 1, "it uses a land play");
    // With a spell on the stack in the main phase.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Teferi, Mage of Zhalfir");
    let arbor = t.hand(P0, "Dryad Arbor");
    cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    assert_eq!(t.stack_len(), 1);
    assert!(can_play_land(&mut t, P0, arbor));
    t.play_land(P0, arbor).expect("Dryad Arbor with flash, stack not empty");
    assert!(t.on_battlefield(arbor));
    // No land play left.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Teferi, Mage of Zhalfir");
    let forest = t.hand(P0, "Forest");
    t.play_land(P0, forest).unwrap();
    let arbor = t.hand(P0, "Dryad Arbor");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_play_land(&mut t, P0, arbor));
    assert!(t.play_land(P0, arbor).is_err());
    // Another player's turn.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Teferi, Mage of Zhalfir");
    let arbor = t.hand(P0, "Dryad Arbor");
    t.set_step(P1, Step::PrecombatMain);
    t.g.turn.priority = Some(P0);
    assert!(!can_play_land(&mut t, P0, arbor));
    assert!(t.play_land(P0, arbor).is_err());
}

#[test]
fn permissions_to_cast_spells_as_though_they_had_flash_dont_apply_to_land_plays() {
    cr!("305.1", "305.9", "702.8a");
    supported("Vedalken Orrery");
    // "You may cast spells as though they had flash": a land is played, never cast.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vedalken Orrery");
    let arbor = t.hand(P0, "Dryad Arbor");
    let forest = t.hand(P0, "Forest");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_play_land(&mut t, P0, arbor));
    assert!(!can_play_land(&mut t, P0, forest));
    assert!(t.play_land(P0, arbor).is_err());
    // Control: the Orrery does let a creature spell be cast then.
    lands_for_cost(&mut t, P0, "Grizzly Bears");
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).try_go().expect("Orrery lets a creature be cast in combat");
}

#[test]
fn scouts_warning_lets_dryad_arbor_be_played_in_combat_and_is_used_up() {
    cr!("305.1", "305.3", "702.8a");
    ruling!(
        "Scout's Warning",
        "You cannot use this effect to play Dryad Arbor during an opponent's turn."
    );
    supported("Scout's Warning");
    // "The next creature card you play this turn can be played as though it had flash."
    let mut t = TestGame::new(2);
    let arbor = t.hand(P0, "Dryad Arbor");
    cast_new(&mut t, P0, "Scout's Warning", &[]);
    t.resolve_all();
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(can_play_land(&mut t, P0, arbor));
    t.play_land(P0, arbor).expect("Dryad Arbor under Scout's Warning in combat");
    assert!(t.on_battlefield(arbor));
    assert_eq!(t.g.player(P0).lands_played_this_turn, 1);
    // Playing Dryad Arbor used the effect up: the next creature card can't be cast now.
    lands_for_cost(&mut t, P0, "Grizzly Bears");
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(t.cast(P0, bears).try_go().is_err(), "the effect is used up");
    // Control: without the land play, the effect lets the creature be cast in combat.
    let mut t = TestGame::new(2);
    cast_new(&mut t, P0, "Scout's Warning", &[]);
    t.resolve_all();
    t.set_step(P0, Step::BeginningOfCombat);
    lands_for_cost(&mut t, P0, "Grizzly Bears");
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).try_go().expect("Scout's Warning gives the creature flash");
    // Not during an opponent's turn.
    let mut t = TestGame::new(2);
    let arbor = t.hand(P0, "Dryad Arbor");
    t.set_step(P1, Step::PrecombatMain);
    t.g.turn.priority = Some(P0);
    lands_for_cost(&mut t, P0, "Scout's Warning");
    let sw = t.hand(P0, "Scout's Warning");
    t.cast(P0, sw).try_go().expect("Scout's Warning is an instant");
    t.resolve_all();
    t.g.turn.priority = Some(P0);
    assert!(!can_play_land(&mut t, P0, arbor));
    assert!(t.play_land(P0, arbor).is_err());
}
