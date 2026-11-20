//! Two instances of Unstable Amulet's "{T}, Pay {E}{E}: Exile the top card of your library.
//! You may play it until you exile another card with this artifact." on one object, gained
//! from two different cards (Myr Welder: "This creature has all activated abilities of all
//! cards exiled with it."; Mairsil, the Pretender: "Mairsil has all activated abilities of all
//! cards you own in exile with cage counters on them."). The ability exiles a card and refers
//! to exiling another one, so it's linked to itself:
//!
//! - 607.1c "An ability printed on an object that fulfills both criteria described in rule
//!   607.1 is linked to itself."
//! - 607.5 "If an object acquires a pair of linked abilities as part of the same effect, the
//!   abilities will be similarly linked to one another on that object even though they
//!   weren’t printed on that object. They can’t be linked to any other ability, regardless of
//!   what other abilities the object may currently have or may have had in the past."
//!
//! So each instance's permission lasts until that instance exiles another card (CR 611.2a),
//! not until the other one does. Before the fix (`permissions::given` told the permissions
//! apart only by their source object) each instance's exile ended the other's permission
//! (found by the November 2026 mutation run of `permissions.rs`, docs/MUTATION.md).

use mtg_engine::ability::AbilityKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

/// The text of Unstable Amulet's activated ability.
const AMULETS: &str = "You may play it until you exile another card";

/// Whether `p` has a way to cast `card` from where it is: a card in exile has one only while a
/// permission covers it.
fn may_cast(t: &TestGame, p: PlayerId, card: ObjectId) -> bool {
    !t.g.cast_options(p, t.g.current(card)).is_empty()
}

/// The indices among `id`'s activated abilities of those whose text contains `text`.
fn indices_of(t: &mut TestGame, id: ObjectId, text: &str) -> Vec<usize> {
    t.g.recompute();
    t.g.obj(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .enumerate()
        .filter(|(_, a)| a.text.contains(text))
        .map(|(i, _)| i)
        .collect()
}

/// `holder`'s two Amulet abilities: the first exiles the Shock, the second the Lightning Bolt
/// (both stay playable); then (after `between`) the first exiles the Memnite (the Shock's
/// permission ends, the Bolt's doesn't); the second exiles the Ornithopter (the Bolt's ends,
/// the Memnite's doesn't).
fn two_amulet_abilities(t: &mut TestGame, holder: ObjectId, between: fn(&mut TestGame)) {
    let amulets = indices_of(t, holder, AMULETS);
    assert_eq!(amulets.len(), 2, "{}", t.dump_log());
    t.g.add_counters(Entity::Player(P0), counters::ENERGY, 8, None);
    let mut exile_top = |t: &mut TestGame, which: usize, name: &str| {
        let card = t.library_top(P0, name);
        t.g.untap(holder);
        t.activate(P0, holder, amulets[which], &[]).unwrap();
        t.resolve_all();
        assert_eq!(t.zone(t.g.current(card)), Zone::Exile, "{name}");
        card
    };
    let shock = exile_top(t, 0, "Shock");
    let bolt = exile_top(t, 1, "Lightning Bolt");
    assert!(may_cast(t, P0, shock), "{}", t.dump_log());
    assert!(may_cast(t, P0, bolt));
    between(t);
    let memnite = exile_top(t, 0, "Memnite");
    assert!(!may_cast(t, P0, shock));
    assert!(may_cast(t, P0, bolt), "{}", t.dump_log());
    assert!(may_cast(t, P0, memnite));
    let thopter = exile_top(t, 1, "Ornithopter");
    assert!(!may_cast(t, P0, bolt));
    assert!(may_cast(t, P0, memnite));
    assert!(may_cast(t, P0, thopter));
}

/// Myr Welder that has imprinted two Unstable Amulets.
#[test]
fn myr_welder_with_two_unstable_amulets_each_ability_ends_only_its_own_permission() {
    cr!("607.1c", "607.5", "611.2a");
    interp!("until-another-per-ability-instance");
    let mut t = TestGame::new(2);
    let welder = t.battlefield(P0, "Myr Welder");
    for _ in 0..2 {
        let amulet = t.graveyard(P0, "Unstable Amulet");
        let i = indices_of(&mut t, welder, "Exile target artifact card")[0];
        t.g.untap(welder);
        t.activate(P0, welder, i, &[Entity::Object(amulet)]).unwrap();
        t.resolve_all();
    }
    two_amulet_abilities(&mut t, welder, |_| {});
}

/// Mairsil, the Pretender with two Unstable Amulets P0 owns in exile with cage counters.
#[test]
fn mairsil_with_two_caged_unstable_amulets_each_ability_ends_only_its_own_permission() {
    cr!("607.1c", "607.5", "611.2a");
    interp!("until-another-per-ability-instance");
    ruling!(
        "Mairsil, the Pretender",
        "If multiple cards exiled with cage counters on them have the same ability, Mairsil will have multiple instances of that ability. Each may be activated once each turn."
    );
    let mut t = TestGame::new(2);
    let mairsil = t.battlefield(P0, "Mairsil, the Pretender");
    for _ in 0..2 {
        let amulet = t.exile(P0, "Unstable Amulet");
        t.g.add_counters(Entity::Object(amulet), "cage", 1, None);
    }
    // Each instance may be activated once each turn: the second round is P0's next turn.
    two_amulet_abilities(&mut t, mairsil, |t| {
        let mairsil = t.g.find_in_zone(Zone::Battlefield, "Mairsil, the Pretender")[0];
        let first = indices_of(t, mairsil, AMULETS)[0];
        t.g.untap(mairsil);
        assert!(t.activate(P0, mairsil, first, &[]).is_err());
        t.clear_answers();
        for p in [P0, P1] {
            for _ in 0..3 {
                t.library_top(p, "Island");
            }
        }
        t.advance_to(P1, Step::PrecombatMain);
        t.advance_to(P0, Step::PrecombatMain);
    });
}
