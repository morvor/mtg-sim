//! CR 406.3, 406.3b: a player instructed to look at cards and then exile them face down
//! may go on looking at them while they remain exiled — also when the looking and the
//! exiling are separate instructions ("Look at the top two cards of target opponent's
//! library and exile those cards face down."). Found by the hidden information property
//! (`mtg_api::hidden`, docs/PROPERTIES.md): a player was offered to cast a face-down card
//! they weren't allowed to look at, so the offer depended on what the card was.

use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn cards_looked_at_then_exiled_face_down_may_still_be_looked_at() {
    cr!("406.3", "406.3b");
    let mut t = TestGame::new(2);
    let bears = t.library_top(P1, "Grizzly Bears");
    let shock = t.library_top(P1, "Shock");
    let larceny = t.hand(P0, "Petty Larceny");
    for ty in mtg_engine::mana::ManaType::ALL {
        t.g.players[0].mana_pool.add_type(ty, 8);
    }
    t.cast(P0, larceny).target(Entity::Player(P1)).go();
    t.resolve_all();
    for c in [bears, shock] {
        let now = t.g.current(c);
        assert_eq!(t.zone(now), Zone::Exile);
        assert!(t.obj(now).face_down);
        assert!(
            zones::may_look(&t.g, P0, now),
            "P0 looked at it before exiling it"
        );
        assert!(!zones::may_look(&t.g, P1, now));
    }
}
