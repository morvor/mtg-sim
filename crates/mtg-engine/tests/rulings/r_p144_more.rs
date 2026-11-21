//! Rulings batch P144 — repeatable impulse draw: Pyre of the World Tree's discard
//! trigger (in the cleanup step too, CR 514.3a), Fires of Mount Doom's targeted trigger,
//! and Parapet Thrasher's "choose one that hasn't been chosen this turn".

use crate::r_p107_common::mana;
use crate::r_p117_common::gain_control;
use crate::r_p144_common::*;
use crate::r_s02_common::can_cast;
use crate::r_s06_common::attach_new;
use crate::r_s09_common::{declare, declare_in_next_combat};
use crate::r_s17_common::enter_transformed;
use crate::r_s29_common::choose_modes;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::*;

const INVASION: &str = "Invasion of Kaldheim // Pyre of the World Tree";

#[test]
fn pyre_triggers_on_any_land_discard_and_in_cleanup_gives_priority() {
    cr!("514.3a", "514.1", "603.2");
    supported(INVASION);
    ruling!(
        "Invasion of Kaldheim // Pyre of the World Tree",
        "The last ability of Pyre of the World Tree will trigger whenever you discard a land card for any reason, not just because you activated its other ability."
    );
    // Tormenting Voice's discard cost: a land card.
    let mut t = TestGame::new(2);
    let lib = on_top(
        &mut t,
        P0,
        &["Lightning Bolt", "Wastes", "Wastes", "Wastes"],
    );
    enter_transformed(&mut t, P0, INVASION);
    let island = t.hand(P0, "Island");
    let voice = t.hand(P0, "Tormenting Voice");
    mana(&mut t, P0, ManaType::R, 2);
    t.answer_choose(P0, &[obj(island)]);
    t.cast(P0, voice).go();
    t.resolve_all();
    assert_eq!(t.zone(island), Zone::Graveyard(P0));
    assert_eq!(t.zone(lib[0]), Zone::Exile);

    ruling!(
        "Invasion of Kaldheim // Pyre of the World Tree",
        "If that ability triggers during the cleanup step because you discarded a land card to bring your hand size down to your maximum hand size, you will get priority after the ability resolves, so you’ll have a chance to cast an instant card or a card with flash you exiled. Whether you cast the exiled card or not, there will be then be another cleanup step before the turn ends."
    );
    // Eight cards in hand at P0's cleanup step: the Island is discarded, the trigger
    // exiles a Lightning Bolt, and P0 gets priority in the cleanup step to cast it.
    let mut t = TestGame::new(2);
    let lib = on_top(
        &mut t,
        P0,
        &["Lightning Bolt", "Wastes", "Wastes", "Wastes"],
    );
    enter_transformed(&mut t, P0, INVASION);
    t.g.players[P0.idx()].hand.clear();
    let island = t.hand(P0, "Island");
    for _ in 0..7 {
        t.hand(P0, "Grizzly Bears");
    }
    t.answer_choose(P0, &[obj(island)]);
    let turn = t.g.turn.number;
    let ok = t.g.run_until(10_000, |g| {
        (g.turn.step == Step::Cleanup && g.turn.stage == Stage::Priority) || g.turn.number != turn
    });
    assert!(
        ok && t.g.turn.number == turn,
        "no priority in the cleanup step"
    );
    assert_eq!(t.zone(island), Zone::Graveyard(P0));
    t.resolve_all();
    assert_eq!(t.zone(lib[0]), Zone::Exile);
    mana(&mut t, P0, ManaType::R, 1);
    assert!(can_cast(&mut t, P0, lib[0], CastMethod::Normal));
    let bolt = t.g.current(lib[0]);
    t.cast(P0, bolt).target(P1).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // Then another cleanup step: a hand over the maximum (a card added now) is discarded
    // down again before the turn ends.
    t.hand(P0, "Grizzly Bears");
    assert_eq!(t.hand_size(P0), 8);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.hand_size(P0), 7);
    assert_eq!(t.g.turn.active, P1);
}

#[test]
fn fires_of_mount_doom_targets_any_opposing_creature_and_can_fizzle() {
    cr!("603.3d", "608.2b");
    supported("Fires of Mount Doom");
    ruling!(
        "Fires of Mount Doom",
        "Fires of Mount Doom's first ability can target a creature that has no Equipment attached to it."
    );
    let mut t = TestGame::new(2);
    let b = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(b)]);
    t.enter(P0, "Fires of Mount Doom");
    t.resolve_all();
    assert!(!t.on_battlefield(b));

    ruling!(
        "Fires of Mount Doom",
        "If the target of Fires of Mount Doom's first ability is illegal by the time the ability tries to resolve, it's removed from the stack. You won't destroy any Equipment."
    );
    // The Wurm carrying P1's Bonesplitter comes under P0's control before the trigger
    // resolves: it's no longer "a creature an opponent controls".
    let mut t = TestGame::new(2);
    let w = t.battlefield(P1, "Craw Wurm");
    let eq = attach_new(&mut t, P1, "Bonesplitter", obj(w));
    t.answer_targets(P0, &[obj(w)]);
    t.enter(P0, "Fires of Mount Doom");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    gain_control(&mut t, P0, w);
    t.resolve_all();
    assert!(t.on_battlefield(eq));
    assert_eq!(t.obj_now(w).damage, 0);
}

#[test]
fn parapet_thrasher_fourth_trigger_has_no_mode() {
    cr!("700.2", "603.3c", "506.1");
    supported("Parapet Thrasher");
    ruling!(
        "Parapet Thrasher",
        "If the triggered ability triggers more than three times in a turn, no mode can be chosen for any instances of the ability after the third, and those will be removed from the stack and do nothing."
    );
    // Double strike (Assault Strobe) and a second combat (Relentless Assault): four
    // triggers. The first three take the three modes; the fourth does nothing.
    let mut t = TestGame::new(2);
    let lib = on_top(&mut t, P0, &["Island", "Swamp", "Wastes"]);
    let th = t.battlefield(P0, "Parapet Thrasher");
    let orn = t.battlefield(P1, "Ornithopter");
    let strobe = t.hand(P0, "Assault Strobe");
    mana(&mut t, P0, ManaType::R, 1);
    t.cast(P0, strobe).target(th).go();
    t.resolve_all();
    choose_modes(&mut t, P0, &[2]);
    choose_modes(&mut t, P0, &[1]);
    declare(&mut t, P0, &[(th, Entity::Player(P1))]);
    t.resolve_all();
    t.advance_to(P0, Step::PostcombatMain);
    t.resolve_all();
    assert_eq!(t.life(P1), 12);
    assert_eq!(t.zone(lib[0]), Zone::Exile);
    let ra = t.hand(P0, "Relentless Assault");
    mana(&mut t, P0, ManaType::R, 4);
    t.cast(P0, ra).go();
    t.resolve_all();
    choose_modes(&mut t, P0, &[0]);
    t.answer_targets(P0, &[obj(orn)]);
    // The fourth: whatever is asked, nothing can be chosen.
    choose_modes(&mut t, P0, &[2]);
    declare_in_next_combat(&mut t, P0, &[(th, Entity::Player(P1))]);
    t.resolve_all();
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.life(P1), 4);
    assert_eq!(t.zone(orn), Zone::Graveyard(P1));
    // No second card exiled by the fourth trigger.
    assert_ne!(t.zone(lib[1]), Zone::Exile);
    assert_eq!(t.stack_len(), 0);
}
