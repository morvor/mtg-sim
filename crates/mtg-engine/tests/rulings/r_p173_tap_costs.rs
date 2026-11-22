//! Rulings batch P173 — costs that tap untapped permanents you control (not {T}): the
//! permanents tapped needn't have been under your control since the turn began, the
//! source included (CR 302.6); noncreature permanents with {T} abilities aren't affected
//! by summoning sickness (CR 302.6); tapping an Aura or Equipment doesn't tap what it's
//! attached to (CR 701.26a, 110.5c); and no player acts while a spell or ability is being
//! paid for (CR 601.2, 602.2): the opponents' first chance to act comes with the costs
//! paid (played through the priority loop, CR 117.3c, 117.4), so what is tapped or counted
//! for a cost is locked in (CR 601.2h).

use crate::r_p174_tap_costs::{no_setup, opp_bears, run, Src, TapCase};
use crate::r_s01_common::{supported, tokens, watch, Seen};
use crate::r_s02_common::can_activate;
use crate::r_s04_common::add_mana;
use crate::r_s06_common::{activate_containing, attach_new};
use crate::r_s24_common::tapped;
use crate::r_s25_common::keep_copy_targets;
use mtg_engine::ability::AbilityKind;
use mtg_engine::decision::{Action, Answer};
use mtg_engine::game::Game;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

fn objs(ids: &[ObjectId]) -> Vec<Entity> {
    ids.iter().map(|i| obj(*i)).collect()
}

fn target_p1(_: &mut TestGame) -> Vec<Entity> {
    vec![Entity::Player(P1)]
}

/// P0 casts Lightning Bolt at P1 (with mana from its pool); the spell is the target.
fn p0_bolts_p1(t: &mut TestGame) -> Vec<Entity> {
    let b = t.hand(P0, "Lightning Bolt");
    add_mana(t, P0, ManaType::R, 1);
    let s = t.cast(P0, b).target(P1).go();
    vec![obj(s)]
}

/// [`p0_bolts_p1`], keeping the target for the copy Sigil Tracer makes of the Bolt.
fn p0_bolts_p1_for_a_copy(t: &mut TestGame) -> Vec<Entity> {
    keep_copy_targets(t, P0);
    p0_bolts_p1(t)
}

/// Chooses P0's Grizzly Bears to tap for the cost.
fn tap_the_bears(t: &mut TestGame) -> Vec<Entity> {
    let b = t.named_on_battlefield("Grizzly Bears")[0];
    t.answer_choose(P0, &[obj(b)]);
    vec![]
}

fn pumped_by_one(t: &mut TestGame, src: ObjectId) -> bool {
    let name = t.obj_now(src).chars.name.clone();
    let base = mtg_engine::card::card(&name)
        .front()
        .chars
        .power
        .unwrap_or(0);
    t.g.recompute();
    t.pt(src).0 == base + 1
}

// --- Summoning sickness doesn't restrict costs without {T} ----------------------------------

#[test]
fn creatures_and_the_source_tapped_for_costs_needn_t_have_been_controlled_since_the_turn_began() {
    cr!("302.6", "602.2b");
    ruling!(
        "Persistent Petitioners",
        "Advisors that you haven't controlled continuously since your turn began may be tapped to pay for the second ability of Persistent Petitioners. This includes Persistent Petitioners itself on the turn it comes under your control."
    );
    run(TapCase {
        card: "Persistent Petitioners",
        needle: "Tap four untapped Advisors",
        src: Src::Battlefield { taps_itself: true },
        helpers: &[
            "Persistent Petitioners",
            "Persistent Petitioners",
            "Persistent Petitioners",
        ],
        lands: &[],
        tap_symbol: false,
        setup: target_p1,
        check: |t, _| t.graveyard_size(P1) == 12,
    });
    ruling!(
        "Embalmer's Tools",
        "Any untapped Zombie you control can be tapped to pay the second ability’s activation cost, even one that just came under your control."
    );
    run(TapCase {
        card: "Embalmer's Tools",
        needle: "Tap an untapped Zombie",
        src: Src::Battlefield { taps_itself: false },
        helpers: &["Walking Corpse"],
        lands: &[],
        tap_symbol: false,
        setup: target_p1,
        check: |t, _| t.graveyard_size(P1) == 1,
    });
    // Tap an untapped creature you control: this creature gets +1/+1.
    ruling!(
        "Bramblesnap",
        "Since Bramblesnap's activated ability doesn't have a tap symbol in its cost, you can tap a creature (including Bramblesnap itself) that hasn't been under your control since your most recent turn began to pay the cost."
    );
    ruling!(
        "Topan Ascetic",
        "Since Topan Ascetic's activated ability doesn't have a tap symbol in its cost, you can tap a creature (including Topan Ascetic itself) that hasn't been under your control since your most recent turn began to pay the cost."
    );
    ruling!(
        "Llanowar Behemoth",
        "Since the activated ability doesn't have a tap symbol in its cost, you can tap a creature (including Llanowar Behemoth itself) that hasn't been under your control since your most recent turn began to pay the cost."
    );
    for card in ["Bramblesnap", "Topan Ascetic", "Llanowar Behemoth"] {
        run(TapCase {
            card,
            needle: "Tap an untapped creature",
            src: Src::Battlefield { taps_itself: true },
            helpers: &[],
            lands: &[],
            tap_symbol: false,
            setup: no_setup,
            check: pumped_by_one,
        });
        // Or another creature that just came under P0's control.
        run(TapCase {
            card,
            needle: "Tap an untapped creature",
            src: Src::Battlefield { taps_itself: false },
            helpers: &["Grizzly Bears"],
            lands: &[],
            tap_symbol: false,
            setup: tap_the_bears,
            check: pumped_by_one,
        });
    }
    ruling!(
        "Sanctuary Lockdown",
        "Since Sanctuary Lockdown’s activated ability doesn’t have a tap symbol in its cost, you can tap creatures that haven’t been under your control since your most recent turn began to pay the cost."
    );
    run(TapCase {
        card: "Sanctuary Lockdown",
        needle: "Tap two untapped Humans",
        src: Src::Battlefield { taps_itself: false },
        helpers: &["Alaborn Trooper", "Alaborn Trooper"],
        lands: &["Wastes", "Wastes"],
        tap_symbol: false,
        setup: opp_bears,
        check: |t, _| {
            let b = t.named_on_battlefield("Grizzly Bears")[0];
            tapped(t, b)
        },
    });
    ruling!(
        "Siege Zombie",
        "Since Siege Zombie's activated ability doesn't have a tap symbol in its cost, you can tap creatures that haven't been under your control since your most recent turn began (including Siege Zombie itself) to pay the cost."
    );
    run(TapCase {
        card: "Siege Zombie",
        needle: "Tap three untapped creatures",
        src: Src::Battlefield { taps_itself: true },
        helpers: &["Grizzly Bears", "Grizzly Bears"],
        lands: &[],
        tap_symbol: false,
        setup: no_setup,
        check: |t, _| t.life(P1) == 19,
    });
    ruling!(
        "Skaab Wrangler",
        "Since Skaab Wrangler's activated ability doesn't have a tap symbol in its cost, you can tap creatures that haven't been under your control since your most recent turn began (including Skaab Wrangler itself) to pay the cost."
    );
    run(TapCase {
        card: "Skaab Wrangler",
        needle: "Tap three untapped creatures",
        src: Src::Battlefield { taps_itself: true },
        helpers: &["Grizzly Bears", "Grizzly Bears"],
        lands: &[],
        tap_symbol: false,
        setup: opp_bears,
        check: |t, _| {
            let b = t.named_on_battlefield("Grizzly Bears");
            b.iter()
                .any(|b| t.obj_now(*b).controller == P1 && tapped(t, *b))
        },
    });
    ruling!(
        "Skystrike Officer",
        "Since Skystrike Officer's activated ability doesn't have a tap symbol in its cost, you can tap creatures that haven't been under your control since your most recent turn began (including Skystrike Officer itself if you'd like) to pay the cost."
    );
    run(TapCase {
        card: "Skystrike Officer",
        needle: "Tap three untapped Soldiers",
        src: Src::Battlefield { taps_itself: true },
        helpers: &["Alaborn Trooper", "Alaborn Trooper"],
        lands: &[],
        tap_symbol: false,
        setup: no_setup,
        check: |t, _| t.hand_size(P0) == 1,
    });
    ruling!(
        "The Archimandrite",
        "Since The Archimandrite’s activated ability doesn’t have a tap symbol in its cost, you can tap creatures that haven’t been under your control since your most recent turn began (including The Archimandrite itself) to pay the cost."
    );
    run(TapCase {
        card: "The Archimandrite",
        needle: "Tap three untapped Advisors",
        src: Src::Battlefield { taps_itself: true },
        helpers: &["Persistent Petitioners", "Persistent Petitioners"],
        lands: &[],
        tap_symbol: false,
        setup: no_setup,
        check: |t, _| t.hand_size(P0) == 1,
    });
    // "other than this creature": a new red creature; the Giant gets +X/+0.
    ruling!(
        "Impelled Giant",
        "Since the activated ability doesn't have a tap symbol in its cost, you can tap creatures that haven't been under your control since your most recent turn began to pay the cost."
    );
    run(TapCase {
        card: "Impelled Giant",
        needle: "Tap an untapped red creature",
        src: Src::Battlefield { taps_itself: false },
        helpers: &["Hill Giant"],
        lands: &[],
        tap_symbol: false,
        setup: no_setup,
        check: |t, src| t.pt(src).0 == 6,
    });
    ruling!(
        "Sigil Tracer",
        "Since the activated ability doesn’t have a tap symbol in its cost, you can tap creatures (including Sigil Tracer itself) that haven’t been under your control since your most recent turn began to pay the cost."
    );
    run(TapCase {
        card: "Sigil Tracer",
        needle: "Tap two untapped Wizards",
        src: Src::Battlefield { taps_itself: true },
        helpers: &["Prodigal Sorcerer"],
        lands: &["Island", "Wastes"],
        tap_symbol: false,
        setup: p0_bolts_p1_for_a_copy,
        check: |t, _| t.life(P1) == 14,
    });
    ruling!(
        "Patron Wizard",
        "Since the cost does not include the {T} symbol, it can tap itself even on the turn it enters."
    );
    run(TapCase {
        card: "Patron Wizard",
        needle: "Tap an untapped Wizard",
        src: Src::Battlefield { taps_itself: true },
        helpers: &[],
        lands: &[],
        tap_symbol: false,
        setup: p0_bolts_p1,
        // Countered: P0 has no mana to pay {1}.
        check: |t, _| t.life(P1) == 20 && t.in_graveyard(P0, "Lightning Bolt"),
    });
    ruling!(
        "Lullmage Mentor",
        "Since the second ability doesn’t have a tap symbol in its cost, you can tap creatures (including Lullmage Mentor itself) that haven’t been under your control since your most recent turn began to pay the cost."
    );
    run(TapCase {
        card: "Lullmage Mentor",
        needle: "Tap seven untapped Merfolk",
        src: Src::Battlefield { taps_itself: true },
        helpers: &["Merfolk of the Pearl Trident"; 6],
        lands: &[],
        tap_symbol: false,
        setup: p0_bolts_p1,
        check: |t, _| t.life(P1) == 20,
    });
    ruling!(
        "Dwarven Bloodboiler",
        "The ability can be activated the turn it enters because the ability does not use the {T} symbol."
    );
    run(TapCase {
        card: "Dwarven Bloodboiler",
        needle: "Tap an untapped Dwarf",
        src: Src::Battlefield { taps_itself: true },
        helpers: &[],
        lands: &[],
        tap_symbol: false,
        setup: opp_bears,
        check: |t, _| {
            let b = t.named_on_battlefield("Grizzly Bears")[0];
            t.pt(b) == (4, 2)
        },
    });
}

// --- Noncreature permanents, Auras and Equipment ---------------------------------------------

#[test]
fn flame_fusillade_noncreature_permanents_tap_the_turn_they_enter() {
    cr!("302.6", "113.3b");
    ruling!(
        "Flame Fusillade",
        "\"Summoning sickness\" applies only to creatures, not to other permanents. You may tap a noncreature permanent to pay for an ability with {T} in its cost even if the permanent entered that turn."
    );
    supported("Flame Fusillade");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    let land = t.battlefield_sick(P0, "Wastes");
    let rock = t.battlefield_sick(P0, "Bonesplitter");
    let bears = t.battlefield_sick(P0, "Grizzly Bears");
    let ff = t.hand(P0, "Flame Fusillade");
    t.cast(P0, ff).go();
    t.resolve_all();
    for src in [land, rock] {
        t.answer_targets(P0, &[Entity::Player(P1)]);
        activate_containing(&mut t, P0, src, "1 damage").expect("noncreature taps");
        t.resolve_all();
    }
    assert_eq!(t.life(P1), 18);
    // The creature that came under P0's control this turn can't.
    t.answer_targets(P0, &[Entity::Player(P1)]);
    assert!(activate_containing(&mut t, P0, bears, "1 damage").is_err());
    assert_eq!(t.life(P1), 18);
}

#[test]
fn flame_fusillade_tapping_an_aura_or_equipment_doesnt_tap_the_creature() {
    cr!("701.26a", "110.5c");
    ruling!(
        "Flame Fusillade",
        "Tapping an Aura or Equipment doesn't tap the creature it's attached to and vice versa. Tapped enchantments work normally. Tapped artifacts work normally unless they say otherwise."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let sword = attach_new(&mut t, P0, "Bonesplitter", bears);
    let aura = attach_new(&mut t, P0, "Holy Strength", bears);
    assert_eq!(t.pt(bears), (5, 4));
    let ff = t.hand(P0, "Flame Fusillade");
    t.cast(P0, ff).go();
    t.resolve_all();
    // The creature taps: its Aura and Equipment don't.
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_containing(&mut t, P0, bears, "1 damage").expect("creature taps");
    t.resolve_all();
    assert!(tapped(&t, bears) && !tapped(&t, sword) && !tapped(&t, aura));
    // An untapped creature: tapping its Equipment and Aura leaves it untapped, and both
    // still work while tapped.
    t.g.objects[bears.0 as usize].tapped = false;
    for src in [sword, aura] {
        t.answer_targets(P0, &[Entity::Player(P1)]);
        activate_containing(&mut t, P0, src, "1 damage").expect("attachment taps");
        t.resolve_all();
    }
    assert!(tapped(&t, sword) && tapped(&t, aura) && !tapped(&t, bears));
    assert_eq!(t.pt(bears), (5, 4));
    assert_eq!(t.life(P1), 17);
}

// --- No actions while a spell or ability is being paid for -----------------------------------
//
// Played through the priority loop: P0 casts or activates from priority, and P1, with
// answers queued for its first priority, tries to interfere. What P1 sees each time it's
// asked to decide anything (the only times it can act) is recorded: its first chance comes
// with the costs already paid.

/// The uid of `src`'s activated ability whose text contains `needle`.
pub fn ability_uid(t: &mut TestGame, src: ObjectId, needle: &str) -> u64 {
    t.g.recompute();
    t.obj_now(src)
        .chars
        .abilities
        .iter()
        .find(|a| matches!(a.kind, AbilityKind::Activated(_)) && a.text.contains(needle))
        .map(|a| a.uid)
        .expect("no such activated ability")
}

/// Queues `p`'s action for the next time it has priority.
pub fn act(t: &mut TestGame, p: PlayerId, a: Action) {
    t.answer(p, DecisionKind::Priority, Answer::Action(a));
}

/// Queues P0's activation of `src`'s ability containing `needle` from priority (the
/// caller queues its choices).
pub fn p0_activates(t: &mut TestGame, src: ObjectId, needle: &str) {
    let ability = ability_uid(t, src, needle);
    act(
        t,
        P0,
        Action::Activate {
            source: src,
            ability,
        },
    );
}

/// P1 gets a Mountain and a Lightning Bolt, cast at `target` at its first priority.
pub fn p1_bolts_at_first_chance(t: &mut TestGame, target: ObjectId) {
    t.battlefield(P1, "Mountain");
    let bolt = t.hand(P1, "Lightning Bolt");
    act(
        t,
        P1,
        Action::Cast {
            card: bolt,
            method: CastMethod::Normal,
        },
    );
    t.answer_targets(P1, &[obj(target)]);
}

/// What P1 sees each time it's asked to decide anything: the number of objects on the
/// stack and the tapped permanents.
pub fn p1_views(t: &mut TestGame) -> Seen<(usize, Vec<ObjectId>)> {
    watch(
        t,
        P1,
        |_| true,
        |g| {
            let tapped = g.permanents().filter(|o| o.tapped).map(|o| o.id).collect();
            (g.stack.len(), tapped)
        },
    )
}

/// What P1 saw the first time it was asked anything.
pub fn first_view(views: &Seen<(usize, Vec<ObjectId>)>) -> (usize, Vec<ObjectId>) {
    views
        .lock()
        .unwrap()
        .first()
        .cloned()
        .expect("P1 was never asked")
}

/// Plays on (both players passing unless answers are queued) until `done`.
pub fn play_until(t: &mut TestGame, done: fn(&Game) -> bool) {
    assert!(t.g.run_until(2000, done), "{}", t.dump_log());
}

/// P1's Lightning Bolt has resolved, and so has everything else.
fn bolt_resolved(g: &Game) -> bool {
    g.stack.is_empty()
        && !g
            .find_in_zone(Zone::Graveyard(P1), "Lightning Bolt")
            .is_empty()
}

#[test]
fn opposition_opponents_cant_tap_your_permanents_while_you_pay() {
    cr!("601.2", "602.2", "117.3c");
    ruling!(
        "Opposition",
        "Once a player announces that they’re casting a spell or activating an ability, no player may take other actions until the spell or ability has been paid for. Notably, other players can’t try to tap that player’s permanents to stop them from paying {T} or to stop them from producing enough mana."
    );
    supported("Opposition");
    // P0 casts Grizzly Bears with its two Forests and, keeping priority, activates
    // Prodigal Pyromancer's {T} ability at P1. P1 (Opposition and two untapped creatures)
    // taps a Forest, then the Pyromancer, at its first chance: too late.
    let mut t = TestGame::new(2);
    let opp = t.battlefield(P1, "Opposition");
    let p1c: Vec<ObjectId> = (0..2).map(|_| t.battlefield(P1, "Grizzly Bears")).collect();
    let forests = t.lands(P0, "Forest", 2);
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    let bears = t.hand(P0, "Grizzly Bears");
    act(
        &mut t,
        P0,
        Action::Cast {
            card: bears,
            method: CastMethod::Normal,
        },
    );
    p0_activates(&mut t, pyro, "1 damage");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let tap = ability_uid(&mut t, opp, "Tap target");
    for (target, cost) in [(forests[0], p1c[0]), (pyro, p1c[1])] {
        act(
            &mut t,
            P1,
            Action::Activate {
                source: opp,
                ability: tap,
            },
        );
        t.answer_targets(P1, &[obj(target)]);
        t.answer_choose(P1, &[obj(cost)]);
    }
    let views = p1_views(&mut t);
    play_until(&mut t, |g| g.stack.is_empty() && g.player(P1).life == 19);
    // P1's first chance: the spell and the ability on the stack, both paid for.
    let (stack, tapped_then) = first_view(&views);
    assert_eq!(stack, 2);
    assert!(forests
        .iter()
        .chain([&pyro])
        .all(|x| tapped_then.contains(x)));
    // P1 did tap them with Opposition (its creatures paid for it), and nothing was undone.
    assert!(p1c.iter().all(|c| tapped(&t, *c)), "{}", t.dump_log());
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 3);
    assert_eq!(t.life(P1), 19);
}

#[test]
fn aryel_x_is_locked_in_once_the_knights_are_tapped() {
    cr!("601.2h", "602.2b", "107.3a");
    ruling!(
        "Aryel, Knight of Windgrace",
        "Once you announce that you're activating Aryel's last ability, no player may take other actions until the ability's been paid for. Notably, players can't try to make the value of X invalid by removing or tapping your Knights."
    );
    supported("Aryel, Knight of Windgrace");
    // X = 2, two Knights tapped. P1's first chance comes after that; its Lightning Bolt
    // kills one of the Knights, and X stays 2: the Bears (power 2) are destroyed.
    let mut t = TestGame::new(2);
    let aryel = t.battlefield(P0, "Aryel, Knight of Windgrace");
    let k1 = t.battlefield(P0, "Knight of Malice");
    let k2 = t.battlefield(P0, "Knight of Malice");
    t.battlefield(P0, "Swamp");
    let bears = t.battlefield(P1, "Grizzly Bears");
    p0_activates(&mut t, aryel, "Destroy target creature");
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer_choose(P0, &objs(&[k1, k2]));
    t.answer_targets(P0, &[obj(bears)]);
    p1_bolts_at_first_chance(&mut t, k1);
    let views = p1_views(&mut t);
    play_until(&mut t, bolt_resolved);
    let (stack, tapped_then) = first_view(&views);
    assert_eq!(stack, 1);
    assert!([aryel, k1, k2].iter().all(|x| tapped_then.contains(x)));
    assert!(!t.on_battlefield(k1), "{}", t.dump_log());
    assert!(t.in_graveyard(P1, "Grizzly Bears"), "{}", t.dump_log());
}

#[test]
fn deathless_ancient_and_otharri_return_even_if_the_tapped_creatures_leave() {
    cr!("601.2h", "602.2b");
    ruling!(
        "Deathless Ancient",
        "Once you announce that you're activating Deathless Ancient's last ability, no player may take other actions until the ability's been paid for. Notably, players can't try to stop the ability by tapping or removing Vampires you control."
    );
    supported("Deathless Ancient");
    // P1's first chance comes with the three Vampires tapped; it Bolts one of them.
    let mut t = TestGame::new(2);
    let da = t.graveyard(P0, "Deathless Ancient");
    let vs: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P0, "Vampire Noble")).collect();
    p0_activates(&mut t, da, "Tap three untapped Vampires");
    t.answer_choose(P0, &objs(&vs));
    p1_bolts_at_first_chance(&mut t, vs[0]);
    let views = p1_views(&mut t);
    play_until(&mut t, bolt_resolved);
    let (stack, tapped_then) = first_view(&views);
    assert_eq!(stack, 1);
    assert!(vs.iter().all(|v| tapped_then.contains(v)));
    assert!(!t.on_battlefield(vs[0]), "{}", t.dump_log());
    assert!(t.in_hand(P0, "Deathless Ancient"), "{}", t.dump_log());

    ruling!(
        "Otharri, Suns' Glory",
        "Tapping a Rebel you control is part of the cost to activate the last ability of Otharri, Suns' Glory while it is in the graveyard. Once a player has begun to activate the ability, other players may not respond by removing that Rebel from the battlefield to prevent it from being tapped."
    );
    supported("Otharri, Suns' Glory");
    // P1's first chance comes with the Rebel tapped; it Bolts the Rebel.
    let mut t = TestGame::new(2);
    let ot = t.graveyard(P0, "Otharri, Suns' Glory");
    let rebel = t.battlefield(P0, "Rebel Informer");
    t.lands(P0, "Plateau", 4);
    p0_activates(&mut t, ot, "Tap an untapped Rebel");
    t.answer_choose(P0, &[obj(rebel)]);
    p1_bolts_at_first_chance(&mut t, rebel);
    let views = p1_views(&mut t);
    play_until(&mut t, bolt_resolved);
    let (stack, tapped_then) = first_view(&views);
    assert_eq!(stack, 1);
    assert!(tapped_then.contains(&rebel));
    assert!(!t.on_battlefield(rebel), "{}", t.dump_log());
    let o = t.named_on_battlefield("Otharri, Suns' Glory");
    assert_eq!(o.len(), 1, "{}", t.dump_log());
    assert!(tapped(&t, o[0]));
}

#[test]
fn burn_at_the_stake_creatures_are_tapped_together_as_a_cost() {
    cr!("601.2b", "601.2h", "603.2c", "117.5");
    ruling!(
        "Burn at the Stake",
        "Players may not respond to the paying of costs. Once Burn at the Stake has been announced and put on the stack, no one can try to respond by tapping any creatures."
    );
    ruling!(
        "Burn at the Stake",
        "All creatures tapped this way become tapped at the same time."
    );
    supported("Burn at the Stake");
    supported("Deeproot Pilgrimage");
    // P0 taps three nontoken Merfolk for the additional cost, under Deeproot Pilgrimage
    // ("Whenever one or more nontoken Merfolk you control become tapped, create a 1/1 blue
    // Merfolk creature token with hexproof."): tapped together, they trigger it once.
    // P1 (Opposition) taps one of them at its first chance, which comes after the cost
    // was paid: the damage is still three times three.
    let mut t = TestGame::new(2);
    let opp = t.battlefield(P1, "Opposition");
    let p1c = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P0, "Deeproot Pilgrimage");
    let ms: Vec<ObjectId> = (0..3)
        .map(|_| t.battlefield(P0, "Merfolk of the Pearl Trident"))
        .collect();
    t.lands(P0, "Mountain", 5);
    let s = t.hand(P0, "Burn at the Stake");
    act(
        &mut t,
        P0,
        Action::Cast {
            card: s,
            method: CastMethod::Normal,
        },
    );
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(3));
    t.answer_choose(P0, &objs(&ms));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let tap = ability_uid(&mut t, opp, "Tap target");
    act(
        &mut t,
        P1,
        Action::Activate {
            source: opp,
            ability: tap,
        },
    );
    t.answer_targets(P1, &[obj(ms[0])]);
    t.answer_choose(P1, &[obj(p1c)]);
    let views = p1_views(&mut t);
    play_until(&mut t, |g| g.stack.is_empty() && g.player(P1).life < 20);
    // P1's first chance: the spell, its cost paid, under the Pilgrimage's one trigger.
    let (stack, tapped_then) = first_view(&views);
    assert_eq!(stack, 2, "{}", t.dump_log());
    assert!(ms.iter().all(|m| tapped_then.contains(m)));
    assert!(tapped(&t, p1c), "P1 responded");
    assert_eq!(t.life(P1), 11);
    assert_eq!(tokens(&t, P0).len(), 1, "{}", t.dump_log());
}

#[test]
fn vodalian_war_machine_taps_merfolk_as_its_cost() {
    cr!("602.2b", "601.2h");
    ruling!(
        "Vodalian War Machine",
        "The Merfolk are tapped during announcement and as a cost. The ability can't be announced if the Merfolk are not in an untapped state."
    );
    supported("Vodalian War Machine");
    let mut t = TestGame::new(2);
    let wm = t.battlefield(P0, "Vodalian War Machine");
    let m = t.battlefield(P0, "Merfolk of the Pearl Trident");
    t.g.objects[m.0 as usize].tapped = true;
    assert!(!can_activate(&mut t, P0, wm));
    t.g.objects[m.0 as usize].tapped = false;
    assert!(can_activate(&mut t, P0, wm));
    t.answer_choose(P0, &[obj(m)]);
    activate_containing(&mut t, P0, wm, "+2/+1").expect("activates");
    // Tapped on activation, before the ability resolves.
    assert!(tapped(&t, m));
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.pt(wm), (2, 5));
}
