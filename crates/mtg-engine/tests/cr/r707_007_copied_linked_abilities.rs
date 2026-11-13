//! CR 707.7, 607.2, 603.7c: linked abilities a copy has are linked to one another on the
//! copy, the way they were on the original — the abilities its keywords stand for, its "as
//! this enters" abilities, delayed triggered abilities they create, and the cards they
//! exile however they exile them. Found by the dynamic copy property
//! (`mtg_engine::properties::copies::check_dynamic`, docs/PROPERTIES.md): each of these
//! worked on the original and not on a copy.

use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// `P0`'s copy of `what` (an object on the battlefield) made by `cloner` entering.
fn copy_of(t: &mut TestGame, cloner: &str, what: ObjectId) -> ObjectId {
    t.answer_choose(P0, &[Entity::Object(what)]);
    let c = t.enter(P0, cloner);
    t.g.current(c)
}

fn fill_pool(t: &mut TestGame) {
    for ty in mtg_engine::mana::ManaType::ALL {
        t.g.players[0].mana_pool.add_type(ty, 8);
    }
}

#[test]
fn a_copys_as_enters_choice_is_seen_by_the_ability_linked_to_it() {
    cr!("707.7", "607.2d", "614.12a");
    // "As this creature enters, choose another creature you control. The chosen creature
    // gets +3/+3 and has flying."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let pitlord = t.battlefield(P1, "Tyrannical Pitlord");
    t.answer_choose(P0, &[Entity::Object(pitlord)]);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    let c = t.enter(P0, "Clone");
    assert_eq!(t.obj_now(c).chars.name.as_str(), "Tyrannical Pitlord");
    assert_eq!(t.pt(bears), (5, 5));
}

#[test]
fn a_copy_can_activate_its_ability_linked_to_the_card_it_exiled() {
    cr!("707.7", "607.2a", "602.2b");
    // "When this creature enters, exile target non-Ooze creature an opponent controls
    // until this creature leaves the battlefield. {X}{B}: Put target creature card with
    // mana value X exiled with this creature into its owner's graveyard."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let cube = t.battlefield(P1, "Gelatinous Cube");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let c = copy_of(&mut t, "Clone", cube);
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    fill_pool(&mut t);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    let exiled = t.g.current(bears);
    t.activate(P0, c, 0, &[Entity::Object(exiled)])
        .expect("the copy's Dissolve can target the card it exiled");
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Graveyard(P1));
}

#[test]
fn a_copys_delayed_trigger_finds_the_card_its_ability_exiled() {
    cr!("707.7", "603.7c", "607.2a");
    // "Whenever this creature attacks, you may exile another target nontoken creature. If
    // you do, this creature becomes a copy of that creature until end of turn. Return the
    // exiled card to the battlefield under its owner's control at the beginning of the
    // next end step."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let thief = t.battlefield(P1, "Identity Thief");
    let c = copy_of(&mut t, "Clone", thief);
    t.g.objects[c.0 as usize].summoning_sick = false;
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.attack(&[(c, Entity::Player(P1))], &[]);
    assert_eq!(t.zone(bears), Zone::Exile);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.zone(bears), Zone::Battlefield);
}

#[test]
fn cards_a_copy_exiles_by_searching_are_exiled_with_it() {
    cr!("707.7", "607.2a");
    // "When this enchantment enters, search your library for seven cards, exile them in a
    // face-down pile, and shuffle that pile. ... If you would draw a card, you may instead
    // put the top card of the pile you exiled into your hand."
    let mut t = TestGame::new(2);
    let pt = t.battlefield(P1, "Parallel Thoughts");
    copy_of(&mut t, "Copy Enchantment", pt);
    t.resolve_all();
    let exiled = t.g.exile.len();
    assert_eq!(exiled, 7);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::Draw);
    t.advance_to_step(Step::PrecombatMain);
    // The draw was replaced: the top card of the pile went to the hand.
    assert_eq!(t.g.exile.len(), exiled - 1);
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn the_abilities_a_copys_keywords_stand_for_are_linked_like_the_keywords() {
    cr!("707.7", "607.2a", "702.75a");
    // "Hideaway 4. Whenever this Vehicle deals combat damage to a player, you may cast the
    // exiled card without paying its mana cost. If you do, return this Vehicle to its
    // owner's hand."
    let mut t = TestGame::new(2);
    let buggy = t.battlefield(P1, "Smuggler's Buggy");
    let c = copy_of(&mut t, "Sculpting Steel", buggy);
    t.resolve_all();
    assert_eq!(t.g.exile.len(), 1, "hideaway exiled a card");
    let o = t.obj_now(c);
    let hideaway = o
        .chars
        .abilities
        .iter()
        .filter(|a| a.text.starts_with("Hideaway"))
        .map(|a| a.link)
        .collect::<Vec<_>>();
    let cast = o
        .chars
        .abilities
        .iter()
        .find(|a| a.text.contains("cast the exiled card"))
        .map(|a| a.link);
    // The keyword and the triggered ability it stands for share the copy's link with the
    // ability that casts the exiled card.
    assert!(hideaway.len() >= 2);
    assert!(
        hideaway.iter().all(|l| Some(*l) == cast),
        "{hideaway:?} {cast:?}"
    );
    let exiled = t.g.exile[0];
    assert!(o
        .linked
        .get(&cast.unwrap())
        .is_some_and(|v| v.contains(&exiled)));
}

#[test]
fn a_copy_of_an_ante_card_has_its_ante_text() {
    cr!("407.3", "707.2");
    // "Remove this card from your deck before playing if you're not playing for ante.
    // {T}: Ante this artifact. If you do, put all other cards you own from the ante into
    // your graveyard, then draw a card."
    let mut t = TestGame::new(2);
    let bird = t.battlefield(P1, "Jeweled Bird");
    let c = copy_of(&mut t, "Sculpting Steel", bird);
    t.activate(P0, c, 0, &[]).expect("activate");
    t.resolve_all();
    assert_eq!(t.zone(c), Zone::Ante);
    assert_eq!(t.hand_size(P0), 1);
}
