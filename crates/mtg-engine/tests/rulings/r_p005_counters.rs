//! Rulings batch P005 — counters seen as a permanent last existed (CR 603.10a, 608.2h)
//! and put on others (CR 122.1, 122.6), counters annihilating as a state-based action at
//! the same time as a creature dies (CR 704.3, 704.5f, 704.5q), distributed counters
//! (CR 601.2d, 608.2b), persist and a card that has left the graveyard (CR 400.7), and
//! exiled cards with aegis counters.

use crate::r_p006_common::{in_graveyard, mark, obj, pl};
use crate::r_p044_common::kill;
use crate::r_p046_common::bounce;
use crate::r_p109_common::{divide, division_of};
use crate::r_s01_common::supported;
use mtg_engine::decision::Decision;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

const ADMIN: &str = "Scolding Administrator";

fn put(t: &mut TestGame, id: ObjectId, kind: &str, n: u32) {
    t.g.add_counters(obj(id), kind, n, None);
    t.g.recompute();
}

/// P0's Scolding Administrator with these counters; returns it.
fn administrator(t: &mut TestGame, with: &[(&str, u32)]) -> ObjectId {
    supported(ADMIN);
    let admin = t.battlefield(P0, ADMIN);
    for (kind, n) in with {
        put(t, admin, kind, *n);
    }
    admin
}

#[test]
fn scolding_administrator_puts_every_kind_of_counter_it_had() {
    cr!("603.10a", "122.1");
    ruling!(
        "Scolding Administrator",
        "Scolding Administrator's last ability puts all counters that were on Scolding Administrator onto the target creature, not just its +1/+1 counters."
    );
    ruling!(
        "Scolding Administrator",
        "Scolding Administrator's last ability doesn't cause you to move counters from Scolding Administrator onto the target creature. Rather, you put the same number of each kind of counter Scolding Administrator had when it died onto the target creature."
    );
    let mut t = TestGame::new(2);
    let admin = administrator(
        &mut t,
        &[(counters::PLUS1, 2), ("flying", 1), ("vigilance", 1)],
    );
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[obj(giant)]);
    kill(&mut t, admin);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.counters(giant, counters::PLUS1), 2);
    assert_eq!(t.counters(giant, "flying"), 1);
    assert_eq!(t.counters(giant, "vigilance"), 1);
    assert_eq!(t.pt(giant), (5, 5));
    // The counters are put on the Hill Giant, so Hardened Scales ("If one or more +1/+1
    // counters would be put on a creature you control, that many plus one +1/+1 counters
    // are put on it instead.") applies.
    let mut t = TestGame::new(2);
    let admin = administrator(&mut t, &[(counters::PLUS1, 2)]);
    t.battlefield(P0, "Hardened Scales");
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[obj(giant)]);
    kill(&mut t, admin);
    t.resolve_all();
    assert_eq!(t.counters(giant, counters::PLUS1), 3);
}

#[test]
fn scolding_administrator_puts_its_minus_counters_too() {
    cr!("603.10a", "704.5f");
    ruling!(
        "Scolding Administrator",
        "If Scolding Administrator has -1/-1 counters on it when it dies, that ability will include those as well. This may result in the recipient of the counters dying."
    );
    // Two -1/-1 counters kill the Administrator (2/2); the Grizzly Bears (2/2) then get
    // two -1/-1 counters and die.
    let mut t = TestGame::new(2);
    let admin = administrator(&mut t, &[]);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    put(&mut t, admin, counters::MINUS1, 2);
    t.settle();
    assert!(in_graveyard(&t, admin));
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(in_graveyard(&t, bears));
}

#[test]
fn scolding_administrator_sees_plus_and_minus_counters_put_on_together() {
    cr!("704.3", "704.5f", "704.5q", "603.10a");
    ruling!(
        "Scolding Administrator",
        "If enough -1/-1 counters are put on Scolding Administrator at the same time to make its toughness 0 or less, its last ability will see all of the +1/+1 counters it had when it died as well as the -1/-1 counters it had, and an equal number of each of those types of counters (plus any other applicable counters) will be put onto the target creature."
    );
    // A 4/4 Administrator (two +1/+1 counters) gets four -1/-1 counters at once (and has
    // a flying counter).
    let mut t = TestGame::new(2);
    let admin = administrator(&mut t, &[(counters::PLUS1, 2), ("flying", 1)]);
    let target = t.battlefield(P0, "Rumbling Baloth");
    t.answer_targets(P0, &[obj(target)]);
    put(&mut t, admin, counters::MINUS1, 4);
    t.settle();
    assert!(in_graveyard(&t, admin));
    t.resolve();
    // The Baloth (4/4) gets two +1/+1, four -1/-1 and a flying counter; the +1/+1 and
    // -1/-1 counters then annihilate in pairs, leaving two -1/-1 counters.
    assert_eq!(t.counters(target, counters::PLUS1), 0);
    assert_eq!(t.counters(target, counters::MINUS1), 2);
    assert_eq!(t.counters(target, "flying"), 1);
    assert_eq!(t.pt(target), (2, 2));
}

#[test]
fn scolding_administrator_and_the_ozolith_both_get_the_counters() {
    cr!("603.10a", "122.1");
    ruling!(
        "Scolding Administrator",
        "In some unusual cases, you may end up putting the appropriate counters on more than one permanent. For example, if you control The Ozolith (a card from the Ikoria: Lair of Behemoths set) when Scolding Administrator dies, you'll put the appropriate number of each kind of counter onto both The Ozolith and the target creature."
    );
    supported("The Ozolith");
    let mut t = TestGame::new(2);
    let ozolith = t.battlefield(P0, "The Ozolith");
    let admin = administrator(&mut t, &[(counters::PLUS1, 2), ("flying", 1)]);
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[obj(giant)]);
    kill(&mut t, admin);
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    for id in [giant, ozolith] {
        assert_eq!(t.counters(id, counters::PLUS1), 2);
        assert_eq!(t.counters(id, "flying"), 1);
    }
}

const GOODIES: CastMethod = CastMethod::Half(1);

/// P0 casts Stolen Goodies (the Adventure of Picnic Ruiner) on these targets with this
/// distribution. Returns the spell.
fn stolen_goodies(t: &mut TestGame, targets: &[Entity], division: &[i64]) -> ObjectId {
    supported("Picnic Ruiner // Stolen Goodies");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 3);
    let card = t.hand(P0, "Picnic Ruiner // Stolen Goodies");
    divide(t, P0, division);
    t.cast(P0, card).method(GOODIES).targets(targets).go()
}

#[test]
fn stolen_goodies_distribution_is_chosen_as_it_is_cast() {
    cr!("601.2d", "601.2c");
    ruling!(
        "Picnic Ruiner // Stolen Goodies",
        "You choose how the counters will be distributed as you cast Stolen Goodies. Each target must receive at least one +1/+1 counter."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let from = mark(&t);
    let spell = stolen_goodies(&mut t, &[obj(a), obj(b)], &[2, 1]);
    assert!(t.asked()[from..].iter().any(|(p, d)| *p == P0
        && matches!(
            d,
            Decision::Divide {
                total: 3,
                min_each: 1,
                ..
            }
        )));
    assert_eq!(division_of(&t, spell), vec![2, 1]);
    t.resolve_all();
    assert_eq!(t.counters(a, counters::PLUS1), 2);
    assert_eq!(t.counters(b, counters::PLUS1), 1);
    // A distribution giving a target none isn't accepted.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let spell = stolen_goodies(&mut t, &[obj(a), obj(b)], &[3, 0]);
    let d = division_of(&t, spell);
    assert!(d.len() == 2 && d.iter().all(|n| *n >= 1), "{d:?}");
}

#[test]
fn stolen_goodies_loses_the_counters_of_an_illegal_target() {
    cr!("608.2b", "601.2d");
    ruling!(
        "Picnic Ruiner // Stolen Goodies",
        "If some of the creatures are illegal targets as Stolen Goodies tries to resolve, the original distribution of counters still applies and the counters that would have been put on illegal targets are lost."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    stolen_goodies(&mut t, &[obj(a), obj(b)], &[2, 1]);
    bounce(&mut t, a);
    t.resolve_all();
    assert_eq!(t.counters(b, counters::PLUS1), 1);
    assert_eq!(t.pt(b), (4, 4));
}

#[test]
fn rhys_persist_doesnt_return_a_card_that_left_the_graveyard() {
    cr!("400.7", "702.79a", "608.2b");
    ruling!(
        "Rhys, the Evermore",
        "If the target creature dies but that card leaves the graveyard before the persist ability resolves, it won't be returned to the battlefield."
    );
    supported("Rhys, the Evermore");
    supported("Tormod's Crypt");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    t.enter(P0, "Rhys, the Evermore");
    t.resolve_all();
    kill(&mut t, bears);
    assert_eq!(t.stack_len(), 1, "persist");
    // In response, P1 exiles P0's graveyard with Tormod's Crypt.
    let crypt = t.battlefield(P1, "Tormod's Crypt");
    t.activate(P1, crypt, 0, &[pl(P0)]).unwrap();
    t.resolve();
    assert_eq!(t.zone(bears), Zone::Exile);
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
}

#[test]
fn livio_returns_every_card_with_an_aegis_counter() {
    cr!("400.7", "122.1");
    ruling!(
        "Livio, Oathsworn Sentinel",
        "Livio's second activated ability will return all exiled cards with aegis counters on them to the battlefield, including ones exiled by another Livio, Oathsworn Sentinel and ones exiled a previous time Livio was on the battlefield."
    );
    const LIVIO: &str = "Livio, Oathsworn Sentinel";
    supported(LIVIO);
    let mut t = TestGame::new(2);
    let livio = t.battlefield(P0, LIVIO);
    // {1}{W}, casting Livio again ({1}{W}) and {2}{W}.
    t.lands(P0, "Plains", 7);
    // P0's Livio exiles P0's Grizzly Bears (P0 chooses to).
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.activate(P0, livio, 0, &[obj(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    assert_eq!(t.counters(bears, "aegis"), 1);
    // P1's Livio exiles P1's Hill Giant.
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    let theirs = t.battlefield(P1, LIVIO);
    t.lands(P1, "Plains", 2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_yes(P1, true);
    t.activate(P1, theirs, 0, &[obj(giant)]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(giant, "aegis"), 1);
    // A card exiled without an aegis counter stays in exile.
    let other = t.exile(P0, "Savannah Lions");
    // P0's Livio leaves and comes back (a new object), then returns them all.
    t.set_step(P0, mtg_engine::turn::Step::PrecombatMain);
    bounce(&mut t, livio);
    let livio = t.g.current(livio);
    t.cast(P0, livio).go();
    t.resolve_all();
    let livio = t.g.current(livio);
    assert!(t.on_battlefield(livio));
    t.g.objects[livio.0 as usize].summoning_sick = false;
    t.activate(P0, livio, 1, &[]).unwrap();
    t.resolve_all();
    let bears = t.g.current(bears);
    let giant = t.g.current(giant);
    assert!(t.on_battlefield(bears) && t.on_battlefield(giant));
    assert_eq!(t.g.obj(bears).controller, P0);
    assert_eq!(t.g.obj(giant).controller, P1);
    assert_eq!(t.zone(other), Zone::Exile);
}
