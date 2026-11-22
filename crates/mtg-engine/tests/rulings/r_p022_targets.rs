//! Rulings batch P022 — modal spells with a "create a token that's a copy of target ..."
//! mode whose targets become illegal: with at least one legal target the spell resolves
//! and does as much as it can; with none it doesn't resolve (CR 608.2b, 700.2).

use crate::r_p022_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Casts `p`'s `spell` (paying `cost`) with `modes` and `targets`; then `kill`s are
/// destroyed before it resolves. Returns the tokens `p` got.
fn cast_then_kill(
    t: &mut TestGame,
    spell: &str,
    cost: &str,
    modes: &[usize],
    targets: &[Entity],
    kill_these: &[ObjectId],
) -> Vec<ObjectId> {
    supported(spell);
    lands_for(t, P0, cost);
    let card = t.hand(P0, spell);
    let before = t.g.battlefield.clone();
    let mut b = t.cast(P0, card).modes(modes);
    for e in targets {
        b = b.target(*e);
    }
    b.go();
    for k in kill_these {
        kill(t, *k);
    }
    t.resolve_all();
    new_tokens(t, P0, &before)
}

#[test]
fn commands_with_an_illegal_copy_target_still_do_the_other_mode() {
    cr!("608.2b", "700.2");
    ruling!("Ashling's Command", "If all of Ashling's Command's targets are illegal as it tries to resolve, it will do nothing. If at least one target is still legal, it will resolve and do as much as it can.");
    ruling!("Brigid's Command", "If all of Brigid's Command's targets are illegal as it tries to resolve, it will do nothing. If at least one target is still legal, it will resolve and do as much as it can.");
    ruling!("Grub's Command", "If all of Grub's Command's targets are illegal as it tries to resolve, it will do nothing. If at least one target is still legal, it will resolve and do as much as it can.");
    ruling!("Sygg's Command", "If all of Sygg's Command's targets are illegal as it tries to resolve, it will do nothing. If at least one target is still legal, it will resolve and do as much as it can.");
    ruling!("Trystan's Command", "If all of Trystan's Command's targets are illegal as it tries to resolve, it will do nothing. If at least one target is still legal, it will resolve and do as much as it can.");
    // (Command, cost, creature of its type, other mode, check after the other mode)
    type Check = fn(&TestGame, usize, ObjectId) -> bool;
    let cases: [(&str, &str, &str, usize, Check); 5] = [
        // "Target player draws two cards."
        (
            "Ashling's Command",
            "{3}{U}{R}",
            "Air Elemental",
            1,
            |t, hand, _| t.hand_size(P0) == hand + 2,
        ),
        // "Target player creates a 1/1 green and white Kithkin creature token."
        (
            "Brigid's Command",
            "{1}{G}{W}",
            "Goldmeadow Harrier",
            1,
            |t, _, _| {
                t.g.battlefield
                    .iter()
                    .any(|o| t.obj(*o).is_token() && t.obj(*o).chars.has_subtype("Kithkin"))
            },
        ),
        // "Target player mills five cards, then puts each Goblin card milled this way
        // into their hand."
        (
            "Grub's Command",
            "{3}{B}{R}",
            "Raging Goblin",
            3,
            |t, _, _| t.graveyard_size(P0) >= 5,
        ),
        // "Target player draws a card."
        (
            "Sygg's Command",
            "{1}{W}{U}",
            "Merfolk of the Pearl Trident",
            2,
            |t, hand, _| t.hand_size(P0) == hand + 1,
        ),
        // "Destroy target creature or enchantment." (P1's Hill Giant)
        (
            "Trystan's Command",
            "{4}{B}{G}",
            "Llanowar Elves",
            2,
            |t, _, giant| !t.on_battlefield(giant),
        ),
    ];
    for (command, cost, creature, other, check) in cases {
        supported(creature);
        // The creature to copy leaves; the other mode still happens.
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, creature);
        let giant = t.battlefield(P1, "Hill Giant");
        let second = if command == "Trystan's Command" {
            obj(giant)
        } else {
            Entity::Player(P0)
        };
        let hand = t.hand_size(P0);
        let toks = cast_then_kill(&mut t, command, cost, &[0, other], &[obj(c), second], &[c]);
        assert!(
            toks.iter().all(|x| name_of(&t, *x) != creature),
            "{command}: no copy of the creature that left"
        );
        assert!(check(&t, hand, giant), "{command}: the other mode happened");
    }
    // Trystan's Command with both targets gone does nothing (the Elf, the Giant).
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P0, "Llanowar Elves");
    let giant = t.battlefield(P1, "Hill Giant");
    let toks = cast_then_kill(
        &mut t,
        "Trystan's Command",
        "{4}{B}{G}",
        &[0, 2],
        &[obj(elves), obj(giant)],
        &[elves, giant],
    );
    assert!(toks.is_empty());
    assert!(t.in_graveyard(P0, "Trystan's Command"));
}

#[test]
fn season_with_every_target_illegal_doesnt_resolve() {
    cr!("608.2b", "700.2");
    ruling!("Season of the Burrow", "If all targets for the chosen modes become illegal before the Season resolves, the spell won't resolve and none of its effects will happen. If at least one target is still legal, the spell will resolve but will have no effect on any illegal targets.");
    supported("Season of the Burrow");
    // {P} Rabbit token and {P}{P} "Exile target nonland permanent. Its controller draws a
    // card." on P1's Hill Giant, which leaves: no Rabbit either.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let toks = cast_then_kill(
        &mut t,
        "Season of the Burrow",
        "{3}{W}{W}",
        &[0, 1],
        &[obj(giant)],
        &[giant],
    );
    assert!(toks.is_empty());
    // {P}{P} on the Giant (which leaves) and {P}{P}{P} "Return target permanent card with
    // mana value 3 or less from your graveyard to the battlefield with an indestructible
    // counter on it" on Grizzly Bears: the Bears returns.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.graveyard(P0, "Grizzly Bears");
    let hand = t.hand_size(P1);
    cast_then_kill(
        &mut t,
        "Season of the Burrow",
        "{3}{W}{W}",
        &[1, 2],
        &[obj(giant), obj(bears)],
        &[giant],
    );
    let bears = t.g.current(bears);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.counters(bears, "indestructible"), 1);
    assert_eq!(t.hand_size(P1), hand, "the Giant's controller didn't draw");
}

#[test]
fn sublime_epiphany_affects_the_remaining_targets() {
    cr!("608.2b", "700.2");
    ruling!(
        "Sublime Epiphany",
        "If any of the targets become illegal, the remaining targets are affected as appropriate."
    );
    supported("Sublime Epiphany");
    // "Return target nonland permanent to its owner's hand", "Create a token that's a copy
    // of target creature you control" and "Target player draws a card": the creature to
    // copy leaves.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let hand = t.hand_size(P0);
    let toks = cast_then_kill(
        &mut t,
        "Sublime Epiphany",
        "{4}{U}{U}",
        &[2, 3, 4],
        &[obj(giant), obj(bears), Entity::Player(P0)],
        &[bears],
    );
    assert!(toks.is_empty());
    assert!(t.in_hand(P1, "Hill Giant"));
    assert_eq!(t.hand_size(P0), hand + 1);
}
