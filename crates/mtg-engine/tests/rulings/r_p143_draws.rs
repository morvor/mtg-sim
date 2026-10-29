//! Rulings batch P143 — draw replacement and draw restriction: Sages of the Anima,
//! Tomorrow, Azami's Familiar and Narset, Parter of Veils.

use crate::r_p143_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

const LIB: &[&str] = &[
    "Grizzly Bears",
    "Island",
    "Llanowar Elves",
    "Forest",
    "Memnite",
    "Swamp",
    "Ornithopter",
];

fn names_in_hand(t: &TestGame, p: PlayerId) -> Vec<String> {
    let mut v: Vec<String> = t
        .g
        .player(p)
        .hand
        .iter()
        .map(|id| t.g.obj(*id).chars.name.to_string())
        .collect();
    v.sort();
    v
}

/// "If you would draw a card, instead reveal the top three cards of your library. Put all
/// creature cards revealed this way into your hand and the rest on the bottom of your
/// library in any order."
#[test]
fn sages_of_the_anima_replaces_each_draw_but_not_other_ways_into_hand() {
    cr!("121.5", "121.2", "614.11");
    supported("Sages of the Anima");
    ruling!(
        "Sages of the Anima",
        "If an effect would cause you to draw multiple cards, each individual draw is replaced by Sages of the Anima’s effect. Process the draws one at a time."
    );
    // Drawing two: the first replacement takes Bears and Elves (Island to the bottom),
    // the second reveals Forest, Memnite, Swamp and takes Memnite.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sages of the Anima");
    let lib = set_library(&mut t, P0, LIB);
    draw(&mut t, P0, 2);
    assert_eq!(
        names_in_hand(&t, P0),
        vec!["Grizzly Bears", "Llanowar Elves", "Memnite"]
    );
    // Ornithopter (never revealed) is now on top; the revealed noncreatures are below.
    assert_eq!(library_top_first(&t, P0)[0], lib[6]);
    assert_eq!(t.library_size(P0), 4);

    ruling!(
        "Sages of the Anima",
        "If a spell or ability causes you to put cards in your hand without specifically using the word “draw,” Sages of the Anima’s ability won’t affect it."
    );
    // Impulse: "Look at the top four cards of your library. Put one of them into your
    // hand and the rest on the bottom of your library in any order."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sages of the Anima");
    let lib = set_library(&mut t, P0, LIB);
    t.answer_choose(P0, &[obj(lib[1])]);
    cast_new(&mut t, P0, "Impulse", &[]);
    t.resolve_all();
    assert_eq!(names_in_hand(&t, P0), vec!["Island"]);
    assert_eq!(library_top_first(&t, P0)[0], lib[4]);
}

#[test]
fn sages_of_the_anima_reveals_what_there_is_and_is_mandatory() {
    cr!("121.6a", "614.11");
    ruling!(
        "Sages of the Anima",
        "If there are fewer than three cards in your library, you’ll reveal all of them."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sages of the Anima");
    set_library(&mut t, P0, &["Island", "Memnite"]);
    draw(&mut t, P0, 1);
    assert_eq!(names_in_hand(&t, P0), vec!["Memnite"]);
    assert_eq!(t.library_size(P0), 1);
    assert!(!t.g.player(P0).drew_from_empty_library);

    ruling!(
        "Sages of the Anima",
        "This effect is mandatory. Even if there are no more creature cards in your library, you have to do this instead of drawing a card."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sages of the Anima");
    let lib = set_library(&mut t, P0, &["Island", "Swamp", "Forest", "Plains"]);
    let from = n_asked(&t);
    draw(&mut t, P0, 1);
    assert_eq!(t.hand_size(P0), 0);
    // Nothing optional was offered, and the three revealed lands went to the bottom.
    assert!(!asked_of(&t, P0, from)
        .iter()
        .any(|d| matches!(d, Decision::YesNo { .. } | Decision::ChooseReplacement { .. })));
    assert_eq!(library_top_first(&t, P0)[0], lib[3]);
}

#[test]
fn two_sages_apply_only_once_to_a_draw() {
    cr!("616.1", "616.1e");
    ruling!(
        "Sages of the Anima",
        "If two or more replacement effects would apply to a card-drawing event, the player who’s drawing the card chooses what order to apply them."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sages of the Anima");
    t.battlefield(P0, "Sages of the Anima");
    set_library(&mut t, P0, LIB);
    draw(&mut t, P0, 1);
    // Only the top three were revealed once: Memnite (fifth) stays in the library.
    assert_eq!(names_in_hand(&t, P0), vec!["Grizzly Bears", "Llanowar Elves"]);
    assert_eq!(t.library_size(P0), 5);
}

/// "If you would draw a card, look at the top three cards of your library instead. Put
/// one of those cards into your hand and the rest on the bottom of your library in any
/// order."
#[test]
fn tomorrow_replaces_each_draw_with_a_look_at_three() {
    cr!("121.2", "614.11", "616.1");
    supported("Tomorrow, Azami's Familiar");
    ruling!(
        "Tomorrow, Azami's Familiar",
        "If you would draw two cards, you look at the top three cards of your library, keep one, and put the rest on the bottom of your library in any order. Then you look at the new top three cards of your library, keep one, and put the rest on the bottom of your library in any order."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tomorrow, Azami's Familiar");
    let lib = set_library(&mut t, P0, LIB);
    let from = n_asked(&t);
    t.answer_choose(P0, &[obj(lib[2])]);
    t.answer_choose(P0, &[obj(lib[5])]);
    draw(&mut t, P0, 2);
    let looks: Vec<usize> = entity_choices(&t, P0, from)
        .iter()
        .map(|(_, _, c)| c.len())
        .collect();
    assert_eq!(looks, vec![3, 3]);
    assert_eq!(names_in_hand(&t, P0), vec!["Llanowar Elves", "Swamp"]);
    // The second look saw the new top three: Forest, Memnite and Swamp.
    let second = &entity_choices(&t, P0, from)[1].2;
    assert!(second.contains(&obj(lib[3])) && second.contains(&obj(lib[5])));

    ruling!(
        "Tomorrow, Azami's Familiar",
        "Having multiple Tomorrows on the battlefield does nothing extra."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tomorrow, Azami's Familiar");
    t.battlefield(P0, "Tomorrow, Azami's Familiar");
    set_library(&mut t, P0, LIB);
    let from = n_asked(&t);
    draw(&mut t, P0, 1);
    // (Before the legend rule is checked.)
    let looks = asked_of(&t, P0, from)
        .iter()
        .filter(|d| matches!(d, Decision::ChooseEntities { prompt, .. } if prompt.contains("take")))
        .count();
    assert_eq!(looks, 1);
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.library_size(P0), 6);
}

#[test]
fn tomorrow_with_a_short_or_empty_library() {
    cr!("614.11", "121.6a", "704.5b");
    ruling!(
        "Tomorrow, Azami's Familiar",
        "If you have fewer than three cards in your library, you keep one and put the rest back. If you have no cards in your library, you do nothing."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tomorrow, Azami's Familiar");
    let lib = set_library(&mut t, P0, &["Island", "Memnite"]);
    t.answer_choose(P0, &[obj(lib[1])]);
    draw(&mut t, P0, 1);
    assert_eq!(names_in_hand(&t, P0), vec!["Memnite"]);
    assert_eq!(library_top_first(&t, P0), vec![lib[0]]);

    ruling!(
        "Tomorrow, Azami's Familiar",
        "Since Tomorrow replaces all your card drawing with looking at cards, you can’t lose the game by being required to draw more cards than are in your library with Tomorrow on the battlefield."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tomorrow, Azami's Familiar");
    set_library(&mut t, P0, &[]);
    draw(&mut t, P0, 3);
    assert_eq!(t.hand_size(P0), 0);
    assert!(!t.has_lost(P0));
}

#[test]
fn tomorrow_looking_is_not_drawing() {
    cr!("121.5", "614.11");
    ruling!(
        "Tomorrow, Azami's Familiar",
        "Tomorrow’s ability makes you look at the top three cards, not draw them. Therefore it won’t trigger abilities that trigger when cards are drawn."
    );
    // Psychosis Crawler: "Whenever you draw a card, each opponent loses 1 life."
    supported("Psychosis Crawler");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Psychosis Crawler");
    draw(&mut t, P0, 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    t.battlefield(P0, "Tomorrow, Azami's Familiar");
    draw(&mut t, P0, 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.hand_size(P0), 2);
}

/// "Each opponent can't draw more than one card each turn."
#[test]
fn narset_lets_each_opponent_draw_one_card_each_turn() {
    cr!("121.2b");
    supported("Narset, Parter of Veils");
    ruling!(
        "Narset, Parter of Veils",
        "Your opponents can each draw a maximum of one card each on each player's turn. Subsequent card draws during that turn are ignored."
    );
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Narset, Parter of Veils");
    draw(&mut t, P1, 1);
    draw(&mut t, P1, 1);
    draw(&mut t, P2, 3);
    assert_eq!((t.hand_size(P1), t.hand_size(P2)), (1, 1));
    // Narset's controller isn't affected.
    draw(&mut t, P0, 2);
    assert_eq!(t.hand_size(P0), 2);
    // On P1's turn, P1 draws for the turn and can't draw more.
    t.advance_to(P1, mtg_engine::turn::Step::PrecombatMain);
    assert_eq!(t.hand_size(P1), 2);
    draw(&mut t, P1, 1);
    assert_eq!(t.hand_size(P1), 2);

    ruling!(
        "Narset, Parter of Veils",
        "If an opponent hasn't drawn any cards in a turn and a spell or ability instructs that player to draw multiple cards, that player will just draw one card. However, if the draws are optional, the player can't choose to draw, even if they could draw one card this way."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Narset, Parter of Veils");
    t.g.turn.active = P1;
    t.lands(P1, "Island", 3);
    cast_new(&mut t, P1, "Divination", &[]);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 1);
    // "You may draw two cards." isn't a choice P1 can take, though one draw is left.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Narset, Parter of Veils");
    t.g.turn.active = P1;
    let may = custom_card("Optional Insight", "Sorcery", "{U}", None, "You may draw two cards.");
    let spell = t.custom(P1, may, Zone::Hand(P1));
    t.lands(P1, "Island", 1);
    t.answer_yes(P1, true);
    t.cast(P1, spell).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 0);
}

#[test]
fn narset_sees_draws_made_before_it_entered() {
    cr!("121.2b");
    ruling!(
        "Narset, Parter of Veils",
        "Narset will \"see\" cards drawn by opponents earlier in the turn she entered the battlefield, although Narset can't affect cards drawn before she entered the battlefield."
    );
    let mut t = TestGame::new(2);
    draw(&mut t, P1, 2);
    t.battlefield(P0, "Narset, Parter of Veils");
    t.g.recompute();
    assert_eq!(t.hand_size(P1), 2);
    draw(&mut t, P1, 1);
    assert_eq!(t.hand_size(P1), 2);
}

#[test]
fn narset_and_replaced_draws() {
    cr!("121.2b", "614.11");
    // Underrealm Lich: "If you would draw a card, instead look at the top three cards of
    // your library, then put one into your hand and the rest into your graveyard."
    supported("Underrealm Lich");
    ruling!(
        "Narset, Parter of Veils",
        "However, if an opponent's first draw is replaced (by Underrealm Lich's ability, for example), that draw didn't happen and Narset won't stop the next draw"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Narset, Parter of Veils");
    t.battlefield(P1, "Underrealm Lich");
    draw(&mut t, P1, 2);
    // Both draws were replaced: two looks at three cards (one kept, two milled each).
    assert_eq!(t.hand_size(P1), 2);
    assert_eq!(t.graveyard_size(P1), 4);

    ruling!(
        "Narset, Parter of Veils",
        "Replacement effects (such as that of Underrealm Lich or the first ability of Jace, Wielder of Mysteries) can't be used to replace draws that Narset disallows."
    );
    let mut t = TestGame::new(2);
    draw(&mut t, P1, 1);
    t.battlefield(P0, "Narset, Parter of Veils");
    t.battlefield(P1, "Underrealm Lich");
    draw(&mut t, P1, 1);
    assert_eq!(t.hand_size(P1), 1);
    assert_eq!(t.graveyard_size(P1), 0);
}
