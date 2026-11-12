//! CR 707.10 audit: copies of spells. A copy copies the choices made for the spell
//! (X, kicker), isn't cast, belongs to and is controlled by the player who put it on the
//! stack, ceases to exist outside the stack (704.5e), and a copy of a permanent spell
//! becomes a token that isn't "created" and is subject to the legend rule (707.10f,
//! 608.3f).

use crate::r603_audit_common::*;
use mtg_engine::mana::ManaType;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Activates Lithoform Engine's `index`th ability ({2}: ability, {3}: instant or sorcery
/// spell, {4}: permanent spell), adding the mana for it, and declines new targets.
fn lithoform(t: &mut TestGame, engine: ObjectId, index: usize, target: ObjectId) {
    t.g.players[0]
        .mana_pool
        .add_type(ManaType::C, 2 + index as u32);
    t.answer_yes(P0, false);
    t.activate(P0, engine, index, &[Entity::Object(target)])
        .expect("activate Lithoform Engine");
}

fn tokens_named_on_battlefield(t: &TestGame, name: &str) -> usize {
    t.named_on_battlefield(name)
        .into_iter()
        .filter(|id| t.g.obj(*id).kind == ObjKind::Token)
        .count()
}

#[test]
fn a_copy_of_a_legendary_permanent_spell_becomes_a_legendary_token() {
    cr!("707.10f", "608.3f", "704.5j");
    ruling!(
        "Lithoform Engine",
        "The rules that apply to a permanent spell becoming a permanent apply to a copy of a spell becoming a token."
    );
    supported("Lithoform Engine");
    let mut t = TestGame::new(2);
    let engine = t.battlefield(P0, "Lithoform Engine");
    t.lands(P0, "Plains", 1);
    let card = t.hand(P0, "Isamaru, Hound of Konda");
    let spell = t.cast(P0, card).go();
    lithoform(&mut t, engine, 2, spell);
    t.resolve();
    // The copy resolves first: a legendary token.
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(
        tokens_named_on_battlefield(&t, "Isamaru, Hound of Konda"),
        1
    );
    let token = t.named_on_battlefield("Isamaru, Hound of Konda")[0];
    assert!(t.obj_now(token).chars.is_legendary());
    assert_eq!(t.obj_now(token).owner, P0);
    // The original resolves: the legend rule. The player keeps the original; the token
    // goes to the graveyard and ceases to exist.
    t.answer_choose_moved(P0, &[Entity::Object(spell)]);
    t.resolve();
    assert_eq!(t.named_on_battlefield("Isamaru, Hound of Konda").len(), 1);
    assert_eq!(
        tokens_named_on_battlefield(&t, "Isamaru, Hound of Konda"),
        0
    );
    assert_eq!(t.graveyard_size(P0), 0);
}

#[test]
fn the_token_a_copy_of_a_permanent_spell_becomes_isnt_created() {
    cr!("707.10f", "608.3f");
    ruling!(
        "Lithoform Engine",
        "The token that a resolving copy of a spell becomes isn't said to have been “created.”"
    );
    supported("Anointed Procession");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Anointed Procession");
    let engine = t.battlefield(P0, "Lithoform Engine");
    t.lands(P0, "Forest", 2);
    let card = t.hand(P0, "Grizzly Bears");
    let spell = t.cast(P0, card).go();
    lithoform(&mut t, engine, 2, spell);
    t.resolve_all();
    // One token (not doubled) and the card.
    assert_eq!(tokens_named_on_battlefield(&t, "Grizzly Bears"), 1);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 2);
}

#[test]
fn a_copy_of_a_kicked_permanent_spell_with_x_is_kicked_and_has_the_same_x_but_isnt_cast() {
    cr!("707.10", "707.10f", "702.33d");
    ruling!(
        "Kavu Titan",
        "If the copied spell is a permanent spell, the token the copy of that spell becomes when it enters is also kicked."
    );
    ruling!(
        "Lithoform Engine",
        "If the spell or ability that's copied has an X whose value was determined as it was cast or activated, the copy will have the same value of X."
    );
    ruling!(
        "Lithoform Engine",
        "Creating the copy won't cause abilities that trigger when a player casts a spell or activates an ability to trigger."
    );
    supported("Kavu Titan");
    supported("Hydroid Krasis");
    let mut t = TestGame::new(2);
    let engine = t.battlefield(P0, "Lithoform Engine");
    t.lands(P0, "Forest", 5);
    let card = t.hand(P0, "Kavu Titan");
    let spell = t.cast(P0, card).kicked(true).go();
    lithoform(&mut t, engine, 2, spell);
    t.resolve_all();
    let titans = t.named_on_battlefield("Kavu Titan");
    assert_eq!(titans.len(), 2);
    for id in titans {
        assert_eq!(t.counters(id, counters::PLUS1), 3);
        assert!(t.obj_now(id).has_keyword(keywords::KeywordKind::Trample));
    }

    // X = 4: the copy's token enters with four +1/+1 counters; only the spell that was
    // cast has its "when you cast" ability trigger (gain 2, draw 2).
    let mut t = TestGame::new(2);
    let engine = t.battlefield(P0, "Lithoform Engine");
    t.lands(P0, "Forest", 3);
    t.lands(P0, "Island", 3);
    let card = t.hand(P0, "Hydroid Krasis");
    let hand = t.hand_size(P0);
    let spell = t.cast(P0, card).x(4).go();
    t.settle();
    lithoform(&mut t, engine, 2, spell);
    t.resolve_all();
    let krases = t.named_on_battlefield("Hydroid Krasis");
    assert_eq!(krases.len(), 2);
    for id in krases {
        assert_eq!(t.counters(id, counters::PLUS1), 4);
    }
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.hand_size(P0), hand - 1 + 2);
}

#[test]
fn a_copy_of_a_kicked_instant_is_kicked() {
    cr!("707.10", "702.33d");
    ruling!(
        "Burst Lightning",
        "If you copy a kicked spell on the stack, the copy is also kicked."
    );
    supported("Burst Lightning");
    let mut t = TestGame::new(2);
    let engine = t.battlefield(P0, "Lithoform Engine");
    t.lands(P0, "Mountain", 5);
    let card = t.hand(P0, "Burst Lightning");
    let spell = t
        .cast(P0, card)
        .kicked(true)
        .target(Entity::Player(P1))
        .go();
    lithoform(&mut t, engine, 1, spell);
    t.resolve_all();
    assert_eq!(t.life(P1), 12);
}

#[test]
fn a_copy_belongs_to_and_is_controlled_by_the_player_who_copied_it() {
    cr!("707.10");
    ruling!(
        "Reverberate",
        "If the copy says that it affects “you,” it affects the controller of the copy, not the controller of the original spell."
    );
    supported("Reverberate");
    supported("Divination");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    t.lands(P1, "Mountain", 2);
    let div = t.hand(P0, "Divination");
    let spell = t.cast(P0, div).go();
    let rev = t.hand(P1, "Reverberate");
    t.cast(P1, rev).target(spell).go();
    let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
    t.resolve();
    let copy = *t.g.stack.last().unwrap();
    assert_ne!(copy, spell);
    assert_eq!(t.g.obj(copy).kind, ObjKind::SpellCopy);
    assert_eq!(t.g.obj(copy).controller, P1);
    assert_eq!(t.g.obj(copy).owner, P1);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), h1 + 2);
    assert_eq!(t.hand_size(P0), h0 + 2);
}

#[test]
fn a_copy_of_a_spell_returned_to_a_hand_ceases_to_exist() {
    cr!("704.5e", "707.10a");
    ruling!(
        "Venser, Shaper Savant",
        "If a copy of a spell is returned to its owner's hand, it's moved there, then it will cease to exist as a state-based action. It can't be recast."
    );
    supported("Venser, Shaper Savant");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    t.lands(P1, "Mountain", 2);
    let div = t.hand(P0, "Divination");
    let spell = t.cast(P0, div).go();
    let rev = t.hand(P1, "Reverberate");
    t.cast(P1, rev).target(spell).go();
    t.resolve();
    let copy = *t.g.stack.last().unwrap();
    let h1 = t.hand_size(P1);
    // P0's Venser returns the copy to its owner's hand: P1's.
    t.answer_targets(P0, &[Entity::Object(copy)]);
    t.enter(P0, "Venser, Shaper Savant");
    t.settle();
    t.resolve();
    assert!(!t.g.stack.contains(&copy));
    assert_eq!(t.hand_size(P1), h1, "it ceased to exist");
    assert!(t
        .player(P1)
        .hand
        .iter()
        .all(|c| t.g.obj(*c).kind == ObjKind::Card));
    t.resolve_all();
    // Only the original resolved.
    assert_eq!(t.hand_size(P1), h1);
}

#[test]
fn a_copy_of_an_adventure_isnt_exiled_on_an_adventure() {
    cr!("715.3c", "715.3d", "704.5e");
    ruling!(
        "Bonecrusher Giant // Stomp",
        "If an effect copies an Adventure spell, that copy is exiled as it resolves. It ceases to exist as a state-based action; it's not possible to cast the copy as a permanent."
    );
    supported("Bonecrusher Giant // Stomp");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    let giant = t.hand(P0, "Bonecrusher Giant // Stomp");
    let stomp = t
        .cast(P0, giant)
        .method(CastMethod::Half(1))
        .target(Entity::Player(P1))
        .go();
    let rev = t.hand(P0, "Reverberate");
    t.answer_yes(P0, false);
    t.cast(P0, rev).target(stomp).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.g.exile.len(), 1, "only the card, on an adventure");
    assert_eq!(t.g.obj(t.g.exile[0]).kind, ObjKind::Card);
}
