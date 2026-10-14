//! Rulings batch P219 — "mill N cards, then return a [kind] card from your graveyard":
//! the card is chosen as the ability resolves, after milling, from all the cards in the
//! graveyard: one just milled or one that was already there (CR 608.2c, 701.17a).

use crate::r_s01_common::*;
use crate::r_s03_common::{in_hand_with_mana, respond};
use crate::r_s04_common::add_mana;
use crate::r_s05_common::enter;
use crate::r_s06_common::activate_containing;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::CardType;
use mtg_engine::*;
use std::cell::Cell;

thread_local! {
    /// The card name [`pick_wanted`] chooses.
    static WANTED: Cell<&'static str> = const { Cell::new("") };
}

/// Chooses one candidate named [`WANTED`] in an entity choice that offers one.
fn pick_wanted(g: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
    let Decision::ChooseEntities { candidates, .. } = d else {
        return None;
    };
    let want = WANTED.with(|w| w.get());
    let pick = candidates
        .iter()
        .find(|e| e.object().is_some_and(|o| g.obj(o).chars.name == want))?;
    Some(Answer::Entities(vec![*pick]))
}

/// Casts the real card `name` from P0's hand with the mana for it.
fn cast(t: &mut TestGame, name: &str) {
    let c = in_hand_with_mana(t, P0, name);
    t.cast(P0, c).go();
}

/// Where the wanted card should end up.
#[derive(Clone, Copy, Debug)]
enum Dest {
    Battlefield,
    Hand,
    LibraryTop,
    Exile,
}

struct Case {
    card: &'static str,
    /// The card to be returned, and whose graveyard (and library) it's in.
    wanted: &'static str,
    owner: PlayerId,
    run: fn(&mut TestGame),
    dest: Dest,
}

fn cases() -> Vec<Case> {
    use Dest::*;
    vec![
        Case {
            card: "Tyvar, Jubilant Brawler",
            wanted: "Grizzly Bears",
            owner: P0,
            run: |t| {
                let ty = t.battlefield(P0, "Tyvar, Jubilant Brawler");
                t.answer_yes(P0, true);
                activate_containing(t, P0, ty, "Mill three").expect("activated");
            },
            dest: Battlefield,
        },
        Case {
            card: "Harald Unites the Elves",
            wanted: "Llanowar Elves",
            owner: P0,
            run: |t| {
                t.answer_yes(P0, true);
                enter(t, P0, "Harald Unites the Elves");
            },
            dest: Battlefield,
        },
        Case {
            card: "Roots of Wisdom",
            wanted: "Forest",
            owner: P0,
            run: |t| cast(t, "Roots of Wisdom"),
            dest: Hand,
        },
        Case {
            card: "Technomancer",
            wanted: "Ornithopter",
            owner: P0,
            run: |t| {
                enter(t, P0, "Technomancer");
            },
            dest: Battlefield,
        },
        Case {
            card: "Halo-Charged Skaab",
            wanted: "Shock",
            owner: P0,
            run: |t| {
                t.answer_yes(P0, true);
                enter(t, P0, "Halo-Charged Skaab");
            },
            dest: LibraryTop,
        },
        Case {
            card: "Dogmeat, Ever Loyal",
            wanted: "Bonesplitter",
            owner: P0,
            run: |t| {
                enter(t, P0, "Dogmeat, Ever Loyal");
            },
            dest: Hand,
        },
        Case {
            card: "Breach the Multiverse",
            wanted: "Grizzly Bears",
            owner: P1,
            run: |t| cast(t, "Breach the Multiverse"),
            dest: Battlefield,
        },
        Case {
            card: "King Narfi's Betrayal",
            wanted: "Grizzly Bears",
            owner: P1,
            run: |t| {
                t.answer_yes(P0, true);
                enter(t, P0, "King Narfi's Betrayal");
            },
            dest: Exile,
        },
        Case {
            card: "Druidic Ritual",
            wanted: "Grizzly Bears",
            owner: P0,
            run: |t| {
                t.answer_yes(P0, true);
                cast(t, "Druidic Ritual");
            },
            dest: Hand,
        },
        Case {
            card: "Tomb Fortress",
            wanted: "Grizzly Bears",
            owner: P0,
            run: |t| {
                let fortress = t.battlefield(P0, "Tomb Fortress");
                add_mana(t, P0, ManaType::B, 3);
                add_mana(t, P0, ManaType::C, 2);
                activate_containing(t, P0, fortress, "Mill four").expect("activated");
            },
            dest: Battlefield,
        },
        Case {
            card: "Sauron, Lord of the Rings",
            wanted: "Hill Giant",
            owner: P0,
            run: |t| cast(t, "Sauron, Lord of the Rings"),
            dest: Battlefield,
        },
        Case {
            card: "Path of the Schemer",
            wanted: "Grizzly Bears",
            owner: P1,
            run: |t| cast(t, "Path of the Schemer"),
            dest: Battlefield,
        },
        Case {
            card: "Another Chance",
            wanted: "Grizzly Bears",
            owner: P0,
            run: |t| {
                t.answer_yes(P0, true);
                cast(t, "Another Chance");
            },
            dest: Hand,
        },
        Case {
            card: "Takenuma, Abandoned Mire",
            wanted: "Grizzly Bears",
            owner: P0,
            run: |t| {
                let takenuma = t.hand(P0, "Takenuma, Abandoned Mire");
                add_mana(t, P0, ManaType::B, 1);
                add_mana(t, P0, ManaType::C, 3);
                activate_containing(t, P0, takenuma, "Mill three").expect("activated");
            },
            dest: Hand,
        },
        Case {
            card: "Glowspore Shaman",
            wanted: "Forest",
            owner: P0,
            run: |t| {
                t.answer_yes(P0, true);
                enter(t, P0, "Glowspore Shaman");
            },
            dest: LibraryTop,
        },
        Case {
            card: "Realmbreaker, the Invasion Tree",
            wanted: "Forest",
            owner: P1,
            run: |t| {
                let tree = t.battlefield(P0, "Realmbreaker, the Invasion Tree");
                t.lands(P0, "Wastes", 2);
                t.answer_targets(P0, &[Entity::Player(P1)]);
                activate_containing(t, P0, tree, "mills three").expect("activated");
            },
            dest: Battlefield,
        },
        Case {
            card: "Eccentric Farmer",
            wanted: "Forest",
            owner: P0,
            run: |t| {
                t.answer_yes(P0, true);
                enter(t, P0, "Eccentric Farmer");
            },
            dest: Hand,
        },
        Case {
            card: "Port of Karfell",
            wanted: "Grizzly Bears",
            owner: P0,
            run: |t| {
                let port = t.battlefield(P0, "Port of Karfell");
                add_mana(t, P0, ManaType::U, 1);
                add_mana(t, P0, ManaType::B, 2);
                add_mana(t, P0, ManaType::C, 3);
                activate_containing(t, P0, port, "Mill four").expect("activated");
            },
            dest: Battlefield,
        },
        Case {
            card: "Summon Undead",
            wanted: "Grizzly Bears",
            owner: P0,
            run: |t| {
                t.answer_yes(P0, true);
                cast(t, "Summon Undead");
            },
            dest: Battlefield,
        },
        Case {
            card: "Incarnation Technique",
            wanted: "Grizzly Bears",
            owner: P0,
            run: |t| {
                // Demonstrate: P0 doesn't copy it.
                t.answer_yes(P0, false);
                cast(t, "Incarnation Technique");
            },
            dest: Battlefield,
        },
        Case {
            card: "Dig Up the Body",
            wanted: "Grizzly Bears",
            owner: P0,
            run: |t| {
                t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(false));
                t.answer_yes(P0, true);
                cast(t, "Dig Up the Body");
            },
            dest: Hand,
        },
        Case {
            card: "Overlord of the Balemurk",
            wanted: "Grizzly Bears",
            owner: P0,
            run: |t| {
                t.answer_yes(P0, true);
                enter(t, P0, "Overlord of the Balemurk");
            },
            dest: Hand,
        },
        Case {
            card: "Witherbloom Command",
            wanted: "Forest",
            owner: P0,
            run: |t| {
                let c = in_hand_with_mana(t, P0, "Witherbloom Command");
                t.cast(P0, c)
                    .modes(&[0, 3])
                    .target(Entity::Player(P0))
                    .target(Entity::Player(P1))
                    .go();
            },
            dest: Hand,
        },
    ]
}

/// Whether the wanted card is where it should be.
fn arrived(t: &TestGame, case: &Case) -> bool {
    let w = case.wanted;
    match case.dest {
        Dest::Battlefield => t
            .named_on_battlefield(w)
            .iter()
            .any(|id| t.obj_now(*id).controller == P0),
        Dest::Hand => t.in_hand(P0, w),
        Dest::LibraryTop => t
            .g
            .library_top(case.owner)
            .is_some_and(|id| t.g.obj(id).chars.name == w),
        Dest::Exile => t.in_exile(w),
    }
}

#[test]
fn the_returned_card_can_be_one_just_milled_or_one_already_there() {
    cr!("608.2c", "701.17a", "115.10");
    ruling!("Tyvar, Jubilant Brawler", "For Tyvar's last ability, you may return any creature card with mana value 2 or less from your graveyard to the battlefield, even if it wasn't one of the three cards milled as the ability resolved.");
    ruling!("Harald Unites the Elves", "For the chapter I ability, the Elf or Tyvar card you put onto the battlefield can be one of the cards you just milled, or it may be one that was already in your graveyard.");
    ruling!("Roots of Wisdom", "Roots of Wisdom doesn’t target any cards in graveyards. You may return a land card or an Elf card that was just milled or one that was already there.");
    ruling!("Technomancer", "The artifact creature cards you choose don't need to be cards that you milled. You may choose cards that were already in your graveyard before you milled.");
    ruling!("Halo-Charged Skaab", "The card you put on top of your library, if any, may be one you just milled, or it can be one that was already in your graveyard.");
    ruling!("Dogmeat, Ever Loyal", "The card you return with Dogmeat's first ability doesn't have to be one of the cards you milled with that ability.");
    ruling!("Breach the Multiverse", "The cards you choose can be ones that were just milled or ones that were already in the graveyard.");
    ruling!("King Narfi's Betrayal", "The chapter I ability doesn't target any cards in graveyards. For each graveyard, you may exile a creature or planeswalker card that was just milled or one that was already there. You choose which cards are exiled.");
    ruling!("Druidic Ritual", "The creature and/or land card you return doesn't need to be one of the cards you just milled. It can be any card that is in your graveyard at that time.");
    ruling!("Tomb Fortress", "The creature card that you return to the battlefield doesn't have to be one of the cards you milled. It can be a card that was already in your graveyard before you milled.");
    ruling!("Sauron, Lord of the Rings", "The creature card you choose doesn't need to be a card you milled. You may choose a creature card that was already in your graveyard before you milled.");
    ruling!("Path of the Schemer", "The creature card you put onto the battlefield may be one that was just milled, or it may be one that was already in a graveyard.");
    ruling!("Another Chance", "The creature cards you return to your hand don't have to be any of the cards you milled. They can be cards that were already in your graveyard before you milled.");
    ruling!("Takenuma, Abandoned Mire", "The creature or planeswalker card you return to your hand is not chosen until after you mill three cards. You may choose a creature or planeswalker card from among all the cards in your graveyard, including the milled cards.");
    ruling!("Glowspore Shaman", "The land card you put on top of your library may be one that you just milled or one that was already there.");
    ruling!("Realmbreaker, the Invasion Tree", "The land card you put onto the battlefield can be one that was just milled, or it can be one that was already in their graveyard.");
    ruling!("Eccentric Farmer", "The land card you return to your hand can be one you just milled or it can be one that was already in your graveyard.");
    ruling!("Port of Karfell", "The last activated ability doesn't target any creature cards in graveyards. You may return a creature card that was just milled or one that was already there.");
    ruling!("Summon Undead", "You can return a creature card that you milled or one that was already in your graveyard.");
    ruling!("Incarnation Technique", "You choose which creature to bring back after you mill five cards.");
    ruling!("Incarnation Technique", "You may choose any creature card in your graveyard, not just one that you milled this way.");
    ruling!("Dig Up the Body", "You may return any creature card from your graveyard to your hand, not just those you milled while resolving Dig Up the Body.");
    ruling!("Overlord of the Balemurk", "You may return any non-Avatar creature card or any planeswalker card from your graveyard to your hand with Overlord of the Balemurk's last ability, not just one you milled while resolving that ability.");
    ruling!("Witherbloom Command", "If you target yourself with the first mode, you may return any land card to your hand, including one you just milled or one that was already there.");
    for case in cases() {
        supported(case.card);
        for already_there in [true, false] {
            let mut t = TestGame::new(2);
            WANTED.with(|w| w.set(case.wanted));
            respond(&mut t, P0, pick_wanted);
            if already_there {
                t.graveyard(case.owner, case.wanted);
            } else {
                t.library_top(case.owner, case.wanted);
            }
            (case.run)(&mut t);
            t.resolve_all();
            assert!(
                arrived(&t, &case),
                "{} (already there: {already_there})\n{}",
                case.card,
                t.dump_log()
            );
        }
    }
}

#[test]
fn tasigurs_opponent_may_choose_a_card_that_wasnt_milled() {
    cr!("608.2c", "701.17a");
    ruling!(
        "Tasigur, the Golden Fang",
        "The chosen opponent can choose any nonland card in your graveyard, not just one of the milled cards."
    );
    supported("Tasigur, the Golden Fang");
    let mut t = TestGame::new(2);
    WANTED.with(|w| w.set("Grizzly Bears"));
    respond(&mut t, P1, pick_wanted);
    t.graveyard(P0, "Grizzly Bears");
    stack_library(&mut t, P0, &["Hill Giant", "Shock"]);
    let tasigur = t.battlefield(P0, "Tasigur, the Golden Fang");
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Wastes", 2);
    activate_containing(&mut t, P0, tasigur, "Mill two").expect("activated");
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Hill Giant") && t.in_graveyard(P0, "Shock"));
}

#[test]
fn path_of_the_schemer_creature_becomes_an_artifact() {
    cr!("608.2c", "205.1b");
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P1, "Grizzly Bears");
    cast(&mut t, "Path of the Schemer");
    t.resolve_all();
    let bears = t.g.current(bears);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj(bears).controller, P0);
    assert!(t.obj(bears).is(CardType::Artifact));
}

#[test]
fn summon_undead_returns_a_creature_even_without_milling() {
    cr!("608.2c", "701.17b");
    ruling!(
        "Summon Undead",
        "You return a creature card if there is one to return, even if you chose not to mill three cards."
    );
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Grizzly Bears");
    t.answer_yes(P0, false);
    cast(&mut t, "Summon Undead");
    t.resolve_all();
    assert_eq!(t.library_size(P0), 30);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn inspiration_from_beyond_cant_return_itself() {
    cr!("608.2c", "608.2n");
    ruling!(
        "Inspiration from Beyond",
        "The instant or sorcery card you return doesn't have to be one of the milled cards. It can't be Inspiration from Beyond itself, which will be on the stack and not in your graveyard as it's resolving."
    );
    supported("Inspiration from Beyond");
    // Nothing else to return: it doesn't return itself.
    let mut t = TestGame::new(2);
    cast(&mut t, "Inspiration from Beyond");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Inspiration from Beyond"));
    assert_eq!(t.hand_size(P0), 0);
    // A card that was already there.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Shock");
    cast(&mut t, "Inspiration from Beyond");
    t.resolve_all();
    assert!(t.in_hand(P0, "Shock"));
    assert!(t.in_graveyard(P0, "Inspiration from Beyond"));
}

#[test]
fn ashcoat_may_return_rats_it_just_milled() {
    cr!("608.2c", "603.12");
    ruling!(
        "Ashcoat of the Shadow Swarm",
        "You choose which Rat creature cards to return to your hand as the last ability resolves. You may choose Rat creature cards milled with this ability."
    );
    supported("Ashcoat of the Shadow Swarm");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ashcoat of the Shadow Swarm");
    stack_library(&mut t, P0, &["Pack Rat", "Forest", "Rat Colony", "Island"]);
    // P0 takes every Rat creature card offered (the two just milled).
    fn take_all(_g: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
        let Decision::ChooseEntities { candidates, .. } = d else {
            return None;
        };
        Some(Answer::Entities(candidates.clone()))
    }
    respond(&mut t, P0, take_all);
    t.answer_yes(P0, true);
    t.set_step(P0, mtg_engine::turn::Step::PostcombatMain);
    t.advance_to(P0, mtg_engine::turn::Step::End);
    t.resolve_all();
    assert!(t.in_hand(P0, "Pack Rat"));
    assert!(t.in_hand(P0, "Rat Colony"));
    assert_eq!(t.g.find_in_zone(Zone::Graveyard(P0), "Forest").len(), 1);
}
