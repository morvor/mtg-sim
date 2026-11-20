//! "You may play it until you exile another card with this artifact" (Unstable Amulet; also
//! Furious Rise, Superior Foes of Spider-Man): the permission lasts as long as it says
//! (CR 611.2a), so it ends when the same ability exiles another card for the same player
//! (`permissions::given`), not when another object's ability (CR 700.7), another ability of
//! the same object (CR 607.1, 607.1c, 607.5) or another player (CR 109.5) exiles one.
//! Written for survivors of the November 2026 mutation run of `permissions.rs`
//! (docs/MUTATION.md).

use mtg_engine::ability::AbilityKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

/// The text of Unstable Amulet's activated ability (which Myr Welder gains).
const AMULETS: &str = "You may play it until you exile another card";

/// Whether `p` has a way to cast `card` from where it is: a card in exile has one only while a
/// permission covers it.
fn may_cast(t: &TestGame, p: PlayerId, card: ObjectId) -> bool {
    !t.g.cast_options(p, t.g.current(card)).is_empty()
}

/// The index among `id`'s activated abilities of the first one whose text contains `text`.
fn index_of(t: &mut TestGame, id: ObjectId, text: &str) -> usize {
    t.g.recompute();
    let all: Vec<String> =
        t.g.obj(id)
            .chars
            .abilities
            .iter()
            .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
            .map(|a| a.text.clone())
            .collect();
    all.iter()
        .position(|a| a.contains(text))
        .unwrap_or_else(|| panic!("no activated ability {text:?} among {all:?}"))
}

/// Kills `permissions.rs:975:64` (in `permissions::given`, `!(gr.terms.until_another &&
/// newer.contains(&gr.source))` with `||`: when an effect gives an "until you exile another
/// card with ~" permission, every earlier such permission ends, whichever object gave it, and so
/// does every earlier permission the same object gave). "This artifact" is that particular
/// Unstable Amulet (CR 700.7): the second Amulet exiling the Lightning Bolt doesn't end the
/// first's permission for the Shock, and the first exiling the Memnite ends that one but not
/// the second's for the Bolt. With the mutant the Bolt's exile ends the Shock's permission, and
/// the Memnite's the Bolt's.
#[test]
fn another_unstable_amulet_exiling_a_card_doesnt_end_this_ones_permission() {
    cr!("611.2a", "700.7");
    let mut t = TestGame::new(2);
    let first = t.battlefield(P0, "Unstable Amulet");
    let second = t.battlefield(P0, "Unstable Amulet");
    // (Put directly onto the battlefield, they gave no energy as they entered.)
    t.g.add_counters(Entity::Player(P0), counters::ENERGY, 6, None);
    let shock = t.library_top(P0, "Shock");
    t.activate(P0, first, 0, &[]).unwrap();
    t.resolve_all();
    let bolt = t.library_top(P0, "Lightning Bolt");
    t.activate(P0, second, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(shock)), Zone::Exile);
    assert_eq!(t.zone(t.g.current(bolt)), Zone::Exile);
    // The Bolt isn't another card exiled with the first Amulet.
    assert!(may_cast(&t, P0, shock), "{}", t.dump_log());
    assert!(may_cast(&t, P0, bolt));
    // The first Amulet exiles another card: that ends the Shock's permission only.
    let memnite = t.library_top(P0, "Memnite");
    t.g.untap(first);
    t.activate(P0, first, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(memnite)), Zone::Exile);
    assert!(!may_cast(&t, P0, shock));
    assert!(may_cast(&t, P0, bolt), "{}", t.dump_log());
    assert!(may_cast(&t, P0, memnite));
}

/// Kills `permissions.rs:969:45` (in `permissions::given`, `gr.terms.until_another &&
/// gr.source.is_some()` with `||`: any permission an object's ability gives with terms ends the
/// "until you exile another card with ~" permissions the same object gave). Unstable Amulet's
/// ability exiles a card and refers to exiling another one "with this artifact", so it's linked
/// to itself (CR 607.1, 607.1c), and Myr Welder, which gains it, keeps it linked to itself only
/// (CR 607.5): the Lightning Bolt Myr Welder exiles with Oracle's Vault's second ability isn't
/// another card exiled with the Amulet's, so the Shock it exiled with that one stays playable
/// until the Amulet's ability exiles the Memnite. With the mutant the Vault's permission ends
/// the Shock's. (The Vault's free-play ability is used because its permission comes with terms:
/// "Until end of turn, you may play that card." alone, like most such texts, gives a bare
/// permission, which `given` never sees.)
#[test]
fn myr_welders_other_exile_ability_doesnt_end_its_unstable_amulet_permission() {
    cr!("607.1", "607.1c", "607.5", "611.2a");
    ruling!(
        "Myr Welder",
        "treat Myr Welder's version of that ability as though it referenced Myr Welder by name instead"
    );
    let mut t = TestGame::new(2);
    let welder = t.battlefield(P0, "Myr Welder");
    let amulet = t.graveyard(P0, "Unstable Amulet");
    let vault = t.graveyard(P0, "Oracle's Vault");
    for card in [amulet, vault] {
        let i = index_of(&mut t, welder, "Exile target artifact card");
        t.activate(P0, welder, i, &[Entity::Object(card)]).unwrap();
        t.resolve_all();
        t.g.untap(welder);
    }
    t.g.add_counters(Entity::Player(P0), counters::ENERGY, 4, None);
    // The Amulet's ability exiles the Shock.
    let shock = t.library_top(P0, "Shock");
    let i = index_of(&mut t, welder, AMULETS);
    t.activate(P0, welder, i, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(shock)), Zone::Exile, "{}", t.dump_log());
    assert!(may_cast(&t, P0, shock));
    // The Vault's second ability ("Activate only if there are three or more brick counters on
    // this artifact": on Myr Welder) exiles the Bolt.
    t.g.add_counters(Entity::Object(welder), "brick", 3, None);
    t.g.untap(welder);
    let bolt = t.library_top(P0, "Lightning Bolt");
    let i = index_of(&mut t, welder, "without paying its mana cost");
    t.activate(P0, welder, i, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(bolt)), Zone::Exile, "{}", t.dump_log());
    assert!(may_cast(&t, P0, bolt));
    assert!(may_cast(&t, P0, shock), "{}", t.dump_log());
    // The Amulet's ability exiles another card: the Shock's permission ends.
    let memnite = t.library_top(P0, "Memnite");
    t.g.untap(welder);
    let i = index_of(&mut t, welder, AMULETS);
    t.activate(P0, welder, i, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(memnite)), Zone::Exile);
    assert!(!may_cast(&t, P0, shock));
    assert!(may_cast(&t, P0, memnite));
}

/// Regression test for the engine bug found triaging `permissions.rs:969:45` and `975:64`; it
/// failed before the fix (commit `09bc166`: in `permissions::given`, an earlier "until you
/// exile another card with ~" permission ends only when the new one is from the same object
/// *for the same player*; before, any player's new permission from that object ended it).
/// "You" in the duration is the player who activated the ability (CR 109.5), so P0 may play
/// the Shock until P0 exiles another card with the Amulet: P1, who took the Amulet with Steal
/// Artifact, exiling a card of their own with it doesn't end that. (`975:64` fails this test
/// too, as it does the first one.)
#[test]
fn another_player_exiling_a_card_with_your_unstable_amulet_doesnt_end_your_permission() {
    cr!("109.5", "611.2a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let amulet = t.battlefield(P0, "Unstable Amulet");
    t.g.add_counters(Entity::Player(P0), counters::ENERGY, 2, None);
    let shock = t.library_top(P0, "Shock");
    t.activate(P0, amulet, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(shock)), Zone::Exile);
    // P1 takes the Amulet and exiles a card of their own with it.
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Island", 4);
    let steal = t.hand(P1, "Steal Artifact");
    t.cast(P1, steal).target(Entity::Object(amulet)).go();
    t.resolve_all();
    t.g.recompute();
    assert_eq!(t.g.obj(amulet).controller, P1);
    t.g.untap(amulet);
    t.g.add_counters(Entity::Player(P1), counters::ENERGY, 2, None);
    let bolt = t.library_top(P1, "Lightning Bolt");
    t.activate(P1, amulet, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(bolt)), Zone::Exile);
    assert!(may_cast(&t, P1, bolt));
    // P0 hasn't exiled another card with it: P0 may still cast the Shock.
    assert!(may_cast(&t, P0, shock), "{}", t.dump_log());
    let shock = t.g.current(shock);
    t.cast(P0, shock).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}
