//! Energy (CR 107.14, 122.1): paying a chosen amount ("you may pay any amount of {E}",
//! "one or more {E}") and what refers to it, "an amount of {E} equal to ...", amounts
//! spelled out ("six {E}", "eight {E}"), a mandatory payment ("Pay {E}{E}. If you
//! can't, ..."), and energy in keyword costs ("Equip—Pay {E}{E}", "Reconfigure—Pay {2}
//! or {E}{E}{E}").

use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::turn::Step;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let c = card(n);
        assert!(
            c.unsupported_text().is_empty(),
            "{n} has unsupported text: {:?}",
            c.unsupported_text()
        );
    }
}

fn energy(t: &TestGame, p: PlayerId) -> u32 {
    t.g.player(p).counter(counters::ENERGY)
}

fn give_energy(t: &mut TestGame, p: PlayerId, n: u32) {
    t.g.add_counters(Entity::Player(p), counters::ENERGY, n, None);
}

#[test]
fn harnessed_lightning_deals_the_energy_paid() {
    cr!("107.14", "122.1");
    assert_supported(&["Harnessed Lightning", "Galvanic Discharge"]);
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 3);
    // Pays 2 of the 3 energy: 2 damage; 1 energy stays.
    let hl = t.hand(P0, "Harnessed Lightning");
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.cast(P0, hl).target(bears).go();
    t.resolve();
    assert!(!t.on_battlefield(bears));
    assert_eq!(energy(&t, P0), 1);
    // A player can't pay more energy than they have (CR 107.14): asking for 9 pays 4.
    let gd = t.hand(P0, "Galvanic Discharge");
    t.answer(P0, DecisionKind::X, Answer::Number(9));
    t.cast(P0, gd).target(giant).go();
    t.resolve();
    assert_eq!(energy(&t, P0), 0);
    assert!(!t.on_battlefield(giant));
}

#[test]
fn die_young_gives_minus_one_for_each_energy_paid() {
    cr!("107.14");
    assert_supported(&["Die Young"]);
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Swamp", 2);
    give_energy(&mut t, P0, 3);
    let dy = t.hand(P0, "Die Young");
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.cast(P0, dy).target(giant).go();
    t.resolve();
    // Hill Giant 3/3 gets -2/-2.
    assert_eq!(t.pt(giant), (1, 1));
    assert_eq!(energy(&t, P0), 3);
}

#[test]
fn aether_refinery_creates_a_token_the_size_of_the_energy_paid() {
    cr!("107.14", "107.3a");
    assert_supported(&["Aether Refinery"]);
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Aether Refinery");
    // Its replacement effect doubles the energy: 2 becomes 4, then {T} gives 2 more.
    give_energy(&mut t, P0, 2);
    assert_eq!(energy(&t, P0), 4);
    t.answer(P0, DecisionKind::X, Answer::Number(5));
    t.activate(P0, r, 0, &[]).unwrap();
    t.resolve_all();
    let tokens = t.named_on_battlefield("Aetherborn Token");
    assert_eq!(tokens.len(), 1);
    assert_eq!(t.pt(tokens[0]), (5, 5));
    assert_eq!(energy(&t, P0), 1);
}

#[test]
fn aether_refinery_paying_no_energy_creates_nothing() {
    cr!("107.14");
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Aether Refinery");
    t.answer(P0, DecisionKind::X, Answer::Number(0));
    t.activate(P0, r, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.named_on_battlefield("Aetherborn Token").is_empty());
    assert_eq!(energy(&t, P0), 2);
}

#[test]
fn wrath_of_the_skies_destroys_up_to_the_energy_paid() {
    cr!("107.14", "107.3a");
    assert_supported(&["Wrath of the Skies"]);
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P1, "Llanowar Elves");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Plains", 4);
    let w = t.hand(P0, "Wrath of the Skies");
    // X = 2: get 2 energy, pay both.
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.cast(P0, w).x(2).go();
    t.resolve();
    assert!(!t.on_battlefield(elves));
    assert!(!t.on_battlefield(bears));
    assert!(t.on_battlefield(giant));
    assert_eq!(energy(&t, P0), 0);
}

#[test]
fn confiscation_coup_gains_control_if_the_energy_is_paid() {
    cr!("107.14", "118.12");
    assert_supported(&["Confiscation Coup", "Jolted Awake"]);
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Island", 5);
    let cc = t.hand(P0, "Confiscation Coup");
    t.answer_yes(P0, true);
    t.cast(P0, cc).target(giant).go();
    t.resolve();
    // 4 energy gained, 4 (its mana value) paid.
    assert_eq!(t.g.obj(t.g.current(giant)).controller, P0);
    assert_eq!(energy(&t, P0), 0);
}

#[test]
fn behemoth_of_vault_0_destroys_its_target_when_paid() {
    cr!("107.14", "603.12");
    assert_supported(&["Behemoth of Vault 0"]);
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let b = t.battlefield(P0, "Behemoth of Vault 0");
    give_energy(&mut t, P0, 5);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.answer_yes(P0, true);
    t.g.destroy(b, None);
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
    assert_eq!(energy(&t, P0), 1);
}

#[test]
fn aethertide_whale_gets_six_energy() {
    cr!("107.14", "122.1");
    assert_supported(&["Aethertide Whale"]);
    let mut t = TestGame::new(2);
    t.enter(P0, "Aethertide Whale");
    t.resolve_all();
    assert_eq!(energy(&t, P0), 6);
}

#[test]
fn greenbelt_rampager_returns_to_hand_unless_it_can_pay() {
    cr!("107.14");
    assert_supported(&["Greenbelt Rampager"]);
    // Without energy it can't pay: it returns and you get {E}.
    let mut t = TestGame::new(2);
    let g = t.enter(P0, "Greenbelt Rampager");
    t.resolve_all();
    assert!(!t.on_battlefield(g));
    assert!(t.in_hand(P0, "Greenbelt Rampager"));
    assert_eq!(energy(&t, P0), 1);
    // With two energy it must pay: it stays.
    let mut t = TestGame::new(2);
    give_energy(&mut t, P0, 2);
    let g = t.enter(P0, "Greenbelt Rampager");
    t.resolve_all();
    assert!(t.on_battlefield(g));
    assert_eq!(energy(&t, P0), 0);
}

#[test]
fn lightning_runner_pays_eight_energy_for_an_extra_combat() {
    cr!("107.14", "500.8");
    assert_supported(&["Lightning Runner"]);
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Lightning Runner");
    give_energy(&mut t, P0, 6);
    t.answer_yes(P0, true);
    t.set_step(P0, turn::Step::BeginningOfCombat);
    let other = t.battlefield(P0, "Grizzly Bears");
    t.g.tap(other);
    t.attack(&[(r, Entity::Player(P1))], &[]);
    // 6 + 2 = 8 energy, all paid: creatures untap, and a combat phase follows this one.
    assert_eq!(energy(&t, P0), 0);
    assert!(!t.obj_now(r).tapped);
    assert!(!t.obj_now(other).tapped);
    assert_eq!(t.g.turn.schedule.first(), Some(&Step::BeginningOfCombat));
    // Declining to pay: nothing untaps, no additional combat.
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Lightning Runner");
    give_energy(&mut t, P0, 6);
    t.answer_yes(P0, false);
    t.set_step(P0, turn::Step::BeginningOfCombat);
    t.attack(&[(r, Entity::Player(P1))], &[]);
    assert_eq!(energy(&t, P0), 8);
    assert!(t.obj_now(r).tapped);
    assert_ne!(t.g.turn.schedule.first(), Some(&Step::BeginningOfCombat));
}

#[test]
fn inventors_axe_equips_by_paying_energy() {
    cr!("702.6a", "107.14");
    assert_supported(&["Inventor's Axe", "Razorfield Ripper"]);
    let mut t = TestGame::new(2);
    let axe = t.battlefield(P0, "Inventor's Axe");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Not enough energy: can't equip.
    give_energy(&mut t, P0, 1);
    assert!(t.activate(P0, axe, 0, &[Entity::Object(bears)]).is_err());
    give_energy(&mut t, P0, 1);
    t.activate(P0, axe, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(
        t.g.obj(t.g.current(axe)).attached_to,
        Some(Entity::Object(bears))
    );
    assert_eq!(energy(&t, P0), 0);
}

#[test]
fn pia_nalaar_creates_a_vehicle_the_size_of_the_energy_paid() {
    cr!("107.14");
    assert_supported(&["Pia Nalaar, Chief Mechanic"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Pia Nalaar, Chief Mechanic");
    give_energy(&mut t, P0, 5);
    // "you may pay one or more {E}": pays 3 of 5.
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let jets = t.named_on_battlefield("Nalaar Aetherjet");
    assert_eq!(jets.len(), 1);
    // A Vehicle has a power and toughness only while it's a creature (CR 301.7a): crew it.
    t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, jets[0], 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(jets[0]), (3, 3));
    assert!(t.obj_now(jets[0]).chars.has_keyword(KeywordKind::Flying));
    assert_eq!(energy(&t, P0), 2);
    // Paying none: no token ("If you do").
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Pia Nalaar, Chief Mechanic");
    give_energy(&mut t, P0, 5);
    t.answer(P0, DecisionKind::X, Answer::Number(0));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.named_on_battlefield("Nalaar Aetherjet").is_empty());
    assert_eq!(energy(&t, P0), 5);
}

#[test]
fn territorial_aetherkite_deals_the_energy_paid_to_each_other_creature() {
    cr!("107.14", "603.12");
    assert_supported(&["Territorial Aetherkite"]);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let mine = t.battlefield(P0, "Llanowar Elves");
    give_energy(&mut t, P0, 1);
    // 1 + 2 = 3 energy; pays 2: 2 damage to each other creature.
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    let k = t.enter(P0, "Territorial Aetherkite");
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(!t.on_battlefield(mine));
    assert!(t.on_battlefield(giant));
    assert_eq!(t.obj_now(giant).damage, 2);
    assert!(t.on_battlefield(k));
    assert_eq!(t.obj_now(k).damage, 0);
    assert_eq!(energy(&t, P0), 1);
}

#[test]
fn localized_destruction_saves_creatures_with_power_equal_to_the_energy_paid() {
    cr!("107.14");
    assert_supported(&["Localized Destruction"]);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 5);
    give_energy(&mut t, P0, 1);
    let ld = t.hand(P0, "Localized Destruction");
    // 1 + 1 = 2 energy, both paid: your power-2 creatures gain indestructible.
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.cast(P0, ld).go();
    t.resolve();
    assert!(t.on_battlefield(bears));
    assert!(!t.on_battlefield(giant));
    assert!(!t.on_battlefield(theirs));
    assert_eq!(energy(&t, P0), 0);
}

#[test]
fn rampaging_aetherhood_puts_a_counter_for_each_energy_paid() {
    cr!("107.14", "122.1");
    assert_supported(&["Rampaging Aetherhood"]);
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Rampaging Aetherhood");
    // Upkeep: 4 energy (its power); pays 3.
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(r, counters::PLUS1), 3);
    assert_eq!(energy(&t, P0), 1);
}

#[test]
fn blaster_hulk_pays_eight_energy_to_deal_eight_damage() {
    cr!("107.14", "603.12");
    // (Its cost reduction is another item's construct; the attack trigger compiles.)
    assert!(!card("Blaster Hulk")
        .unsupported_text()
        .iter()
        .any(|t| t.contains("attacks")));
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Blaster Hulk");
    let giant = t.battlefield(P1, "Hill Giant");
    give_energy(&mut t, P0, 6);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(h, Entity::Player(P1))], &[]);
    // All 8 damage to the one target.
    assert!(!t.on_battlefield(giant));
    assert_eq!(energy(&t, P0), 0);
    // Without enough energy it can't pay: nothing happens.
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Blaster Hulk");
    let giant = t.battlefield(P1, "Hill Giant");
    give_energy(&mut t, P0, 5);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(h, Entity::Player(P1))], &[]);
    assert!(t.on_battlefield(giant));
    assert_eq!(energy(&t, P0), 7);
}

#[test]
fn razorfield_ripper_reconfigures_for_energy_or_mana() {
    cr!("702.151a", "107.14");
    use mtg_engine::ability::{AbilityKind, CostPart};
    let mut t = TestGame::new(2);
    let ripper = t.battlefield(P0, "Razorfield Ripper");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.recompute();
    let activated: Vec<_> = t
        .obj_now(ripper)
        .chars
        .abilities
        .iter()
        .filter_map(|a| match &a.kind {
            AbilityKind::Activated(act) => Some(act.cost.clone()),
            _ => None,
        })
        .collect();
    let by_energy = activated
        .iter()
        .position(|c| c.parts.iter().any(|p| matches!(p, CostPart::PayEnergy(_))))
        .expect("an energy reconfigure ability");
    let by_mana = activated
        .iter()
        .position(|c| c.mana.is_some())
        .expect("a mana reconfigure ability");
    // Two energy isn't enough.
    give_energy(&mut t, P0, 2);
    assert!(t.activate(P0, ripper, by_energy, &[Entity::Object(bears)]).is_err());
    give_energy(&mut t, P0, 1);
    t.activate(P0, ripper, by_energy, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(ripper).attached_to, Some(Entity::Object(bears)));
    assert_eq!(energy(&t, P0), 0);
    // Or {2}.
    let mut t = TestGame::new(2);
    let ripper = t.battlefield(P0, "Razorfield Ripper");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 2);
    t.activate(P0, ripper, by_mana, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(ripper).attached_to, Some(Entity::Object(bears)));
}
