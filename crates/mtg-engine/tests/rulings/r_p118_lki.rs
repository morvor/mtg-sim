//! Rulings batch P118 — abilities that use their source's power after it has left the
//! battlefield: its power as it last existed there is used (CR 603.10a, 608.2h, 113.7a),
//! not the power of the card in its new zone. Each test gives the source +1/+1 counters
//! first, so the last known power differs from the printed power of the card in the
//! graveyard (or exile).

use crate::r_p118_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::*;

/// P0's `name` with `n` +1/+1 counters is destroyed, its dies trigger targeting `targets`
/// (if any); returns the game after the stack has resolved.
fn dies_with(t: &mut TestGame, name: &str, n: u32, targets: &[Entity]) -> ObjectId {
    supported(name);
    let c = with_counters(t, P0, name, n);
    if !targets.is_empty() {
        t.answer_targets(P0, targets);
    }
    destroy(t, c);
    assert!(!t.on_battlefield(c), "{name}");
    t.resolve_all();
    c
}

#[test]
fn dies_triggers_use_the_last_known_power() {
    cr!("603.10a", "608.2h", "113.7a");
    ruling!(
        "Balduvian Berserker",
        "To determine the amount of damage it deals, use Balduvian Berserker’s power as it last existed on the battlefield, not its power in the graveyard."
    );
    ruling!(
        "Infernal Phantom",
        "Use Infernal Phantom's power as it last existed on the battlefield to determine how much damage it deals due to its last ability."
    );
    ruling!(
        "Mortis Dogs",
        "The loss of life is equal to Mortis Dogs’s power as it last existed on the battlefield."
    );
    ruling!(
        "Heartfire Hero",
        "Use Heartfire Hero's power as it last existed on the battlefield to determine how much damage it deals due to its last ability."
    );
    ruling!(
        "Bottle Golems",
        "Use Bottle Golems' power when it was on the battlefield to determine the amount of life you gain. This may be different than its power in the graveyard."
    );
    // (card, counters, P1's life lost, P0's life gained); printed powers 1, 2, 2, 1, 3.
    let cases: [(&str, u32, i32, i32); 5] = [
        ("Balduvian Berserker", 3, 4, 0),
        ("Infernal Phantom", 2, 4, 0),
        ("Mortis Dogs", 2, 4, 0),
        ("Heartfire Hero", 2, 3, 0),
        ("Bottle Golems", 2, 0, 5),
    ];
    for (name, n, loss, gain) in cases {
        let mut t = TestGame::new(2);
        dies_with(&mut t, name, n, &[Entity::Player(P1)]);
        assert_eq!(lost(&t, P1), loss, "{name}");
        assert_eq!(t.life(P0), 20 + gain, "{name}");
    }
}

#[test]
fn goblin_fireleaper_deals_its_last_known_power_to_a_creature() {
    cr!("603.10a", "608.2h");
    ruling!(
        "Goblin Fireleaper",
        "To determine the amount of damage it deals, use Goblin Fireleaper's power as it last existed on the battlefield, not its power in the graveyard."
    );
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    dies_with(&mut t, "Goblin Fireleaper", 2, &[obj(wurm)]);
    assert_eq!(damage(&t, wurm), 3);
}

#[test]
fn rapacious_guest_leaving_uses_its_last_known_power() {
    cr!("603.10a", "608.2h");
    ruling!(
        "Rapacious Guest",
        "To determine the amount of life the target opponent loses from Rapacious Guest's last ability, use Rapacious Guest's power as it last existed on the battlefield, not its power in the graveyard."
    );
    supported("Rapacious Guest");
    let mut t = TestGame::new(2);
    let g = with_counters(&mut t, P0, "Rapacious Guest", 3);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    exile(&mut t, g);
    t.resolve_all();
    assert_eq!(lost(&t, P1), 5);
}

#[test]
fn feral_ghoul_gives_rad_counters_equal_to_its_last_known_power() {
    cr!("603.10a", "608.2h", "122.1");
    ruling!(
        "Feral Ghoul",
        "Use Feral Ghoul’s power as it last existed on the battlefield to determine how many rad counters each opponent gets."
    );
    let mut t = TestGame::new(3);
    dies_with(&mut t, "Feral Ghoul", 2, &[]);
    assert_eq!(player_counters(&t, P1, "rad"), 4);
    assert_eq!(player_counters(&t, P2, "rad"), 4);
    assert_eq!(player_counters(&t, P0, "rad"), 0);
}

#[test]
fn elenda_and_lifeblood_hydra_use_their_last_known_power() {
    cr!("603.10a", "608.2h");
    ruling!(
        "Elenda, the Dusk Rose",
        "To determine how many Vampire tokens are created, use Elenda's power as it last existed on the battlefield."
    );
    ruling!(
        "Lifeblood Hydra",
        "Use Lifeblood Hydra's power when it died (including any +1/+1 counters it had) to determine how much life to gain and how many cards to draw."
    );
    let mut t = TestGame::new(2);
    dies_with(&mut t, "Elenda, the Dusk Rose", 3, &[]);
    assert_eq!(tokens_named(&t, P0, "Vampire"), 4);
    let mut t = TestGame::new(2);
    for _ in 0..6 {
        t.library_top(P0, "Island");
    }
    let hand = t.hand_size(P0);
    dies_with(&mut t, "Lifeblood Hydra", 4, &[]);
    assert_eq!(t.life(P0), 24);
    assert_eq!(t.hand_size(P0), hand + 4);
}

#[test]
fn furnace_gremlin_incubates_its_last_known_power() {
    cr!("603.10a", "608.2h", "701.53a");
    ruling!(
        "Furnace Gremlin",
        "Use Furnace Gremlin's power from when it was last on the battlefield to determine the value of X."
    );
    let mut t = TestGame::new(2);
    dies_with(&mut t, "Furnace Gremlin", 3, &[]);
    let incubator = t
        .g
        .permanents()
        .find(|o| o.controller == P0 && o.chars.has_subtype("Incubator"))
        .map(|o| o.id)
        .expect("no Incubator");
    assert_eq!(plus1(&t, incubator), 4);
}

#[test]
fn fiendish_panda_target_legality_uses_its_last_known_power() {
    cr!("603.10a", "608.2h", "115.1");
    ruling!(
        "Fiendish Panda",
        "Use Fiendish Panda's power as it last existed on the battlefield to determine which creature cards may be targeted with its last ability. (Feel free to think of this as a metric of just how fiendish the panda was.)"
    );
    // Serra Angel has mana value 5: too much for a 3-power Panda, enough for a 5-power one.
    for (n, returns) in [(2, true), (0, false)] {
        let mut t = TestGame::new(2);
        let angel = t.graveyard(P0, "Serra Angel");
        dies_with(&mut t, "Fiendish Panda", n, &[obj(angel)]);
        assert_eq!(t.named_on_battlefield("Serra Angel").len(), returns as usize, "{n}");
    }
}

#[test]
fn attendant_of_vraska_gains_its_last_known_power() {
    cr!("603.10a", "608.2h");
    ruling!(
        "Attendant of Vraska",
        "The amount of life gained is equal to Attendant of Vraska’s power as it last existed on the battlefield."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vraska, Golgari Queen");
    dies_with(&mut t, "Attendant of Vraska", 2, &[]);
    assert_eq!(t.life(P0), 25);
}

#[test]
fn attendant_of_vraska_needs_a_vraska_as_it_resolves() {
    cr!("603.4", "603.10a");
    ruling!(
        "Attendant of Vraska",
        "If you don’t control a Vraska planeswalker as Attendant of Vraska’s triggered ability resolves, possibly because Vraska left the battlefield at the same time as Attendant of Vraska, you won’t gain life."
    );
    // Vraska leaves while the trigger is on the stack.
    let mut t = TestGame::new(2);
    let vraska = t.battlefield(P0, "Vraska, Golgari Queen");
    let a = t.battlefield(P0, "Attendant of Vraska");
    destroy(&mut t, a);
    assert_eq!(stack(&t), 1);
    exile(&mut t, vraska);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    // Both are destroyed at once (Planar Cleansing: "Destroy all nonland permanents.").
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vraska, Golgari Queen");
    t.battlefield(P0, "Attendant of Vraska");
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    cast_new(&mut t, P1, "Planar Cleansing", &[]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Attendant of Vraska"));
    assert!(t.in_graveyard(P0, "Vraska, Golgari Queen"));
    assert_eq!(t.life(P0), 20);
    // Positive control: with Vraska still there, life is gained.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vraska, Golgari Queen");
    let a = t.battlefield(P0, "Attendant of Vraska");
    destroy(&mut t, a);
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
}

#[test]
fn marionette_master_and_spikeshot_goblin_use_their_power_as_the_ability_resolves() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Marionette Master",
        "The amount of life lost is determined as Marionette Master's last ability resolves. If Marionette Master is no longer on the battlefield, use its power as it last existed on the battlefield to determine how much life is lost."
    );
    ruling!(
        "Spikeshot Goblin",
        "Spikeshot Goblin's power is determined as its ability resolves. If Spikeshot Goblin has left the battlefield, use its power as it last existed on the battlefield."
    );
    supported("Marionette Master");
    supported("Spikeshot Goblin");
    // Marionette Master (1/3): pumped in response, then (second case) exiled.
    for leaves in [false, true] {
        let mut t = TestGame::new(2);
        let mm = t.battlefield(P0, "Marionette Master");
        let thopter = t.battlefield(P0, "Ornithopter");
        t.answer_targets(P0, &[Entity::Player(P1)]);
        destroy(&mut t, thopter);
        assert_eq!(stack(&t), 1);
        give_plus1(&mut t, mm, 2);
        if leaves {
            exile(&mut t, mm);
        }
        t.resolve_all();
        assert_eq!(lost(&t, P1), 3, "leaves: {leaves}");
    }
    // Spikeshot Goblin (1/2).
    for leaves in [false, true] {
        let mut t = TestGame::new(2);
        let sg = t.battlefield(P0, "Spikeshot Goblin");
        t.lands(P0, "Mountain", 1);
        t.activate(P0, sg, 0, &[Entity::Player(P1)]).unwrap();
        assert_eq!(stack(&t), 1);
        give_plus1(&mut t, sg, 3);
        if leaves {
            exile(&mut t, sg);
        }
        t.resolve_all();
        assert_eq!(lost(&t, P1), 4, "leaves: {leaves}");
    }
}

#[test]
fn wild_beastmaster_uses_its_last_known_power() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Wild Beastmaster",
        "The value of X is determined when the triggered ability resolves. If Wild Beastmaster is no longer on the battlefield at that time, use its last known power to determine the value of X."
    );
    supported("Wild Beastmaster");
    for leaves in [false, true] {
        let mut t = TestGame::new(2);
        let wb = t.battlefield(P0, "Wild Beastmaster");
        let bears = t.battlefield(P0, "Grizzly Bears");
        attack_with(&mut t, &[(wb, Entity::Player(P1))]);
        assert_eq!(stack(&t), 1);
        give_plus1(&mut t, wb, 2);
        if leaves {
            exile(&mut t, wb);
        }
        t.resolve_all();
        assert_eq!(t.pt(bears), (5, 5), "leaves: {leaves}");
    }
}

#[test]
fn cultivator_of_blades_x_is_fixed_on_resolution_or_last_known() {
    cr!("608.2h", "113.7a", "611.2c");
    ruling!(
        "Cultivator of Blades",
        "The value of X is determined as Cultivator of Blades's last ability resolves. It won't change later, even if its power changes. If Cultivator of Blades is no longer on the battlefield, use its power as it last existed on the battlefield to determine the value of X."
    );
    supported("Cultivator of Blades");
    for leaves in [false, true] {
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, "Cultivator of Blades");
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.answer(P0, DecisionKind::YesNo, Answer::Bool(true));
        attack_with(
            &mut t,
            &[(c, Entity::Player(P1)), (bears, Entity::Player(P1))],
        );
        give_plus1(&mut t, c, 2);
        if leaves {
            exile(&mut t, c);
        }
        t.resolve_all();
        assert_eq!(t.pt(bears), (5, 5), "leaves: {leaves}");
        if !leaves {
            give_plus1(&mut t, c, 2);
            assert_eq!(t.pt(bears), (5, 5));
        }
    }
}
