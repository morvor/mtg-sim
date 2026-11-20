//! Item `rulings-residue-shapechange-typal` — interaction rulings about typal payoffs
//! (batch P184): counts and type checks made as an ability resolves or as damage is dealt
//! (CR 608.2h, 615), last known information of a creature that left (CR 608.2h),
//! "is dealt damage" triggers (CR 603.2, 120.3), simultaneous zone changes (CR 603.6a,
//! 603.10a), choices made in APNAP order (CR 101.4) and mana with delayed triggers
//! (CR 106.6).

use crate::r_residue_shapechange_typal_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// ---------------------------------------------------------------------------
// Patriarch's Bidding
// ---------------------------------------------------------------------------

#[test]
fn patriarchs_bidding_choices_in_turn_order_and_returns_at_once() {
    cr!("101.4", "603.6a", "205.3m");
    ruling!(
        "Patriarch's Bidding",
        "The player whose turn it is chooses a creature type first, followed by each other player in turn order. Then, all creature cards that have one or more of the chosen types are returned to the battlefield at the same time."
    );
    ruling!(
        "Patriarch's Bidding",
        "You must choose a creature type, such as Elf or Shaman. You can't choose other card types such as artifact or supertypes such as legendary."
    );
    supported("Patriarch's Bidding");
    supported("Soul Warden");
    // P1's turn; P1 casts it.
    let mut t = TestGame::new(3);
    t.set_step(P1, Step::PrecombatMain);
    t.graveyard(P0, "Soul Warden");
    t.graveyard(P1, "Grizzly Bears");
    t.graveyard(P2, "Llanowar Elves");
    choose_creature_type(&mut t, P0, "Cleric");
    choose_creature_type(&mut t, P1, "Bear");
    choose_creature_type(&mut t, P2, "Elf");
    let from = t.asked().len();
    cast_new(&mut t, P1, "Patriarch's Bidding", &[]);
    t.resolve_all();
    let asked: Vec<(PlayerId, Vec<String>)> = asked_since(&t, from)
        .into_iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseOption {
                prompt, options, ..
            } if prompt == "Choose a creature type" => Some((p, options)),
            _ => None,
        })
        .collect();
    assert_eq!(
        asked.iter().map(|a| a.0).collect::<Vec<_>>(),
        vec![P1, P2, P0]
    );
    // Only creature types are offered.
    for (_, options) in &asked {
        for not in ["Artifact", "Legendary", "Creature", "Land", "Kindred"] {
            assert!(!options.iter().any(|o| o == not), "{not} offered");
        }
        assert!(options.iter().any(|o| o == "Elf"));
    }
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.named_on_battlefield("Llanowar Elves").len(), 1);
    // Soul Warden entered with the other two: it saw both enter.
    assert_eq!(t.life(P0), 22);
}

// ---------------------------------------------------------------------------
// Battletide Alchemist
// ---------------------------------------------------------------------------

#[test]
fn battletide_alchemist_prevents_for_each_player_separately() {
    cr!("615.1a", "615.10", "120.3");
    ruling!(
        "Battletide Alchemist",
        "If a source would deal damage to multiple players at the same time, you may prevent X damage that source would deal to each of those players. The choice for each player is made separately."
    );
    supported("Battletide Alchemist");
    supported("Flame Rift");
    // Two Clerics: X = 2. Prevent for one player and not the other (either order).
    let mut t = main_phase(2);
    t.battlefield(P0, "Battletide Alchemist");
    t.battlefield(P0, "Soul Warden");
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    cast_resolve(&mut t, P0, "Flame Rift", &[]);
    let mut losses = vec![20 - t.life(P0), 20 - t.life(P1)];
    losses.sort();
    assert_eq!(losses, vec![2, 4]);
    // Both prevented.
    let mut t = main_phase(2);
    t.battlefield(P0, "Battletide Alchemist");
    t.battlefield(P0, "Soul Warden");
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    cast_resolve(&mut t, P0, "Flame Rift", &[]);
    assert_eq!((t.life(P0), t.life(P1)), (18, 18));
}

#[test]
fn battletide_alchemist_prevents_each_time_and_counts_clerics_as_damage_is_dealt() {
    cr!("615.1", "615.1a", "615.10");
    ruling!(
        "Battletide Alchemist",
        "If the same source would deal damage to a player at different times, you may prevent X of that damage each time."
    );
    ruling!(
        "Battletide Alchemist",
        "You count the number of Clerics you control at the time the damage would be dealt."
    );
    supported("Pestilence");

    // Pestilence ({B}: 1 damage to each creature and each player), twice: prevented each
    // time.
    let mut t = main_phase(2);
    t.battlefield(P0, "Battletide Alchemist");
    let pest = t.battlefield(P1, "Pestilence");
    t.lands(P1, "Swamp", 2);
    for _ in 0..4 {
        t.answer_yes(P0, true);
    }
    for _ in 0..2 {
        t.activate(P1, pest, 0, &[]).unwrap();
        t.resolve_all();
    }
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
    // Lightning Bolt (3 to P0) cast with one Cleric; a second one enters before it
    // resolves.
    let mut t = main_phase(2);
    t.battlefield(P0, "Battletide Alchemist");
    t.answer_yes(P0, true);
    cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
    t.enter(P0, "Soul Warden");
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
    // And the other way: two Clerics as it's cast, one left as it resolves.
    let mut t = main_phase(2);
    t.battlefield(P0, "Battletide Alchemist");
    let warden = t.battlefield(P0, "Soul Warden");
    t.answer_yes(P0, true);
    cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
    destroy(&mut t, warden);
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
}

// ---------------------------------------------------------------------------
// Wrathful Raptors
// ---------------------------------------------------------------------------

#[test]
fn wrathful_raptors_any_target_that_isnt_a_dinosaur() {
    cr!("115.4", "205.3m");
    ruling!(
        "Wrathful Raptors",
        "\"Any target that isn't a Dinosaur\" means any non-Dinosaur creature, any planeswalker or battle, or any player (no matter how old they are)."
    );
    supported("Wrathful Raptors");
    let mut t = main_phase(2);
    let raptors = t.battlefield(P0, "Wrathful Raptors");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let maw = t.battlefield(P1, "Colossal Dreadmaw");
    let jace = t.battlefield(P1, "Jace Beleren");
    let battle = t.battlefield(P1, "Invasion of Belenon");
    let from = t.asked().len();
    t.answer_targets(P0, &[Entity::Player(P1)]);
    cast_resolve(&mut t, P1, "Shock", &[obj(raptors)]);
    let cands = crate::r_s02_common::target_candidates(&t, P0, from);
    let c = cands.last().expect("asked for a target");
    for e in [
        obj(bears),
        obj(jace),
        obj(battle),
        Entity::Player(P0),
        Entity::Player(P1),
    ] {
        assert!(c.contains(&e), "{e:?} not a candidate");
    }
    assert!(!c.contains(&obj(maw)));
    assert!(!c.contains(&obj(raptors)));
    assert_eq!(t.life(P1), 18);
}

#[test]
fn wrathful_raptors_triggers_for_each_dinosaur_dealt_damage_even_lethal() {
    cr!("603.2", "603.10", "120.3", "704.5g");
    ruling!(
        "Wrathful Raptors",
        "If Wrathful Raptors and one or more other Dinosaurs you control are dealt damage at the same time, Wrathful Raptors's ability triggers for itself and for each of those Dinosaurs, regardless of whether or not Wrathful Raptors was dealt lethal damage."
    );
    ruling!(
        "Wrathful Raptors",
        "If lethal damage is dealt to a Dinosaur you control while you control Wrathful Raptors, Wrathful Raptors's last ability still triggers."
    );
    supported("Pyroclasm");
    supported("Sun-Collared Raptor");
    let mut t = main_phase(2);
    let raptors = t.battlefield(P0, "Wrathful Raptors");
    t.battlefield(P0, "Sun-Collared Raptor");
    // Three damage already marked: Pyroclasm's 2 is lethal for the Raptors too.
    damage(&mut t, raptors, 3);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    cast_new(&mut t, P0, "Pyroclasm", &[]);
    t.resolve();
    // Both died, and both triggers are on the stack.
    assert!(t.in_graveyard(P0, "Wrathful Raptors"));
    assert!(t.in_graveyard(P0, "Sun-Collared Raptor"));
    assert_eq!(triggers_on_stack(&t, "that much damage"), 2);
    t.resolve_all();
    // Each Dinosaur (as it last existed) dealt 2 to P1.
    assert_eq!(t.life(P1), 16);
}

#[test]
fn wrathful_raptors_triggers_once_for_simultaneous_damage_from_several_sources() {
    cr!("603.2c", "510.2", "120.3");
    ruling!(
        "Wrathful Raptors",
        "If multiple sources deal damage to a Dinosaur you control while you control Wrathful Raptors, most likely because multiple creatures blocked that Dinosaur, Wrathful Raptors's last ability triggers only once."
    );
    supported("Looming Altisaur");
    let mut t = main_phase(2);
    t.battlefield(P0, "Wrathful Raptors");
    let alt = t.battlefield(P0, "Looming Altisaur");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(alt, Entity::Player(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(b1, alt), (b2, alt)]),
    );
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "that much damage"), 1);
    t.resolve_all();
    // 4 damage (2 from each Bears) to P1.
    assert_eq!(t.life(P1), 16);
}

#[test]
fn triumphant_chomp_amount_is_determined_on_resolution() {
    cr!("608.2h");
    ruling!(
        "Triumphant Chomp",
        "The amount of damage Triumphant Chomp deals is determined as it resolves. If you don't control any Dinosaurs at that time, or if the Dinosaurs you control are unusually small, Triumphant Chomp will deal 2 damage."
    );
    supported("Triumphant Chomp");
    // Cast with a 6-power Dinosaur that's gone as it resolves: 2 damage.
    let mut t = main_phase(2);
    let maw = t.battlefield(P0, "Colossal Dreadmaw");
    let wurm = t.battlefield(P1, "Craw Wurm");
    cast_new(&mut t, P0, "Triumphant Chomp", &[obj(wurm)]);
    destroy(&mut t, maw);
    t.resolve_all();
    assert_eq!(t.obj_now(wurm).damage, 2);
    // A 1-power Dinosaur: 2 damage. One that grew in response: its new power.
    let mut t = main_phase(2);
    let small = t.battlefield(P0, "Sun-Collared Raptor");
    let wurm = t.battlefield(P1, "Craw Wurm");
    cast_resolve(&mut t, P0, "Triumphant Chomp", &[obj(wurm)]);
    assert_eq!(t.obj_now(wurm).damage, 2);
    let alt = t.battlefield(P1, "Looming Altisaur");
    cast_new(&mut t, P0, "Triumphant Chomp", &[obj(alt)]);
    giant_growth(&mut t, small);
    t.resolve_all();
    assert_eq!(t.obj_now(alt).damage, 4);
}

// ---------------------------------------------------------------------------
// Armored Kincaller, Priest of the Wakening Sun
// ---------------------------------------------------------------------------

#[test]
fn armored_kincaller_checks_for_another_dinosaur_on_resolution() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Armored Kincaller",
        "Whether or not you control another Dinosaur is checked only as Armored Kincaller's ability is resolving. It doesn't matter if you still control Armored Kincaller at the time, or if it's still a Dinosaur."
    );
    supported("Armored Kincaller");
    supported("Turn to Frog");
    // No other Dinosaur, none revealed: no life.
    let mut t = main_phase(2);
    t.answer_yes(P0, false);
    t.enter(P0, "Armored Kincaller");
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    // Another Dinosaur arrives while the trigger waits: 3 life.
    let mut t = main_phase(2);
    t.answer_yes(P0, false);
    t.enter(P0, "Armored Kincaller");
    t.settle();
    t.battlefield(P0, "Colossal Dreadmaw");
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    // Kincaller destroyed in response: still 3 life for the other Dinosaur.
    let mut t = main_phase(2);
    t.battlefield(P0, "Colossal Dreadmaw");
    t.answer_yes(P0, false);
    let k = t.enter(P0, "Armored Kincaller");
    t.settle();
    destroy(&mut t, k);
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    // Kincaller turned into a Frog in response: still 3 life.
    let mut t = main_phase(2);
    t.battlefield(P0, "Colossal Dreadmaw");
    t.answer_yes(P0, false);
    let k = t.enter(P0, "Armored Kincaller");
    t.settle();
    cast_new(&mut t, P1, "Turn to Frog", &[obj(k)]);
    t.resolve();
    assert!(!crate::r_residue_type_change_common::subtype(
        &mut t, k, "Dinosaur"
    ));
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
}

/// P0's upkeep begins with the given Priests of the Wakening Sun on the battlefield.
fn priest_upkeep(t: &mut TestGame) {
    t.advance_to(P0, Step::Upkeep);
    t.settle();
}

#[test]
fn priest_of_the_wakening_sun_reveals() {
    cr!("603.3b", "701.20a", "608.2c");
    ruling!(
        "Priest of the Wakening Sun",
        "While resolving Priest of the Wakening Sun’s triggered ability, you can’t reveal multiple Dinosaur cards to gain more life."
    );
    ruling!(
        "Priest of the Wakening Sun",
        "You can reveal the same Dinosaur card for multiple Priests of the Wakening Sun or for the same one over multiple turns."
    );
    supported("Priest of the Wakening Sun");
    // Two Dinosaur cards in hand: 2 life.
    let mut t = main_phase(2);
    t.battlefield(P0, "Priest of the Wakening Sun");
    t.hand(P0, "Colossal Dreadmaw");
    t.hand(P0, "Sun-Collared Raptor");
    for _ in 0..3 {
        t.answer_yes(P0, true);
    }
    priest_upkeep(&mut t);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    // Two Priests, one Dinosaur card: 4 life, and 4 more the next turn.
    let mut t = main_phase(2);
    t.battlefield(P0, "Priest of the Wakening Sun");
    t.battlefield(P0, "Priest of the Wakening Sun");
    t.hand(P0, "Colossal Dreadmaw");
    for _ in 0..8 {
        t.answer_yes(P0, true);
    }
    priest_upkeep(&mut t);
    t.resolve_all();
    assert_eq!(t.life(P0), 24);
    t.advance_to(P1, Step::Upkeep);
    priest_upkeep(&mut t);
    t.resolve_all();
    assert_eq!(t.life(P0), 28);
}

#[test]
fn priest_of_the_wakening_sun_reveal_is_chosen_on_resolution() {
    cr!("608.2c", "603.3", "603.5");
    ruling!(
        "Priest of the Wakening Sun",
        "You don’t choose whether to reveal a Dinosaur card from your hand until the triggered ability of Priest of the Wakening Sun resolves. You may respond to the triggered ability by taking an action to get a Dinosaur card into your hand, such as activating its second ability."
    );
    let mut t = main_phase(2);
    let priest = t.battlefield(P0, "Priest of the Wakening Sun");
    t.library_top(P0, "Colossal Dreadmaw");
    priest_upkeep(&mut t);
    assert_eq!(triggers_on_stack(&t, "reveal a Dinosaur"), 1);
    assert!(!t.in_hand(P0, "Colossal Dreadmaw"));
    // In response: sacrifice the Priest to search for the Dreadmaw.
    t.lands(P0, "Plains", 2);
    t.lands(P0, "Wastes", 3);
    let maw =
        t.g.find_in_zone(mtg_engine::object::Zone::Library(P0), "Colossal Dreadmaw")[0];
    t.answer_choose(P0, &[obj(maw)]);
    t.activate(P0, priest, 0, &[]).unwrap();
    t.resolve();
    assert!(t.in_hand(P0, "Colossal Dreadmaw"));
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
}

// ---------------------------------------------------------------------------
// Scaled Nurturer
// ---------------------------------------------------------------------------

#[test]
fn scaled_nurturer_mana_triggers_per_mana_and_without_the_nurturer() {
    cr!("106.6", "603.7a", "603.7e");
    ruling!(
        "Scaled Nurturer",
        "If more than one {G} produced by a Scaled Nurturer is spent to cast a single Dragon creature spell, the delayed triggered ability associated with each mana spent will trigger. You will gain 2 life that many times."
    );
    ruling!(
        "Scaled Nurturer",
        "The delayed triggered ability will trigger whether Scaled Nurturer is still on the battlefield or not."
    );
    ruling!(
        "Scaled Nurturer",
        "The mana created by Scaled Nurturer can be spent on anything, not just Dragon creature spells."
    );
    supported("Scaled Nurturer");
    // Two Nurturers' {G}{G} for a Dragon whose cost has generic mana: 4 life.
    let mut t = main_phase(2);
    let n1 = t.battlefield(P0, "Scaled Nurturer");
    let n2 = t.battlefield(P0, "Scaled Nurturer");
    t.lands(P0, "Mountain", 4);
    t.activate(P0, n1, 0, &[]).unwrap();
    t.activate(P0, n2, 0, &[]).unwrap();
    let dragon = t.hand(P0, "Shivan Dragon");
    t.cast(P0, dragon).go();
    t.settle();
    // The Dragon spell and one delayed trigger for each {G}.
    assert_eq!(on_stack(&t), 3);
    t.resolve_all();
    assert_eq!(t.life(P0), 24);
    // A Nurturer that left the battlefield after its mana was made: still 2 life.
    let mut t = main_phase(2);
    let n = t.battlefield(P0, "Scaled Nurturer");
    t.lands(P0, "Mountain", 5);
    t.activate(P0, n, 0, &[]).unwrap();
    destroy(&mut t, n);
    let dragon = t.hand(P0, "Shivan Dragon");
    t.cast(P0, dragon).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    // Spent on something else: no life, but it pays.
    let mut t = main_phase(2);
    let n = t.battlefield(P0, "Scaled Nurturer");
    t.lands(P0, "Wastes", 1);
    t.activate(P0, n, 0, &[]).unwrap();
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.resolve_all();
    assert!(t.on_battlefield(bears) || !t.named_on_battlefield("Grizzly Bears").is_empty());
    assert_eq!(t.life(P0), 20);
}

// ---------------------------------------------------------------------------
// Tiamat
// ---------------------------------------------------------------------------

#[test]
fn tiamat_triggers_when_cast_from_any_zone() {
    cr!("603.4", "601.2a", "903.8");
    ruling!(
        "Tiamat",
        "Tiamat's enters-the-battlefield ability will trigger if you cast Tiamat from any zone, not just from your hand. Still thinking command zone over here."
    );
    supported("Tiamat");
    supported("Future Sight");
    // Cast from the top of the library with Future Sight.
    let mut t = main_phase(2);
    t.battlefield(P0, "Future Sight");
    t.library_top(P0, "Shivan Dragon");
    let tiamat = t.library_top(P0, "Tiamat");
    lands_for_cost(&mut t, P0, "Tiamat");
    t.cast(P0, tiamat).go();
    t.resolve();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "search your library"), 1);
    t.answer_choose(P0, &[]);
    t.resolve_all();
    // Cast from the command zone as a commander (CR 903.8): it finds a Dragon.
    let mut t = TestGame::with_config(2, mtg_engine::game::GameConfig::commander_game());
    t.set_step(P0, Step::PrecombatMain);
    let tiamat = t.command(P0, "Tiamat");
    t.g.objects[tiamat.0 as usize].is_commander = true;
    t.g.players[0].commander_names.push("Tiamat".into());
    let shivan = t.library_top(P0, "Shivan Dragon");
    lands_for_cost(&mut t, P0, "Tiamat");
    t.cast(P0, tiamat).go();
    t.resolve();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "search your library"), 1);
    t.answer_choose_moved(P0, &[obj(shivan)]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Shivan Dragon"));
    // Put onto the battlefield without being cast: no trigger.
    let mut t = main_phase(2);
    t.enter(P0, "Tiamat");
    t.settle();
    assert_eq!(triggers_on_stack(&t, "search your library"), 0);
}

// ---------------------------------------------------------------------------
// Overgrowth Elemental
// ---------------------------------------------------------------------------

#[test]
fn overgrowth_elemental_dying_with_others() {
    cr!("603.10a", "603.6c", "400.7");
    ruling!(
        "Overgrowth Elemental",
        "If Overgrowth Elemental dies at the same time as one or more other creatures you control, Overgrowth Elemental’s last ability triggers for each of those other creatures. You will gain 1 life for each of them, but if any of those creatures were Elementals, Overgrowth Elemental won’t get a +1/+1 counter since it’s already left the battlefield."
    );
    supported("Overgrowth Elemental");
    supported("Air Elemental");
    let mut t = main_phase(2);
    let og = t.battlefield(P0, "Overgrowth Elemental");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Air Elemental");
    cast_new(&mut t, P0, "Wrath of God", &[]);
    t.resolve();
    assert_eq!(triggers_on_stack(&t, "gain 1 life"), 2);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    let og_now = t.g.current(og);
    assert_eq!(plus1(&t, og_now), 0);
    assert!(t.in_graveyard(P0, "Overgrowth Elemental"));
    // Without it dying too, the Elemental gives it a counter.
    let mut t = main_phase(2);
    let og = t.battlefield(P0, "Overgrowth Elemental");
    let air = t.battlefield(P0, "Air Elemental");
    destroy(&mut t, air);
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    assert_eq!(plus1(&t, og), 1);
}

#[test]
fn overgrowth_elemental_cant_save_you_from_simultaneous_lethal_damage() {
    cr!("704.3", "704.5a", "117.5");
    ruling!(
        "Overgrowth Elemental",
        "If a creature you control dies at the same time that your life total becomes 0 or less, you lose the game before Overgrowth Elemental’s last ability can save you."
    );
    supported("Earthquake");
    let mut t = main_phase(2);
    t.battlefield(P0, "Overgrowth Elemental");
    t.battlefield(P0, "Llanowar Elves");
    t.g.players[0].life = 1;
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 1);
    let quake = t.hand(P0, "Earthquake");
    t.cast(P0, quake).x(1).go();
    t.resolve();
    assert!(t.has_lost(P0));
    assert_eq!(t.life(P0), 0);
}

// ---------------------------------------------------------------------------
// Thornmantle Striker
// ---------------------------------------------------------------------------

/// P0's Thornmantle Striker enters choosing mode `mode` with `target`; the trigger is on
/// the stack.
fn striker_enters(t: &mut TestGame, mode: usize, target: Entity) -> ObjectId {
    supported("Thornmantle Striker");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![mode]));
    t.answer_targets(P0, &[target]);
    let s = t.enter(P0, "Thornmantle Striker");
    t.settle();
    s
}

#[test]
fn thornmantle_striker_counts_elves_on_resolution() {
    cr!("608.2h");
    ruling!(
        "Thornmantle Striker",
        "For each mode, use the number of Elves you control as the enters-the-battlefield ability resolves, including Thornmantle Striker itself if it’s still under your control."
    );
    // Striker and Llanowar Elves: X = 2, the Bears die.
    let mut t = main_phase(2);
    t.battlefield(P0, "Llanowar Elves");
    let bears = t.battlefield(P1, "Grizzly Bears");
    striker_enters(&mut t, 1, obj(bears));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    // The Striker bounced in response: X = 1.
    let mut t = main_phase(2);
    t.battlefield(P0, "Llanowar Elves");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let s = striker_enters(&mut t, 1, obj(bears));
    cast_new(&mut t, P1, "Unsummon", &[obj(s)]);
    t.resolve();
    t.resolve_all();
    assert_eq!(pt_now(&mut t, bears), (1, 1));
    // An Elf entering in response counts: X = 3 for the first mode.
    let mut t = main_phase(2);
    let jace = t.battlefield(P1, "Jace Beleren");
    striker_enters(&mut t, 0, obj(jace));
    t.battlefield(P0, "Llanowar Elves");
    t.battlefield(P0, "Llanowar Elves");
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Jace Beleren"));
}

#[test]
fn thornmantle_striker_removes_counters_and_a_planeswalker_without_loyalty_dies() {
    cr!("704.5i", "608.2c");
    ruling!(
        "Thornmantle Striker",
        "A planeswalker with no loyalty counters on it is put into its owner’s graveyard."
    );
    ruling!(
        "Thornmantle Striker",
        "For the first mode, you choose which counters are removed as the ability resolves. If the permanent has fewer counters on it than the number of Elves you control, you’ll remove all of them."
    );
    // Jace Beleren (3 loyalty), X = 3: it dies.
    let mut t = main_phase(2);
    t.battlefield(P0, "Llanowar Elves");
    t.battlefield(P0, "Llanowar Elves");
    let jace = t.battlefield(P1, "Jace Beleren");
    striker_enters(&mut t, 0, obj(jace));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Jace Beleren"));
    // Fewer counters than Elves: all of them go.
    let mut t = main_phase(2);
    for _ in 0..3 {
        t.battlefield(P0, "Llanowar Elves");
    }
    let bears = t.battlefield(P1, "Grizzly Bears");
    give_plus1(&mut t, bears, 1);
    t.g.add_counters(Entity::Object(bears), "stun", 1, None);
    striker_enters(&mut t, 0, obj(bears));
    t.resolve_all();
    assert_eq!(plus1(&t, bears), 0);
    assert_eq!(t.counters(bears, "stun"), 0);
    // A choice among kinds, made as the ability resolves: X = 1 with a +1/+1 counter and
    // a stun counter; P0's choice is the one removed (the second kind offered).
    let mut t = main_phase(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    give_plus1(&mut t, bears, 1);
    t.g.add_counters(Entity::Object(bears), "stun", 1, None);
    striker_enters(&mut t, 0, obj(bears));
    // The trigger is on the stack: no counter has been chosen yet.
    let from = t.asked().len();
    let choices = |t: &TestGame| {
        asked_since(t, from)
            .into_iter()
            .filter_map(|(p, d)| match d {
                Decision::ChooseOption {
                    prompt, options, ..
                } if p == P0 && prompt == "Choose a counter to remove" => Some(options),
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    assert!(choices(&t).is_empty());
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.resolve_all();
    let asked = choices(&t);
    assert_eq!(asked.len(), 1);
    assert_eq!(asked[0].len(), 2);
    let (plus, stun) = (plus1(&t, bears), t.counters(bears, "stun"));
    assert_eq!(plus + stun, 1);
    if asked[0][1].contains("stun") {
        assert_eq!(stun, 0);
    } else {
        assert_eq!(plus, 0);
    }
}

// ---------------------------------------------------------------------------
// Arboreal Alliance, Tajuru Paragon
// ---------------------------------------------------------------------------

#[test]
fn arboreal_alliance_with_x_0_makes_a_0_0_token() {
    cr!("107.3m", "704.5f");
    ruling!(
        "Arboreal Alliance",
        "If Arboreal Alliance enters the battlefield without being cast, or if you cast it with X equal to 0, you'll create a 0/0 green Treefolk creature token. Unless something else is raising its toughness, it will immediately die."
    );
    supported("Arboreal Alliance");
    // Put onto the battlefield without being cast, or cast with X = 0: the token is
    // created (Soul Warden sees it enter) and dies at once.
    for cast in [false, true] {
        let mut t = main_phase(2);
        t.battlefield(P0, "Soul Warden");
        if cast {
            t.lands(P0, "Forest", 2);
            let aa = t.hand(P0, "Arboreal Alliance");
            t.cast(P0, aa).x(0).go();
        } else {
            t.enter(P0, "Arboreal Alliance");
        }
        t.resolve_all();
        assert!(!t.named_on_battlefield("Arboreal Alliance").is_empty());
        assert_eq!(t.life(P0), 21, "cast {cast}");
        assert_eq!(crate::r_s25_common::creature_tokens(&t, P0), 0);
    }
    // Something raising its toughness: with Glorious Anthem, the 0/0 survives as a 1/1.
    for cast in [false, true] {
        let mut t = main_phase(2);
        t.battlefield(P0, "Glorious Anthem");
        if cast {
            t.lands(P0, "Forest", 2);
            let aa = t.hand(P0, "Arboreal Alliance");
            t.cast(P0, aa).x(0).go();
        } else {
            t.enter(P0, "Arboreal Alliance");
        }
        t.resolve_all();
        let toks = crate::r_s01_common::tokens(&t, P0);
        assert_eq!(toks.len(), 1, "cast {cast}");
        assert_eq!(pt_now(&mut t, toks[0]), (1, 1));
        assert!(crate::r_residue_type_change_common::subtype(
            &mut t, toks[0], "Treefolk"
        ));
        assert!(has_color(&mut t, toks[0], Color::Green));
    }
}

#[test]
fn tajuru_paragon_uses_its_last_known_creature_types() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Tajuru Paragon",
        "If Tajuru Paragon leaves the battlefield before its triggered ability resolves, use its creature types as it last existed on the battlefield to determine which cards you may put into your hand."
    );
    supported("Tajuru Paragon");
    supported("Mirror Entity");
    // The Paragon (an Elf Cleric Rogue Warrior Wizard) dies with its trigger on the stack.
    // With Mirror Entity's "gain all creature types until end of turn" in response, it had
    // every creature type as it last existed (the card in the graveyard is no Bear): a
    // Bear shares one of them. Without it, the Bear can't be taken.
    for mirror in [false, true] {
        let mut t = main_phase(2);
        let entity = t.battlefield(P0, "Mirror Entity");
        let bears = t.library_top(P0, "Grizzly Bears");
        t.lands(P0, "Forest", 1);
        t.lands(P0, "Wastes", 4);
        let para = t.hand(P0, "Tajuru Paragon");
        t.cast(P0, para).kicked(true).go();
        t.resolve();
        t.settle();
        assert_eq!(triggers_on_stack(&t, "reveal the top six"), 1);
        if mirror {
            t.lands(P0, "Wastes", 3);
            t.answer(P0, DecisionKind::X, Answer::Number(3));
            t.activate(P0, entity, 0, &[]).unwrap();
            t.resolve();
        }
        let para_now = t.g.current(para);
        assert_eq!(
            crate::r_residue_type_change_common::subtype(&mut t, para_now, "Bear"),
            mirror
        );
        destroy(&mut t, para_now);
        let card = t.g.current(para_now);
        assert!(t.in_graveyard(P0, "Tajuru Paragon"));
        assert!(!crate::r_residue_type_change_common::subtype(
            &mut t, card, "Bear"
        ));
        t.answer_choose_moved(P0, &[obj(bears)]);
        t.resolve_all();
        assert_eq!(
            t.in_hand(P0, "Grizzly Bears"),
            mirror,
            "Mirror Entity {mirror}"
        );
    }
}
