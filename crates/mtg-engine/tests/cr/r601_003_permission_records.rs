//! What playing a card with an effect's permission records (CR 400.7h, 601.3;
//! `permissions::note_way`, `permissions::record_use`). The permissions one effect gives
//! share a way of their own, which the delayed triggered ability the same effect creates
//! ("When you play a card this way") looks for: a card played with another activation's
//! permission doesn't trigger it, even one from the same source. And a card played with one
//! of the permissions an effect gives with a limit ("You may play up to two of those cards")
//! counts toward that limit: once that many are played, the other cards' permissions end.
//! Written for survivors of the November 2026 mutation run of `permissions.rs`
//! (docs/MUTATION.md).

use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Kills `permissions.rs:917:5` (`note_way` replaced with `()`) and `917:15` (`before >=
/// g.play_grants.len()` with `<`: it returns before noting the way whenever the effect gave a
/// permission). Without it each activation's permission keeps way 0 and its delayed ability
/// carries no way, so `permissions::played_this_way` takes the branch for a static ability's
/// permission and matches any card played with a way-0 permission from Fires: the Ornithopter
/// played with the first activation's permission triggers both activations' abilities (16/16
/// instead of 18/18), and the Memnite then triggers none.
#[test]
fn each_fires_of_mount_doom_activation_deals_damage_only_for_its_own_card() {
    cr!("400.7h", "603.7a");
    ruling!(
        "Fires of Mount Doom",
        "If you activate Fires of Mount Doom's last ability but don't play the exiled card this turn using that casting permission"
    );
    let mut t = TestGame::new(2);
    let fires = t.battlefield(P0, "Fires of Mount Doom");
    t.lands(P0, "Mountain", 6);
    let memnite = t.library_top(P0, "Memnite");
    let thopter = t.library_top(P0, "Ornithopter");
    // The first activation exiles the Ornithopter, the second the Memnite.
    t.activate(P0, fires, 0, &[]).unwrap();
    t.resolve_all();
    t.activate(P0, fires, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(thopter), Zone::Exile);
    assert_eq!(t.zone(memnite), Zone::Exile);
    // The Ornithopter is cast with the first activation's permission: only that
    // activation's delayed ability triggers.
    let thopter = t.g.current(thopter);
    t.cast(P0, thopter).go();
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (18, 18));
    // The second activation's was still waiting for its own card.
    let memnite = t.g.current(memnite);
    t.cast(P0, memnite).go();
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (16, 16));
}

/// Kills `permissions.rs:811:72` (in `permissions::record_use`, `x.terms.group == group` with
/// `!=`: when a card is played with one of the permissions an effect gave with a limit of
/// two or more, the count left is written onto every other permission instead of that
/// effect's, whose count never goes down). March of Reckless Joy exiles three cards and lets
/// P0 play up to two of them, chosen as they're played: once the Ornithopter is cast and the
/// Mountain played with its permissions, the Memnite's permission has ended. With the mutant
/// the limit stays two, and the Memnite (or any number of the cards) can still be cast.
#[test]
fn march_of_reckless_joy_plays_only_two_of_the_exiled_cards() {
    cr!("601.3");
    ruling!(
        "March of Reckless Joy",
        "You choose which cards that you are playing from exile as you play them, not as March of Reckless Joy is resolving."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    let memnite = t.library_top(P0, "Memnite");
    let mountain = t.library_top(P0, "Mountain");
    let thopter = t.library_top(P0, "Ornithopter");
    let march = t.hand(P0, "March of Reckless Joy");
    // X is 3: {3}{R}, with no red card exiled from the hand.
    t.cast(P0, march).x(3).go();
    t.resolve_all();
    let (thopter, mountain, memnite) = (
        t.g.current(thopter),
        t.g.current(mountain),
        t.g.current(memnite),
    );
    for c in [thopter, mountain, memnite] {
        assert_eq!(t.zone(c), Zone::Exile, "{}", t.dump_log());
    }
    // Each of the three may still be played.
    t.g.turn.priority = Some(P0);
    let actions = t.g.legal_actions(P0);
    assert!(
        actions.contains(&Action::PlayLand { card: mountain }),
        "{actions:?}"
    );
    for c in [thopter, memnite] {
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, Action::Cast { card, .. } if *card == c)),
            "{actions:?}"
        );
    }
    // Two of them: the Ornithopter, then the Mountain as P0's land play.
    t.cast(P0, thopter).go();
    t.resolve_all();
    t.play_land(P0, mountain).unwrap();
    assert!(t.on_battlefield(thopter) && t.on_battlefield(mountain));
    // The Memnite's permission has ended with them.
    assert!(t.g.cast_options(P0, memnite).is_empty());
    assert!(t.cast(P0, memnite).try_go().is_err());
    assert_eq!(t.zone(memnite), Zone::Exile);
}
