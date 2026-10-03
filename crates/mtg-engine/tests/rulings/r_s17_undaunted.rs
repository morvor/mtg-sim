//! Rulings batch S17 — undaunted (CR 702.125): the total cost is locked in once it's
//! determined.

use crate::r_s01_common::*;
use crate::r_s04_common::untapped_lands;
use mtg_engine::ability::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;
use std::sync::Arc;

/// Replaces "each opponent" in every "loses the game" instruction of `e` with `who`.
fn retarget_loss(e: &mut Effect, who: PlayerId) {
    match e {
        Effect::Seq(v) => v.iter_mut().for_each(|x| retarget_loss(x, who)),
        Effect::LoseGame { who: w } => *w = PlayerRef::Player(who),
        _ => {}
    }
}

/// A land with "{T}: Add {C}. [loser] loses the game." — a mana ability (CR 605.1a) whose
/// activation, while a spell's costs are being paid, makes an opponent lose.
fn doom_spring(loser: PlayerId) -> mtg_engine::card::CardDef {
    let mut def = custom_card(
        "Doom Spring",
        "Land",
        "",
        None,
        "{T}: Add {C}. Each opponent loses the game.",
    );
    let abilities = &mut def.faces[0].chars.abilities;
    for a in abilities.iter_mut() {
        let mut d = (**a).clone();
        if let AbilityKind::Activated(act) = &mut d.kind {
            assert!(act.is_mana_ability);
            retarget_loss(&mut act.body.effect, loser);
        }
        *a = Arc::new(d);
    }
    def
}

#[test]
fn an_opponent_losing_after_the_total_cost_is_determined_doesnt_raise_it() {
    cr!("601.2f", "601.2g", "605.1a", "702.125a", "800.4a");
    ruling!(
        "Sublime Exhalation",
        "Causing an opponent to lose the game after you've announced that you're casting a spell with undaunted and determined its total cost won't cause you to have to pay more mana."
    );
    supported("Sublime Exhalation");
    // Four players: Sublime Exhalation ({6}{W}, undaunted) costs {3}{W}. P0 has exactly
    // four mana sources, one of which makes P3 lose the game as P0 pays.
    let mut t = TestGame::new(4);
    t.lands(P0, "Plains", 3);
    let spring = t.custom(P0, doom_spring(P3), Zone::Battlefield);
    let se = t.hand(P0, "Sublime Exhalation");
    let spell = t.cast(P0, se).go();
    // P3 lost during the payment; the cost stayed {3}{W}.
    assert!(!t.g.player(P3).in_game());
    assert!(t.obj(spring).tapped);
    assert_eq!(untapped_lands(&t, P0), 0);
    assert_eq!(t.zone(spell), Zone::Stack);
    // With P3 gone, casting another costs {4}{W}: the same four sources aren't enough.
    let mut t2 = TestGame::new(4);
    t2.lands(P0, "Plains", 4);
    t2.g.perform_action(P3, mtg_engine::decision::Action::Concede)
        .unwrap();
    t2.settle();
    let se2 = t2.hand(P0, "Sublime Exhalation");
    assert!(t2.cast(P0, se2).try_go().is_err());
}
