//! Rulings batch P221 — rebound (CR 702.88) on Ephemerate, Trumpeting Herd and the
//! creature Jeskai Baller: a permanent spell with rebound cast from its owner's hand is
//! exiled as it resolves instead of entering the battlefield; cast again from exile, it
//! enters as a creature.

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s04_common::next_upkeep;
use crate::r_s14_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Goes to P0's next upkeep and returns the number of triggered abilities put on the
/// stack (the rebound triggers).
fn upkeep_triggers(t: &mut TestGame) -> usize {
    next_upkeep(t, P0);
    triggers_on_stack_now(t)
}

fn athletes(t: &TestGame) -> usize {
    t.named_on_battlefield("Athlete Token").len()
}

#[test]
fn jeskai_baller_cast_from_hand_is_exiled_instead_of_entering() {
    cr!("702.88a", "608.3", "603.2");
    ruling!(
        "Jeskai Baller",
        "If you cast Jeskai Baller from your hand and the spell resolves, you must exile the spell due to its rebound ability. You cannot choose to have it enter the battlefield as a creature."
    );
    ruling!(
        "Jeskai Baller",
        "If you cast Jeskai Baller from any zone other than your hand (including from exile using its rebound ability) and the spell resolves, Jeskai Baller enters the battlefield as a creature."
    );
    supported("Jeskai Baller");
    let mut t = TestGame::new(2);
    let spell = cast_from_hand(&mut t, P0, "Jeskai Baller", &[]);
    t.resolve_all();
    // The cast trigger made an Athlete; the spell was exiled, not put onto the battlefield.
    assert_eq!(athletes(&t), 1);
    assert_eq!(t.zone(spell), Zone::Exile);
    assert!(t.named_on_battlefield("Jeskai Baller").is_empty());
    // At P0's next upkeep it's cast from exile (another Athlete) and enters as a creature.
    assert_eq!(upkeep_triggers(&mut t), 1);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Jeskai Baller").len(), 1);
    assert_eq!(athletes(&t), 2);
    assert_eq!(upkeep_triggers(&mut t), 0);
}

/// P0 casts `name` from hand (targeting a Grizzly Bears if `target`); P1 counters it with
/// Cancel. The card ends up in P0's graveyard and doesn't rebound.
fn countered(name: &str, target: bool) {
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let targets = if target { vec![Entity::Object(bears)] } else { vec![] };
    let spell = cast_from_hand(&mut t, P0, name, &targets);
    cast_from_hand(&mut t, P1, "Cancel", &[Entity::Object(spell)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, name), "{name}");
    assert_eq!(upkeep_triggers(&mut t), 0, "{name}");
    assert!(t.in_graveyard(P0, name), "{name}");
}

#[test]
fn a_countered_rebound_spell_goes_to_the_graveyard() {
    cr!("702.88a", "701.6a");
    ruling!(
        "Jeskai Baller",
        "If a spell with rebound that you cast from your hand doesn’t resolve for any reason, including being countered, the spell will be put into its owner’s graveyard and you won’t get to cast it again on your next turn."
    );
    ruling!(
        "Trumpeting Herd",
        "If a spell with rebound that you cast from your hand is countered for any reason, that spell won’t resolve and none of its effects will happen, including rebound. The spell will be put into its owner’s graveyard and you won’t get to cast it again on your next turn."
    );
    ruling!(
        "Ephemerate",
        "If a spell with rebound that you cast from your hand is countered or doesn't resolve (most likely because its targets have become illegal), none of its effects will happen, including rebound. The spell will be put into its owner's graveyard and you won't get to cast it again on your next turn."
    );
    countered("Jeskai Baller", false);
    countered("Trumpeting Herd", false);
    countered("Ephemerate", true);
    // Ephemerate whose target became illegal (the Bears died in response).
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_from_hand(&mut t, P0, "Ephemerate", &[Entity::Object(bears)]);
    destroy(&mut t, bears);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Ephemerate"));
    assert_eq!(upkeep_triggers(&mut t), 0);
}

/// P0 casts `name` from hand (targeting a Grizzly Bears if `target`) and it's exiled; at
/// the next upkeep P0 declines to cast it (`decline`), or can't: P1's Drannith Magistrate
/// prohibits it (`prohibited`) or Ephemerate has no legal target (`no target`). The card
/// stays exiled for good. Returns the number of P0's creatures at the end.
fn not_cast_again(name: &str, how: &str, target: bool) -> usize {
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let targets = if target { vec![Entity::Object(bears)] } else { vec![] };
    let spell = cast_from_hand(&mut t, P0, name, &targets);
    t.resolve_all();
    assert_eq!(t.zone(spell), Zone::Exile, "{name}");
    match how {
        "decline" => {
            t.answer_yes(P0, false);
        }
        "prohibited" => {
            t.battlefield(P1, "Drannith Magistrate");
        }
        "no target" => {
            let b = t.g.current(bears);
            destroy(&mut t, b);
        }
        _ => unreachable!(),
    }
    let n = creatures(&t, P0).len();
    assert_eq!(upkeep_triggers(&mut t), 1, "{name} {how}");
    t.resolve_all();
    assert_eq!(t.zone(spell), Zone::Exile, "{name} {how}");
    assert_eq!(creatures(&t, P0).len(), n, "{name} {how}: nothing new");
    assert_eq!(upkeep_triggers(&mut t), 0, "{name} {how}");
    assert_eq!(t.zone(spell), Zone::Exile, "{name} {how}");
    n
}

#[test]
fn casting_the_rebound_card_is_optional_and_it_stays_exiled_otherwise() {
    cr!("702.88a", "101.2");
    ruling!(
        "Jeskai Baller",
        "Casting the card again due to rebound’s delayed triggered ability is optional. If you choose not to cast the card, or if you can’t because an effect prohibits it, the card will stay exiled. You won’t get another chance to cast it on a future turn."
    );
    ruling!(
        "Trumpeting Herd",
        "Casting the card again due to rebound’s delayed triggered ability is optional. If you choose not to cast the card, or if you can’t because an effect prohibits it, the card will stay exiled. You won’t get another chance to cast it on a future turn. If you do cast the card, it’s put into its owner’s graveyard as normal once it resolves."
    );
    ruling!(
        "Ephemerate",
        "Casting the card again due to rebound's delayed triggered ability is optional. If you choose not to cast the card, or if you can't (perhaps because there are no legal targets available), the card will stay exiled. You won't get another chance to cast it on a future turn. If you do cast the card, it's put into its owner's graveyard as normal once it resolves."
    );
    supported("Drannith Magistrate");
    for how in ["decline", "prohibited"] {
        not_cast_again("Jeskai Baller", how, false);
        not_cast_again("Trumpeting Herd", how, false);
    }
    for how in ["decline", "no target"] {
        not_cast_again("Ephemerate", how, true);
    }
    // Cast from exile, Trumpeting Herd and Ephemerate go to the graveyard as they resolve.
    let mut t = TestGame::new(2);
    let spell = cast_from_hand(&mut t, P0, "Trumpeting Herd", &[]);
    t.resolve_all();
    assert_eq!(t.zone(spell), Zone::Exile);
    assert_eq!(upkeep_triggers(&mut t), 1);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Elephant Token").len(), 2);
    assert!(t.in_graveyard(P0, "Trumpeting Herd"));
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_from_hand(&mut t, P0, "Ephemerate", &[Entity::Object(bears)]);
    t.resolve_all();
    let bears = t.named_on_battlefield("Grizzly Bears")[0];
    t.answer_targets(P0, &[Entity::Object(bears)]);
    assert_eq!(upkeep_triggers(&mut t), 1);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Ephemerate"));
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}
