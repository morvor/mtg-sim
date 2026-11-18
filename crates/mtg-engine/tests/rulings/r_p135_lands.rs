//! Rulings batch P135 — land destruction: spells whose target land becomes illegal do
//! nothing (CR 608.2b), the rest of the spell still happens when the land survives
//! (indestructible, regeneration), two-target spells with one target left, which lands
//! can be targeted (snow, {C}, Mountain subtype, Forest, names), mandatory triggered
//! abilities that must target your own land, X = 0, and the opponent's choice of targets.

use crate::r_p108_common::put_counters;
use crate::r_p135_common::*;
use crate::r_s02_common::can_activate;
use crate::r_s25_common::targets_of;
use mtg_engine::ability::Modification;
use mtg_engine::decision::Decision;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// ---------------------------------------------------------------------------------------
// An illegal target: none of the spell's effects happen.
// ---------------------------------------------------------------------------------------

#[test]
fn land_destruction_with_an_illegal_target_does_nothing() {
    cr!("608.2b");
    ruling!("Lithoform Blight", "If the target land is an illegal target by the time Lithoform Blight tries to resolve, the spell doesn't resolve. It doesn't enter the battlefield, so you don't draw a card.");
    ruling!("Molten Rain", "If the target land is an illegal target by the time Molten Rain tries to resolve, the spell doesn't resolve. No player is dealt 2 damage.");
    ruling!("Tectonic Rift", "If the target land is an illegal target by the time Tectonic Rift tries to resolve, the spell doesn’t resolve. Creatures without flying will be able to block as normal.");
    ruling!("Survey the Wreckage", "If the target land is an illegal target when Survey the Wreckage tries to resolve, it won’t resolve and none of its effects will happen. You won’t get the creature token.");
    ruling!("Desecrated Earth", "If the targeted land is an illegal target by the time Desecrated Earth resolves, the entire spell doesn’t resolve. No one discards a card.");
    ruling!("Maw of the Mire", "If the targeted land is an illegal target by the time Maw of the Mire resolves, it won’t resolve and none of its effects will occur. You won’t gain 4 life.");
    ruling!("Melt Terrain", "If the targeted land is an illegal target by the time Melt Terrain resolves, the spell won’t resolve. No damage will be dealt.");
    ruling!("Roiling Terrain", "If the targeted land is an illegal target by the time Roiling Terrain resolves, the entire spell doesn’t resolve. No damage will be dealt.");
    ruling!("Scorch the Fields", "If the targeted land is an illegal target when Scorch the Fields tries to resolve, it won’t resolve and none of its effects will happen. No damage will be dealt to any Human.");
    ruling!("Peak Eruption", "If the target Mountain is an illegal target when Peak Eruption tries to resolve, it won't resolve and none of its effects will happen. No damage will be dealt.");
    for name in [
        "Lithoform Blight",
        "Molten Rain",
        "Tectonic Rift",
        "Survey the Wreckage",
        "Desecrated Earth",
        "Maw of the Mire",
        "Melt Terrain",
        "Roiling Terrain",
        "Scorch the Fields",
        "Peak Eruption",
    ] {
        let mut t = TestGame::new(2);
        // A 2/1 Human (Scorch the Fields), a card in hand (Desecrated Earth), a land card
        // in the graveyard (Roiling Terrain).
        let human = t.battlefield(P1, "Elite Vanguard");
        t.hand(P1, "Grizzly Bears");
        t.graveyard(P1, "Forest");
        let land = if name == "Peak Eruption" {
            t.battlefield(P1, "Stomping Ground")
        } else {
            t.battlefield(P1, "Mutavault")
        };
        let s = cast_modes(&mut t, P0, name, &[], &[ob(land)]);
        fizzles(&mut t, s, |t| kill(t, land));
        t.g.recompute();
        assert!(t.g.can_block_at_all(human), "{name}");
        assert_eq!(dealt(&t, human), 0, "{name}");
    }
}

#[test]
fn land_destruction_that_doesnt_destroy_still_does_the_rest() {
    cr!("608.2c", "702.12b", "701.19c");
    ruling!("Peak Eruption", "However, if Peak Eruption resolves and the Mountain isn't destroyed (perhaps because it has indestructible), damage will be dealt.");
    ruling!("Molten Rain", "If the target is legal but not destroyed (most likely because it has indestructible), its controller is dealt 2 damage.");
    ruling!("Tectonic Rift", "If the target is legal but not destroyed (most likely because it has indestructible), creatures without flying will be unable to block.");
    ruling!(
        "Roiling Terrain",
        "If the targeted land regenerates or has indestructible, Roiling Terrain still deals damage to the targeted land’s controller."
    );
    // (card, target land, P1's life afterwards)
    let cases: [(&str, &str, i32); 3] = [
        ("Peak Eruption", "Mountain", 17),
        ("Molten Rain", "Mutavault", 18),
        ("Tectonic Rift", "Mutavault", 20),
    ];
    for (name, land, life) in cases {
        let mut t = TestGame::new(2);
        let human = t.battlefield(P1, "Elite Vanguard");
        let l = t.battlefield(P1, land);
        indestructible(&mut t, l);
        cast_modes(&mut t, P0, name, &[], &[ob(l)]);
        t.resolve_all();
        assert!(t.on_battlefield(l), "{name}");
        assert_eq!(t.life(P1), life, "{name}");
        if name == "Tectonic Rift" {
            t.g.recompute();
            assert!(!t.g.can_block_at_all(human));
        }
    }
    // Roiling Terrain: two land cards in the graveyard; the land survives either way.
    for regen in [false, true] {
        let mut t = TestGame::new(2);
        t.graveyard(P1, "Forest");
        t.graveyard(P1, "Island");
        let l = t.battlefield(P1, "Mutavault");
        if regen {
            regen_shield(&mut t, l);
        } else {
            indestructible(&mut t, l);
        }
        cast_modes(&mut t, P0, "Roiling Terrain", &[], &[ob(l)]);
        t.resolve_all();
        assert!(t.on_battlefield(l));
        assert_eq!(t.life(P1), 18, "regen {regen}");
    }
}

#[test]
fn roiling_terrain_counts_the_land_it_just_destroyed() {
    cr!("608.2c", "701.8a");
    ruling!("Roiling Terrain", "The effects of the spell happen in sequence. By the time Roiling Terrain checks how many land cards are in the affected player’s graveyard, the land Roiling Terrain destroyed is (most likely) already in that graveyard.");
    let mut t = TestGame::new(2);
    t.graveyard(P1, "Forest");
    let l = t.battlefield(P1, "Mutavault");
    cast_modes(&mut t, P0, "Roiling Terrain", &[], &[ob(l)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

// ---------------------------------------------------------------------------------------
// Two targets.
// ---------------------------------------------------------------------------------------

#[test]
fn lavaball_trap_damages_every_creature_unless_both_lands_are_illegal() {
    cr!("608.2b", "115.10");
    ruling!(
        "Lavaball Trap",
        "All creatures are dealt damage, not just ones controlled by the targeted lands' controller(s)."
    );
    ruling!("Lavaball Trap", "If both targeted lands are illegal targets by the time Lavaball Trap resolves, the entire spell doesn't resolve. No creatures are dealt damage.");
    supported("Lavaball Trap");
    // One land gone: it resolves, destroys the other, and damages everyone's creatures.
    let mut t = TestGame::new(3);
    let mine = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Craw Wurm");
    let other = t.battlefield(P2, "Colossal Dreadmaw");
    let a = t.battlefield(P1, "Mutavault");
    let b = t.battlefield(P1, "Forest");
    cast_modes(&mut t, P0, "Lavaball Trap", &[], &[ob(a), ob(b)]);
    kill(&mut t, a);
    t.resolve_all();
    assert!(!t.on_battlefield(b));
    assert!(!t.on_battlefield(mine) && !t.on_battlefield(theirs));
    assert_eq!(damage_on(&t, other), 4);
    // Both gone: nothing.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Craw Wurm");
    let a = t.battlefield(P1, "Mutavault");
    let b = t.battlefield(P1, "Forest");
    let s = cast_modes(&mut t, P0, "Lavaball Trap", &[], &[ob(a), ob(b)]);
    fizzles(&mut t, s, |t| {
        kill(t, a);
        kill(t, b);
    });
    assert_eq!(dealt(&t, theirs), 0);
}

#[test]
fn two_target_destruction_with_one_target_left_affects_it() {
    cr!("608.2b", "601.2c");
    ruling!(
        "Boom // Bust",
        "If one of Boom's targets becomes an illegal target, the other is still destroyed."
    );
    ruling!("Into the Maw of Hell", "If one of Into the Maw of Hell’s targets is illegal by the time it resolves, Into the Maw of Hell will still affect the remaining legal target. If both targets are illegal at this time, Into the Maw of Hell won’t resolve.");
    ruling!("Into the Maw of Hell", "Into the Maw of Hell targets both the land and the creature. You can only cast it if you can choose a legal target for both.");
    ruling!("Spiteful Blow", "You can't cast Spiteful Blow without a legal creature and land to target. If one of those targets is illegal when Spiteful Below tries to resolve, the other one will still be destroyed.");
    // Boom: either land gone, the other is destroyed.
    supported("Boom // Bust");
    for gone in 0..2 {
        let mut t = TestGame::new(2);
        lands_for_cost(&mut t, P0, "Boom // Bust");
        let mine = t.battlefield(P0, "Mutavault");
        let theirs = t.battlefield(P1, "Mutavault");
        let card = t.hand(P0, "Boom // Bust");
        t.cast(P0, card)
            .method(CastMethod::Half(0))
            .targets(&[ob(mine)])
            .targets(&[ob(theirs)])
            .go();
        let (a, b) = if gone == 0 {
            (mine, theirs)
        } else {
            (theirs, mine)
        };
        indestructible(&mut t, a);
        spoil(&mut t, a, Spoil::Stolen);
        t.resolve_all();
        assert!(t.on_battlefield(a));
        assert!(!t.on_battlefield(b), "gone {gone}");
    }
    // Into the Maw of Hell and Spiteful Blow: no creature, can't be cast.
    for name in ["Into the Maw of Hell", "Spiteful Blow"] {
        let mut t = TestGame::new(2);
        let l = t.battlefield(P1, "Mutavault");
        assert!(
            try_cast(&mut t, P0, name, &[], &[&[ob(l)], &[]]).is_err(),
            "{name}"
        );
        // One target gone: the other is affected.
        for gone in 0..2 {
            let mut t = TestGame::new(2);
            let land = t.battlefield(P1, "Mutavault");
            let c = t.battlefield(P1, "Craw Wurm");
            let (first, second) = if name == "Spiteful Blow" {
                (ob(c), ob(land))
            } else {
                (ob(land), ob(c))
            };
            cast_modes(&mut t, P0, name, &[], &[first, second]);
            let (a, b) = if gone == 0 { (land, c) } else { (c, land) };
            spoil(&mut t, a, Spoil::Hexproof);
            t.resolve_all();
            assert!(t.on_battlefield(a), "{name} {gone}");
            assert!(!t.on_battlefield(b), "{name} {gone}");
        }
    }
    // Into the Maw of Hell, both gone: it doesn't resolve.
    let mut t = TestGame::new(2);
    let land = t.battlefield(P1, "Mutavault");
    let c = t.battlefield(P1, "Craw Wurm");
    let s = cast_modes(&mut t, P0, "Into the Maw of Hell", &[], &[ob(land), ob(c)]);
    fizzles(&mut t, s, |t| {
        kill(t, land);
        kill(t, c);
    });
}

#[test]
fn burning_of_xinye_targets_an_opponent_destroys_two_players_lands_and_damages_all() {
    cr!("608.2b", "701.19c", "701.8a");
    ruling!("Burning of Xinye", "A land with a regeneration shield on it will regenerate from Burning of Xinye’s destruction effect. However, the spell or ability that creates that shield must be cast or activated before Burning of Xinye starts to resolve.");
    ruling!("Burning of Xinye", "Burning of Xinye targets only an opponent. If that player is an illegal target by the time Burning of Xinye resolves, the entire spell doesn’t resolve.");
    ruling!("Burning of Xinye", "In a multiplayer game, only lands controlled by two players (you and the targeted opponent) are destroyed. However, Burning of Xinye will deal 4 damage to each creature, even those controlled by other players.");
    supported("Burning of Xinye");
    let mut t = TestGame::new(3);
    let p1_lands = t.lands(P1, "Forest", 5);
    let p2_lands = t.lands(P2, "Island", 4);
    let wurm = t.battlefield(P2, "Craw Wurm");
    regen_shield(&mut t, p1_lands[0]);
    t.answer_choose(
        P1,
        &p1_lands[..4].iter().map(|l| ob(*l)).collect::<Vec<_>>(),
    );
    cast_modes(&mut t, P0, "Burning of Xinye", &[], &[Entity::Player(P1)]);
    t.resolve_all();
    // P0's six lands (all it has) are down to two; P1 keeps the regenerated Forest and
    // the one it didn't choose; P2's lands are untouched.
    assert_eq!(controlled(&t, P0, CardType::Land), 2);
    assert!(t.on_battlefield(p1_lands[0]) && t.on_battlefield(p1_lands[4]));
    assert_eq!(controlled(&t, P1, CardType::Land), 2);
    assert!(p2_lands.iter().all(|l| t.on_battlefield(*l)));
    assert!(!t.on_battlefield(wurm), "P2's creature was dealt 4");
    // The opponent becomes an illegal target: nothing happens.
    let mut t = TestGame::new(2);
    t.lands(P1, "Forest", 5);
    t.battlefield(P1, "Craw Wurm");
    let s = cast_modes(&mut t, P0, "Burning of Xinye", &[], &[Entity::Player(P1)]);
    fizzles(&mut t, s, |t| {
        t.battlefield(P1, "Leyline of Sanctity");
        t.settle();
    });
}

// ---------------------------------------------------------------------------------------
// Which lands can be targeted.
// ---------------------------------------------------------------------------------------

#[test]
fn break_the_ice_targets_snow_lands_and_lands_that_could_make_colorless() {
    cr!("205.4a", "115.1", "106.7");
    ruling!(
        "Break the Ice",
        "A land \"is snow\" if it has the snow supertype."
    );
    ruling!("Break the Ice", "Break the Ice cares about what kind of mana could be produced by abilities of a land, even if that ability couldn't currently be activated. For example, a land with \"{T}: Add {C}\" is still a legal target even if it is currently tapped.");
    supported("Break the Ice");
    let mut t = TestGame::new(2);
    let snow = t.battlefield(P1, "Snow-Covered Forest");
    let forest = t.battlefield(P1, "Forest");
    let wastes = t.battlefield(P1, "Wastes");
    crate::r_p127_common::tap(&mut t, wastes);
    let from = t.asked().len();
    cast_modes(&mut t, P0, "Break the Ice", &[], &[ob(wastes)]);
    let offered = target_candidates(&t, P0, from);
    assert!(offered[0].contains(&ob(snow)));
    assert!(offered[0].contains(&ob(wastes)));
    assert!(!offered[0].contains(&ob(forest)));
    t.resolve_all();
    assert!(!t.on_battlefield(wastes));
}

#[test]
fn peak_eruption_and_nissas_defeat_check_land_subtypes() {
    cr!("205.3i", "305.6", "115.1");
    ruling!("Peak Eruption", "Peak Eruption can target any land with the subtype Mountain, not just those named Mountain. It can't target a land that doesn't have that subtype, even if the land can tap for {R}.");
    ruling!("Nissa's Defeat", "Most nonbasic lands that produce green mana aren’t Forests. For example, Desert of the Indomitable isn’t a Forest. Some nonbasic lands (such as Sheltered Thicket from the Amonkhet set) do have basic land types printed on the type line and may be Forests.");
    // (card, a land without the type that makes its color, a land with the type)
    for (name, without, with) in [
        ("Peak Eruption", "Shivan Reef", "Stomping Ground"),
        (
            "Nissa's Defeat",
            "Desert of the Indomitable",
            "Sheltered Thicket",
        ),
    ] {
        let mut t = TestGame::new(2);
        let w = t.battlefield(P1, without);
        let w2 = t.battlefield(P1, with);
        let from = t.asked().len();
        cast_modes(&mut t, P0, name, &[], &[ob(w2)]);
        let offered = target_candidates(&t, P0, from);
        assert!(!offered[0].contains(&ob(w)), "{name}: {without}");
        assert!(offered[0].contains(&ob(w2)), "{name}: {with}");
        let mut t = TestGame::new(2);
        let w = t.battlefield(P1, with);
        cast_modes(&mut t, P0, name, &[], &[ob(w)]);
        t.resolve_all();
        assert!(!t.on_battlefield(w), "{name}: {with}");
    }
}

#[test]
fn wake_of_destruction_looks_only_at_the_name() {
    cr!("201.2", "608.2c");
    ruling!(
        "Wake of Destruction",
        "Only looks at the card name, so it will not destroy lands that just share the destroyed land’s type."
    );
    let mut t = TestGame::new(2);
    let target = t.battlefield(P1, "Forest");
    let mine = t.battlefield(P0, "Forest");
    let snow = t.battlefield(P1, "Snow-Covered Forest");
    let arbor = t.battlefield(P1, "Dryad Arbor");
    cast_modes(&mut t, P0, "Wake of Destruction", &[], &[ob(target)]);
    t.resolve_all();
    assert!(!t.on_battlefield(target) && !t.on_battlefield(mine));
    assert!(t.on_battlefield(snow) && t.on_battlefield(arbor));
}

#[test]
fn icequake_checks_snow_only_on_resolution() {
    cr!("608.2h", "115.1");
    ruling!("Icequake", "Checks only on resolution to see if the land has the supertype snow; it’s not part of the targeting condition.");
    for (land, life) in [("Forest", 20), ("Snow-Covered Forest", 19)] {
        let mut t = TestGame::new(2);
        let l = t.battlefield(P1, land);
        cast_modes(&mut t, P0, "Icequake", &[], &[ob(l)]);
        t.resolve_all();
        assert!(!t.on_battlefield(l));
        assert_eq!(t.life(P1), life, "{land}");
    }
}

#[test]
fn befoul_isnt_modal_and_checks_its_target_on_resolution() {
    cr!("115.1", "608.2b", "700.2");
    ruling!("Befoul", "This is not modal. You do not have to choose the target type. The target has to either be a land or non-black creature when announced and on resolution.");
    // A land or a nonblack creature, with no mode chosen.
    for name in ["Mutavault", "Grizzly Bears"] {
        let mut t = TestGame::new(2);
        let x = t.battlefield(P1, name);
        let from = t.asked().len();
        cast_modes(&mut t, P0, "Befoul", &[], &[ob(x)]);
        assert!(t.asked()[from..]
            .iter()
            .all(|(_, d)| !matches!(d, Decision::ChooseModes { .. })));
        t.resolve_all();
        assert!(!t.on_battlefield(x), "{name}");
    }
    // A black creature can't be targeted.
    let mut t = TestGame::new(2);
    let black = t.battlefield(P1, "Walking Corpse");
    let land = t.battlefield(P1, "Mutavault");
    let from = t.asked().len();
    cast_modes(&mut t, P0, "Befoul", &[], &[ob(land)]);
    let offered = target_candidates(&t, P0, from);
    assert!(offered[0].contains(&ob(land)));
    assert!(!offered[0].contains(&ob(black)));
    // The creature turns black before it resolves: it doesn't resolve.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let s = cast_modes(&mut t, P0, "Befoul", &[], &[ob(bears)]);
    fizzles(&mut t, s, |t| {
        modify(
            t,
            P1,
            bears,
            vec![Modification::SetColors(ColorSet::single(Color::Black))],
        );
    });
    assert!(t.on_battlefield(bears));
}

// ---------------------------------------------------------------------------------------
// Mandatory triggered abilities with only your own lands to target.
// ---------------------------------------------------------------------------------------

#[test]
fn mandatory_land_destruction_triggers_target_your_own_land() {
    cr!("603.3d", "115.1");
    ruling!(
        "Ravenous Baboons",
        "You have to pick one of your own non-basic lands if the opponent does not have any and you do."
    );
    ruling!(
        "Ravenous Baboons",
        "If there is no non-basic land to destroy, the ability does nothing."
    );
    ruling!("Goblin Gardener", "The ability is not optional. You have to choose one of your own lands if your opponent has none.");
    ruling!("Detritivore", "The triggered ability is mandatory. If no other player has any nonbasic lands on the battlefield, you must target one of your own.");
    for name in ["Ravenous Baboons", "Goblin Gardener", "Detritivore"] {
        supported(name);
    }
    // Ravenous Baboons: only P0 has a nonbasic land (P1 has a Forest).
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Mutavault");
    let forest = t.battlefield(P1, "Forest");
    t.answer_targets(P0, &[]);
    t.enter(P0, "Ravenous Baboons");
    t.resolve_all();
    assert!(!t.on_battlefield(mine));
    assert!(t.on_battlefield(forest));
    // No nonbasic land at all: nothing happens.
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P1, "Forest");
    let own = t.battlefield(P0, "Forest");
    t.enter(P0, "Ravenous Baboons");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert!(t.on_battlefield(forest) && t.on_battlefield(own));
    // Goblin Gardener dies; P1 has no land.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Forest");
    let gg = t.battlefield(P0, "Goblin Gardener");
    t.answer_targets(P0, &[]);
    destroy(&mut t, gg);
    t.resolve_all();
    assert!(!t.on_battlefield(mine));
    // Detritivore: a time counter is removed while it's exiled; P1 has only basics.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Mutavault");
    let forest = t.battlefield(P1, "Forest");
    let d = t.exile(P0, "Detritivore");
    put_counters(&mut t, d, counters::TIME, 2);
    t.answer_targets(P0, &[]);
    t.g.remove_counters(ob(d), counters::TIME, 1);
    t.g.flush_events();
    t.resolve_all();
    assert!(!t.on_battlefield(mine));
    assert!(t.on_battlefield(forest));
}

// ---------------------------------------------------------------------------------------
// X = 0, Catch, and other single rulings.
// ---------------------------------------------------------------------------------------

#[test]
fn land_destruction_with_x_zero_destroys_nothing() {
    cr!("107.3", "601.2b", "115.1");
    ruling!(
        "Volcanic Eruption",
        "Can be used with X equal to zero. This is useful if no Mountains are on the battlefield."
    );
    ruling!("Scorched Earth", "X can be zero to destroy no lands.");
    for name in ["Volcanic Eruption", "Scorched Earth"] {
        supported(name);
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let forest = t.battlefield(P1, "Forest");
        lands_for_cost(&mut t, P0, name);
        let card = t.hand(P0, name);
        let s = t.cast(P0, card).x(0).go();
        t.resolve_all();
        assert!(resolved(&t, s), "{name}");
        assert!(
            t.on_battlefield(forest) && t.on_battlefield(bears),
            "{name}"
        );
        assert_eq!((t.life(P0), t.life(P1)), (20, 20), "{name}");
    }
}

#[test]
fn catch_can_target_your_own_untapped_permanent() {
    cr!("115.1", "702.10b");
    ruling!(
        "Catch // Release",
        "Catch can target any permanent, including one that’s untapped or one you already control."
    );
    supported("Catch // Release");
    let mut t = TestGame::new(2);
    let bears = t.battlefield_sick(P0, "Grizzly Bears");
    lands_for_cost(&mut t, P0, "Catch // Release");
    let card = t.hand(P0, "Catch // Release");
    t.cast(P0, card)
        .method(CastMethod::Half(0))
        .target(bears)
        .go();
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P0);
    assert!(!t.obj_now(bears).tapped);
    assert!(
        crate::r_s07_common::has_kw(&t, bears, keywords::KeywordKind::Haste),
        "it has haste"
    );
}

#[test]
fn destroy_the_evidence_mills_the_whole_library_without_lands() {
    cr!("701.20a", "608.2c");
    ruling!("Destroy the Evidence", "If the controller of the target land has no land cards in their library, all cards from that library will be revealed and put into their graveyard.");
    let mut t = TestGame::new(2);
    let l = t.battlefield(P1, "Mutavault");
    let n = t.library_size(P1);
    cast_modes(&mut t, P0, "Destroy the Evidence", &[], &[ob(l)]);
    t.resolve_all();
    assert_eq!(t.library_size(P1), 0);
    assert_eq!(t.graveyard_size(P1), n + 1);
}

#[test]
fn scorch_the_fields_doesnt_target_humans() {
    cr!("115.1", "609.3");
    ruling!("Scorch the Fields", "Scorch the Fields doesn’t target any Humans. You can cast Scorch the Fields even if there aren’t any Humans on the battlefield.");
    // No Humans: it's cast and destroys the land.
    let mut t = TestGame::new(2);
    let l = t.battlefield(P1, "Mutavault");
    cast_modes(&mut t, P0, "Scorch the Fields", &[], &[ob(l)]);
    t.resolve_all();
    assert!(!t.on_battlefield(l));
    // A Human with hexproof is still dealt damage.
    let mut t = TestGame::new(2);
    let l = t.battlefield(P1, "Mutavault");
    let h = t.battlefield(P1, "Elite Vanguard");
    spoil(&mut t, h, Spoil::Hexproof);
    cast_modes(&mut t, P0, "Scorch the Fields", &[], &[ob(l)]);
    t.resolve_all();
    assert!(!t.on_battlefield(h));
}

#[test]
fn obsidian_charmaw_counts_tapped_colorless_lands() {
    cr!("601.2f", "106.7");
    ruling!("Obsidian Charmaw", "Obsidian Charmaw's first ability counts each land your opponents control that would produce {C} if one of that land's abilities resolved. It doesn't matter whether that land's controller could currently activate the ability or not.");
    supported("Obsidian Charmaw");
    let mut t = TestGame::new(2);
    for w in t.lands(P1, "Wastes", 2) {
        crate::r_p127_common::tap(&mut t, w);
    }
    // {3}{R}{R} minus {2}: three lands pay for it.
    t.lands(P0, "Mountain", 3);
    let c = t.hand(P0, "Obsidian Charmaw");
    t.cast(P0, c).go();
}

#[test]
fn volcanic_offering_opponent_chooses_targets_as_it_is_cast() {
    cr!("601.2c", "115.1");
    ruling!("Volcanic Offering", "Unlike the other Offerings, you choose the opponents for Volcanic Offering as you cast the spell because those players are involved in choosing the spell’s targets.");
    supported("Volcanic Offering");
    let mut t = TestGame::new(3);
    let l1 = t.battlefield(P1, "Mutavault");
    let l2 = t.battlefield(P2, "Mutavault");
    let c1 = t.battlefield(P1, "Grizzly Bears");
    let c2 = t.battlefield(P2, "Grizzly Bears");
    lands_for_cost(&mut t, P0, "Volcanic Offering");
    let card = t.hand(P0, "Volcanic Offering");
    t.answer_targets(P0, &[ob(l1)]);
    t.answer_targets(P0, &[ob(c1)]);
    for p in [P1, P2] {
        t.answer_targets(p, &[ob(l2)]);
        t.answer_targets(p, &[ob(c2)]);
    }
    let from = t.asked().len();
    let s = t.cast(P0, card).go();
    // An opponent chose targets during casting, before the spell was on the stack.
    assert!(t.asked()[from..]
        .iter()
        .any(|(p, d)| *p != P0 && matches!(d, Decision::ChooseTargets { .. })));
    assert_eq!(targets_of(&t, s).len(), 4);
    t.resolve_all();
    assert!(!t.on_battlefield(l1) && !t.on_battlefield(l2));
    assert!(!t.on_battlefield(c1) && !t.on_battlefield(c2));
}

#[test]
fn world_breaker_returns_only_from_the_graveyard() {
    cr!("602.5", "113.6k");
    ruling!(
        "World Breaker",
        "You can activate the last ability only if World Breaker is in your graveyard."
    );
    supported("World Breaker");
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 4);
    let wb = t.battlefield(P0, "World Breaker");
    assert!(!can_activate(&mut t, P0, wb));
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 4);
    let wb = t.graveyard(P0, "World Breaker");
    assert!(can_activate(&mut t, P0, wb));
    t.activate(P0, wb, 0, &[]).expect("from the graveyard");
    t.resolve_all();
    assert!(t.in_hand(P0, "World Breaker"));
}

/// Goblin Grenadiers attacks P1 unblocked.
fn grenadiers_attack(t: &mut TestGame) -> ObjectId {
    let g = t.battlefield(P0, "Goblin Grenadiers");
    crate::r_p123_common::to_blockers(t, &[(g, Entity::Player(P1))], &[]);
    t.settle();
    g
}

#[test]
fn goblin_grenadiers_needs_a_creature_and_a_land_to_target() {
    cr!("603.3d", "115.3", "115.1");
    ruling!("Goblin Grenadiers", "A permanent that’s both a land and a creature may be chosen as both of this ability’s targets.");
    ruling!("Goblin Grenadiers", "Both a creature and a land must be targeted when the ability triggers. If there aren’t enough legal targets, the ability doesn’t go on the stack and Goblin Grenadiers can’t be sacrificed this way.");
    supported("Goblin Grenadiers");
    // No land anywhere: no ability, no sacrifice.
    let mut t = TestGame::new(2);
    t.answer_yes(P0, true);
    let g = grenadiers_attack(&mut t);
    assert_eq!(t.stack_len(), 0);
    t.resolve_all();
    assert!(t.on_battlefield(g));
    // Dryad Arbor is both targets.
    let mut t = TestGame::new(2);
    let arbor = t.battlefield(P1, "Dryad Arbor");
    t.answer_targets(P0, &[ob(arbor)]);
    t.answer_targets(P0, &[ob(arbor)]);
    t.answer_yes(P0, true);
    let g = grenadiers_attack(&mut t);
    assert_eq!(t.stack_len(), 1);
    let top = *t.g.stack.last().unwrap();
    assert_eq!(targets_of(&t, top), vec![ob(arbor), ob(arbor)]);
    t.resolve_all();
    assert!(!t.on_battlefield(g));
    assert!(!t.on_battlefield(arbor));
}

#[test]
fn numot_targets_on_trigger_and_pays_on_resolution() {
    cr!("603.3d", "608.2c", "115.4");
    ruling!("Numot, the Devastator", "You choose the targets when the ability is put on the stack, and you choose whether to pay when it resolves. You may choose zero targets. You can pay the mana even if you chose zero targets.");
    supported("Numot, the Devastator");
    for targets in [0usize, 2] {
        let mut t = TestGame::new(2);
        let numot = t.battlefield(P0, "Numot, the Devastator");
        t.lands(P0, "Mountain", 3);
        let p1 = t.lands(P1, "Forest", 2);
        let chosen: Vec<Entity> = p1[..targets].iter().map(|l| ob(*l)).collect();
        t.answer_targets(P0, &chosen);
        crate::r_p123_common::to_blockers(&mut t, &[(numot, Entity::Player(P1))], &[]);
        t.advance_to(P0, Step::CombatDamage);
        t.settle();
        assert_eq!(t.stack_len(), 1, "the trigger");
        let top = *t.g.stack.last().unwrap();
        assert_eq!(targets_of(&t, top).len(), targets);
        assert_eq!(
            crate::r_s01_common::tapped_lands(&t, P0),
            0,
            "nothing paid yet"
        );
        t.answer_yes(P0, true);
        t.resolve();
        assert_eq!(
            crate::r_s01_common::tapped_lands(&t, P0),
            3,
            "paid {targets}"
        );
        assert_eq!(
            p1.iter().filter(|l| t.on_battlefield(**l)).count(),
            2 - targets
        );
    }
}
