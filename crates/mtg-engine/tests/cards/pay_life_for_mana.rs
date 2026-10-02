//! "Until end of turn, any time you could activate a mana ability, you may pay 1 life. If
//! you do, add {C}." (Channel): a repeatable action for the spell's controller, until end
//! of turn (CR 116.2c, 605.3a, 119.4).

use mtg_engine::decision::{Action, SpecialAction};
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::*;

fn channel_action(t: &mut TestGame, p: PlayerId) -> Option<Action> {
    t.g.turn.priority = Some(p);
    t.g.recompute();
    t.g.legal_actions(p)
        .into_iter()
        .find(|a| matches!(a, Action::Special(SpecialAction::Offer { .. })))
}

#[test]
fn channel_pays_life_for_colorless_mana_until_end_of_turn() {
    cr!("116.2c", "119.4", "605.3a");
    let def = card("Channel");
    assert!(
        def.unsupported_text().is_empty(),
        "{:?}",
        def.unsupported_text()
    );
    let mut t = TestGame::new(2);
    assert!(channel_action(&mut t, P0).is_none());
    t.lands(P0, "Forest", 2);
    let channel = t.hand(P0, "Channel");
    t.cast(P0, channel).go();
    t.resolve_all();
    // Only Channel's controller may take the action, any number of times.
    assert!(channel_action(&mut t, P1).is_none());
    for n in 1..=3 {
        let pay = channel_action(&mut t, P0).expect("Channel's action");
        t.g.perform_action(P0, pay).unwrap();
        t.settle();
        assert_eq!(t.life(P0), 20 - n);
    }
    let pool: Vec<ManaType> = t.g.player(P0).mana_pool.mana.iter().map(|m| m.ty).collect();
    assert_eq!(pool, vec![ManaType::C; 3]);
    // The mana pays for a spell: Mind Stone ({2}).
    let stone = t.hand(P0, "Mind Stone");
    t.cast(P0, stone).go();
    t.resolve_all();
    assert!(t.on_battlefield(stone));
    assert_eq!(t.g.player(P0).mana_pool.mana.len(), 1);
    // It lasts until end of turn.
    t.advance_to(P1, mtg_engine::turn::Step::Upkeep);
    assert!(channel_action(&mut t, P0).is_none());
}
