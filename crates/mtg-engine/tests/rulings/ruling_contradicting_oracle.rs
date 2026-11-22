//! Rulings whose words contradict the card's own Oracle text give way to it (the user's
//! decision of Nov 5; register `ruling-contradicting-oracle-gives-way`). The parts of such
//! a ruling that contradict the card are listed in `docs/rulings-exemptions/` and logged in
//! `docs/RULING_DISCREPANCIES.md`; these tests pin what the Oracle text says instead.
//! (Homicide Investigator: `r_p005_dies.rs`; Rimescale Dragon: `r_p096_lockdown.rs`; Run Out
//! of Town: `r_p140_tuck.rs`.)

use crate::r_p056_common::{tap, tapped, untap_step_of};
use crate::r_s01_common::{attack_with, supported};
use mtg_engine::events::MoveCause;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn combat_celebrant_untaps_all_other_creatures_but_not_itself() {
    cr!("701.43a", "508.1f");
    interp!("ruling-contradicting-oracle-gives-way");
    // Oracle: "If this creature hasn't been exerted this turn, you may exert it as it
    // attacks. When you do, untap all other creatures you control and after this phase,
    // there is an additional combat phase." Its ruling (2017-04-18) "Combat Celebrant's
    // ability untaps all of your creatures, not just the ones that are attacking." is cited
    // for "not just the ones that are attacking" (r_p056_extra_phases.rs); its "all of your
    // creatures" would include Combat Celebrant itself, which "other" leaves tapped
    // (exempt in docs/rulings-exemptions/rulings-P056.tsv).
    supported("Combat Celebrant");
    let mut t = TestGame::new(2);
    let celebrant = t.battlefield(P0, "Combat Celebrant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let idle = t.battlefield(P0, "Hill Giant");
    tap(&mut t, idle);
    t.answer_yes(P0, true);
    attack_with(
        &mut t,
        &[(celebrant, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    t.resolve_all();
    assert!(t.obj_now(celebrant).exerted);
    // The other attacker and the tapped creature that didn't attack untap ...
    assert!(!tapped(&t, bears));
    assert!(!tapped(&t, idle));
    // ... and Combat Celebrant, tapped by attacking (CR 508.1f), stays tapped.
    assert!(tapped(&t, celebrant));
    // Exerted, it doesn't untap during its controller's next untap step either (CR
    // 701.43a), while the Hill Giant, tapped again, does.
    tap(&mut t, idle);
    untap_step_of(&mut t, P0);
    assert!(tapped(&t, celebrant));
    assert!(!tapped(&t, idle));
}

/// The +1/+1 counters on the permanent.
fn plus_counters(t: &TestGame, id: ObjectId) -> u32 {
    t.obj_now(id).counter(mtg_engine::types::counters::PLUS1)
}

/// `p` casts an Ornithopter (an artifact spell costing {0}) in their own main phase, and
/// it resolves.
fn cast_ornithopter(t: &mut TestGame, p: PlayerId) {
    t.set_step(p, Step::PrecombatMain);
    let spell = t.hand(p, "Ornithopter");
    t.cast(p, spell).go();
    t.resolve_all();
}

#[test]
fn citanul_druid_grows_for_each_opponents_artifact_spells_but_not_its_controllers() {
    cr!("102.2", "102.3", "603.2");
    interp!("ruling-contradicting-oracle-gives-way");
    // Oracle: "Whenever an opponent casts an artifact spell, put a +1/+1 counter on this
    // creature." Its ruling (2004-10-04) "In multiplayer games it affects all players." would
    // count its controller's and teammates' artifact spells too; the multiplayer note of the
    // same template says "all opponents" on the cards whose abilities apply only to
    // opponents (exempt in docs/rulings-exemptions/rulings-P178.tsv). Every opponent counts,
    // and no one else.
    supported("Citanul Druid");
    supported("Ornithopter");
    // Two players: the other player is its controller's opponent (CR 102.2).
    let mut t = TestGame::new(2);
    let druid = t.battlefield(P0, "Citanul Druid");
    cast_ornithopter(&mut t, P0);
    assert_eq!(plus_counters(&t, druid), 0);
    cast_ornithopter(&mut t, P1);
    assert_eq!(plus_counters(&t, druid), 1);
    // Free-for-all: both opponents' artifact spells count; its controller's doesn't.
    let mut t = TestGame::new(3);
    let druid = t.battlefield(P0, "Citanul Druid");
    cast_ornithopter(&mut t, P0);
    assert_eq!(plus_counters(&t, druid), 0);
    cast_ornithopter(&mut t, P1);
    assert_eq!(plus_counters(&t, druid), 1);
    cast_ornithopter(&mut t, P2);
    assert_eq!(plus_counters(&t, druid), 2);
    // Teams: P2 is P0's teammate, not an opponent (CR 102.3); P1 and P3 are opponents.
    let mut t = TestGame::with_config(
        4,
        GameConfig {
            teams: Some(vec![0, 1, 0, 1]),
            ..Default::default()
        },
    );
    let druid = t.battlefield(P0, "Citanul Druid");
    cast_ornithopter(&mut t, P2);
    cast_ornithopter(&mut t, P0);
    assert_eq!(plus_counters(&t, druid), 0);
    cast_ornithopter(&mut t, P1);
    cast_ornithopter(&mut t, P3);
    assert_eq!(plus_counters(&t, druid), 2);
}

/// Puts one card of each of `names` into `p`'s graveyard.
fn fill_graveyard(t: &mut TestGame, p: PlayerId, names: &[&str]) -> Vec<ObjectId> {
    names.iter().map(|n| t.graveyard(p, n)).collect()
}

/// Five cards with five different mana values (0 to 4).
const FIVE_MANA_VALUES: [&str; 5] = [
    "Forest",
    "Llanowar Elves",
    "Grizzly Bears",
    "Centaur Courser",
    "Hill Giant",
];

#[test]
fn sanguine_spy_counts_mana_values_among_cards_in_your_graveyard() {
    cr!("603.4", "202.3");
    interp!("ruling-contradicting-oracle-gives-way");
    // Oracle: "At the beginning of your end step, if there are five or more mana values
    // among cards in your graveyard, you may pay 2 life. If you do, draw a card." Its ruling
    // (2022-04-29) counts them "among cards you control", where the same note in the
    // Streets of New Capenna Release Notes (2022-04-20) says "among cards in your
    // graveyard" (a copying error; exempt in docs/rulings-exemptions/rulings-P093.tsv).
    // What it says of the check holds: it's made as the ability triggers and again as it
    // resolves.
    ruling!(
        "Sanguine Spy",
        "If you no longer have five or more mana values among those cards when the ability resolves, you may not pay 2 life."
    );
    supported("Sanguine Spy");
    // Five mana values in the graveyard: it triggers, and P0 pays 2 life to draw.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sanguine Spy");
    fill_graveyard(&mut t, P0, &FIVE_MANA_VALUES);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1, "it triggers");
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.hand_size(P0), hand + 1);
    // One of them leaves the graveyard before it resolves: four mana values, so no life is
    // paid and no card drawn.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sanguine Spy");
    let cards = fill_graveyard(&mut t, P0, &FIVE_MANA_VALUES);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.g.move_object(cards[4], Zone::Exile, MoveCause::Effect, None);
    t.g.flush_events();
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.hand_size(P0), hand);
    // Permanents with five mana values don't count: four in the graveyard, no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sanguine Spy");
    for name in FIVE_MANA_VALUES {
        t.battlefield(P0, name);
    }
    fill_graveyard(&mut t, P0, &FIVE_MANA_VALUES[..4]);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0, "four mana values in the graveyard");
    assert_eq!(t.life(P0), 20);
}
