//! Rulings batch S08 — ferocious (an ability word, CR 207.2c): "If you control a creature
//! with power 4 or greater, ...". On instants and sorceries it's checked as the spell
//! resolves (CR 608.2c); with "instead" it replaces the normal effect (CR 614.1a).

use crate::r_s01_common::*;
use crate::r_s02_common::*;
use crate::r_s08_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0 casts Feed the Clan ("You gain 5 life. Ferocious — You gain 10 life instead if you
/// control a creature with power 4 or greater."); `before_resolving` runs while it's on
/// the stack. P0's life total afterwards.
fn feed_the_clan(big: bool, before_resolving: impl FnOnce(&mut TestGame, Option<ObjectId>)) -> i32 {
    let mut t = TestGame::new(2);
    let wurm = big.then(|| t.battlefield(P0, "Craw Wurm"));
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    let feed = t.hand(P0, "Feed the Clan");
    t.cast(P0, feed).go();
    before_resolving(&mut t, wurm);
    t.resolve_all();
    t.life(P0)
}

#[test]
fn an_instead_ferocious_spell_gives_only_the_upgraded_effect() {
    cr!("207.2c", "608.2c", "614.1a");
    ruling!(
        "Feed the Clan",
        "For these, you only get the upgraded effect, not both effects."
    );
    supported("Feed the Clan");
    // Craw Wurm is 6/4: 10 life, not 5 + 10.
    assert_eq!(feed_the_clan(true, |_, _| {}), 30);
    // Only Grizzly Bears (2/2): the normal effect.
    assert_eq!(feed_the_clan(false, |_, _| {}), 25);
    // The Wurm is gone as the spell resolves: the normal effect.
    assert_eq!(
        feed_the_clan(true, |t, wurm| destroy(t, wurm.unwrap())),
        25
    );
}

/// P1 casts Lightning Bolt at P0 (P1 has one more Mountain to pay with), and P0 responds
/// with Stubborn Denial ("Counter target noncreature spell unless its controller pays {1}.
/// Ferocious — If you control a creature with power 4 or greater, counter that spell
/// instead."). Returns P0's life and whether P1 was offered to pay.
fn denial(big: bool) -> (i32, bool) {
    let mut t = TestGame::new(2);
    if big {
        t.battlefield(P0, "Craw Wurm");
    }
    t.lands(P1, "Mountain", 2);
    t.lands(P0, "Island", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(P0).go();
    let denial = t.hand(P0, "Stubborn Denial");
    t.cast(P0, denial).target(bolt).go();
    let from = t.asked().len();
    t.answer_yes(P1, true);
    t.resolve_all();
    let offered = t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P1 && matches!(d, Decision::YesNo { .. }));
    (t.life(P0), offered)
}

#[test]
fn an_upgraded_counterspell_doesnt_also_offer_to_pay() {
    cr!("207.2c", "608.2c", "614.1a");
    ruling!(
        "Stubborn Denial",
        "For these, you only get the upgraded effect, not both effects."
    );
    supported("Stubborn Denial");
    // Without a big creature, P1 pays {1} and the Bolt resolves.
    assert_eq!(denial(false), (17, true));
    // With one, the Bolt is just countered: P1 isn't offered to pay.
    assert_eq!(denial(true), (20, false));
}

/// P0 casts Icy Blast with X = 1 on P1's Grizzly Bears ("Tap X target creatures.
/// Ferocious — If you control a creature with power 4 or greater, those creatures don't
/// untap during their controllers' next untap steps."). If `grow`, P0 responds by giving
/// its own Grizzly Bears +3/+3 with Giant Growth. Whether P1's Bears are still tapped in
/// P1's next upkeep.
fn icy_blast(grow: bool) -> bool {
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    t.lands(P0, "Forest", 1);
    let blast = t.hand(P0, "Icy Blast");
    t.cast(P0, blast).x(1).target(theirs).go();
    if grow {
        let growth = t.hand(P0, "Giant Growth");
        t.cast(P0, growth).target(mine).go();
    }
    t.resolve_all();
    // Tapped either way: the ferocious part is additional.
    assert!(is_tapped(&t, theirs));
    t.advance_to(P1, Step::Upkeep);
    is_tapped(&t, theirs)
}

#[test]
fn a_ferocious_spell_without_instead_adds_its_effect_if_the_condition_holds_on_resolution() {
    cr!("207.2c", "608.2c");
    ruling!(
        "Icy Blast",
        "will provide an additional effect if you control a creature with power 4 or greater as they resolve."
    );
    supported("Icy Blast");
    // No creature with power 4 or greater: the Bears untap normally.
    assert!(!icy_blast(false));
    // P0's Bears became 5/5 before Icy Blast resolved: they also stay tapped.
    assert!(icy_blast(true));
}

#[test]
fn barrage_of_boulders_deals_its_damage_either_way() {
    cr!("207.2c", "608.2c");
    ruling!(
        "Barrage of Boulders",
        "will provide an additional effect if you control a creature with power 4 or greater as they resolve."
    );
    supported("Barrage of Boulders");
    // Barrage of Boulders: "deals 1 damage to each creature you don't control. Ferocious —
    // If you control a creature with power 4 or greater, creatures can't block this turn."
    for big in [false, true] {
        let mut t = TestGame::new(2);
        let attacker = t.battlefield(P0, if big { "Craw Wurm" } else { "Grizzly Bears" });
        let elves = t.battlefield(P1, "Llanowar Elves");
        let giant = t.battlefield(P1, "Hill Giant");
        t.lands(P0, "Mountain", 3);
        let barrage = t.hand(P0, "Barrage of Boulders");
        t.cast(P0, barrage).go();
        t.resolve_all();
        assert!(t.in_graveyard(P1, "Llanowar Elves"));
        assert!(!t.on_battlefield(elves));
        assert_eq!(t.obj_now(giant).damage, 1);
        // The additional effect: the Hill Giant can't block.
        t.set_step(P0, Step::BeginningOfCombat);
        attack_with(&mut t, &[(attacker, Entity::Player(P1))]);
        block_and_finish(&mut t, P1, &[(giant, attacker)]);
        let blocked = t.life(P1) == 20;
        assert_eq!(blocked, !big, "big: {big}");
    }
}
