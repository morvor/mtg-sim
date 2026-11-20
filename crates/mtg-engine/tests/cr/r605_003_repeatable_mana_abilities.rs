//! CR 605.3a, 601.2g–h, 118.3: a mana ability whose cost doesn't tap or sacrifice its
//! permanent (Ashnod's Altar's "Sacrifice a creature: Add {C}{C}.") can be activated again
//! while a cost is being paid, as long as its cost can be paid. The automatic payment
//! (`automatic-mana-payment`) plans such an ability once per activation the payment needs,
//! and what all the planned activations sacrifice between them must be distinct objects.

use mtg_engine::decision::Action;
use mtg_engine::mana::ManaType;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn main_phase() -> TestGame {
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t
}

/// Whether casting `card` is among `p`'s legal actions.
fn offered(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    let card = t.g.current(card);
    t.g.legal_actions(p).contains(&Action::Cast {
        card,
        method: CastMethod::Normal,
    })
}

fn on_stack(t: &TestGame, name: &str) -> bool {
    t.g.stack
        .iter()
        .any(|id| t.g.obj(*id).chars.name.as_str() == name)
}

#[test]
fn ashnods_altar_is_activated_once_per_creature_to_pay_four() {
    cr!("605.3a", "601.2g", "601.2h");
    interp!("automatic-mana-payment");
    interp!("repeatable-mana-abilities-planned-per-activation");
    let mut t = main_phase();
    t.battlefield(P0, "Ashnod's Altar");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let tome = t.hand(P0, "Jayemdae Tome");
    assert!(offered(&mut t, P0, tome));
    t.cast(P0, tome).go();
    assert!(on_stack(&t, "Jayemdae Tome"));
    assert!(!t.on_battlefield(a) && !t.on_battlefield(b));
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
}

#[test]
fn two_creatures_pay_for_four_activations_of_two_altars_at_most_two() {
    cr!("605.3a", "118.3", "601.2h");
    interp!("automatic-mana-payment");
    interp!("repeatable-mana-abilities-planned-per-activation");
    // Ashnod's Altar ({C}{C}) and Phyrexian Altar (one mana of any color) both sacrifice a
    // creature: with two creatures, two activations between them, so {4} and not {5}.
    let mut t = main_phase();
    t.battlefield(P0, "Ashnod's Altar");
    t.battlefield(P0, "Phyrexian Altar");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    let golem = t.hand(P0, "Precursor Golem");
    let tome = t.hand(P0, "Jayemdae Tome");
    assert!(!offered(&mut t, P0, golem));
    assert!(offered(&mut t, P0, tome));
    t.cast(P0, tome).go();
    assert!(on_stack(&t, "Jayemdae Tome"));
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
}

#[test]
fn diamond_kaleidoscope_sacrifices_both_prisms_for_inspired_ultimatum() {
    cr!("605.3a", "601.2g");
    interp!("automatic-mana-payment");
    interp!("repeatable-mana-abilities-planned-per-activation");
    let mut t = main_phase();
    let dk = t.battlefield(P0, "Diamond Kaleidoscope");
    // Two Prisms, made with its first ability ("{3}, {T}: Create a 0/1 colorless Prism
    // artifact creature token."), untapped again for the second.
    for _ in 0..2 {
        t.g.players[P0.idx()].mana_pool.add_type(ManaType::C, 3);
        t.activate(P0, dk, 0, &[]).expect("make a Prism");
        t.resolve_all();
        t.g.untap(dk);
    }
    assert_eq!(t.named_on_battlefield("Prism Token").len(), 2);
    for land in ["Plains", "Plains", "Island", "Island", "Mountain"] {
        t.battlefield(P0, land);
    }
    // {U}{U}{R}{R}{R}{W}{W}: the lands pay five, each Prism one of the {R}s.
    let iu = t.hand(P0, "Inspired Ultimatum");
    assert!(offered(&mut t, P0, iu));
    t.cast(P0, iu)
        .targets(&[Entity::Player(P0), Entity::Player(P1)])
        .go();
    assert!(on_stack(&t, "Inspired Ultimatum"));
    assert!(t.named_on_battlefield("Prism Token").is_empty());
}

#[test]
fn skirk_prospector_pays_three_red_and_is_sacrificed_last() {
    cr!("605.3a", "601.2g", "400.7");
    interp!("automatic-mana-payment");
    interp!("repeatable-mana-abilities-planned-per-activation");
    ruling!(
        "Skirk Prospector",
        "if you’d like to sacrifice multiple Goblins including Skirk Prospector for mana, be sure to sacrifice Skirk Prospector last"
    );
    let mut t = main_phase();
    let sp = t.battlefield(P0, "Skirk Prospector");
    t.battlefield(P0, "Raging Goblin");
    t.battlefield(P0, "Raging Goblin");
    let ball = t.hand(P0, "Ball Lightning");
    assert!(offered(&mut t, P0, ball));
    // Each time the player would rather sacrifice Skirk Prospector itself: until its last
    // activation it isn't among the choices (sacrificed first, its ability couldn't be
    // activated again, CR 400.7).
    for _ in 0..3 {
        t.answer_choose(P0, &[Entity::Object(sp)]);
    }
    t.cast(P0, ball).go();
    assert!(on_stack(&t, "Ball Lightning"));
    assert!(t.named_on_battlefield("Raging Goblin").is_empty());
    assert!(!t.on_battlefield(sp));
    // Its first two sacrifices were offered without it.
    let offers: Vec<Vec<Entity>> = t
        .asked()
        .into_iter()
        .filter_map(|(_, d)| match d {
            mtg_engine::decision::Decision::ChooseEntities {
                candidates, prompt, ..
            } if prompt.contains("sacrifice") => Some(candidates),
            _ => None,
        })
        .collect();
    assert!(offers.len() >= 2, "{offers:?}");
    for c in &offers[..2] {
        assert!(!c.contains(&Entity::Object(sp)), "{c:?}");
    }
}

#[test]
fn krark_clan_ironworks_sacrifices_itself_last_to_pay_four() {
    cr!("605.3a", "601.2g");
    interp!("automatic-mana-payment");
    interp!("repeatable-mana-abilities-planned-per-activation");
    let mut t = main_phase();
    let kci = t.battlefield(P0, "Krark-Clan Ironworks");
    let other = t.battlefield(P0, "Ornithopter");
    let tome = t.hand(P0, "Jayemdae Tome");
    assert!(offered(&mut t, P0, tome));
    t.answer_choose(P0, &[Entity::Object(kci)]);
    t.cast(P0, tome).go();
    assert!(on_stack(&t, "Jayemdae Tome"));
    assert!(!t.on_battlefield(kci) && !t.on_battlefield(other));
}

#[test]
fn a_blood_pet_cant_both_sacrifice_itself_and_be_sacrificed_to_ashnods_altar() {
    cr!("118.3", "400.7", "601.2h");
    interp!("automatic-mana-payment");
    interp!("repeatable-mana-abilities-planned-per-activation");
    // Blood Pet ("Sacrifice this creature: Add {B}.") is the only creature: either it
    // sacrifices itself ({B}) or it's sacrificed to Ashnod's Altar ({C}{C}), not both.
    let mut t = main_phase();
    t.battlefield(P0, "Ashnod's Altar");
    let pet = t.battlefield(P0, "Blood Pet");
    let powerstone = t.hand(P0, "Worn Powerstone");
    assert!(!offered(&mut t, P0, powerstone));
    // With a land too, the Altar's {C}{C} and the land pay {3}.
    t.battlefield(P0, "Wastes");
    assert!(offered(&mut t, P0, powerstone));
    t.cast(P0, powerstone).go();
    assert!(on_stack(&t, "Worn Powerstone"));
    assert!(!t.on_battlefield(pet));
}

#[test]
fn counters_and_life_bound_the_activations() {
    cr!("605.3a", "118.3", "118.3b");
    interp!("automatic-mana-payment");
    interp!("repeatable-mana-abilities-planned-per-activation");
    // Workhorse ("This creature enters with four +1/+1 counters on it."; "Remove a +1/+1
    // counter from this creature: Add {C}."): one activation per counter, {4} and not {5}.
    let mut t = main_phase();
    let horse = t.enter(P0, "Workhorse");
    assert_eq!(t.counters(horse, "+1/+1"), 4);
    let golem = t.hand(P0, "Precursor Golem");
    let tome = t.hand(P0, "Jayemdae Tome");
    assert!(!offered(&mut t, P0, golem));
    assert!(offered(&mut t, P0, tome));
    t.cast(P0, tome).go();
    assert!(on_stack(&t, "Jayemdae Tome"));
    assert_eq!(t.counters(horse, "+1/+1"), 0);
    // Treasonous Ogre ("Pay 3 life: Add {R}."): {R}{R}{R} with 10 life, not with 8.
    let mut t = main_phase();
    t.battlefield(P0, "Treasonous Ogre");
    let ball = t.hand(P0, "Ball Lightning");
    t.g.players[P0.idx()].life = 8;
    assert!(!offered(&mut t, P0, ball));
    t.g.players[P0.idx()].life = 10;
    assert!(offered(&mut t, P0, ball));
    t.cast(P0, ball).go();
    assert!(on_stack(&t, "Ball Lightning"));
    assert_eq!(t.life(P0), 1);
}

#[test]
fn what_each_activation_sacrifices_is_matched_to_a_different_object() {
    cr!("118.3", "400.7", "601.2h");
    interp!("automatic-mana-payment");
    interp!("repeatable-mana-abilities-planned-per-activation");
    // Two Ashnod's Altars, a Grizzly Bears and Molt Tender ("{T}, Exile a card from your
    // graveyard: Add one mana of any color."), with a card in the graveyard: the Altars
    // sacrifice the Bears and Molt Tender for {C}{C}{C}{C}, or one of them sacrifices the
    // Bears and Molt Tender adds one more: {4} at most, never {5} (Molt Tender can't be
    // both tapped for mana and sacrificed first, nor the Bears sacrificed twice).
    let mut t = main_phase();
    t.battlefield(P0, "Ashnod's Altar");
    t.battlefield(P0, "Ashnod's Altar");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let tender = t.battlefield(P0, "Molt Tender");
    t.graveyard(P0, "Island");
    let golem = t.hand(P0, "Precursor Golem");
    let tome = t.hand(P0, "Jayemdae Tome");
    assert!(!offered(&mut t, P0, golem));
    assert!(offered(&mut t, P0, tome));
    t.cast(P0, tome).go();
    assert!(on_stack(&t, "Jayemdae Tome"));
    assert!(!t.on_battlefield(bears) && !t.on_battlefield(tender));
}

#[test]
fn phyrexian_altar_pays_each_colored_symbol_with_another_creature() {
    cr!("605.3a", "601.2g");
    interp!("automatic-mana-payment");
    interp!("repeatable-mana-abilities-planned-per-activation");
    // Phyrexian Altar ("Sacrifice a creature: Add one mana of any color.") and three
    // creatures pay {R}{R}{R}; with two, they don't.
    let mut t = main_phase();
    t.battlefield(P0, "Phyrexian Altar");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    let ball = t.hand(P0, "Ball Lightning");
    assert!(!offered(&mut t, P0, ball));
    t.battlefield(P0, "Grizzly Bears");
    assert!(offered(&mut t, P0, ball));
    t.cast(P0, ball).go();
    assert!(on_stack(&t, "Ball Lightning"));
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
}
