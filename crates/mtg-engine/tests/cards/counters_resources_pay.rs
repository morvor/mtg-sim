//! Paying resources while an ability resolves: "you may pay {E}{E} / 2 life / {2}. If you
//! do, ..." and "When you do, ..." (CR 118.12, 603.12), "[effect] unless you pay {E}{E}"
//! (CR 118.12a), and energy costs written with a number word ("Pay six {E}").

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
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
    t.g.player(p).counter("energy")
}

fn give_energy(t: &mut TestGame, p: PlayerId, n: u32) {
    t.g.add_counters(Entity::Player(p), "energy", n, None);
}

#[test]
fn thriving_rats_pays_energy_for_a_counter() {
    cr!("107.14", "118.12");
    ruling!(
        "Thriving Rats",
        "To pay one or more {E}, you lose that many energy counters."
    );
    assert_supported(&["Thriving Rats"]);
    let mut t = TestGame::new(2);
    let rats = t.enter(P0, "Thriving Rats");
    t.resolve_all();
    assert_eq!(energy(&t, P0), 2);
    t.g.objects[rats.0 as usize].summoning_sick = false;
    t.answer_yes(P0, true);
    t.attack(&[(rats, Entity::Player(P1))], &[]);
    assert_eq!(t.counters(rats, "+1/+1"), 1);
    assert_eq!(energy(&t, P0), 0);
}

#[test]
fn thriving_rats_cant_pay_energy_it_doesnt_have() {
    cr!("107.14", "118.3");
    ruling!(
        "Thriving Rats",
        "You can't pay more energy counters than you have."
    );
    let mut t = TestGame::new(2);
    let rats = t.battlefield(P0, "Thriving Rats");
    give_energy(&mut t, P0, 1);
    t.answer_yes(P0, true);
    t.attack(&[(rats, Entity::Player(P1))], &[]);
    assert_eq!(t.counters(rats, "+1/+1"), 0);
    assert_eq!(energy(&t, P0), 1);
}

#[test]
fn declining_to_pay_energy_does_nothing() {
    cr!("118.12");
    assert_supported(&["Aetherstream Leopard"]);
    let mut t = TestGame::new(2);
    let leo = t.battlefield(P0, "Aetherstream Leopard");
    give_energy(&mut t, P0, 1);
    t.answer_yes(P0, false);
    t.attack(&[(leo, Entity::Player(P1))], &[]);
    assert_eq!(energy(&t, P0), 1);
    // Unblocked 2/3 trample dealt 2, not 4.
    assert_eq!(t.life(P1), 18);
}

#[test]
fn erebos_pays_two_life_to_draw() {
    cr!("119.4", "118.12");
    ruling!(
        "Erebos, Bleak-Hearted",
        "you can't pay more than 2 life to draw more than one card"
    );
    assert_supported(&["Erebos, Bleak-Hearted"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Erebos, Bleak-Hearted");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let hand = t.hand_size(P0);
    // First death: pay 2 life and draw one card.
    t.answer_yes(P0, true);
    t.g.destroy(a, None);
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.hand_size(P0), hand + 1);
    // Second death: decline, nothing happens.
    t.answer_yes(P0, false);
    t.g.destroy(b, None);
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn may_pay_mana_after_an_intervening_if() {
    cr!("603.4", "118.12");
    assert_supported(&["Markov Purifier"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Markov Purifier");
    t.lands(P0, "Plains", 2);
    let tapped_plains = |t: &TestGame| {
        t.g.battlefield
            .iter()
            .filter(|&&l| t.g.obj(l).name() == "Plains" && t.g.obj(l).tapped)
            .count()
    };
    // No life gained this turn: the ability doesn't trigger, so nothing is paid or drawn.
    t.answer_yes(P0, true);
    let hand = t.hand_size(P0);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(tapped_plains(&t), 0);
    t.clear_answers();
    // A later turn in which P0 gained life: pay {2} and draw.
    t.advance_to(P0, Step::PrecombatMain);
    t.g.gain_life(P0, 1);
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(tapped_plains(&t), 2);
}

#[test]
fn when_you_pay_life_a_reflexive_trigger_targets() {
    cr!("603.12", "119.4");
    assert_supported(&["Ambulatory Edifice"]);
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(bear)]);
    t.enter(P0, "Ambulatory Edifice");
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.pt(bear), (1, 1));
}

#[test]
fn lathnu_hellion_is_sacrificed_unless_energy_is_paid() {
    cr!("118.12a", "107.14");
    assert_supported(&["Lathnu Hellion"]);
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Lathnu Hellion");
    give_energy(&mut t, P0, 2);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.on_battlefield(h));
    assert_eq!(energy(&t, P0), 0);
    // Its next end step: no energy left, so it's sacrificed.
    t.advance_to(P1, Step::End);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(!t.on_battlefield(h));
    assert!(t.in_graveyard(P0, "Lathnu Hellion"));
}

#[test]
fn pay_six_energy_cost() {
    cr!("107.14", "118.3");
    assert_supported(&["Roil Cartographer"]);
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Roil Cartographer");
    give_energy(&mut t, P0, 5);
    assert!(t.activate(P0, r, 0, &[]).is_err());
    give_energy(&mut t, P0, 1);
    let hand = t.hand_size(P0);
    t.activate(P0, r, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(energy(&t, P0), 0);
    assert_eq!(t.hand_size(P0), hand + 3);
}

#[test]
fn an_alternative_cost_isnt_paid_as_the_spell_resolves() {
    cr!("118.9");
    // "If you control a Swamp, you may pay 4 life rather than pay this spell's mana
    // cost" is an alternative cost chosen as the spell is cast, not a payment offered
    // while it resolves.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    let bear = t.battlefield(P1, "Grizzly Bears");
    let s = t.hand(P0, "Snuff Out");
    t.answer_yes(P0, true);
    t.cast(P0, s).target(bear).go();
    t.resolve_all();
    assert!(!t.on_battlefield(bear));
    assert_eq!(t.life(P0), 20);
}
