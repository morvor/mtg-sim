//! Rulings batch P121 — costs and choices made while casting or activating: cost
//! reductions apply to the total cost after increases and only to generic mana, and don't
//! change mana value (CR 601.2f, 118.7, 202.3); mana that can be spent only on artifact
//! spells and abilities of artifacts (CR 106.6); a single optional payment; sacrificing the
//! source to pay its own cost (CR 602.2); dividing counters or damage among targets as a
//! spell is cast or an ability is put on the stack, at least one to each (CR 601.2d).

use crate::r_p121_common::*;
use crate::r_s01_common::tapped_lands;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The Divide decisions `p` was asked since decision `from`: (total, recipients, minimum
/// each).
fn divides_since(t: &TestGame, from: usize) -> Vec<(u32, usize, u32)> {
    t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::Divide {
                total,
                recipients,
                min_each,
                ..
            } => Some((*total, recipients.len(), *min_each)),
            _ => None,
        })
        .collect()
}

// --- cost reductions ------------------------------------------------------------------------

#[test]
fn the_earth_crystal_reduces_only_generic_mana() {
    cr!("601.2f", "118.7");
    ruling!(
        "The Earth Crystal",
        "The cost reduction applies only to generic mana in the total cost of green spells you cast."
    );
    supported("The Earth Crystal");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Earth Crystal");
    t.lands(P0, "Forest", 1);
    let tusker = t.hand(P0, "Kalonian Tusker");
    assert!(t.cast(P0, tusker).try_go().is_err(), "{{G}}{{G}} stays");
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(t.cast(P0, bears).try_go().is_ok(), "{{1}}{{G}} is {{G}}");
}

#[test]
fn reductions_apply_to_the_total_cost_and_dont_change_mana_value() {
    cr!("601.2f", "202.3", "118.7");
    ruling!(
        "The Earth Crystal",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying (such as a flashback cost), add any cost increases (such as kicker costs), then apply any cost reductions (such as that of The Earth Crystal's first ability). The mana value of the spell is determined by only its mana cost, no matter what the total cost to cast that spell was."
    );
    ruling!(
        "Hamza, Guardian of Arashin",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying, add any cost increases, then apply any cost reductions (such as that of Hamza, Guardian of Arashin). The total cost is locked in before any costs are paid. The mana value of the spell is determined only by its mana cost, no matter what the total cost to cast the spell was."
    );
    ruling!(
        "Prehistoric Turtlesaurus",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying, add any cost increases, then apply any cost reductions (such as that of Prehistoric Turtlesaurus's first ability). The mana value of the spell remains unchanged, no matter what the total cost to cast it was."
    );
    supported("Hamza, Guardian of Arashin");
    supported("Prehistoric Turtlesaurus");
    supported("Gnarlid Colony");
    // The Earth Crystal: Gnarlid Colony ({1}{G}) kicked ({2}{G}) costs {2}{G}{G}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Earth Crystal");
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Wastes", 2);
    let colony = t.hand(P0, "Gnarlid Colony");
    let spell = t.cast(P0, colony).kicked(true).go();
    assert_eq!(tapped_lands(&t, P0), 4);
    assert_eq!(t.g.mana_value_of(spell), 2);
    t.resolve_all();
    // Kicked: two +1/+1 counters, doubled by The Earth Crystal.
    let c = t.named_on_battlefield("Gnarlid Colony")[0];
    assert_eq!(plus1(&t, c), 4);
    // Hamza: two creatures with counters, Hill Giant ({3}{R}) costs {1}{R}. Paying
    // the locked-in cost doesn't change it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hamza, Guardian of Arashin");
    with_counters(&mut t, P0, "Grizzly Bears", 1);
    with_counters(&mut t, P0, "Grizzly Bears", 1);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 1);
    let giant = t.hand(P0, "Hill Giant");
    let spell = t.cast(P0, giant).go();
    assert_eq!(tapped_lands(&t, P0), 2);
    assert_eq!(t.g.mana_value_of(spell), 4);
    // Prehistoric Turtlesaurus ({4}{G}) costs {3}{G}.
    let mut t = TestGame::new(2);
    with_counters(&mut t, P0, "Grizzly Bears", 1);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 3);
    let turtle = t.hand(P0, "Prehistoric Turtlesaurus");
    let spell = t.cast(P0, turtle).go();
    assert_eq!(t.g.mana_value_of(spell), 5);
}

// --- mana and payments ----------------------------------------------------------------------

#[test]
fn power_depot_mana_cant_pay_for_cycling_an_artifact_card() {
    cr!("106.6", "702.29a");
    ruling!(
        "Power Depot",
        "The mana produced by the last activated ability can't be spent to activate abilities of artifact cards in other zones, such as cycling or unearth costs."
    );
    supported("Power Depot");
    supported("Indatha Crystal");
    let mut t = TestGame::new(2);
    let depot = t.battlefield(P0, "Power Depot");
    t.lands(P0, "Wastes", 1);
    // Depot's restricted mana goes into P0's pool.
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.activate(P0, depot, 1, &[]).expect("mana ability");
    assert_eq!(crate::r_p119_common::pool_total(&t, P0), 1);
    // Cycling {2}: the restricted mana can't help.
    let crystal = t.hand(P0, "Indatha Crystal");
    let cycling = t
        .obj_now(crystal)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, mtg_engine::ability::AbilityKind::Activated(_)))
        .position(|a| a.text.to_lowercase().contains("cycling"))
        .expect("cycling");
    assert!(t.activate(P0, crystal, cycling, &[]).is_err());
    assert!(t.in_hand(P0, "Indatha Crystal"));
    // It can pay for an artifact spell: Scrapwork Mutt ({2}).
    let mutt = t.hand(P0, "Scrapwork Mutt");
    assert!(t.cast(P0, mutt).try_go().is_ok());
}

#[test]
fn animation_module_pays_only_once_per_trigger() {
    cr!("603.5", "118.8");
    ruling!(
        "Animation Module",
        "You can pay {1} only once each time Animation Module's first ability resolves. You can't pay more to create extra Servo tokens."
    );
    supported("Animation Module");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Animation Module");
    t.lands(P0, "Wastes", 3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    give_plus1(&mut t, bears, 3);
    t.resolve_all();
    assert_eq!(tokens_named(&t, P0, "Servo"), 1);
    assert_eq!(tapped_lands(&t, P0), 1);
}

#[test]
fn scrapyard_recombiner_can_sacrifice_itself_to_pay_its_cost() {
    cr!("602.2", "118.3", "701.21a");
    ruling!(
        "Scrapyard Recombiner",
        "You can sacrifice Scrapyard Recombiner to pay the cost of its second ability."
    );
    supported("Scrapyard Recombiner");
    let mut t = TestGame::new(2);
    let rec = with_counters(&mut t, P0, "Scrapyard Recombiner", 2);
    t.library_top(P0, "Hangarback Walker");
    t.answer_choose(P0, &[obj(rec)]);
    t.activate(P0, rec, 0, &[]).expect("activate");
    assert!(t.in_graveyard(P0, "Scrapyard Recombiner"));
    t.resolve_all();
    assert!(t.in_hand(P0, "Hangarback Walker"));
}

// --- dividing among targets ----------------------------------------------------------------

#[test]
fn biogenic_upgrade_divides_as_it_is_cast_at_least_one_each() {
    cr!("601.2d", "115.7");
    ruling!(
        "Biogenic Upgrade",
        "You choose how many targets Biogenic Upgrade has and how the counters are distributed as you cast it. Each target must receive at least one counter. This means, for example, that you can't target three creatures and assign them two, one, and zero counters."
    );
    supported("Biogenic Upgrade");
    for (answer, want) in [(vec![2, 1], (4, 2)), (vec![3, 0], (0, 0))] {
        let mut t = TestGame::new(2);
        let a = t.battlefield(P0, "Grizzly Bears");
        let b = t.battlefield(P0, "Grizzly Bears");
        t.answer(P0, DecisionKind::Divide, Answer::Numbers(answer.clone()));
        let from = t.asked().len();
        cast_slots(&mut t, P0, "Biogenic Upgrade", &[&[obj(a), obj(b)]]);
        assert_eq!(divides_since(&t, from), vec![(3, 2, 1)]);
        t.resolve_all();
        let got = (plus1(&t, a), plus1(&t, b));
        if want.0 > 0 {
            assert_eq!(got, want);
        } else {
            // An assignment of zero isn't legal: each target got at least one.
            assert!(got.0 >= 2 && got.1 >= 2, "{answer:?}: {got:?}");
            assert_eq!(got.0 + got.1, 6);
        }
    }
}

#[test]
fn the_earth_crystal_divides_as_it_is_activated() {
    cr!("601.2d", "602.2b");
    ruling!(
        "The Earth Crystal",
        "You choose how many targets The Earth Crystal's last ability has and how the counters are distributed as you activate it. Each target must receive at least one counter. This means, for example, that you can't target two creatures and assign them two and zero counters respectively."
    );
    let mut t = TestGame::new(2);
    let crystal = t.battlefield(P0, "The Earth Crystal");
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Wastes", 4);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![1, 1]));
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(a), obj(b)]);
    t.activate(P0, crystal, 0, &[]).expect("activate");
    assert_eq!(divides_since(&t, from), vec![(2, 2, 1)]);
    assert_eq!((plus1(&t, a), plus1(&t, b)), (0, 0));
    t.resolve_all();
    // One each, doubled by The Earth Crystal's replacement effect.
    assert_eq!((plus1(&t, a), plus1(&t, b)), (2, 2));
}

#[test]
fn court_of_garenbrig_divides_as_its_trigger_is_put_on_the_stack() {
    cr!("601.2d", "603.3d");
    ruling!(
        "Court of Garenbrig",
        "You choose how the counters will be distributed as you put Court of Garenbrig's last ability on the stack. Each target must receive at least one +1/+1 counter."
    );
    supported("Court of Garenbrig");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Court of Garenbrig");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P1, Step::End);
    t.answer_targets(P0, &[obj(a), obj(b)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![1, 1]));
    let from = t.asked().len();
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(stack(&t), 1);
    assert_eq!(divides_since(&t, from), vec![(2, 2, 1)]);
    t.resolve_all();
    // P0 isn't the monarch: no doubling.
    assert_eq!((plus1(&t, a), plus1(&t, b)), (1, 1));
}

#[test]
fn skarrgan_hellkite_divides_as_it_is_activated() {
    cr!("601.2d", "602.2b");
    ruling!(
        "Skarrgan Hellkite",
        "You divide the damage as you activate Skarrgan Hellkite's ability, not as it resolves. Each target must be assigned at least 1 damage. In other words, as you activate the ability, you choose whether to have Skarrgan Hellkite deal 2 damage to a single target, or deal 1 damage to each of two targets."
    );
    supported("Skarrgan Hellkite");
    let mut t = TestGame::new(2);
    let h = with_counters(&mut t, P0, "Skarrgan Hellkite", 1);
    t.lands(P0, "Mountain", 4);
    let a = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(a), Entity::Player(P1)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![1, 1]));
    let from = t.asked().len();
    t.activate(P0, h, 0, &[]).expect("activate");
    assert_eq!(divides_since(&t, from), vec![(2, 2, 1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.obj_now(a).damage, 1);
}
