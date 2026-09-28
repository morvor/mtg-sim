//! Rulings batch S22 — Nicol Bolas, God-Pharaoh's first ability ("+2: Target opponent
//! exiles cards from the top of their library until they exile a nonland card. Until end
//! of turn, you may cast that card without paying its mana cost."): a permission to cast
//! the exiled card this turn without paying its mana cost (CR 118.9), following the
//! timing rules, whether or not Nicol Bolas is still around.

use crate::r_s01_common::*;
use crate::r_s08_common::legal_cast_methods;
use crate::r_s22_common::*;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Activates Nicol Bolas's +2 targeting P1 and resolves it.
fn bolas_plus_two(t: &mut TestGame) {
    let bolas = t.named_on_battlefield("Nicol Bolas, God-Pharaoh")[0];
    t.activate(P0, bolas, 0, &[Entity::Player(P1)])
        .expect("activate +2");
    t.resolve_all();
}

/// Puts `name` on top of P1's library (under two lands, which are exiled first), with
/// Nicol Bolas on P0's battlefield.
fn bolas(t: &mut TestGame, name: &str) -> ObjectId {
    t.battlefield(P0, "Nicol Bolas, God-Pharaoh");
    stack_library(t, P1, &["Forest", "Island", name])[2]
}

fn run_bolas(t: &mut TestGame, card: ObjectId, answers: &dyn Fn(&mut TestGame)) {
    bolas_plus_two(t);
    answers(t);
    let _ = t
        .cast(P0, t.g.current(card))
        .method(CastMethod::Free)
        .try_go();
    t.resolve_all();
    t.clear_answers();
}

#[test]
fn nicol_bolas_casts_without_paying_but_additional_costs_are_paid() {
    cr!("118.9a", "118.9b", "118.8a", "601.2b", "601.2f");
    ruling!(
        "Nicol Bolas, God-Pharaoh",
        "If you cast a card \"without paying its mana cost,\" you can't choose to cast it for any alternative costs, such as emerge costs. You can, however, pay additional costs. If the card has any mandatory additional costs, such as that of Tormenting Voice, you must pay those to cast the card."
    );
    supported("Nicol Bolas, God-Pharaoh");
    check_free_cast_costs(&FreeCaster {
        instants_only: false,
        place: bolas,
        run: run_bolas,
    });
    // Cyclonic Rift can be cast only without paying its mana cost, not for its overload
    // cost (a permission to cast it without paying its mana cost doesn't allow another
    // alternative cost, CR 118.9a).
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Hill Giant");
    let rift = bolas(&mut t, "Cyclonic Rift");
    t.lands(P0, "Island", 7);
    bolas_plus_two(&mut t);
    let methods = legal_cast_methods(&mut t, P0, rift);
    assert_eq!(methods, vec![CastMethod::Free]);
}

#[test]
fn nicol_bolas_exiles_face_up_until_a_nonland_card() {
    cr!("406.3", "701.20a");
    ruling!(
        "Nicol Bolas, God-Pharaoh",
        "The cards exiled by Nicol Bolas's first and second abilities are exiled face up."
    );
    supported("Nicol Bolas, God-Pharaoh");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Nicol Bolas, God-Pharaoh");
    let cards = stack_library(&mut t, P1, &["Forest", "Island", "Grizzly Bears", "Swamp"]);
    bolas_plus_two(&mut t);
    for c in &cards[..3] {
        assert_eq!(t.zone(*c), Zone::Exile);
        assert!(!t.obj_now(*c).face_down);
    }
    // The Swamp under Grizzly Bears stays.
    assert_eq!(t.zone(cards[3]), Zone::Library(P1));
    // Only the nonland card may be cast.
    assert!(legal_cast_methods(&mut t, P0, cards[0]).is_empty());
    assert!(t.play_land(P0, t.g.current(cards[0])).is_err());
    t.lands(P0, "Wastes", 2);
    assert!(!legal_cast_methods(&mut t, P0, cards[2]).is_empty());
}

#[test]
fn nicol_bolas_the_card_is_cast_following_timing_rules() {
    cr!("307.1", "601.3");
    ruling!(
        "Nicol Bolas, God-Pharaoh",
        "Casting the card exiled with Nicol Bolas's first ability follows the normal timing rules for casting that card. For example, if the card is a creature card, you can cast that card only during your main phase while the stack is empty."
    );
    ruling!(
        "Nicol Bolas, God-Pharaoh",
        "You may cast the nonland card exiled by Nicol Bolas's first ability that turn even if Nicol Bolas is no longer on the battlefield or under your control."
    );
    supported("Nicol Bolas, God-Pharaoh");
    let mut t = TestGame::new(2);
    let bears = bolas(&mut t, "Grizzly Bears");
    bolas_plus_two(&mut t);
    // Nicol Bolas leaves the battlefield.
    let b = t.named_on_battlefield("Nicol Bolas, God-Pharaoh")[0];
    let _ = t.g.destroy(b, None);
    t.settle();
    assert!(t.named_on_battlefield("Nicol Bolas, God-Pharaoh").is_empty());
    // Not while a spell is on the stack.
    crate::r_s04_common::add_mana(&mut t, P0, mtg_engine::mana::ManaType::R, 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    assert!(legal_cast_methods(&mut t, P0, bears).is_empty());
    t.resolve_all();
    // Not in combat either.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(legal_cast_methods(&mut t, P0, bears).is_empty());
    // In the main phase with an empty stack: yes, without paying its mana cost.
    t.set_step(P0, Step::PostcombatMain);
    t.cast(P0, t.g.current(bears))
        .method(CastMethod::Free)
        .go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.obj_now(bears).controller, P0);
}

#[test]
fn nicol_bolas_an_uncast_card_stays_exiled() {
    cr!("611.2a", "514.2");
    ruling!(
        "Nicol Bolas, God-Pharaoh",
        "If you don't cast the card exiled by Nicol Bolas's first ability that turn, it will remain exiled."
    );
    supported("Nicol Bolas, God-Pharaoh");
    let mut t = TestGame::new(2);
    let bears = bolas(&mut t, "Grizzly Bears");
    bolas_plus_two(&mut t);
    t.advance_to(P1, Step::PrecombatMain);
    assert_eq!(t.zone(bears), Zone::Exile);
    // The permission has ended: P0 can't cast it on a later turn.
    t.advance_to(P0, Step::PrecombatMain);
    t.lands(P0, "Forest", 2);
    assert!(legal_cast_methods(&mut t, P0, bears).is_empty());
}
