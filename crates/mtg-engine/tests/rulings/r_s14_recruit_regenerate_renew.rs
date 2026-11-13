//! Rulings batch S14 — recruit (CR 701.70), regenerate (CR 701.19) and renew: no player
//! can act while a spell or ability resolves (CR 608.2), a cost is paid as the ability is
//! activated (CR 602.2), and the active player receives priority first after a spell or
//! ability resolves (CR 117.3b).

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s05_common::{enter, tokens_with_subtype};
use crate::r_s06_common::{attach_new, damage};
use crate::r_s14_common::*;
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn nobody_can_act_while_recruit_draws_discards_and_creates_the_token() {
    cr!("701.70a", "608.2", "117.3b");
    ruling!(
        "Esgaroth Garrison",
        "Once a spell or ability that causes you to recruit begins to resolve, no player may take any other actions until it's done. Any responses made to the spell or ability must be made before you draw, discard, and potentially create a token."
    );
    supported("Esgaroth Garrison");
    // Esgaroth Garrison: "When this creature enters, recruit." P0 holds Lightning Bolt and
    // draws a Forest; P0 discards the Bolt and creates a Soldier. Every priority decision
    // (of either player) sees the recruit either not started or finished: never the card
    // drawn but not yet discarded, nor the card discarded but no token yet.
    let mut t = TestGame::new(2);
    t.hand(P0, "Lightning Bolt");
    t.library_top(P0, "Forest");
    let is_priority = |d: &Decision| matches!(d, Decision::Priority { .. });
    let state = |g: &mtg_engine::game::Game| {
        let soldiers = g
            .permanents()
            .filter(|o| o.is_token() && o.chars.has_subtype("Soldier"))
            .count();
        (
            g.player(P0).hand.len(),
            g.player(P0).graveyard.len(),
            soldiers,
        )
    };
    let seen0 = watch(&mut t, P0, is_priority, state);
    let seen1 = watch(&mut t, P1, is_priority, state);
    let bolt_first = |g: &mtg_engine::game::Game, d: &Decision| match d {
        Decision::ChooseEntities { candidates, .. } => candidates
            .iter()
            .find(|e| {
                e.object()
                    .is_some_and(|o| g.obj(o).chars.name == "Lightning Bolt")
            })
            .map(|e| mtg_engine::decision::Answer::Entities(vec![*e])),
        _ => None,
    };
    crate::r_s03_common::respond(&mut t, P0, bolt_first);
    enter(&mut t, P0, "Esgaroth Garrison");
    assert_eq!(t.stack_len(), 1);
    // Both players pass; the trigger resolves; then both players pass in the main phase.
    t.advance_to(P0, mtg_engine::turn::Step::BeginningOfCombat);
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    assert!(t.in_hand(P0, "Forest"));
    assert_eq!(tokens_with_subtype(&t, P0, "Soldier").len(), 1);
    let all: Vec<(usize, usize, usize)> = seen0
        .lock()
        .unwrap()
        .iter()
        .chain(seen1.lock().unwrap().iter())
        .copied()
        .collect();
    assert!(!all.is_empty());
    for s in &all {
        assert!(
            *s == (1, 0, 0) || *s == (1, 1, 1),
            "a player had priority in the middle of recruiting: {s:?}"
        );
    }
}

/// Grizzly Bears enchanted by Carapace (+0/+2), with `marked` damage; P0 sacrifices
/// Carapace to regenerate the Bears. Returns the game and the Bears.
fn carapace_bears(marked: i32) -> (TestGame, ObjectId) {
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let carapace = attach_new(&mut t, P0, "Carapace", bears);
    assert_eq!(t.pt(bears), (2, 4));
    let bolt = t.battlefield(P1, "Mogg Fanatic");
    damage(&mut t, bolt, marked, bears);
    assert!(t.on_battlefield(bears));
    t.activate(P0, carapace, 0, &[])
        .expect("sacrifice Carapace");
    assert!(!t.on_battlefield(carapace));
    (t, bears)
}

#[test]
fn sacrificing_carapace_removes_its_bonus_before_the_regeneration_shield() {
    cr!("701.19a", "602.2", "704.5g");
    ruling!(
        "Carapace",
        "This leaves the battlefield when you activate its activated ability, but the enchanted creature won't be regenerated until the ability resolves. In the intervening time, the creature will no longer have the bonus that this had been giving it. This may cause the creature to be destroyed before the regeneration shield is created."
    );
    supported("Carapace");
    // 3 damage on the 2/4 Bears: once Carapace is sacrificed, it's a 2/2 with lethal
    // damage, and it's destroyed before the ability resolves.
    let (mut t, bears) = carapace_bears(3);
    assert_eq!(t.pt(bears), (2, 2));
    t.settle();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    // 1 damage: it survives, and the ability gives the Bears a regeneration shield.
    let (mut t, bears) = carapace_bears(1);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    destroy(&mut t, bears);
    assert!(t.on_battlefield(bears), "regenerated");
    assert!(t.obj(bears).tapped);
    assert_eq!(t.obj(bears).damage, 0);
}

/// P0's answer to a priority decision while Champion of Dusan is in P0's graveyard and
/// the stack is empty: activate its renew ability.
fn renew_champion(g: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
    let Decision::Priority { actions } = d else {
        return None;
    };
    actions
        .iter()
        .find(|a| {
            matches!(a, Action::Activate { source, .. }
                if g.obj(*source).chars.name == "Champion of Dusan"
                    && g.obj(*source).zone == Zone::Graveyard(P0))
        })
        .map(|a| Answer::Action(a.clone()))
}

#[test]
fn a_renew_card_put_into_the_graveyard_on_your_turn_can_be_activated_first() {
    cr!("117.3b", "602.5d", "117.1b");
    ruling!(
        "Champion of Dusan",
        "If a card with a renew ability is put into your graveyard during your turn, you can activate that ability if it’s legal to do so before any other player can take any actions."
    );
    supported("Champion of Dusan");
    // Champion of Dusan: "Renew — {1}{G}, Exile this card from your graveyard: Put a +1/+1
    // counter and a trample counter on target creature. Activate only as a sorcery." P0's
    // Champion dies to P0's own Lightning Bolt in P0's main phase. As the Bolt resolves,
    // P0 receives priority first and activates renew: P1 never has priority while the
    // card is in P0's graveyard.
    let mut t = TestGame::new(2);
    let champion = t.battlefield(P0, "Champion of Dusan");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    let in_graveyard = |g: &mtg_engine::game::Game| {
        g.player(P0)
            .graveyard
            .iter()
            .any(|c| g.obj(*c).chars.name == "Champion of Dusan")
    };
    let is_priority = |d: &Decision| matches!(d, Decision::Priority { .. });
    let p1_saw = watch(&mut t, P1, is_priority, in_graveyard);
    crate::r_s03_common::respond(&mut t, P0, renew_champion);
    cast_from_hand(&mut t, P0, "Lightning Bolt", &[Entity::Object(champion)]);
    // Renew's target.
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.advance_to(P0, mtg_engine::turn::Step::BeginningOfCombat);
    assert!(t.in_exile("Champion of Dusan"), "renew was activated");
    assert_eq!(t.pt(bears), (3, 3));
    assert!(t
        .obj(bears)
        .has_keyword(mtg_engine::keywords::KeywordKind::Trample));
    let seen = p1_saw.lock().unwrap().clone();
    assert!(!seen.is_empty());
    assert!(
        seen.iter().all(|x| !x),
        "P1 had priority while the renew card was in P0's graveyard"
    );
}
