//! Replacement effects on putting counters (CR 614.1a, 122.6) apply to the counters they
//! name, and only when one or more are put. "If you would put one or more +1/+1 counters
//! on a creature you control, put that many plus one +1/+1 counters on it instead."
//! (Prairie Dog) doesn't add to counters of another kind (Blastoderm's fade counters). A
//! permanent that "enters with X additional +1/+1 counters" for X = 0 (Abuelo's Awakening)
//! is put no counters: Prairie Dog has none to add one to, and two Vorinclexes leave its
//! controller nothing to choose an order for (CR 616.1).

use crate::r614_audit_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

/// P0 activates Prairie Dog's "{4}{W}: Until end of turn, if you would put one or more
/// +1/+1 counters on a creature you control, put that many plus one +1/+1 counters on it
/// instead." and it resolves.
fn prairie_dog_active(t: &mut TestGame) {
    supported("Prairie Dog");
    let dog = t.battlefield(P0, "Prairie Dog");
    t.lands(P0, "Plains", 5);
    t.activate(P0, dog, 0, &[]).expect("Prairie Dog's ability");
    t.resolve_all();
}

/// P0 casts Abuelo's Awakening ("Return target artifact or non-Aura enchantment card from
/// your graveyard to the battlefield with X additional +1/+1 counters on it. It's a 1/1
/// Spirit creature with flying in addition to its other types.") for `x` on a Memnite in
/// P0's graveyard. Returns the Memnite it returns.
fn awaken_memnite(t: &mut TestGame, x: i64) -> ObjectId {
    supported("Abuelo's Awakening");
    let memnite = t.graveyard(P0, "Memnite");
    t.lands(P0, "Plains", 4 + x as usize);
    let spell = t.hand(P0, "Abuelo's Awakening");
    t.cast(P0, spell).x(x).target(memnite).go();
    t.resolve_all();
    let m = t.g.current(memnite);
    assert!(t.on_battlefield(m), "Memnite returns (X = {x})");
    m
}

#[test]
fn prairie_dog_adds_only_to_plus1_counters_and_only_when_some_are_put() {
    cr!("614.1a", "614.1c", "122.6", "702.32a");
    ruling!(
        "Prairie Dog",
        "If a creature you control would enter the battlefield with a number of +1/+1 counters on it on a turn where Prairie Dog’s last ability has resolved once, it enters with that many plus one instead."
    );
    supported("Blastoderm");
    let mut t = TestGame::new(2);
    prairie_dog_active(&mut t);
    // Spike Feeder enters with two +1/+1 counters, and one more.
    let feeder = t.enter(P0, "Spike Feeder");
    assert_eq!(t.counters(feeder, "+1/+1"), 3);
    // Blastoderm (fading 3) enters with three fade counters: not +1/+1 counters.
    let blastoderm = t.enter(P0, "Blastoderm");
    assert_eq!(t.counters(blastoderm, "fade"), 3);
    assert_eq!(t.counters(blastoderm, "+1/+1"), 0);
    // X = 0: Memnite returns with no +1/+1 counters, so there are none to add one to.
    let memnite = awaken_memnite(&mut t, 0);
    assert_eq!(t.counters(memnite, "+1/+1"), 0);
}

/// P0 and P1 each control Vorinclex, Monstrous Raider: the counters P0 puts are doubled by
/// P0's ("If you would put one or more counters on a permanent or player, put twice that
/// many ...") and halved by P1's ("If an opponent would put one or more counters ..., they
/// put half that many ..."), in the order P0 chooses.
#[test]
fn no_counters_put_leaves_two_vorinclexes_nothing_to_order() {
    cr!("614.1a", "616.1", "122.6", "122.6a");
    ruling!(
        "Vorinclex, Monstrous Raider",
        "If two or more effects attempt to modify how many counters would be put onto a permanent you control, you choose the order to apply those effects, no matter who controls the sources of those effects."
    );
    ruling!(
        "Vorinclex, Monstrous Raider",
        "Unlike many similar effects, Vorinclex cares deeply about who is putting the counters on the permanent or player to determine which of its two last abilities applies."
    );
    ruling!(
        "Vorinclex, Monstrous Raider",
        "If the effect doesn’t specify a player, the object’s controller puts those counters on it."
    );
    supported("Vorinclex, Monstrous Raider");
    for x in [1, 0] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Vorinclex, Monstrous Raider");
        t.battlefield(P1, "Vorinclex, Monstrous Raider");
        let memnite = awaken_memnite(&mut t, x);
        let choices = replacement_choices(&t);
        if x == 1 {
            // P0, Memnite's controller, puts its counter: P0's Vorinclex's first effect and
            // P1's second apply, and P0 chooses which to apply first.
            assert_eq!(choices.len(), 1, "{choices:?}");
            let (who, options) = &choices[0];
            assert_eq!(*who, P0);
            assert_eq!(options.len(), 2, "{options:?}");
            for needle in ["if you would put", "if an opponent would put"] {
                assert!(
                    options.iter().any(|o| o.to_lowercase().contains(needle)),
                    "{needle}: {options:?}"
                );
            }
        } else {
            // No counters are put, so neither effect applies. (Either would leave Memnite
            // with none: twice 0, or 0 less half of 0. What would show is the choice.)
            assert!(choices.is_empty(), "{choices:?}");
            assert_eq!(t.counters(memnite, "+1/+1"), 0);
        }
    }
}
