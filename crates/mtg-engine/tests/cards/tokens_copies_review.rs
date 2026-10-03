//! Review checks for the token and copy grammar: tokens joining combat attacking a
//! specified player or blocking a specified creature among several (CR 508.4, 509.4),
//! "that attacking player" as the active player (CR 506.2), and spell-copy exceptions
//! that become part of the copy's copiable values (CR 707.9b).

use mtg_engine::object::ObjKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn tokens(t: &TestGame, p: PlayerId, subtype: &str) -> Vec<ObjectId> {
    t.g.battlefield
        .iter()
        .copied()
        .filter(|&o| {
            let x = t.g.obj(o);
            x.kind == ObjKind::Token && x.controller == p && x.chars.has_subtype(subtype)
        })
        .collect()
}

#[test]
fn combat_calligrapher_inkling_attacks_the_attacked_opponent_not_the_first_one() {
    cr!("508.4", "506.2");
    // Four players: P3 controls the Calligrapher; P0 attacks P2. P1 (the first option
    // the attacking token could otherwise pick) must not be the one attacked.
    let mut t = TestGame::new(4);
    t.battlefield(P3, "Combat Calligrapher");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bears, Entity::Player(P2))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    t.resolve_all();
    let ink = tokens(&t, P0, "Inkling");
    assert_eq!(ink.len(), 1, "{}", t.dump_log());
    assert!(tokens(&t, P3, "Inkling").is_empty());
    let c = t.g.combat.as_ref().unwrap();
    assert_eq!(c.attack_target(ink[0]), Some(Entity::Player(P2)));
}

#[test]
fn curse_of_shallow_graves_gives_the_zombie_to_the_attacking_player() {
    cr!("506.2");
    let mut t = TestGame::new(3);
    // P1 curses P2 on P1's turn; on P0's turn P0 attacks P2.
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Swamp", 3);
    let curse = t.hand(P1, "Curse of Shallow Graves");
    t.cast(P1, curse).target(Entity::Player(P2)).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Curse of Shallow Graves").len(), 1);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bears, Entity::Player(P2))]),
    );
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::DeclareAttackers);
    t.resolve_all();
    let z = tokens(&t, P0, "Zombie");
    assert_eq!(z.len(), 1, "{}", t.dump_log());
    assert!(t.g.obj(z[0]).tapped);
    assert!(tokens(&t, P1, "Zombie").is_empty());
    assert!(tokens(&t, P2, "Zombie").is_empty());
}

#[test]
fn brimaz_cat_blocks_the_creature_brimaz_blocked_among_several_attackers() {
    cr!("509.4");
    let mut t = TestGame::new(2);
    let brimaz = t.battlefield(P0, "Brimaz, King of Oreskos");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.set_step(P1, Step::BeginningOfCombat);
    t.answer(
        P1,
        DecisionKind::Attackers,
        Answer::Attackers(vec![
            (bears, Entity::Player(P0)),
            (giant, Entity::Player(P0)),
        ]),
    );
    t.answer(
        P0,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(brimaz, giant)]),
    );
    t.advance_to(P1, Step::DeclareBlockers);
    t.resolve_all();
    let cats = tokens(&t, P0, "Cat");
    assert_eq!(cats.len(), 1, "{}", t.dump_log());
    let c = t.g.combat.as_ref().unwrap();
    assert!(c.blockers_of(giant).contains(&cats[0]));
    assert!(!c.blockers_of(bears).contains(&cats[0]));
}

#[test]
fn a_clone_of_a_nonlegendary_spell_copy_token_is_nonlegendary() {
    cr!("707.9b", "608.3f");
    // Double Major: "Copy target creature spell you control, except it isn't legendary
    // if the spell is legendary." The exception is part of the copy's copiable values,
    // so a Clone copying the resulting token isn't legendary either.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Island", 1);
    let isamaru = t.hand(P0, "Isamaru, Hound of Konda");
    let spell = t.cast(P0, isamaru).go();
    let dm = t.hand(P0, "Double Major");
    t.cast(P0, dm).target(spell).go();
    t.resolve_all();
    let token: Vec<ObjectId> = t
        .named_on_battlefield("Isamaru, Hound of Konda")
        .into_iter()
        .filter(|&o| t.g.obj(o).kind == ObjKind::Token)
        .collect();
    assert_eq!(token.len(), 1, "{}", t.dump_log());
    assert!(!t.obj_now(token[0]).copiable.is_legendary());
    t.lands(P0, "Island", 4);
    let clone = t.hand(P0, "Clone");
    t.cast(P0, clone).go();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(token[0])]);
    t.resolve_all();
    let clone = t.g.current(clone);
    assert!(t.on_battlefield(clone), "{}", t.dump_log());
    assert_eq!(t.obj_now(clone).chars.name.as_str(), "Isamaru, Hound of Konda");
    assert!(!t.obj_now(clone).chars.is_legendary());
    // All three Isamarus survive the legend rule (only the card is legendary).
    assert_eq!(t.named_on_battlefield("Isamaru, Hound of Konda").len(), 3);
}

#[test]
fn acorn_catapult_gives_a_targeted_player_the_squirrel() {
    cr!("111.2");
    // "That permanent's controller or that player creates ...": with a player target,
    // that player gets the token.
    let mut t = TestGame::new(2);
    let cat = t.battlefield(P0, "Acorn Catapult");
    t.lands(P0, "Wastes", 1);
    t.activate(P0, cat, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(tokens(&t, P1, "Squirrel").len(), 1, "{}", t.dump_log());
    assert!(tokens(&t, P0, "Squirrel").is_empty());
}

#[test]
fn spelltwine_casts_copies_of_both_exiled_cards() {
    cr!("707.12");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 6);
    for _ in 0..6 {
        t.library_top(P0, "Island");
    }
    let mine = t.graveyard(P0, "Divination");
    let theirs = t.graveyard(P1, "Divination");
    let st = t.hand(P0, "Spelltwine");
    let before = t.hand_size(P0);
    t.cast(P0, st)
        .targets(&[Entity::Object(mine), Entity::Object(theirs)])
        .go();
    t.resolve_all();
    // Spelltwine left the hand; each copy of Divination drew two cards.
    assert_eq!(t.hand_size(P0), before - 1 + 4, "{}", t.dump_log());
    assert!(t.in_exile("Spelltwine"));
    assert_eq!(t.g.exile.iter().filter(|&&o| t.g.obj(o).chars.name.as_str() == "Divination").count(), 2);
}

#[test]
fn ajanis_chosen_cant_move_an_aura_that_cant_enchant_the_cat() {
    cr!("303.4", "701.3b");
    ruling!(
        "Ajani's Chosen",
        "it won't become attached to that token"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ajani's Chosen");
    let forest = t.lands(P0, "Forest", 1)[0];
    let wg = t.hand(P0, "Wild Growth");
    t.cast(P0, wg).target(forest).go();
    t.answer_yes(P0, true);
    t.resolve_all();
    let cat = tokens(&t, P0, "Cat");
    assert_eq!(cat.len(), 1, "{}", t.dump_log());
    let aura = t.named_on_battlefield("Wild Growth");
    assert_eq!(aura.len(), 1, "{}", t.dump_log());
    assert_eq!(t.obj_now(aura[0]).attached_to, Some(Entity::Object(forest)));
}
