//! CR 118.8, 601.2b, 601.2f: optional additional costs that another object's static
//! ability offers for the spells a player casts ("As an additional cost to cast green
//! permanent spells, you may pay 2 life. Those spells cost {G} less to cast if you paid
//! life this way."; "As an additional cost to cast creature spells, you may pay any amount
//! of mana."), see `kw/offered_costs.rs`.

use mtg_engine::card::card;
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn supported(name: &str) {
    let c = card(name);
    assert!(
        c.is_fully_supported(),
        "{name}: unsupported {:?}",
        c.unsupported_text()
    );
}

fn add_mana(t: &mut TestGame, p: PlayerId, ty: ManaType, n: u32) {
    t.g.players[p.idx()].mana_pool.add_type(ty, n);
}

fn pool(t: &TestGame, p: PlayerId, ty: ManaType) -> usize {
    t.player(p).mana_pool.count(ty)
}

/// The optional costs `p` was asked about.
fn optional_costs_asked(t: &TestGame) -> Vec<String> {
    t.asked()
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::OptionalCost { name, .. } => Some(name),
            _ => None,
        })
        .collect()
}

fn can_cast(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    t.g.legal_actions(p).contains(&Action::Cast {
        card,
        method: CastMethod::Normal,
    })
}

#[test]
fn an_offered_optional_cost_is_announced_and_paid_with_the_spell() {
    cr!("118.8", "118.8a", "118.8b", "601.2b", "601.2f", "601.2h", "118.7");
    supported("Defiler of Vigor");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Defiler of Vigor");
    // River Boa {1}{G}: with 2 life paid, {1}.
    let boa = t.hand(P0, "River Boa");
    add_mana(&mut t, P0, ManaType::G, 2);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.cast(P0, boa).go();
    assert_eq!(t.life(P0), 18);
    assert_eq!(pool(&t, P0, ManaType::G), 1);
    assert_eq!(optional_costs_asked(&t).len(), 1);
    // Declined: the full cost.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Defiler of Vigor");
    let boa = t.hand(P0, "River Boa");
    add_mana(&mut t, P0, ManaType::G, 2);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(false));
    t.cast(P0, boa).go();
    assert_eq!(t.life(P0), 20);
    assert_eq!(pool(&t, P0, ManaType::G), 0);
}

#[test]
fn an_offered_optional_cost_applies_only_to_the_spells_it_names() {
    cr!("118.8", "601.2b");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Defiler of Vigor");
    // Giant Growth is green but not a permanent spell; Savannah Lions isn't green.
    let growth = t.hand(P0, "Giant Growth");
    let lions = t.hand(P0, "Savannah Lions");
    let bears = t.battlefield(P0, "Grizzly Bears");
    add_mana(&mut t, P0, ManaType::G, 1);
    add_mana(&mut t, P0, ManaType::W, 1);
    t.cast(P0, growth).target(Entity::Object(bears)).go();
    t.resolve_all();
    t.cast(P0, lions).go();
    assert!(optional_costs_asked(&t).is_empty());
    assert_eq!(t.life(P0), 20);
}

#[test]
fn a_spell_that_could_be_cast_only_with_the_reduction_can_be_begun() {
    cr!("601.2f", "118.7");
    // Llanowar Elves {G} with no mana: only paying 2 life makes it castable.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Defiler of Vigor");
    let elves = t.hand(P0, "Llanowar Elves");
    assert!(can_cast(&mut t, P0, elves));
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.cast(P0, elves).go();
    assert_eq!(t.life(P0), 18);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Llanowar Elves").len(), 1);
    // Without the Defiler it can't be cast.
    let mut t = TestGame::new(2);
    let elves = t.hand(P0, "Llanowar Elves");
    assert!(!can_cast(&mut t, P0, elves));
}

#[test]
fn each_object_offers_its_own_cost_once() {
    cr!("118.8a", "601.2b", "601.2f");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Defiler of Vigor");
    t.battlefield(P0, "Defiler of Vigor");
    // Leatherback Baloth {G}{G}{G}: 4 life for {G}{G} less.
    let baloth = t.hand(P0, "Leatherback Baloth");
    add_mana(&mut t, P0, ManaType::G, 1);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.cast(P0, baloth).go();
    assert_eq!(optional_costs_asked(&t).len(), 2);
    assert_eq!(t.life(P0), 16);
    assert_eq!(pool(&t, P0, ManaType::G), 0);
    // Paying for one Defiler only reduces by one {G}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Defiler of Vigor");
    t.battlefield(P0, "Defiler of Vigor");
    let baloth = t.hand(P0, "Leatherback Baloth");
    add_mana(&mut t, P0, ManaType::G, 1);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(false));
    assert!(t.cast(P0, baloth).try_go().is_err());
    assert_eq!(t.life(P0), 20);
}

#[test]
fn an_amount_of_mana_paid_as_an_additional_cost_gives_counters() {
    cr!("118.8", "601.2b", "601.2f", "614.1c", "122.6");
    supported("Chorus of the Conclave");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Chorus of the Conclave");
    // Grizzly Bears {1}{G} plus {3}: it enters with three +1/+1 counters.
    let bears = t.hand(P0, "Grizzly Bears");
    add_mana(&mut t, P0, ManaType::G, 5);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(3));
    t.cast(P0, bears).go();
    assert_eq!(pool(&t, P0, ManaType::G), 0);
    t.resolve_all();
    let bears = t.named_on_battlefield("Grizzly Bears")[0];
    assert_eq!(t.counters(bears, "+1/+1"), 3);
    assert_eq!(t.pt(bears), (5, 5));
    // Paying nothing: no counters.
    let ogre = t.hand(P0, "Gray Ogre");
    add_mana(&mut t, P0, ManaType::R, 3);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(0));
    t.cast(P0, ogre).go();
    t.resolve_all();
    let ogre = t.named_on_battlefield("Gray Ogre")[0];
    assert_eq!(t.counters(ogre, "+1/+1"), 0);
    // A noncreature spell isn't offered the cost.
    let n = optional_costs_asked(&t).len();
    let growth = t.hand(P0, "Giant Growth");
    add_mana(&mut t, P0, ManaType::G, 1);
    t.cast(P0, growth).target(Entity::Object(ogre)).go();
    assert_eq!(optional_costs_asked(&t).len(), n);
}
