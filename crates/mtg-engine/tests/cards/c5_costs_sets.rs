//! Costs over sets and choices made as additional costs, round 5 (cards5-costs-sets,
//! `oracle/patterns/costs_6.rs`, `additional_cost_choices.rs`, `cost_sets.rs`).

use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

fn in_gy(t: &TestGame, o: ObjectId) -> bool {
    matches!(t.zone(t.g.current(o)), Zone::Graveyard(_))
}

fn pool(t: &TestGame, p: PlayerId) -> Vec<ManaType> {
    t.g.player(p).mana_pool.mana.iter().map(|m| m.ty).collect()
}

#[test]
fn benthic_explorers_untaps_an_opponents_land_for_its_mana() {
    cr!("106.7", "118.3", "605.1a");
    ruling!("Benthic Explorers", "If no opponent controls a tapped land, the ability can't be activated");
    ruling!("Benthic Explorers", "This ability is not targeted any more");
    assert_supported(&["Benthic Explorers"]);
    let mut t = TestGame::new(2);
    let explorers = t.battlefield(P0, "Benthic Explorers");
    let forest = t.battlefield(P1, "Forest");
    // P1's Forest is untapped: nothing to untap.
    assert!(t.activate(P0, explorers, 0, &[]).is_err());
    assert!(!t.g.obj(explorers).tapped);
    // P0's own tapped land doesn't do.
    let island = t.battlefield(P0, "Island");
    t.g.objects[island.0 as usize].tapped = true;
    assert!(t.activate(P0, explorers, 0, &[]).is_err());
    t.g.objects[forest.0 as usize].tapped = true;
    t.activate(P0, explorers, 0, &[]).unwrap();
    assert!(!t.g.obj(forest).tapped, "the opponent's Forest is untapped");
    assert!(t.g.obj(island).tapped);
    assert!(t.g.obj(explorers).tapped);
    assert_eq!(pool(&t, P0), vec![ManaType::G], "a mana the Forest could produce");
}

#[test]
fn fabrication_foundry_exiles_artifacts_totaling_exactly_x() {
    cr!("107.3a", "601.2c", "601.2h", "602.2b");
    assert_supported(&["Fabrication Foundry"]);
    let setup = || {
        let mut t = TestGame::new(2);
        let foundry = t.battlefield(P0, "Fabrication Foundry");
        t.lands(P0, "Plains", 3);
        let ring = t.battlefield(P0, "Sol Ring");
        let splitter = t.battlefield(P0, "Bonesplitter");
        let stone = t.graveyard(P0, "Millstone");
        (t, foundry, ring, splitter, stone)
    };
    // X = 2: Sol Ring and Bonesplitter (total 2) return Millstone (mana value 2).
    let (mut t, foundry, ring, splitter, stone) = setup();
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer_choose(P0, &[Entity::Object(ring), Entity::Object(splitter)]);
    // The mana abilities of Sol Ring aren't used to pay ({2}{W} from the Plains).
    t.activate(P0, foundry, 1, &[Entity::Object(stone)]).unwrap();
    assert_eq!(t.zone(t.g.current(ring)), Zone::Exile);
    assert_eq!(t.zone(t.g.current(splitter)), Zone::Exile);
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(stone)));
    // A choice that doesn't total X is fitted to one that does.
    let (mut t, foundry, ring, splitter, stone) = setup();
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer_choose(P0, &[Entity::Object(ring)]);
    t.activate(P0, foundry, 1, &[Entity::Object(stone)]).unwrap();
    assert_eq!(t.zone(t.g.current(ring)), Zone::Exile);
    assert_eq!(t.zone(t.g.current(splitter)), Zone::Exile);
    // X = 1 can't target Millstone (mana value 2 > X).
    let (mut t, foundry, _, _, stone) = setup();
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    let r = t.activate(P0, foundry, 1, &[Entity::Object(stone)]);
    t.resolve_all();
    assert!(r.is_err() || !t.on_battlefield(t.g.current(stone)));
    // X = 3: no set of the other artifacts totals 3.
    let (mut t, foundry, ring, splitter, stone) = setup();
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    let r = t.activate(P0, foundry, 1, &[Entity::Object(stone)]);
    assert!(r.is_err());
    assert!(t.on_battlefield(ring) && t.on_battlefield(splitter));
}

#[test]
fn back_from_the_brink_pays_the_exiled_cards_mana_cost() {
    cr!("107.3h", "118.3", "400.7j");
    ruling!("Back from the Brink", "If the exiled creature card has {X} in its mana cost, X is considered to be zero");
    assert_supported(&["Back from the Brink"]);
    let mut t = TestGame::new(2);
    let brink = t.battlefield(P0, "Back from the Brink");
    let giant = t.graveyard(P0, "Hill Giant");
    t.lands(P0, "Mountain", 3);
    // {3}{R} can't be paid with three lands.
    assert!(t.activate(P0, brink, 0, &[]).is_err());
    assert!(in_gy(&t, giant));
    t.lands(P0, "Mountain", 1);
    t.activate(P0, brink, 0, &[]).unwrap();
    assert_eq!(t.zone(t.g.current(giant)), Zone::Exile);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    // An {X} creature costs nothing: X is 0.
    let mut t = TestGame::new(2);
    let brink = t.battlefield(P0, "Back from the Brink");
    let one = t.graveyard(P0, "Endless One");
    t.activate(P0, brink, 0, &[]).unwrap();
    assert_eq!(t.zone(t.g.current(one)), Zone::Exile);
}

#[test]
fn sword_of_the_ages_deals_the_sacrificed_creatures_power_then_exiles_them() {
    cr!("400.7", "400.7j", "608.2h");
    ruling!("Sword of the Ages", "checks the power the exiled creatures had as they last existed");
    assert_supported(&["Sword of the Ages"]);
    let mut t = TestGame::new(2);
    let sword = t.battlefield(P0, "Sword of the Ages");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer_choose(P0, &[Entity::Object(bears), Entity::Object(giant)]);
    t.activate(P0, sword, 0, &[Entity::Player(P1)]).unwrap();
    assert!(in_gy(&t, t.g.current(bears)));
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
    for o in [sword, bears, giant] {
        assert_eq!(t.zone(t.g.current(o)), Zone::Exile, "{}", t.g.describe(o));
    }
    assert!(t.on_battlefield(elves));
}

#[test]
fn close_encounter_uses_the_chosen_creatures_power() {
    cr!("601.2b", "601.2h", "608.2h");
    ruling!("Close Encounter", "use that creature’s power as it last existed on the battlefield");
    assert_supported(&["Close Encounter"]);
    // Without a creature or a warped creature card to choose, it can't be cast.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    let foe = t.battlefield(P1, "Hill Giant");
    let ce = t.hand(P0, "Close Encounter");
    assert!(t.cast(P0, ce).target(foe).try_go().is_err());
    // P0 chooses Hill Giant (3 power); it leaves before the spell resolves: its last
    // known power still counts.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    let mine = t.battlefield(P0, "Hill Giant");
    let foe = t.battlefield(P1, "Hill Giant");
    let ce = t.hand(P0, "Close Encounter");
    t.answer_choose(P0, &[Entity::Object(mine)]);
    t.cast(P0, ce).target(foe).go();
    t.g.destroy(mine, None);
    t.resolve_all();
    assert!(in_gy(&t, t.g.current(foe)), "3 damage to the 3/3");
}

#[test]
fn close_encounter_may_choose_a_warped_creature_card_in_exile() {
    cr!("702.185b", "601.2b");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    let foe = t.battlefield(P1, "Hill Giant");
    let exiled = t.exile(P0, "Hill Giant");
    let ce = t.hand(P0, "Close Encounter");
    // Not warped: it can't be chosen.
    assert!(t.cast(P0, ce).target(foe).try_go().is_err());
    t.clear_answers();
    mtg_engine::special_actions::mark(&mut t.g, exiled, KeywordKind::Warp);
    t.answer_choose(P0, &[Entity::Object(exiled)]);
    t.cast(P0, ce).target(foe).go();
    t.resolve_all();
    assert!(in_gy(&t, t.g.current(foe)));
}

#[test]
fn celestial_reunion_puts_a_card_of_the_chosen_type_onto_the_battlefield() {
    cr!("601.2b", "701.4a", "614.1a", "701.23a");
    ruling!("Celestial Reunion", "choose two creatures you control of that type on the battlefield");
    assert_supported(&["Celestial Reunion"]);
    let run = |pay: bool, find: &str| {
        let mut t = TestGame::new(2);
        t.lands(P0, "Forest", 3);
        t.battlefield(P0, "Grizzly Bears");
        t.battlefield(P0, "Grizzly Bears");
        let found = t.library_top(P0, find);
        let spell = t.hand(P0, "Celestial Reunion");
        t.answer_yes(P0, pay);
        t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(pay));
        t.answer_choose(P0, &[Entity::Object(found)]);
        t.cast(P0, spell).x(2).go();
        t.resolve_all();
        let now = t.g.current(found);
        let zone = t.zone(now);
        (t, zone)
    };
    // The cost paid (Bear chosen, two Bears beheld) and a Bear found: onto the battlefield.
    let (_, z) = run(true, "Grizzly Bears");
    assert_eq!(z, Zone::Battlefield);
    // Not paid: into the hand.
    let (_, z) = run(false, "Grizzly Bears");
    assert_eq!(z, Zone::Hand(P0));
    // Paid, but the card found isn't a Bear.
    let (_, z) = run(true, "Llanowar Elves");
    assert_eq!(z, Zone::Hand(P0));
}

#[test]
fn primitive_justice_destroys_another_artifact_for_each_additional_payment() {
    cr!("601.2b", "601.2c", "601.2f");
    ruling!("Primitive Justice", "can't target the same artifact more than once");
    assert_supported(&["Primitive Justice"]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    t.lands(P0, "Forest", 2);
    let a = t.battlefield(P1, "Sol Ring");
    let b = t.battlefield(P1, "Millstone");
    let c = t.battlefield(P1, "Mind Stone");
    let spell = t.hand(P0, "Primitive Justice");
    // {1}{R} once more and {1}{G} once: three artifacts, 1 life.
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(1));
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(1));
    t.cast(P0, spell).target(a).target(b).target(c).go();
    t.resolve_all();
    for o in [a, b, c] {
        assert!(in_gy(&t, t.g.current(o)));
    }
    assert_eq!(t.life(P0), 21);
    // Not paid: one artifact.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let a = t.battlefield(P1, "Sol Ring");
    let b = t.battlefield(P1, "Millstone");
    let spell = t.hand(P0, "Primitive Justice");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(0));
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(0));
    t.cast(P0, spell).target(a).go();
    t.resolve_all();
    assert!(in_gy(&t, t.g.current(a)));
    assert!(t.on_battlefield(b));
    assert_eq!(t.life(P0), 20);
    // The same artifact can't be the target of two of its instances of "target".
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    let a = t.battlefield(P1, "Sol Ring");
    let spell = t.hand(P0, "Primitive Justice");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(1));
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(0));
    assert!(t.cast(P0, spell).target(a).target(a).try_go().is_err());
}
