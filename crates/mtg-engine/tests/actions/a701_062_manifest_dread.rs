//! CR 701.62: manifest dread.

use crate::a701_028_071_common::*;
use mtg_engine::ability::*;
use mtg_engine::events::MoveCause;
use mtg_engine::kwa::manifest::{MANIFESTED, MANIFESTED_DREAD};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn manifest_dread_manifests_one_of_the_top_two_and_mills_the_other() {
    cr!("701.62a");
    ruling!(
        "Paranormal Analyst",
        "To manifest dread, look at the top two cards of your library. Manifest one (by putting it onto the battlefield face down) and put the other into your graveyard."
    );
    supported("Innocuous Rat");
    // "When this creature dies, manifest dread."
    let mut t = TestGame::new(2);
    let bears = t.library_top(P0, "Grizzly Bears");
    let giant = t.library_top(P0, "Hill Giant");
    let rat = t.battlefield(P0, "Innocuous Rat");
    t.g.move_object(rat, Zone::Graveyard(P0), MoveCause::Destroy, None);
    // Manifest the second card, not the top one.
    choose(&mut t, P0, &[bears]);
    t.resolve_all();
    let m = t.g.current(bears);
    assert!(t.obj(m).face_down && t.on_battlefield(m));
    assert_eq!(t.obj(m).choices.text.as_deref(), Some(MANIFESTED));
    assert_eq!(t.zone(giant), Zone::Graveyard(P0));
    // Only the two top cards were looked at.
    let looked: Vec<Vec<Entity>> = t
        .asked()
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates),
            _ => None,
        })
        .collect();
    assert_eq!(
        looked,
        vec![vec![Entity::Object(giant), Entity::Object(bears)]]
    );
}

#[test]
fn with_one_card_it_is_manifested_and_nothing_goes_to_the_graveyard() {
    cr!("701.62a");
    ruling!(
        "Paranormal Analyst",
        "If your library contains only one card when you manifest dread, you'll look at that card and put it onto the battlefield face down."
    );
    let mut t = TestGame::new(2);
    t.g.players[0].library.clear();
    let only = t.library_top(P0, "Hill Giant");
    run(&mut t, P0, None, ka(KeywordAction::ManifestDread, Sel::None, 1), &[]);
    assert!(t.obj_now(only).face_down);
    assert_eq!(t.graveyard_size(P0), 0);
}

#[test]
fn whenever_you_manifest_dread_triggers_even_if_nothing_could_be_done() {
    cr!("701.62b");
    ruling!(
        "Paranormal Analyst",
        "In circumstances where you are instructed to manifest dread but can't perform some or all of the steps of manifesting dread (probably because your library has one or fewer cards in it), these abilities will still trigger."
    );
    let mut t = TestGame::new(2);
    let watcher = text_card(
        "Dread Watcher",
        "Enchantment",
        "{0}",
        None,
        "Whenever you manifest dread, you gain 2 life.",
    );
    t.custom(P0, watcher, Zone::Battlefield);
    t.g.players[0].library.clear();
    run(&mut t, P0, None, ka(KeywordAction::ManifestDread, Sel::None, 1), &[]);
    t.resolve_all();
    assert_eq!(custom_events(&t, MANIFESTED_DREAD).len(), 1);
    assert_eq!(t.life(P0), 22);
    assert!(t.g.battlefield.iter().all(|o| !t.obj(*o).face_down));
}
