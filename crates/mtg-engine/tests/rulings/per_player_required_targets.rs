//! Item ruling-conflicts-7: a required target chosen for each player ("for each player,
//! choose target permanent that player controls", "for each opponent, destroy target
//! noncreature permanent that player controls"). The text defines the number of targets,
//! one for each player it names (CR 601.2c), so if any of those players has no legal
//! target, the spell can't be cast (CR 601.2) and a triggered ability is removed from the
//! stack (CR 603.3d), as Vaevictis Asmadi, the Dire's ruling says. A player who has one
//! must get one (Sylvan Primordial's ruling). "Up to one target" for each player (Mass
//! Mutiny) allows a player none (CR 115.6). Register entry `per-player-required-targets`.

use crate::r_p138_common::*;
use crate::r_s01_common::{custom_card, stack_library};
use mtg_engine::decision::{Action, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// The targets of the stack object `id`, in order.
fn targets_of(t: &TestGame, id: ObjectId) -> Vec<Entity> {
    t.obj(id)
        .stack
        .as_ref()
        .expect("on the stack")
        .chosen
        .iter()
        .flat_map(|cm| cm.targets.iter().flatten().copied())
        .collect()
}

/// The targets of the ability or spell on top of the stack, or None if the stack is empty.
fn top_targets(t: &TestGame) -> Option<Vec<Entity>> {
    let id = *t.g.stack.last()?;
    Some(targets_of(t, id))
}

/// The minimum of each target choice asked since `from`.
fn target_minimums(t: &TestGame, from: usize) -> Vec<u32> {
    t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseTargets { min, .. } => Some(*min),
            _ => None,
        })
        .collect()
}

/// Whether `p`'s legal actions include casting `card`.
fn castable(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    t.g.legal_actions(p)
        .iter()
        .any(|a| matches!(a, Action::Cast { card: c, .. } if *c == card))
}

/// Whether `p`'s legal actions include activating an ability of `source`.
fn activatable(t: &mut TestGame, p: PlayerId, source: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    t.g.legal_actions(p)
        .iter()
        .any(|a| matches!(a, Action::Activate { source: s, .. } if *s == source))
}

#[test]
fn vaevictis_ability_is_removed_from_the_stack_if_a_player_controls_no_permanents() {
    cr!("603.3d", "601.2c", "115.1d");
    ruling!(
        "Vaevictis Asmadi, the Dire",
        "If any player controls no permanents, Vaevictis's ability won't have sufficient legal targets, so it's removed from the stack without any effect."
    );
    interp!("per-player-required-targets");
    supported("Vaevictis Asmadi, the Dire");
    // P1 controls no permanents: the ability triggers, but has no legal target for P1.
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Vaevictis Asmadi, the Dire");
    let forest = t.battlefield(P0, "Forest");
    let top = stack_library(&mut t, P0, &["Hill Giant"])[0];
    t.answer_targets(P0, &[obj(forest)]);
    attack_with(&mut t, &[(v, pl(P1))]);
    assert_eq!(t.stack_len(), 0, "removed from the stack");
    t.resolve_all();
    assert!(t.on_battlefield(forest), "nothing is sacrificed");
    assert!(t.on_battlefield(v));
    assert_eq!(t.zone(top), Zone::Library(P0), "nothing is revealed or put onto the battlefield");

    // In a multiplayer game, one player without a permanent is enough.
    let mut t = TestGame::new(3);
    let v = t.battlefield(P0, "Vaevictis Asmadi, the Dire");
    let forest = t.battlefield(P0, "Forest");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(forest)]);
    t.answer_targets(P0, &[obj(bears)]);
    attack_with(&mut t, &[(v, pl(P1))]);
    assert_eq!(t.stack_len(), 0, "P2 controls no permanents");
    t.resolve_all();
    assert!(t.on_battlefield(forest));
    assert!(t.on_battlefield(bears));
}

#[test]
fn vaevictis_must_target_one_of_your_own_permanents_too() {
    cr!("601.2c", "603.3d");
    ruling!(
        "Vaevictis Asmadi, the Dire",
        "You must target one of your permanents to sacrifice as well."
    );
    interp!("per-player-required-targets");
    // Every player controls a permanent: the ability goes on the stack with a target for
    // each of them, its controller included. Trying to choose none of P0's own isn't a
    // legal choice: one of them is targeted anyway.
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Vaevictis Asmadi, the Dire");
    let forest = t.battlefield(P0, "Forest");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[]);
    t.answer_targets(P0, &[obj(bears)]);
    let from = t.asked().len();
    attack_with(&mut t, &[(v, pl(P1))]);
    assert_eq!(target_minimums(&t, from), vec![1, 1]);
    let targets = top_targets(&t).expect("on the stack");
    assert_eq!(targets.len(), 2);
    assert!(targets.contains(&obj(bears)));
    let mine = targets[0];
    assert!(mine == obj(v) || mine == obj(forest), "P0's own permanent: {mine:?}");
    t.resolve();
    assert!(!t.on_battlefield(bears));
    assert_ne!(
        t.on_battlefield(v),
        t.on_battlefield(forest),
        "one of P0's permanents is sacrificed"
    );
}

#[test]
fn sylvan_primordial_must_target_a_permanent_of_each_opponent_who_has_one() {
    cr!("601.2c", "603.3d");
    ruling!(
        "Sylvan Primordial",
        "If an opponent has any legal permanents to target, you must target one of them."
    );
    interp!("per-player-required-targets");
    supported("Sylvan Primordial");
    // Each opponent has a noncreature permanent: one must be targeted for each, even if
    // the controller tries to choose none.
    let mut t = TestGame::new(3);
    let forest = t.battlefield(P1, "Forest");
    let island = t.battlefield(P2, "Island");
    t.answer_targets(P0, &[]);
    t.answer_targets(P0, &[]);
    let from = t.asked().len();
    t.enter(P0, "Sylvan Primordial");
    t.settle();
    assert_eq!(target_minimums(&t, from), vec![1, 1]);
    assert_eq!(top_targets(&t), Some(vec![obj(forest), obj(island)]));
    t.resolve_all();
    assert!(!t.on_battlefield(forest));
    assert!(!t.on_battlefield(island));
}

/// Puts `card` onto the battlefield under P0's control in a game of `players` players,
/// where each `(player, name)` of `permanents` controls that card, and returns how many
/// targets its enters-the-battlefield ability has, or None if it isn't on the stack. Each
/// target choice asked is answered with the next of those permanents.
fn enters_targets(players: usize, card: &str, permanents: &[(PlayerId, &str)]) -> Option<usize> {
    supported(card);
    let mut t = TestGame::new(players);
    for (p, name) in permanents {
        let id = t.battlefield(*p, name);
        t.answer_targets(P0, &[obj(id)]);
    }
    t.enter(P0, card);
    t.settle();
    top_targets(&t).map(|v| v.len())
}

#[test]
fn enters_abilities_with_a_required_target_for_each_player_need_one_for_every_player() {
    cr!("603.3d", "601.2c", "115.1d", "115.6");
    interp!("per-player-required-targets");
    ruling!(
        "Molten Primordial",
        "You can choose a number of targets up to the number of opponents you have, one target per opponent."
    );
    // (card, players, permanents, number of targets of the ability on the stack, or None
    // if it's removed from the stack)
    let cases: &[(&str, usize, &[(PlayerId, &str)], Option<usize>)] = &[
        // "for each opponent, destroy target noncreature permanent that player controls"
        ("Sylvan Primordial", 3, &[(P1, "Forest")], None),
        // A creature isn't a noncreature permanent.
        ("Sylvan Primordial", 3, &[(P1, "Forest"), (P2, "Grizzly Bears")], None),
        ("Sylvan Primordial", 3, &[(P1, "Forest"), (P2, "Island")], Some(2)),
        // "for each opponent, goad target creature that player controls": a creature
        // with hexproof isn't a legal target.
        ("Frenzied Gorespawn", 3, &[(P1, "Grizzly Bears"), (P2, "Forest")], None),
        ("Frenzied Gorespawn", 3, &[(P1, "Grizzly Bears"), (P2, "Gladecover Scout")], None),
        ("Frenzied Gorespawn", 3, &[(P1, "Grizzly Bears"), (P2, "Hill Giant")], Some(2)),
        // "for each player, put a contested counter on target land that player controls":
        // its controller too.
        ("Turf War", 2, &[(P1, "Forest")], None),
        ("Turf War", 2, &[(P0, "Mountain"), (P1, "Forest")], Some(2)),
        // "for each opponent, gain control of up to one target creature that player
        // controls": none for an opponent without a creature.
        ("Molten Primordial", 3, &[(P1, "Grizzly Bears")], Some(1)),
        ("Molten Primordial", 3, &[], Some(0)),
    ];
    for (card, players, permanents, expected) in cases {
        assert_eq!(
            enters_targets(*players, card, permanents),
            *expected,
            "{card} with {permanents:?}"
        );
    }
}

#[test]
fn turf_war_counters_go_on_a_land_of_each_player() {
    cr!("603.3d", "608.2c");
    interp!("per-player-required-targets");
    supported("Turf War");
    let mut t = TestGame::new(2);
    let mountain = t.battlefield(P0, "Mountain");
    let forest = t.battlefield(P1, "Forest");
    t.enter(P0, "Turf War");
    t.resolve_all();
    assert_eq!(t.counters(mountain, "contested"), 1);
    assert_eq!(t.counters(forest, "contested"), 1);
}

#[test]
fn blatant_thievery_cant_be_cast_unless_each_opponent_has_a_legal_target() {
    cr!("601.2", "601.2c", "115.1a", "733.1");
    interp!("per-player-required-targets");
    supported("Blatant Thievery");
    // "For each opponent, gain control of target permanent that player controls." P2
    // controls no permanents, then only a creature with hexproof: Blatant Thievery isn't
    // offered, and trying to cast it is reversed.
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    lots(&mut t, P0);
    lots(&mut t, P0);
    let spell = t.hand(P0, "Blatant Thievery");
    assert!(!castable(&mut t, P0, spell));
    t.answer_targets(P0, &[obj(bears)]);
    assert!(t.cast(P0, spell).try_go().is_err());
    assert!(t.in_hand(P0, "Blatant Thievery"));
    assert_eq!(t.stack_len(), 0);
    let scout = t.battlefield(P2, "Gladecover Scout");
    assert!(!castable(&mut t, P0, spell));
    t.clear_answers();
    t.answer_targets(P0, &[obj(bears)]);
    assert!(t.cast(P0, spell).try_go().is_err());
    assert!(t.in_hand(P0, "Blatant Thievery"));
    // P2 gets a permanent it can be cast for.
    let elves = t.battlefield(P2, "Llanowar Elves");
    assert!(castable(&mut t, P0, spell));
    t.clear_answers();
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_targets(P0, &[obj(elves)]);
    let id = t.cast(P0, spell).go();
    assert_eq!(targets_of(&t, id), vec![obj(bears), obj(elves)]);
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P0);
    assert_eq!(t.obj_now(elves).controller, P0);
    assert_eq!(t.obj_now(scout).controller, P2);
}

#[test]
fn guff_rewrites_history_needs_a_target_for_its_caster_too() {
    cr!("601.2", "601.2c");
    interp!("per-player-required-targets");
    supported("Guff Rewrites History");
    // "For each player, choose target nonenchantment, nonland permanent that player
    // controls." Its caster controls only a land: it can't be cast.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Forest");
    t.battlefield(P1, "Grizzly Bears");
    lots(&mut t, P0);
    let spell = t.hand(P0, "Guff Rewrites History");
    assert!(!castable(&mut t, P0, spell));
    assert!(t.cast(P0, spell).try_go().is_err());
    assert!(t.in_hand(P0, "Guff Rewrites History"));
    let elves = t.battlefield(P0, "Llanowar Elves");
    assert!(castable(&mut t, P0, spell));
    t.clear_answers();
    let id = t.cast(P0, spell).go();
    assert_eq!(targets_of(&t, id).len(), 2);
    assert!(targets_of(&t, id).contains(&obj(elves)));
}

#[test]
fn an_activated_ability_with_a_required_target_for_each_opponent_needs_one_for_each() {
    cr!("602.2", "602.2b", "601.2c");
    interp!("per-player-required-targets");
    // No card in the pool has a required target for each player on an activated ability,
    // so a custom one, compiled from its text: P2 controls no creature, so it can't be
    // activated, and an attempt is reversed before its cost is paid.
    let def = custom_card(
        "Per-Player Warden",
        "Artifact",
        "{1}",
        None,
        "{T}: For each opponent, tap target creature that player controls.",
    );
    let mut t = TestGame::new(3);
    let warden = t.custom(P0, def, Zone::Battlefield);
    let bears = t.battlefield(P1, "Grizzly Bears");
    assert!(!activatable(&mut t, P0, warden));
    assert!(t.activate(P0, warden, 0, &[obj(bears)]).is_err());
    assert!(!t.obj_now(warden).tapped);
    assert_eq!(t.stack_len(), 0);
    let giant = t.battlefield(P2, "Hill Giant");
    assert!(activatable(&mut t, P0, warden));
    t.clear_answers();
    t.activate(P0, warden, 0, &[obj(bears), obj(giant)])
        .expect("activated");
    t.resolve_all();
    assert!(t.obj_now(bears).tapped);
    assert!(t.obj_now(giant).tapped);
}

#[test]
fn mass_mutiny_is_cast_with_no_target_for_an_opponent_without_a_creature() {
    cr!("601.2c", "115.6");
    ruling!(
        "Mass Mutiny",
        "You can cast Mass Mutiny even if an opponent doesn't control any creatures. You simply won't choose a target for that opponent."
    );
    interp!("per-player-required-targets");
    // "For each opponent, gain control of up to one target creature that player controls
    // until end of turn." P2 controls no creatures.
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P2, "Forest");
    lots(&mut t, P0);
    let spell = t.hand(P0, "Mass Mutiny");
    assert!(castable(&mut t, P0, spell));
    t.answer_targets(P0, &[obj(bears)]);
    let id = t.cast(P0, spell).go();
    assert_eq!(targets_of(&t, id), vec![obj(bears)]);
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P0);
}

#[test]
fn mysterious_stranger_targets_only_graveyards_with_an_instant_or_sorcery_card() {
    cr!("601.2c", "603.3d");
    interp!("per-player-required-targets");
    supported("Mysterious Stranger");
    // "for each graveyard with an instant or sorcery card in it, exile target instant or
    // sorcery card from that graveyard": P0's and P2's graveyards have none, so the
    // ability needs only P1's.
    let mut t = TestGame::new(3);
    let bolt = t.graveyard(P1, "Lightning Bolt");
    t.graveyard(P2, "Grizzly Bears");
    t.enter(P0, "Mysterious Stranger");
    t.settle();
    assert_eq!(top_targets(&t), Some(vec![obj(bolt)]));
    t.resolve();
    assert_eq!(t.zone(bolt), Zone::Exile);

    // A graveyard with an instant card in it is counted even if the card can't be
    // targeted ("Cards in graveyards can't be the targets of spells or abilities."): the
    // ability is removed from the stack.
    let mut t = TestGame::new(3);
    let bolt = t.graveyard(P1, "Lightning Bolt");
    t.battlefield(P1, "Ground Seal");
    t.enter(P0, "Mysterious Stranger");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.zone(bolt), Zone::Graveyard(P1));
}
