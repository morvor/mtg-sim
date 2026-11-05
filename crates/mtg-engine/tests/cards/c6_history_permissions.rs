//! Turn history and permissions VII (cards6-history-permissions): permissions over the top
//! card of another player's library (Xanathar, Guild Kingpin), block history anchored on
//! a target (Glyph of Reincarnation, Glyph of Delusion), and damage dealt by one of the
//! sorcery spells a chosen player cast this turn (Backdraft).

use crate::combat2_common::supported;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn sees(t: &TestGame, p: PlayerId, id: ObjectId) -> bool {
    mtg_engine::zones::can_see_in_library(&t.g, p, t.g.current(id))
}

#[test]
fn xanathar_plays_the_top_card_of_the_target_opponents_library() {
    cr!("401.5", "601.3", "305.2", "609.4b", "611.2a");
    ruling!(
        "Xanathar, Guild Kingpin",
        "You'll still pay all costs for spells cast this way"
    );
    ruling!(
        "Xanathar, Guild Kingpin",
        "you can look at the top card of the chosen opponent's library whenever you want"
    );
    ruling!(
        "Xanathar, Guild Kingpin",
        "You must still follow all normal timing rules and land play limits."
    );
    supported("Xanathar, Guild Kingpin");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Xanathar, Guild Kingpin");
    let mountain = t.library_top(P1, "Mountain");
    let forest = t.library_top(P1, "Forest");
    let axe = t.library_top(P1, "Lava Axe");
    let bolt = t.library_top(P1, "Lightning Bolt");
    // (P1 draws this one during their turn before P0's upkeep.)
    t.library_top(P1, "Island");
    let shock = t.hand(P1, "Shock");
    assert!(!sees(&t, P0, bolt));
    t.advance_to(P0, Step::Upkeep);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    // P0 may look at the top card of P1's library; P1 can't cast spells.
    assert!(sees(&t, P0, bolt));
    t.g.players[1].mana_pool.add_type(ManaType::R, 1);
    assert!(t.cast_with(P1, shock, &[Entity::Player(P0)]).is_err());
    t.g.players[1].mana_pool = Default::default();
    // An instant at instant speed, paid with black mana spent as though it were red.
    t.g.players[0].mana_pool.add_type(ManaType::B, 1);
    t.cast_with(P0, bolt, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.g.players[0].mana_pool.total(), 0, "the cost was paid");
    assert!(t.in_graveyard(P1, "Lightning Bolt"), "its owner's graveyard");
    // The new top card: a sorcery, cast only at sorcery speed.
    assert!(sees(&t, P0, axe));
    t.g.players[0].mana_pool.add_type(ManaType::B, 5);
    assert!(t.cast_with(P0, axe, &[Entity::Player(P1)]).is_err());
    t.advance_to(P0, Step::PrecombatMain);
    t.g.players[0].mana_pool.add_type(ManaType::B, 5);
    t.cast_with(P0, axe, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 12);
    // Lands: the land play is P0's, and only one.
    t.play_land(P0, forest).unwrap();
    let f = t.g.current(forest);
    assert!(t.on_battlefield(f));
    assert_eq!(t.obj_now(f).controller, P0);
    assert_eq!(t.obj_now(f).owner, P1);
    assert!(sees(&t, P0, mountain));
    assert!(t.play_land(P0, mountain).is_err(), "one land a turn");
    // Until end of turn only.
    t.advance_to(P1, Step::Upkeep);
    assert!(!sees(&t, P0, mountain));
}
