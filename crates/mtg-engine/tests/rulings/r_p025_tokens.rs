//! Rulings batch P025 — token copies: what the token copies (CR 707.2, 111.3), its mana
//! cost (CR 707.2, 202.3), copies of double-faced cards (CR 707.8) and face-down
//! permanents (CR 708.2), copies of something that's copying something else (CR 707.3),
//! the order tokens enter, and "for each opponent" counts in multiplayer games (CR
//! 800.4a).

use crate::r_p025_common::*;
use mtg_engine::game::GameConfig;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{ObjKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0's Soul Separator is activated targeting `card` in P0's graveyard; returns the new
/// tokens in the order they were created.
fn soul_separate(t: &mut TestGame, card: ObjectId) -> Vec<ObjectId> {
    supported("Soul Separator");
    let sep = t.battlefield(P0, "Soul Separator");
    t.lands(P0, "Wastes", 5);
    let before = t.g.battlefield.clone();
    t.activate(P0, sep, 0, &[obj(card)]).expect("activation");
    t.resolve_all();
    let mut toks = new_tokens(t, P0, &before);
    toks.sort();
    assert_eq!(toks.len(), 2);
    toks
}

#[test]
fn soul_separators_spirit_has_the_cards_mana_cost_and_enters_first() {
    cr!("707.2", "707.9b", "111.3", "202.3");
    ruling!(
        "Soul Separator",
        "The Spirit token has the color and mana cost (and thus mana value) of the original card; the Zombie token has a mana value of 0."
    );
    ruling!(
        "Soul Separator",
        "The Spirit token will be on the battlefield before the Zombie token enters."
    );
    let mut t = TestGame::new(2);
    let card = t.graveyard(P0, "Hill Giant");
    let toks = soul_separate(&mut t, card);
    let (spirit, zombie) = (toks[0], toks[1]);
    let s = t.obj_now(spirit);
    assert_eq!(s.chars.name, "Hill Giant");
    assert_eq!(s.chars.colors, red());
    assert!(has_subtype(&t, spirit, "Spirit") && has_kw(&t, spirit, KeywordKind::Flying));
    assert_eq!(t.pt(spirit), (1, 1));
    assert_eq!(mv_now(&mut t, spirit), 4);
    let z = t.obj_now(zombie);
    assert_eq!(z.chars.colors, ColorSet::single(Color::Black));
    assert!(has_subtype(&t, zombie, "Zombie"));
    assert_eq!(t.pt(zombie), (3, 3));
    assert_eq!(mv_now(&mut t, zombie), 0);
    // The Spirit entered first (an earlier timestamp).
    assert!(t.obj_now(spirit).timestamp < t.obj_now(zombie).timestamp);
}

#[test]
fn soul_separators_spirit_copy_of_a_double_faced_card_is_double_faced() {
    cr!("707.8", "712.2", "701.27a");
    ruling!(
        "Soul Separator",
        "If you copy a double-faced creature card with Soul Separator’s ability, the resulting Spirit token will be a double-faced token. The listed exceptions will apply to both the front and back faces of the Spirit token. The Zombie token will not be a double-faced token."
    );
    let mut t = TestGame::new(2);
    let card = t.graveyard(P0, "Delver of Secrets // Insectile Aberration");
    let toks = soul_separate(&mut t, card);
    let (spirit, zombie) = (toks[0], toks[1]);
    assert_eq!(name_of(&t, spirit), "Delver of Secrets");
    crate::r_s17_common::transform(&mut t, spirit);
    assert_eq!(name_of(&t, spirit), "Insectile Aberration");
    assert!(has_subtype(&t, spirit, "Spirit") && has_kw(&t, spirit, KeywordKind::Flying));
    assert_eq!(t.pt(spirit), (1, 1));
    // The Zombie can't transform.
    let zname = name_of(&t, zombie);
    crate::r_s17_common::transform(&mut t, zombie);
    assert_eq!(name_of(&t, zombie), zname);
    assert_eq!(t.pt(zombie), (1, 1));
}

#[test]
fn altered_ego_copies_the_face_thats_up_and_cant_transform() {
    cr!("707.8", "712.14", "701.27c");
    ruling!(
        "Altered Ego",
        "If you copy a double-faced creature, Altered Ego will be a copy of the face that's up when Altered Ego enters. Because Altered Ego is not a double-faced card, it won't be able to transform. If an effect instructs you to return it to the battlefield transformed when it leaves the battlefield, it won't return and will remain in its new zone."
    );
    supported("Altered Ego");
    let mut t = TestGame::new(2);
    let delver = t.battlefield(P1, "Delver of Secrets // Insectile Aberration");
    crate::r_s17_common::transform(&mut t, delver);
    assert_eq!(name_of(&t, delver), "Insectile Aberration");
    let ego = enter_copying(&mut t, P0, "Altered Ego", delver);
    assert_eq!(name_of(&t, ego), "Insectile Aberration");
    assert_eq!(t.pt(ego), (3, 2));
    crate::r_s17_common::transform(&mut t, ego);
    assert_eq!(name_of(&t, ego), "Insectile Aberration");
    // Put onto the battlefield transformed from the graveyard: it stays there.
    let card = t.graveyard(P0, "Altered Ego");
    assert!(crate::r_s17_common::put_onto_battlefield(&mut t, P0, card, true).is_none());
    assert_eq!(t.zone(t.g.current(card)), Zone::Graveyard(P0));
}

#[test]
fn supplant_form_copying_a_face_down_creature() {
    cr!("707.2", "708.2", "708.9");
    ruling!(
        "Supplant Form",
        "If you return a face-down creature to its owner’s hand, the token will be a face-up colorless 2/2 creature with no name, no abilities, and no creature types."
    );
    let mut t = TestGame::new(2);
    let fd = crate::r_s12_common::morph(&mut t, P0, "Den Protector");
    let toks = cast_for_tokens(&mut t, P0, "Supplant Form", "{4}{U}{U}", &[obj(fd)]);
    assert!(t.in_hand(P0, "Den Protector"));
    assert_eq!(toks.len(), 1);
    let o = t.obj_now(toks[0]);
    assert!(!o.face_down);
    assert!(o.chars.name.is_empty());
    assert_eq!(o.chars.colors, ColorSet::NONE);
    assert!(o.chars.subtypes.is_empty() && o.chars.abilities.is_empty());
    assert_eq!(t.pt(toks[0]), (2, 2));
}

#[test]
fn echoing_assault_copying_a_clone() {
    cr!("707.3", "707.9b", "508.3a");
    ruling!(
        "Echoing Assault",
        "In the case where the target creature is copying something else when Echoing Assault's second ability resolves, then the token enters as whatever that creature copied, except it's a 1/1."
    );
    supported("Echoing Assault");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Echoing Assault");
    let clone = cloned_angel(&mut t, P0);
    unsick(&mut t, clone);
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer_targets(P0, &[obj(clone)]);
    let before = t.g.battlefield.clone();
    t.attack(&[(clone, Entity::Player(P1))], &[]);
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1, "{}", t.dump_log());
    assert_eq!(name_of(&t, toks[0]), "Serra Angel");
    assert!(has_kw(&t, toks[0], KeywordKind::Flying));
    assert_eq!(t.pt(toks[0]), (1, 1));
    assert!(t.g.is_attacking(toks[0]));
}

#[test]
fn sarkhan_copies_what_the_dragon_is_copying_by_then() {
    cr!("707.3", "707.9b", "608.2h");
    ruling!(
        "Sarkhan, Soul Aflame",
        "In some unusual cases, the Dragon that entered the battlefield will be copying something else by the time Sarkhan's ability resolves. In such a case, Sarkhan will copy whatever the Dragon is copying, with the noted exceptions."
    );
    supported("Sarkhan, Soul Aflame");
    let mut t = TestGame::new(2);
    let sarkhan = t.battlefield(P0, "Sarkhan, Soul Aflame");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_yes(P0, true);
    let dragon = t.enter(P0, "Shivan Dragon");
    t.settle();
    // In response, the Dragon becomes a copy of the Hill Giant (Cytoshape).
    lands_for(&mut t, P0, "{1}{G}{U}");
    let cyto = t.hand(P0, "Cytoshape");
    t.answer_choose(P0, &[obj(giant)]);
    t.cast(P0, cyto).target(obj(dragon)).go();
    t.resolve();
    assert_eq!(name_of(&t, dragon), "Hill Giant");
    t.resolve_all();
    let o = t.obj_now(sarkhan);
    assert_eq!(o.chars.name, "Sarkhan, Soul Aflame");
    assert!(has_subtype(&t, sarkhan, "Giant") && legendary(&t, sarkhan));
    assert_eq!(t.pt(sarkhan), (3, 3));
}

#[test]
fn miirym_token_copies_only_the_printed_dragon() {
    cr!("707.2", "707.9b");
    ruling!(
        "Miirym, Sentinel Wyrm",
        "Miirym, Sentinel Wyrm's triggered ability copies the printed values of the Dragon that entered the battlefield. It won't copy any counters on that creature or any effects that have changed its power, toughness, types, color, and so on."
    );
    supported("Miirym, Sentinel Wyrm");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Miirym, Sentinel Wyrm");
    let before = t.g.battlefield.clone();
    let dragon = t.enter(P0, "Shivan Dragon");
    t.settle();
    dress(&mut t, P0, dragon);
    t.resolve_all();
    let toks: Vec<_> = new_tokens(&t, P0, &before)
        .into_iter()
        .filter(|x| *x != dragon)
        .collect();
    assert_eq!(toks.len(), 1);
    assert_printed(&t, toks[0], "Shivan Dragon", red(), Some((5, 5)), true);
}

#[test]
fn soul_foundry_token_has_the_cards_mana_cost() {
    cr!("707.2", "202.3", "702.92a");
    ruling!(
        "Soul Foundry",
        "Most creature tokens have no mana cost and a mana value of 0, but a creature token put onto the battlefield by Soul Foundry has the same mana cost and mana value as the card it copies."
    );
    supported("Soul Foundry");
    let mut t = TestGame::new(2);
    let card = t.hand(P0, "Hill Giant");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(card)]);
    let foundry = t.enter(P0, "Soul Foundry");
    t.settle();
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"));
    t.lands(P0, "Wastes", 4);
    let before = t.g.battlefield.clone();
    t.answer(P0, DecisionKind::X, Answer::Number(4));
    t.activate(P0, foundry, 0, &[]).expect("activation");
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    assert_eq!(name_of(&t, toks[0]), "Hill Giant");
    assert!(t.obj_now(toks[0]).chars.mana_cost.is_some());
    assert_eq!(mv_now(&mut t, toks[0]), 4);
}

#[test]
fn second_harvest_copies_every_token_as_it_was_created() {
    cr!("707.2", "111.3");
    ruling!(
        "Second Harvest",
        "Second Harvest copies all tokens, not only creature tokens."
    );
    ruling!(
        "Second Harvest",
        "Second Harvest copies the original characteristics of each token as stated by the effect that created that token."
    );
    let mut t = TestGame::new(2);
    let wolf = wolf_token(&mut t, P0);
    dress_up(&mut t, wolf);
    // A Treasure token (a noncreature artifact).
    let treasure = mtg_engine::tokens::predefined("Treasure").expect("Treasure");
    token_of(&mut t, P0, treasure);
    let toks = cast_for_tokens(&mut t, P0, "Second Harvest", "{2}{G}{G}", &[]);
    assert_eq!(toks.len(), 2);
    let wolf_copy = toks
        .iter()
        .copied()
        .find(|x| name_of(&t, *x) == "Wolf")
        .expect("a Wolf");
    assert!(toks.iter().any(|x| has_subtype(&t, *x, "Treasure")));
    assert_eq!(t.pt(wolf_copy), (2, 2));
    assert!(fresh(&t, wolf_copy));
    assert_eq!(t.obj_now(wolf_copy).chars.colors, ColorSet::single(Color::Green));
}

#[test]
fn shaun_exiles_every_synth_token_you_control() {
    cr!("603.6c", "603.10a");
    ruling!(
        "Shaun, Father of Synths",
        "Shaun’s last ability will exile all Synth tokens you control, not just Synth tokens created by its first ability."
    );
    supported("Shaun, Father of Synths");
    let mut t = TestGame::new(2);
    let shaun = t.battlefield(P0, "Shaun, Father of Synths");
    let synth = token_of(&mut t, P0, creature_spec("Synth", Color::Blue, 1));
    let wolf = wolf_token(&mut t, P0);
    let theirs = token_of(&mut t, P1, creature_spec("Synth", Color::Blue, 1));
    t.g.destroy(shaun, None);
    t.settle();
    t.resolve_all();
    assert!(!t.on_battlefield(synth));
    assert!(t.on_battlefield(wolf) && t.on_battlefield(theirs));
}

#[test]
fn wire_surgeons_encore_ignores_opponents_who_left_the_game() {
    cr!("702.141a", "800.4a");
    ruling!(
        "Wire Surgeons",
        "Opponents who have left the game aren’t counted when determining how many tokens to create."
    );
    supported("Wire Surgeons");
    let mut t = TestGame::with_config(4, GameConfig::default());
    t.battlefield(P0, "Wire Surgeons");
    t.g.lose_game(PlayerId(2));
    assert!(!t.g.player(PlayerId(2)).in_game());
    // Bronze Sable ({2}): encore {2}.
    let card = t.graveyard(P0, "Bronze Sable");
    t.lands(P0, "Wastes", 2);
    let before = t.g.battlefield.clone();
    activate_containing(&mut t, P0, card, "ncore").expect("encore");
    t.resolve_all();
    let toks: Vec<_> = new_tokens(&t, P0, &before)
        .into_iter()
        .filter(|x| t.obj_now(*x).kind == ObjKind::Token)
        .collect();
    assert_eq!(toks.len(), 2);
}
