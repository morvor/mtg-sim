//! CR 407.3-407.4: ante cards that ante an object from whichever zone it's in, and that
//! exchange or change the ownership of cards (Jeweled Bird, Tempest Efreet, Timmerian
//! Fiends, Bronze Tablet, Rebirth, Amulet of Quoz).

use crate::r703_common::{run_effect, supported};
use mtg_engine::ability::*;
use mtg_engine::ante;
use mtg_engine::card::CardDef;
use mtg_engine::events::MoveCause;
use mtg_engine::game::GameConfig;
use mtg_engine::object::{Characteristics, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;
use smol_str::SmolStr;
use std::sync::Arc;

fn ante_game() -> TestGame {
    TestGame::with_config(
        2,
        GameConfig {
            ante: true,
            ..Default::default()
        },
    )
}

fn stake(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    t.custom(
        p,
        CardDef::custom(Characteristics {
            name: SmolStr::new(name),
            rules_text: Arc::from(""),
            ..Default::default()
        }),
        Zone::Ante,
    )
}

#[test]
fn jeweled_bird_antes_itself_and_replaces_your_whole_contribution() {
    cr!("407.3", "407.4");
    ruling!(
        "Jeweled Bird",
        "The card is exchanged for your entire contribution to the ante. This means that it replaces all the cards if you have more than one already contributed."
    );
    supported("Jeweled Bird");
    // "{T}: Ante this artifact. If you do, put all other cards you own from the ante into
    // your graveyard, then draw a card."
    let mut t = ante_game();
    let a = stake(&mut t, P0, "Stake A");
    let b = stake(&mut t, P0, "Stake B");
    let theirs = stake(&mut t, P1, "Their Stake");
    t.library_top(P0, "Grizzly Bears");
    let bird = t.battlefield(P0, "Jeweled Bird");
    t.activate(P0, bird, 0, &[]).unwrap();
    t.resolve_all();
    let bird_now = t.g.current(bird);
    assert_eq!(t.zone(bird_now), Zone::Ante);
    assert_eq!(t.zone(a), Zone::Graveyard(P0));
    assert_eq!(t.zone(b), Zone::Graveyard(P0));
    assert_eq!(t.zone(theirs), Zone::Ante);
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert_eq!(t.g.ante.len(), 2);
}

#[test]
fn only_its_owner_can_ante_jeweled_bird() {
    cr!("407.4");
    // P1 controls P0's Jeweled Bird: P1 can't ante it, so nothing else happens.
    let mut t = ante_game();
    let mine = stake(&mut t, P1, "Their Stake");
    t.library_top(P1, "Grizzly Bears");
    let bird = t.battlefield(P0, "Jeweled Bird");
    t.g.objects[bird.0 as usize].base_controller = P1;
    t.g.recompute();
    let hand = t.hand_size(P1);
    t.activate(P1, bird, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(bird));
    assert_eq!(t.zone(mine), Zone::Ante);
    assert_eq!(t.hand_size(P1), hand);
}

/// P0 activates Tempest Efreet targeting P1, whose hand is one Hill Giant: "Target opponent
/// may pay 10 life. If that player doesn't, they reveal a card at random from their hand.
/// Exchange ownership of the revealed card and Tempest Efreet. Put the revealed card into
/// your hand and Tempest Efreet from anywhere into that player's graveyard."
fn run_efreet(pays: bool) -> (TestGame, ObjectId, ObjectId) {
    let mut t = ante_game();
    let giant = t.hand(P1, "Hill Giant");
    let efreet = t.battlefield(P0, "Tempest Efreet");
    t.answer_yes(P1, pays);
    t.activate(P0, efreet, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    (t, efreet, giant)
}

#[test]
fn tempest_efreet_exchanges_ownership_of_the_revealed_card_and_itself() {
    cr!("407.3", "108.3");
    supported("Tempest Efreet");
    let (t, efreet, giant) = run_efreet(false);
    let giant_now = t.g.current(giant);
    assert_eq!(t.zone(giant_now), Zone::Hand(P0));
    assert_eq!(t.obj(giant_now).owner, P0);
    let efreet_now = t.g.current(efreet);
    assert_eq!(t.zone(efreet_now), Zone::Graveyard(P1));
    assert_eq!(t.obj(efreet_now).owner, P1);
    // Paying 10 life, nothing changes hands.
    let (t, efreet, giant) = run_efreet(true);
    assert_eq!(t.life(P1), 10);
    assert_eq!(t.zone(t.g.current(giant)), Zone::Hand(P1));
    let efreet_now = t.g.current(efreet);
    assert_eq!(t.zone(efreet_now), Zone::Graveyard(P0));
    assert_eq!(t.obj(efreet_now).owner, P0);
}

/// P0 activates Timmerian Fiends targeting P1's Ornithopter: "The owner of target artifact
/// may ante the top card of their library. If that player doesn't, exchange ownership of
/// that artifact and Timmerian Fiends. Put the artifact card into your graveyard and
/// Timmerian Fiends from anywhere into that player's graveyard."
fn run_fiends(antes: bool) -> (TestGame, ObjectId, ObjectId, ObjectId) {
    let mut t = ante_game();
    let top = t.library_top(P1, "Grizzly Bears");
    let thopter = t.battlefield(P1, "Ornithopter");
    let fiends = t.battlefield(P0, "Timmerian Fiends");
    t.lands(P0, "Swamp", 3);
    t.answer_yes(P1, antes);
    t.activate(P0, fiends, 0, &[Entity::Object(thopter)]).unwrap();
    t.resolve_all();
    (t, fiends, thopter, top)
}

#[test]
fn timmerian_fiends_exchanges_ownership_unless_the_owner_antes() {
    cr!("407.3", "407.4");
    supported("Timmerian Fiends");
    let (t, fiends, thopter, top) = run_fiends(false);
    let thopter_now = t.g.current(thopter);
    assert_eq!(t.zone(thopter_now), Zone::Graveyard(P0));
    assert_eq!(t.obj(thopter_now).owner, P0);
    let fiends_now = t.g.current(fiends);
    assert_eq!(t.zone(fiends_now), Zone::Graveyard(P1));
    assert_eq!(t.obj(fiends_now).owner, P1);
    assert_eq!(t.zone(top), Zone::Library(P1));
    // The owner antes the top card of their library instead.
    let (t, fiends, thopter, top) = run_fiends(true);
    assert_eq!(t.zone(top), Zone::Ante);
    assert!(t.on_battlefield(thopter));
    assert_eq!(t.obj(thopter).owner, P1);
    assert_eq!(t.zone(t.g.current(fiends)), Zone::Graveyard(P0));
}

/// P0 activates Bronze Tablet targeting P1's Grizzly Bears: "Exile this artifact and target
/// nontoken permanent an opponent owns. That player may pay 10 life. If they do, put this
/// card into its owner's graveyard. Otherwise, that player owns this card and you own the
/// other exiled card."
fn run_tablet(t: &mut TestGame, pays: bool, before: impl FnOnce(&mut TestGame, ObjectId)) -> (ObjectId, ObjectId) {
    let bears = t.battlefield(P1, "Grizzly Bears");
    let tablet = t.battlefield(P0, "Bronze Tablet");
    t.g.objects[tablet.0 as usize].tapped = false;
    t.lands(P0, "Island", 4);
    t.answer_yes(P1, pays);
    t.activate(P0, tablet, 0, &[Entity::Object(bears)]).unwrap();
    before(t, tablet);
    t.resolve_all();
    (tablet, bears)
}

#[test]
fn bronze_tablet_trades_ownership_unless_its_target_owner_pays_10_life() {
    cr!("407.3");
    supported("Bronze Tablet");
    let mut t = ante_game();
    let (tablet, bears) = run_tablet(&mut t, false, |_, _| {});
    let tablet_now = t.g.current(tablet);
    let bears_now = t.g.current(bears);
    assert_eq!(t.zone(tablet_now), Zone::Exile);
    assert_eq!(t.zone(bears_now), Zone::Exile);
    assert_eq!(t.obj(tablet_now).owner, P1);
    assert_eq!(t.obj(bears_now).owner, P0);
    // Paying 10 life: Bronze Tablet goes to its owner's graveyard; the other card stays
    // exiled with its owner.
    let mut t = ante_game();
    let (tablet, bears) = run_tablet(&mut t, true, |_, _| {});
    assert_eq!(t.life(P1), 10);
    let tablet_now = t.g.current(tablet);
    assert_eq!(t.zone(tablet_now), Zone::Graveyard(P0));
    assert_eq!(t.obj(tablet_now).owner, P0);
    let bears_now = t.g.current(bears);
    assert_eq!(t.zone(bears_now), Zone::Exile);
    assert_eq!(t.obj(bears_now).owner, P1);
}

#[test]
fn bronze_tablet_off_the_battlefield_isnt_exiled_but_you_still_get_their_card() {
    cr!("407.3", "608.2b");
    ruling!(
        "Bronze Tablet",
        "If the tablet is not still on the battlefield when the ability resolves, it is not exiled. The other player still has the choice to pay 10 life, and you still become the owner of their card if they choose not to do so."
    );
    let mut t = ante_game();
    let (tablet, bears) = run_tablet(&mut t, false, |t, tablet| {
        t.g.move_object(tablet, Zone::Hand(P0), MoveCause::Effect, None);
    });
    let tablet_now = t.g.current(tablet);
    assert_eq!(t.zone(tablet_now), Zone::Hand(P0));
    let bears_now = t.g.current(bears);
    assert_eq!(t.zone(bears_now), Zone::Exile);
    assert_eq!(t.obj(bears_now).owner, P0);
}

#[test]
fn bronze_tablet_a_player_with_less_than_10_life_cant_pay() {
    cr!("407.3", "119.4");
    ruling!(
        "Bronze Tablet",
        "You can't choose to pay 10 life if you have less than 10 life"
    );
    let mut t = ante_game();
    t.g.players[1].life = 9;
    let (_, bears) = run_tablet(&mut t, true, |_, _| {});
    assert_eq!(t.life(P1), 9);
    assert_eq!(t.obj(t.g.current(bears)).owner, P0);
}

#[test]
fn rebirth_sets_the_life_of_each_player_who_antes() {
    cr!("407.3", "407.4");
    supported("Rebirth");
    // "Each player may ante the top card of their library. If a player does, that
    // player's life total becomes 20."
    let mut t = ante_game();
    t.g.players[0].life = 5;
    t.g.players[1].life = 7;
    let mine = t.library_top(P0, "Grizzly Bears");
    let theirs = t.library_top(P1, "Hill Giant");
    t.lands(P0, "Forest", 6);
    let rebirth = t.hand(P0, "Rebirth");
    t.cast(P0, rebirth).go();
    t.answer_yes(P0, true);
    t.answer_yes(P1, false);
    t.resolve();
    assert_eq!(t.zone(mine), Zone::Ante);
    assert_eq!(t.zone(theirs), Zone::Library(P1));
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 7);
}

#[test]
fn amulet_of_quoz_flips_a_coin_unless_the_opponent_antes() {
    cr!("407.3", "705.1");
    supported("Amulet of Quoz");
    // "Target opponent may ante the top card of their library. If they don't, you flip a
    // coin. If you win the flip, that player loses the game. If you lose the flip, you
    // lose the game. Activate only during your upkeep."
    let setup = |antes: bool| {
        let mut t = ante_game();
        t.set_step(P0, Step::Upkeep);
        let top = t.library_top(P1, "Grizzly Bears");
        let amulet = t.battlefield(P0, "Amulet of Quoz");
        t.answer_yes(P1, antes);
        t.activate(P0, amulet, 0, &[Entity::Player(P1)]).unwrap();
        t.resolve_all();
        (t, top)
    };
    let (t, top) = setup(true);
    assert_eq!(t.zone(top), Zone::Ante);
    assert!(!t.has_lost(P0) && !t.has_lost(P1));
    let (t, top) = setup(false);
    assert_eq!(t.zone(top), Zone::Library(P1));
    // Exactly one of the players lost.
    assert!(t.has_lost(P0) != t.has_lost(P1), "{}", t.dump_log());
}

#[test]
fn only_ante_cards_exchange_ownership() {
    cr!("407.3");
    let mut t = ante_game();
    let giant = t.hand(P1, "Hill Giant");
    let other = t.battlefield(P0, "Grizzly Bears");
    run_effect(
        &mut t,
        P0,
        Some(other),
        Effect::Seq(vec![
            Effect::Store {
                var: vars::IT,
                sel: Sel::All(Filter::InZone(ZoneKind::Hand)),
            },
            Effect::Custom(ante::EXCHANGE_OWNERSHIP.into()),
        ]),
        &[],
    );
    assert_eq!(t.obj(giant).owner, P1);
    assert_eq!(t.obj(other).owner, P0);
}
