//! Rulings batch P018 — lands that become creatures: becoming a creature isn't entering
//! the battlefield (CR 603.6a), "summoning sickness" by control (CR 302.6), the
//! power/toughness an animation sets overwrites earlier setting effects but not
//! modifications and counters (CR 613.4), counters stay when it stops being a creature
//! (CR 122.2), "if you control two or more other lands" as lands enter at the same time
//! (CR 614.12, 305.7), several instances of a gained ability (CR 113.2c), and ward gained
//! after the land became a target (CR 702.21a).

use crate::r_p018_common::*;
use crate::r_s01_common::{supported, tokens};
use crate::r_s04_common::add_mana;
use crate::r_s06_common::{activate_containing, attach_new, give_control};
use crate::r_s09_common::legal_attack;
use crate::r_s20_common::{tap_for_mana, to_beginning_of_combat};
use mtg_engine::ability::{Modification, Value};
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Animates the manland: its ability that makes it a creature (X = 2 when it has an X).
fn animate(t: &mut TestGame, p: PlayerId, land: ObjectId) {
    t.answer(p, DecisionKind::X, Answer::Number(2));
    activate_and_resolve(t, p, land, "becomes");
    assert!(is_creature_now(t, land));
}

#[test]
fn turning_a_land_into_a_creature_isnt_a_creature_entering() {
    cr!("603.6a", "613.1d");
    ruling!(
        "Awaken the Ancient",
        "This doesn't count as a creature entering the battlefield. The Mountain was already on the battlefield; it only changed types."
    );
    ruling!(
        "Ignition Team",
        "This doesn’t count as a creature entering the battlefield. The land was already on the battlefield; it only changed types."
    );
    supported("Awaken the Ancient");
    supported("Ignition Team");
    supported("Soul Warden");
    // Soul Warden: "Whenever another creature enters, you gain 1 life."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Soul Warden");
    let mountain = t.battlefield(P0, "Mountain");
    attach_new(&mut t, P0, "Awaken the Ancient", mountain);
    t.settle();
    assert!(is_creature_now(&mut t, mountain));
    assert_eq!(t.pt(mountain), (7, 7));
    let team = t.battlefield(P0, "Ignition Team");
    put_counters(&mut t, team, counters::PLUS1, 1);
    let forest = t.battlefield(P0, "Forest");
    t.answer_targets(P0, &[Entity::Object(forest)]);
    activate_and_resolve(&mut t, P0, team, "becomes");
    assert!(is_creature_now(&mut t, forest));
    assert_eq!(t.pt(forest), (4, 4));
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn a_land_awakened_by_liege_of_the_tangle_depends_only_on_its_counter() {
    cr!("611.2b", "122.1");
    ruling!(
        "Liege of the Tangle",
        "Whether an affected land remains an 8/8 green Elemental creature depends only on whether it still has an awakening counter on it, not on whether Liege of the Tangle is still on the battlefield."
    );
    supported("Liege of the Tangle");
    let mut t = TestGame::new(2);
    let liege = t.battlefield(P0, "Liege of the Tangle");
    let forest = t.battlefield(P0, "Forest");
    t.answer_targets(P0, &[Entity::Object(forest)]);
    t.attack(&[(liege, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 12);
    assert_eq!(t.counters(forest, "awakening"), 1);
    assert!(is_creature_now(&mut t, forest));
    // Liege leaves the battlefield: the Forest is still an 8/8 green Elemental creature.
    crate::r_s02_common::destroy(&mut t, liege);
    assert!(!t.on_battlefield(liege));
    assert!(is_creature_now(&mut t, forest));
    assert_eq!(t.pt(forest), (8, 8));
    assert!(t.obj_now(forest).chars.colors.contains(Color::Green));
    assert!(t.obj_now(forest).chars.has_subtype("Elemental"));
    assert!(t.obj_now(forest).is(CardType::Land));
    // It loses the counter: it isn't a creature any more (and the effect is over).
    let f = t.g.current(forest);
    t.g.remove_counters(Entity::Object(f), "awakening", 1);
    assert!(!is_creature_now(&mut t, forest));
    put_counters(&mut t, forest, "awakening", 1);
    assert!(!is_creature_now(&mut t, forest));
}

#[test]
fn a_manland_animated_the_turn_you_gained_control_of_it_cant_tap_or_attack() {
    cr!("302.6", "508.1a");
    ruling!(
        "Wandering Fumarole",
        "A land that becomes a creature may be affected by “summoning sickness.” You can't attack with it or use any of its {T} abilities (including its mana abilities) unless it began your most recent turn on the battlefield under your control. Note that summoning sickness cares about when that permanent came under your control, not when it became a creature nor when it entered the battlefield."
    );
    ruling!(
        "Raging Ravine",
        "If this land becomes a creature but you haven’t controlled it continuously since your most recent turn began, you won’t be able to activate its mana ability or attack with it that turn."
    );
    ruling!(
        "Den of the Bugbear",
        "If you turn Den of the Bugbear into a creature but haven't controlled it continuously since your most recent turn began, you won't be able to activate its mana ability or attack with it."
    );
    ruling!(
        "Hall of Storm Giants",
        "If you turn Hall of Storm Giants into a creature but haven't controlled it continuously since your most recent turn began, you won't be able to activate its mana ability or attack with it."
    );
    ruling!(
        "Lair of the Hydra",
        "If you turn Lair of the Hydra into a creature but haven't controlled it continuously since your most recent turn began, you won't be able to activate its mana ability or attack with it."
    );
    for name in [
        "Wandering Fumarole",
        "Raging Ravine",
        "Den of the Bugbear",
        "Hall of Storm Giants",
        "Lair of the Hydra",
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        // Long on the battlefield under P1's control; P0 gains control of it this turn.
        let stolen = t.battlefield(P1, name);
        give_control(&mut t, stolen, P0);
        // P0's own one has been under P0's control since the turn began.
        let own = t.battlefield(P0, name);
        for land in [stolen, own] {
            animate(&mut t, P0, land);
        }
        assert!(!tap_for_mana(&mut t, P0, stolen, "Add"), "{name}");
        assert!(tap_for_mana(&mut t, P0, own, "Add"), "{name}");
        empty_pool(&mut t, P0);
        t.g.untap(own);
        to_beginning_of_combat(&mut t, P0);
        assert!(
            !legal_attack(&mut t, &[(stolen, Entity::Player(P1))]),
            "{name}"
        );
        assert!(legal_attack(&mut t, &[(own, Entity::Player(P1))]), "{name}");
    }
}

#[test]
fn a_manland_animation_overwrites_earlier_pt_setting_but_not_counters() {
    cr!("613.4b", "613.4c", "613.4d", "613.7a");
    ruling!(
        "Hissing Quagmire",
        "For example, if Hissing Quagmire has been made a 0/0 creature with three +1/+1 counters on it, activating its last ability will turn it into a 5/5 creature that's still a land."
    );
    ruling!(
        "Lumbering Falls",
        "For example, if Lumbering Falls has been made a 0/0 creature with three +1/+1 counters on it, activating its last ability will turn it into a 6/6 creature that's still a land."
    );
    ruling!(
        "Needle Spires",
        "For example, if Needle Spires has been made a 0/0 creature with three +1/+1 counters on it, activating its last ability will turn it into a 5/4 creature that's still a land."
    );
    ruling!(
        "Wandering Fumarole",
        "For example, if Wandering Fumarole has been made a 0/0 creature with three +1/+1 counters on it, activating its last ability will turn it into a 4/7 creature that's still a land."
    );
    for (name, pt) in [
        ("Hissing Quagmire", (5, 5)),
        ("Lumbering Falls", (6, 6)),
        ("Needle Spires", (5, 4)),
        ("Wandering Fumarole", (4, 7)),
    ] {
        supported(name);
        for switched in [false, true] {
            let mut t = TestGame::new(2);
            let land = t.battlefield(P0, name);
            // As an awaken spell would: a 0/0 creature with three +1/+1 counters.
            put_counters(&mut t, land, counters::PLUS1, 3);
            make_zero_zero_creature(&mut t, land);
            assert_eq!(t.pt(land), (3, 3));
            // A modification from before the animation still applies (+2/+0), and so
            // does a switch.
            let mut mods = vec![Modification::ModifyPT(Value::Const(2), Value::Const(0))];
            if switched {
                mods.push(Modification::SwitchPT);
            }
            modify(&mut t, land, mods);
            animate(&mut t, P0, land);
            let expected = if switched {
                (pt.1, pt.0 + 2)
            } else {
                (pt.0 + 2, pt.1)
            };
            assert_eq!(t.pt(land), expected, "{name}, switched: {switched}");
            assert!(t.obj_now(land).is(CardType::Land));
        }
    }
}

#[test]
fn restless_bivouacs_counters_stay_when_it_stops_being_a_creature() {
    cr!("122.1", "514.2");
    ruling!(
        "Restless Bivouac",
        "Counters on Restless Bivouac remain on it when it stops being a creature. If it becomes a creature later, they'll apply to it."
    );
    supported("Restless Bivouac");
    let mut t = TestGame::new(2);
    let bivouac = t.battlefield(P0, "Restless Bivouac");
    animate(&mut t, P0, bivouac);
    put_counters(&mut t, bivouac, counters::PLUS1, 1);
    assert_eq!(t.pt(bivouac), (3, 3));
    // The turn ends: it's no longer a creature, but the counter stays.
    t.advance_to(P1, Step::Upkeep);
    assert!(!is_creature_now(&mut t, bivouac));
    assert_eq!(t.counters(bivouac, counters::PLUS1), 1);
    t.advance_to(P0, Step::PrecombatMain);
    animate(&mut t, P0, bivouac);
    assert_eq!(t.pt(bivouac), (3, 3));
}

#[test]
fn a_land_entering_with_other_lands_doesnt_count_them() {
    cr!("614.12", "305.4");
    ruling!(
        "Den of the Bugbear",
        "If Den of the Bugbear enters the battlefield at the same time as one or more other lands, it doesn't take those lands into consideration when determining how many other lands you control."
    );
    ruling!(
        "Hall of Storm Giants",
        "If Hall of Storm Giants enters the battlefield at the same time as one or more other lands, it doesn't take those lands into consideration when determining how many other lands you control."
    );
    ruling!(
        "Lair of the Hydra",
        "If Lair of the Hydra enters the battlefield at the same time as one or more other lands, it doesn't take those lands into consideration when determining how many other lands you control."
    );
    ruling!(
        "Manabond",
        "Putting a land onto the battlefield does not count as playing a land."
    );
    supported("Manabond");
    // Manabond: "At the beginning of your end step, you may reveal your hand and put all
    // land cards from it onto the battlefield. If you do, discard your hand."
    for (name, others, tapped) in [
        ("Den of the Bugbear", 1, false),
        ("Hall of Storm Giants", 1, false),
        ("Lair of the Hydra", 1, false),
        // With two other lands already there, it enters tapped.
        ("Den of the Bugbear", 2, true),
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Manabond");
        t.lands(P0, "Plains", others);
        let land = t.hand(P0, name);
        let m1 = t.hand(P0, "Mountain");
        let m2 = t.hand(P0, "Mountain");
        t.answer_yes(P0, true);
        t.advance_to(P0, Step::End);
        t.resolve_all();
        for c in [land, m1, m2] {
            assert!(t.on_battlefield(c), "{name}");
        }
        assert_eq!(t.obj_now(land).tapped, tapped, "{name} with {others} lands");
        // Putting lands onto the battlefield isn't playing them.
        assert_eq!(t.g.players[P0.idx()].lands_played_this_turn, 0);
    }
}

#[test]
fn den_of_the_bugbear_activated_twice_makes_two_goblins() {
    cr!("113.2c", "603.2");
    ruling!(
        "Den of the Bugbear",
        "If Den of the Bugbear's last ability is activated multiple times in one turn, it will have multiple instances of the triggered ability that it grants. For example, if you activate it twice and attack, you will create two 1/1 red Goblin creature tokens that are tapped and attacking."
    );
    let mut t = TestGame::new(2);
    let den = t.battlefield(P0, "Den of the Bugbear");
    animate(&mut t, P0, den);
    animate(&mut t, P0, den);
    t.attack(&[(den, Entity::Player(P1))], &[]);
    let goblins = tokens(&t, P0);
    assert_eq!(goblins.len(), 2);
    // 3 from the Den and 1 from each Goblin.
    assert_eq!(t.life(P1), 15);
}

/// P1 casts Stone Rain targeting `land` (P1's main phase); returns the spell.
fn stone_rain(t: &mut TestGame, land: ObjectId) -> ObjectId {
    t.set_step(P1, Step::PrecombatMain);
    add_mana(t, P1, ManaType::R, 3);
    let rain = t.hand(P1, "Stone Rain");
    t.cast(P1, rain).target(land).go()
}

#[test]
fn hall_of_storm_giants_gaining_ward_after_being_targeted_doesnt_trigger_it() {
    cr!("702.21a", "603.2");
    ruling!(
        "Hall of Storm Giants",
        "If an opponent casts a spell or ability that targets Hall of Storm Giants, and you activate the last ability in response, the ward ability Hall of Storm Giants gains won't trigger. (However, if it already had ward at the moment it became the target of the spell or ability, that instance of ward would trigger.)"
    );
    supported("Stone Rain");
    // Animated in response: no ward trigger, and Stone Rain destroys it.
    let mut t = TestGame::new(2);
    let hall = t.battlefield(P0, "Hall of Storm Giants");
    let rain = stone_rain(&mut t, hall);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    plenty_of_mana(&mut t, P0);
    activate_containing(&mut t, P0, hall, "becomes").expect("animate");
    empty_pool(&mut t, P0);
    t.resolve();
    assert!(is_creature_now(&mut t, hall));
    assert_eq!(t.g.stack, vec![rain]);
    t.resolve_all();
    assert!(!t.on_battlefield(hall));
    // Animated before it's targeted: ward triggers and counters Stone Rain.
    let mut t = TestGame::new(2);
    let hall = t.battlefield(P0, "Hall of Storm Giants");
    t.set_step(P1, Step::PrecombatMain);
    animate(&mut t, P0, hall);
    stone_rain(&mut t, hall);
    t.settle();
    assert_eq!(t.stack_len(), 2, "Stone Rain and a ward trigger");
    t.resolve_all();
    assert!(t.on_battlefield(hall));
    assert!(t.in_graveyard(P1, "Stone Rain"));
}

#[test]
fn hall_of_storm_giants_activated_twice_has_two_wards() {
    cr!("702.21a", "113.2c");
    ruling!(
        "Hall of Storm Giants",
        "If you activate Hall of Storm Giants's last ability multiple times, it gains multiple instances of ward {3}. Each of those instances will trigger separately and ask the opponent who controls the spell or ability to pay {3}. If they don't for any of those instances, the spell or ability will be countered."
    );
    let mut t = TestGame::new(2);
    let hall = t.battlefield(P0, "Hall of Storm Giants");
    t.set_step(P1, Step::PrecombatMain);
    animate(&mut t, P0, hall);
    animate(&mut t, P0, hall);
    stone_rain(&mut t, hall);
    t.settle();
    assert_eq!(t.stack_len(), 3, "Stone Rain and two ward triggers");
    // P1 can pay for one of them, not both: Stone Rain is countered.
    add_mana(&mut t, P1, ManaType::C, 3);
    t.answer_yes(P1, true);
    t.answer_yes(P1, true);
    t.resolve_all();
    assert_eq!(t.g.players[P1.idx()].mana_pool.total(), 0);
    assert!(t.on_battlefield(hall));
    assert!(t.in_graveyard(P1, "Stone Rain"));
}

#[test]
fn staff_of_titanias_dryad_keeps_its_creature_types_when_its_land_type_changes() {
    cr!("205.3i", "205.3m", "305.7");
    ruling!(
        "Staff of Titania",
        "Forest is a land subtype. Dryad is a creature subtype. If an effect causes the token to have a different land subtype, that won't affect its creature subtypes (or color), and vice versa."
    );
    supported("Staff of Titania");
    supported("Spreading Seas");
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    let staff = t.battlefield(P0, "Staff of Titania");
    assert!(t.g.attach(staff, Entity::Object(bear)));
    t.attack(&[(bear, Entity::Player(P1))], &[]);
    let dryads = tokens(&t, P0);
    assert_eq!(dryads.len(), 1);
    let dryad = dryads[0];
    assert!(t.obj_now(dryad).chars.has_subtype("Forest"));
    assert!(t.obj_now(dryad).chars.has_subtype("Dryad"));
    // Spreading Seas: "Enchanted land is an Island." It's an Island Dryad, still green.
    attach_new(&mut t, P1, "Spreading Seas", dryad);
    let o = t.obj_now(dryad);
    assert!(o.chars.has_subtype("Island") && !o.chars.has_subtype("Forest"));
    assert!(o.chars.has_subtype("Dryad"));
    assert!(o.is(CardType::Creature) && o.is(CardType::Land));
    assert!(o.chars.colors.contains(Color::Green));
    // Vice versa: losing its creature types doesn't touch its land type.
    modify(&mut t, dryad, vec![Modification::RemoveAllCreatureTypes]);
    let o = t.obj_now(dryad);
    assert!(!o.chars.has_subtype("Dryad"));
    assert!(o.chars.has_subtype("Island"));
}
