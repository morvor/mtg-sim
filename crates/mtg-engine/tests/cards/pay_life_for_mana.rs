//! "Until end of turn, any time you could activate a mana ability, you may pay 1 life. If
//! you do, add {C}." (Channel): a repeatable action for the spell's controller, until end
//! of turn (CR 116.2c), that can be taken any time they could activate a mana ability —
//! with priority, while casting a spell, or when an attack cost or an effect asks for a
//! mana payment (CR 605.3a) — and not for more life than they have (CR 119.4).

use mtg_engine::decision::{Action, Answer, SpecialAction};
use mtg_engine::mana::ManaType;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn channel_action(t: &mut TestGame, p: PlayerId) -> Option<Action> {
    t.g.turn.priority = Some(p);
    t.g.recompute();
    t.g.legal_actions(p)
        .into_iter()
        .find(|a| matches!(a, Action::Special(SpecialAction::Offer { .. })))
}

/// P0 casts Channel with two Forests (which stay tapped), and it resolves.
fn cast_channel(t: &mut TestGame) {
    let forests = t.lands(P0, "Forest", 2);
    let channel = t.hand(P0, "Channel");
    t.cast(P0, channel).go();
    t.resolve_all();
    assert!(forests.iter().all(|f| t.obj(*f).tapped));
    assert!(t.g.player(P0).mana_pool.mana.is_empty());
}

/// Whether P0 may cast `card` now.
fn castable(t: &mut TestGame, card: ObjectId) -> bool {
    t.g.turn.priority = Some(P0);
    t.g.legal_actions(P0).iter().any(|a| {
        matches!(a, Action::Cast { card: c, method: CastMethod::Normal } if *c == card)
    })
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
    cast_channel(&mut t);
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
    t.advance_to(P1, Step::Upkeep);
    assert!(channel_action(&mut t, P0).is_none());
}

#[test]
fn channel_pays_life_while_a_spell_is_being_cast() {
    cr!("605.3a", "601.2g", "601.2h", "119.4");
    // With Channel's effect and no other mana, Mind Stone ({2}) can be cast: P0 pays 1
    // life for each {C} as the cost is paid.
    let mut t = TestGame::new(2);
    let stone = t.hand(P0, "Mind Stone");
    assert!(!castable(&mut t, stone));
    cast_channel(&mut t);
    assert!(castable(&mut t, stone));
    t.cast(P0, stone).go();
    assert_eq!(t.life(P0), 18);
    assert!(t.g.player(P0).mana_pool.mana.is_empty());
    t.resolve_all();
    assert!(t.on_battlefield(stone));
    // Mana abilities that don't cost life are used first: Grizzly Bears ({1}{G}) is paid
    // with a Forest and the Mind Stone.
    let forest = t.battlefield(P0, "Forest");
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    assert!(t.obj(forest).tapped && t.obj_now(stone).tapped);
    assert_eq!(t.life(P0), 18);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    // Not for more life than P0 has: at 2 life, Hill Giant ({3}{R}) with a Mountain
    // can't be cast (three {C} would cost 3 life).
    t.g.players[0].life = 2;
    let giant = t.hand(P0, "Hill Giant");
    t.battlefield(P0, "Mountain");
    assert!(!castable(&mut t, giant));
    t.g.players[0].life = 3;
    assert!(castable(&mut t, giant));
    t.cast(P0, giant).go();
    assert_eq!(t.life(P0), 0);
}

#[test]
fn channel_pays_life_for_mana_an_effect_asks_for() {
    cr!("605.3a");
    // P1 casts Mana Leak ("Counter target spell unless its controller pays {3}.") at
    // P0's Mind Stone: as it resolves, P0 pays {3} with 3 life, without priority.
    let mut t = TestGame::new(2);
    cast_channel(&mut t);
    let stone = t.hand(P0, "Mind Stone");
    t.lands(P0, "Wastes", 2);
    let spell = t.cast(P0, stone).go();
    assert_eq!(t.life(P0), 20);
    t.lands(P1, "Island", 2);
    let leak = t.hand(P1, "Mana Leak");
    t.g.turn.priority = Some(P1);
    t.cast(P1, leak).target(spell).go();
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(t.life(P0), 17);
    t.resolve_all();
    assert!(t.on_battlefield(stone));
}

#[test]
fn channel_pays_life_for_an_attack_cost() {
    cr!("605.3a", "508.1h", "508.1i");
    // P1's Propaganda: "Creatures can't attack you unless their controller pays {2} for
    // each creature they control that's attacking you." P0's mana pool empties as the
    // beginning of combat step ends; P0 pays the {2} with 2 life as attackers are
    // declared.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Propaganda");
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_channel(&mut t);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bears, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    assert!(t.g.is_attacking(bears));
    assert_eq!(t.life(P0), 18);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn channel_and_phyrexian_mana_share_the_life_total() {
    cr!("119.4", "107.4f", "605.3a");
    // Porcelain Legionnaire ({2}{W/P}) with no other mana: {2} for 2 life with Channel,
    // {W/P} for 2 more — 4 life, so not at 3.
    let mut t = TestGame::new(2);
    cast_channel(&mut t);
    let legionnaire = t.hand(P0, "Porcelain Legionnaire");
    t.g.players[0].life = 3;
    assert!(!castable(&mut t, legionnaire));
    t.g.players[0].life = 4;
    assert!(castable(&mut t, legionnaire));
    t.cast(P0, legionnaire).go();
    assert_eq!(t.life(P0), 0);
    assert!(t.g.player(P0).mana_pool.mana.is_empty());
}
