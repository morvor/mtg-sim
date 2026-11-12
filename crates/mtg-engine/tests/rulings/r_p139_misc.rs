//! Rulings batch P139 — the rest: what a resolving effect does in which order (Living End,
//! Scrap Mastery, Collision of Realms, Hallowed Burial, Terminus, Undercity's Embrace),
//! choices made during resolution (Dark Intimations, Infernal Offering, Liliana's piles,
//! Spawnsire of Ulamog, The Grim Captain), when abilities can be activated (Liliana,
//! Sheoldred, Ajani), Reweave, villainous choices, phasing (Teferi) and Weeping Angel.

use crate::r_p137_common::{choosers_since, obj};
use crate::r_s01_common::{attack_with, give_mana_for, supported, tokens};
use crate::r_s02_common::{can_attack, can_play_land, create_token, destroy};
use crate::r_s04_common::{add_mana, top_of_stack};
use crate::r_s10_common::attacking;
use crate::r_s25_common::cast_new;
use crate::r_s33_common::shuffled_since;
use mtg_engine::ability::{AbilityKind, Effect};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

fn count_of(t: &TestGame, p: PlayerId, ty: CardType) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is(ty))
        .count()
}

/// The spell effect of the real card `name`.
fn spell_effect(name: &str) -> Effect {
    mtg_engine::card::card(name)
        .front()
        .chars
        .abilities
        .iter()
        .find_map(|a| match &a.kind {
            AbilityKind::Spell(s) => Some(s.body.effect.clone()),
            _ => None,
        })
        .expect("a spell ability")
}

// ---------------------------------------------------------------------------------------
// Each player does A, then B, then C
// ---------------------------------------------------------------------------------------

#[test]
fn living_end_exiles_then_sacrifices_then_returns() {
    cr!("608.2c", "701.21a");
    ruling!(
        "Living End",
        "While Living End is resolving, first each player exiles their graveyard at the same time, then each player sacrifices all creatures they control at the same time, and finally each player puts their exiled cards onto the battlefield at the same time."
    );
    supported("Living End");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lions = t.battlefield(P1, "Savannah Lions");
    t.graveyard(P0, "Hill Giant");
    t.graveyard(P1, "Craw Wurm");
    let le = t.exile(P0, "Living End");
    crate::r_s03_common::run_effect(&mut t, Some(le), P0, spell_effect("Living End"), &[]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    assert_eq!(t.named_on_battlefield("Craw Wurm").len(), 1);
    assert!(!t.on_battlefield(bears) && !t.on_battlefield(lions));
    // The sacrificed creatures stay in the graveyards.
    assert!(t.in_graveyard(P0, "Grizzly Bears") && t.in_graveyard(P1, "Savannah Lions"));
}

#[test]
fn scrap_mastery_returns_only_the_cards_its_exile_instruction_exiled() {
    cr!("614.1a", "608.2c");
    ruling!(
        "Scrap Mastery",
        "“Exiled this way” refers to the exile instruction that’s part of Scrap Mastery’s effect. If the sacrificed artifacts are instead exiled because of a replacement effect, those cards won’t be put onto the battlefield."
    );
    supported("Scrap Mastery");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Rest in Peace");
    let ring = t.battlefield(P0, "Sol Ring");
    t.graveyard(P0, "Mind Stone");
    cast_new(&mut t, P0, "Scrap Mastery", &[]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Mind Stone").len(), 1);
    assert!(t.named_on_battlefield("Sol Ring").is_empty());
    assert_eq!(t.zone(t.g.current(ring)), Zone::Exile);
}

#[test]
fn hallowed_burial_puts_every_creature_on_the_bottom_at_once() {
    cr!("701.8a", "111.7", "702.12b");
    ruling!(
        "Hallowed Burial",
        "All creatures leave the battlefield simultaneously. They’re not destroyed; they’re just put on the bottom of their owners’ libraries. This includes token creatures, though they’ll cease to exist as soon as Hallowed Burial finishes resolving."
    );
    supported("Hallowed Burial");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let myr = t.battlefield(P1, "Darksteel Myr");
    create_token(&mut t, P1, "Soldier");
    cast_new(&mut t, P0, "Hallowed Burial", &[]);
    t.resolve_all();
    assert_eq!(t.g.permanents().filter(|o| o.is(CardType::Creature)).count(), 0);
    assert_eq!(t.g.players[0].library.first().copied(), Some(t.g.current(bears)));
    assert_eq!(t.g.players[1].library.first().copied(), Some(t.g.current(myr)));
    assert!(tokens(&t, P1).is_empty());
    assert_eq!(t.library_size(P1), 31, "the token ceased to exist");
}

#[test]
fn terminus_each_owner_orders_their_creatures() {
    cr!("401.4");
    ruling!(
        "Terminus",
        "Each player chooses the order that creatures they own are put onto the bottom of their library. This order isn't revealed to other players."
    );
    supported("Terminus");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.battlefield(P0, "Savannah Lions");
    let from = t.asked().len();
    cast_new(&mut t, P0, "Terminus", &[]);
    t.resolve_all();
    let orderers: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::Order { .. } | Decision::PutOnBottom { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(orderers, vec![P1], "P1 orders its two creatures; P0 has only one");
    let bottom: Vec<ObjectId> = t.g.players[1].library[..2].to_vec();
    assert!(bottom.contains(&t.g.current(a)) && bottom.contains(&t.g.current(b)));
}

#[test]
fn collision_of_realms_one_creature_each_triggers_in_apnap_order() {
    cr!("701.24a", "101.4", "603.3b");
    ruling!(
        "Collision of Realms",
        "All creatures are shuffled into their owners' libraries at the same time. Then all players who shuffled nontoken creatures into their library this way reveal cards. Then all creature cards revealed this way enter the battlefield at the same time. Then all other cards are put on the bottom of their owners' libraries. Any abilities that triggered during this process are then put on the stack, starting by those controlled by the active player in an order of that player's choice, then continuing with each other player's abilities in turn order. The last player to put their abilities on the stack will be the first player to have their abilities resolve. A player who controls no creatures or who controls only creature tokens will still shuffle their library."
    );
    ruling!(
        "Collision of Realms",
        "Each player reveals and puts onto the battlefield one creature card or fewer. They don't reveal a card for each nontoken creature they shuffle into their library; it's just one creature card at most."
    );
    supported("Collision of Realms");
    let mut t = TestGame::new(4);
    // P0 shuffles two Elvish Visionaries (the only creature cards in its library then);
    // P2 one; P1 only a token; P3 nothing.
    t.battlefield(P0, "Elvish Visionary");
    t.battlefield(P0, "Elvish Visionary");
    t.battlefield(P2, "Elvish Visionary");
    create_token(&mut t, P1, "Soldier");
    let from_ev = t.g.turn_events.len();
    cast_new(&mut t, P0, "Collision of Realms", &[]);
    t.resolve();
    for p in [P0, P1, P2, P3] {
        assert!(shuffled_since(&t, from_ev, p), "{p:?} shuffled");
    }
    let visionaries = |p: PlayerId| {
        t.g.permanents()
            .filter(|o| o.controller == p && o.chars.name == "Elvish Visionary")
            .count()
    };
    assert_eq!(visionaries(P0), 1, "one creature card at most");
    assert_eq!(visionaries(P2), 1);
    assert!(tokens(&t, P1).is_empty());
    // The two enters triggers: P0's (the active player's) below P2's.
    assert_eq!(t.stack_len(), 2);
    assert_eq!(t.g.obj(top_of_stack(&t)).controller, P2);
    assert_eq!(t.g.obj(t.g.stack[0]).controller, P0);
}

#[test]
fn undercitys_embrace_checks_power_after_the_sacrifice() {
    cr!("608.2c", "610.3", "603.3");
    ruling!(
        "Undercity's Embrace",
        "Whether you control a creature with power 4 or greater is checked only after your opponent has sacrificed a creature. If a creature you own with power 4 or greater was exiled until the sacrificed creature left the battlefield, that creature will return to the battlefield before the check, so it will count. However, any abilities that trigger on the sacrificed creature’s death won’t have resolved yet when you check what creatures you control."
    );
    supported("Undercity's Embrace");
    // Banisher Priest exiled P0's Craw Wurm until it leaves the battlefield.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Craw Wurm");
    t.answer_targets(P1, &[obj(wurm)]);
    t.enter(P1, "Banisher Priest");
    t.resolve_all();
    assert!(t.named_on_battlefield("Craw Wurm").is_empty());
    cast_new(&mut t, P0, "Undercity's Embrace", &[Entity::Player(P1)]);
    t.resolve();
    assert_eq!(t.named_on_battlefield("Craw Wurm").len(), 1);
    assert_eq!(t.life(P0), 24);
    // Perilous Myr's dies trigger (2 damage to any target) hasn't resolved yet: P0's Ball
    // Lightning (6/1) still counts.
    let mut t = TestGame::new(2);
    let ball = t.battlefield(P0, "Ball Lightning");
    t.battlefield(P1, "Perilous Myr");
    t.answer_targets(P1, &[obj(ball)]);
    cast_new(&mut t, P0, "Undercity's Embrace", &[Entity::Player(P1)]);
    t.resolve();
    assert_eq!(t.life(P0), 24);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(!t.on_battlefield(ball));
}

// ---------------------------------------------------------------------------------------
// Choices made while resolving
// ---------------------------------------------------------------------------------------

#[test]
fn dark_intimations_returns_the_card_chosen_as_it_resolves() {
    cr!("608.2c", "608.2d");
    ruling!(
        "Dark Intimations",
        "You choose the creature or planeswalker card to return to your hand while Dark Intimations is resolving. No player may take actions between the time you choose one and the time you put it into your hand."
    );
    supported("Dark Intimations");
    let mut t = TestGame::new(2);
    let giant = t.graveyard(P0, "Hill Giant");
    t.graveyard(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(giant)]);
    let from = t.asked().len();
    cast_new(&mut t, P0, "Dark Intimations", &[]);
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseEntities { .. } | Decision::ChooseTargets { .. })));
    // The returned card is chosen with the spell resolving; no priority in between.
    let seen = crate::r_s01_common::watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseEntities { .. }),
        |g| g.stack.len(),
    );
    t.resolve();
    assert!(t.in_hand(P0, "Hill Giant") && t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(*seen.lock().unwrap(), vec![1], "chosen while the spell is resolving");
}

#[test]
fn infernal_offering_you_choose_first_then_the_opponent_without_targets() {
    cr!("608.2c", "101.4", "115.1");
    ruling!(
        "Infernal Offering",
        "You choose which creature card you’re returning to the battlefield, then the chosen opponent chooses which creature card they are returning. These choices aren’t made until Infernal Offering resolves. Neither card is a target of Infernal Offering."
    );
    supported("Infernal Offering");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Savannah Lions");
    let mine = t.graveyard(P0, "Hill Giant");
    t.graveyard(P0, "Elvish Visionary");
    let theirs = t.graveyard(P1, "Craw Wurm");
    t.graveyard(P1, "Llanowar Elves");
    let from = t.asked().len();
    cast_new(&mut t, P0, "Infernal Offering", &[]);
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseTargets { .. })));
    // The sacrifices put the Bears and the Lions into the graveyards first.
    t.answer_choose(P0, &[obj(mine)]);
    t.answer_choose(P1, &[obj(theirs)]);
    let from = t.asked().len();
    t.resolve_all();
    let mut order = choosers_since(&t, from);
    order.dedup();
    assert_eq!(order[order.len() - 2..], [P0, P1]);
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    assert_eq!(t.named_on_battlefield("Craw Wurm").len(), 1);
}

#[test]
fn liliana_of_the_veil_piles_can_split_an_aura_from_its_creature() {
    cr!("700.3", "303.4c");
    ruling!(
        "Liliana of the Veil",
        "When Liliana’s third ability resolves, you put each permanent the player controls into one of the two piles. For example, you could put a creature into one pile and an Aura enchanting that creature into the other pile."
    );
    supported("Liliana of the Veil");
    let mut t = TestGame::new(2);
    let lili = t.battlefield(P0, "Liliana of the Veil");
    t.g.objects[lili.0 as usize]
        .counters
        .insert("loyalty".into(), 7);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let aura = t.battlefield(P1, "Holy Strength");
    t.g.attach(aura, Entity::Object(bears));
    // Pile one: the Bears; pile two: the Aura. P1 sacrifices the Aura's pile.
    t.answer_choose(P0, &[obj(bears)]);
    t.answer(P1, DecisionKind::Option, Answer::Index(1));
    t.activate(P0, lili, 2, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert!(t.in_graveyard(P1, "Holy Strength"));
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn spawnsire_casts_eldrazi_during_resolution_then_they_resolve_in_turn() {
    cr!("608.2g", "601.2", "307.1");
    interp!("outside-the-game-is-the-sideboard");
    ruling!(
        "Spawnsire of Ulamog",
        "You cast Eldrazi cards from outside the game as part of the resolution of Spawnsire of Ulamog's last ability. You may cast those cards in any order. Timing restrictions based on the card's type (such as creature or sorcery) are ignored. Other restrictions are not (such as the one from Rule of Law). You cast all of the cards you like, putting them onto the stack, then the ability finishes resolving. The spells you cast this way will then resolve as normal, one at a time, in the opposite order that they were put on the stack. They'll remain in the game for the remainder of that game."
    );
    supported("Spawnsire of Ulamog");
    let mut t = TestGame::new(2);
    // On P1's turn: creature spells are cast anyway.
    t.set_step(P1, Step::Upkeep);
    let spawnsire = t.battlefield(P0, "Spawnsire of Ulamog");
    let side = t.g.add_to_sideboard(
        P0,
        vec![
            mtg_engine::card::card("Eldrazi Devastator"),
            mtg_engine::card::card("Ulamog's Crusher"),
        ],
    );
    add_mana(&mut t, P0, ManaType::C, 20);
    t.answer_choose(P0, &[obj(side[0]), obj(side[1])]);
    t.activate(P0, spawnsire, 1, &[]).unwrap();
    t.resolve();
    assert_eq!(t.stack_len(), 2, "both cast, neither resolved");
    t.resolve();
    assert_eq!(t.named_on_battlefield("Eldrazi Devastator").len() + t.named_on_battlefield("Ulamog's Crusher").len(), 1);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Eldrazi Devastator").len(), 1);
    assert_eq!(t.named_on_battlefield("Ulamog's Crusher").len(), 1);
    // Rule of Law: only one of them can be cast.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Rule of Law");
    let spawnsire = t.battlefield(P0, "Spawnsire of Ulamog");
    let side = t.g.add_to_sideboard(
        P0,
        vec![
            mtg_engine::card::card("Eldrazi Devastator"),
            mtg_engine::card::card("Ulamog's Crusher"),
        ],
    );
    add_mana(&mut t, P0, ManaType::C, 20);
    t.answer_choose(P0, &[obj(side[0]), obj(side[1])]);
    t.activate(P0, spawnsire, 1, &[]).unwrap();
    t.resolve_all();
    assert_eq!(
        t.named_on_battlefield("Eldrazi Devastator").len()
            + t.named_on_battlefield("Ulamog's Crusher").len(),
        1
    );
}

// ---------------------------------------------------------------------------------------
// When abilities can be activated
// ---------------------------------------------------------------------------------------

#[test]
fn activation_conditions() {
    cr!("606.3", "602.5b");
    ruling!(
        "Liliana of the Veil",
        "You can activate Liliana’s first ability even if some or all players will be unable to discard a card."
    );
    supported("Liliana of the Veil");
    let mut t = TestGame::new(2);
    let lili = t.battlefield(P0, "Liliana of the Veil");
    assert!(t.activate(P0, lili, 0, &[]).is_ok());
    t.resolve_all();
    assert_eq!(t.counters(lili, "loyalty"), 4);

    ruling!(
        "Sheoldred // The True Scriptures",
        "You can activate Sheoldred’s last ability if any opponent has eight or more cards in their graveyard, not necessarily all of them."
    );
    supported("Sheoldred // The True Scriptures");
    for (n, ok) in [(7, false), (8, true)] {
        let mut t = TestGame::new(3);
        let sheoldred = t.battlefield(P0, "Sheoldred // The True Scriptures");
        for _ in 0..n {
            t.graveyard(P1, "Island");
        }
        add_mana(&mut t, P0, ManaType::B, 1);
        add_mana(&mut t, P0, ManaType::C, 4);
        assert_eq!(t.activate(P0, sheoldred, 0, &[]).is_ok(), ok, "{n} cards");
    }
}

#[test]
fn ajani_avenger_loyalty_abilities_the_turn_he_enters_only_at_sorcery_speed() {
    cr!("606.3", "307.1");
    ruling!(
        "Ajani, Nacatl Pariah // Ajani, Nacatl Avenger",
        "You can activate one of Ajani, Nacatl Avenger's loyalty abilities the turn he enters the battlefield. However, you may do so only during one of your main phases when the stack is empty. For example, if Ajani, Nacatl Avenger enters the battlefield during combat, there will be an opportunity for your opponent to remove him before you can activate one of his abilities."
    );
    const AJANI: &str = "Ajani, Nacatl Pariah // Ajani, Nacatl Avenger";
    supported(AJANI);
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::BeginningOfCombat);
    let card = t.graveyard(P0, AJANI);
    let ajani = crate::r_s17_common::put_onto_battlefield(&mut t, P0, card, true).unwrap();
    assert!(t.activate(P0, ajani, 0, &[]).is_err(), "during combat");
    t.advance_to(P0, Step::PostcombatMain);
    // A spell on the stack.
    let bolt = t.hand(P1, "Lightning Bolt");
    t.lands(P1, "Mountain", 1);
    t.cast_with(P1, bolt, &[Entity::Player(P0)]).unwrap();
    assert!(t.activate(P0, ajani, 0, &[]).is_err(), "the stack isn't empty");
    t.resolve_all();
    assert!(t.activate(P0, ajani, 0, &[]).is_ok(), "main phase, empty stack");
}

#[test]
fn ajani_avenger_zero_ability_targets_with_a_reflexive_trigger() {
    cr!("603.12", "115.1");
    ruling!(
        "Ajani, Nacatl Pariah // Ajani, Nacatl Avenger",
        "You don't choose a target for Ajani, Nacatl Avenger's second ability when you activate it. Rather, if you control a red permanent other than Ajani, Nacatl Avenger, a second \"reflexive\" ability triggers when you create a Cat Warrior creature token this way. You choose a target for that ability as it goes on the stack. Each player may respond to this triggered ability as normal."
    );
    const AJANI: &str = "Ajani, Nacatl Pariah // Ajani, Nacatl Avenger";
    supported(AJANI);
    for red in [true, false] {
        let mut t = TestGame::new(2);
        let card = t.graveyard(P0, AJANI);
        let ajani = crate::r_s17_common::put_onto_battlefield(&mut t, P0, card, true).unwrap();
        if red {
            t.battlefield(P0, "Raging Goblin");
        }
        let from = t.asked().len();
        t.activate(P0, ajani, 1, &[]).unwrap();
        assert!(!t.asked()[from..]
            .iter()
            .any(|(_, d)| matches!(d, Decision::ChooseTargets { .. })));
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.resolve();
        assert_eq!(tokens(&t, P0).len(), 1);
        if red {
            assert_eq!(t.stack_len(), 1, "the reflexive trigger, which can be responded to");
            t.resolve_all();
            // Raging Goblin and the token.
            assert_eq!(t.life(P1), 18);
        } else {
            assert_eq!(t.stack_len(), 0);
            assert_eq!(t.life(P1), 20);
        }
    }
}

#[test]
fn limited_resources_stops_land_plays_not_lands_put_onto_the_battlefield() {
    cr!("305.2", "305.4");
    ruling!(
        "Limited Resources",
        "You can put lands onto the battlefield using effects, you just can’t play lands from your hand as an action."
    );
    supported("Limited Resources");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Limited Resources");
    t.lands(P0, "Forest", 5);
    t.lands(P1, "Island", 5);
    let land = t.hand(P0, "Forest");
    assert!(!can_play_land(&mut t, P0, land));
    t.library_top(P0, "Plains");
    cast_new(&mut t, P0, "Rampant Growth", &[]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Plains").len(), 1);
}

#[test]
fn rootcast_apprenticeship_modes_need_legal_targets_and_may_repeat() {
    cr!("700.2a", "700.2d", "601.2c");
    ruling!(
        "Rootcast Apprenticeship",
        "You can select a mode only if there's a legal target available for that mode. Ignore the targeting requirements for modes you don't choose. Each time you select that mode, you can choose a different target, or you can choose the same target."
    );
    supported("Rootcast Apprenticeship");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_mana_for(&mut t, P0, "Rootcast Apprenticeship");
    let card = t.hand(P0, "Rootcast Apprenticeship");
    // No token to copy: that mode can't be chosen. The same mode twice, with the same
    // target.
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0, 0, 2]));
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, card).go();
    let available = t.asked()[from..]
        .iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseModes { available, .. } => Some(available.clone()),
            _ => None,
        })
        .unwrap();
    assert!(!available.contains(&1), "{available:?}");
    assert!(available.contains(&0) && available.contains(&2) && available.contains(&3));
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 4);
    assert_eq!(tokens(&t, P1).len(), 1);
}

// ---------------------------------------------------------------------------------------
// Reweave
// ---------------------------------------------------------------------------------------

#[test]
fn reweave_reveals_only_after_a_sacrifice_and_finds_a_permanent_card() {
    cr!("701.21a", "110.4b", "308.1");
    ruling!(
        "Reweave",
        "The player reveals cards only if the targeted permanent gets sacrificed."
    );
    supported("Reweave");
    // Sigarda: "Spells and abilities your opponents control can't cause you to sacrifice
    // permanents."
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Sigarda, Host of Herons");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.library_top(P1, "Hill Giant");
    cast_new(&mut t, P0, "Reweave", &[obj(bears)]);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.zone(giant), Zone::Library(P1));
    assert!(crate::r_s31_common::revealed_cards(&t).is_empty());

    ruling!(
        "Reweave",
        "With the introduction of the kindred type it became possible for an instant or sorcery card to share a type with the sacrificed permanent (ie, a kindred instant card shares a type with a kindred enchantment card). Reweave has received errata so that the effect looks for a *permanent* card that shares a type with the sacrificed permanent. A permanent card is a card that could be put onto the battlefield, which is any card that isn't an instant or sorcery."
    );
    // Bitterblossom (Kindred Enchantment — Faerie); Crib Swap (Kindred Instant) on top,
    // then Glorious Anthem.
    let mut t = TestGame::new(2);
    let bb = t.battlefield(P1, "Bitterblossom");
    t.library_top(P1, "Glorious Anthem");
    t.library_top(P1, "Crib Swap");
    cast_new(&mut t, P0, "Reweave", &[obj(bb)]);
    t.resolve_all();
    assert!(!t.on_battlefield(bb));
    assert_eq!(t.named_on_battlefield("Glorious Anthem").len(), 1);
    assert!(t.named_on_battlefield("Crib Swap").is_empty());
}

#[test]
fn ulamogs_dreadsire_face_down_permanents_cant_pay_its_ward() {
    cr!("708.2a", "702.21a", "202.3");
    ruling!("Ulamog's Dreadsire", "The mana value of a face-down permanent is 0.");
    supported("Ulamog's Dreadsire");
    let mut t = TestGame::new(2);
    let dreadsire = t.battlefield(P0, "Ulamog's Dreadsire");
    let hidden = crate::r_s11_common::manifest_card(&mut t, P1, "Craw Wurm");
    assert_eq!(crate::r_s08_common::mana_value(&t, hidden), 0);
    // P1 can't pay the ward cost with it: the Bolt is countered.
    t.answer_yes(P1, true);
    t.answer(P1, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_choose(P1, &[obj(hidden)]);
    cast_new(&mut t, P1, "Lightning Bolt", &[obj(dreadsire)]);
    t.resolve_all();
    assert!(t.on_battlefield(hidden));
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    assert_eq!(crate::r_s07_common::damage_on(&t, dreadsire), 0);
}

// ---------------------------------------------------------------------------------------
// Combat
// ---------------------------------------------------------------------------------------

/// P0 crews and attacks P1 with Midnight Crusader Shuttle; P1 lets P0 gain control of
/// `stolen`. Returns the shuttle.
fn shuttle_steals(t: &mut TestGame, stolen: ObjectId) -> ObjectId {
    let shuttle = t.battlefield(P0, "Midnight Crusader Shuttle");
    let crew = t.battlefield(P0, "Grizzly Bears");
    assert!(crate::r_s04_common::crew(t, P0, shuttle, &[crew]));
    t.answer(P1, DecisionKind::Option, Answer::Index(1));
    t.answer_choose(P0, &[obj(stolen)]);
    attack_with(t, &[(shuttle, Entity::Player(P1))]);
    t.resolve_all();
    shuttle
}

#[test]
fn midnight_crusader_shuttle_stolen_creature_attacks_without_being_declared() {
    cr!("508.4", "508.3a", "702.3b");
    ruling!(
        "Midnight Crusader Shuttle",
        "When a creature becomes attacking this way, it wasn't declared as an attacker, so triggered abilities that trigger when a creature attacks won't trigger. Similar, all restrictions or costs to attack are ignored. For example, if the chosen creature has defender, it could still end up being an attacking creature this way."
    );
    supported("Midnight Crusader Shuttle");
    // Defender.
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P1, "Wall of Stone");
    shuttle_steals(&mut t, wall);
    assert_eq!(t.g.obj(wall).controller, P0);
    assert!(attacking(&t, wall) && t.obj_now(wall).tapped);
    // An attack trigger (Thraximundar: "defending player sacrifices a creature").
    let mut t = TestGame::new(2);
    let thrax = t.battlefield(P1, "Thraximundar");
    let bears = t.battlefield(P1, "Savannah Lions");
    shuttle_steals(&mut t, thrax);
    assert!(attacking(&t, thrax));
    assert!(t.on_battlefield(bears), "its attack trigger didn't trigger");
}

#[test]
fn teferi_phasing_out_removes_an_attacker_and_phasing_in_isnt_summoning_sickness() {
    cr!("702.26b", "506.4", "302.6");
    ruling!(
        "Teferi, Master of Time",
        "An attacking creature that phases out is removed from combat."
    );
    supported("Teferi, Master of Time");
    let mut t = TestGame::new(2);
    let teferi = t.battlefield(P0, "Teferi, Master of Time");
    t.g.objects[teferi.0 as usize]
        .counters
        .insert("loyalty".into(), 5);
    t.set_step(P1, Step::BeginningOfCombat);
    let giant = t.battlefield(P1, "Hill Giant");
    attack_with(&mut t, &[(giant, Entity::Player(P0))]);
    t.activate(P0, teferi, 1, &[obj(giant)]).unwrap();
    t.resolve_all();
    assert!(!attacking(&t, giant));
    t.advance_to(P1, Step::PostcombatMain);
    assert_eq!(t.life(P0), 20);

    ruling!(
        "Teferi, Master of Time",
        "Any creatures that phase in during their controller's untap step will be able to attack and pay a cost of {T} during that turn."
    );
    let mut t = TestGame::new(2);
    let teferi = t.battlefield(P0, "Teferi, Master of Time");
    t.g.objects[teferi.0 as usize]
        .counters
        .insert("loyalty".into(), 5);
    let pyro = t.battlefield(P1, "Prodigal Pyromancer");
    t.activate(P0, teferi, 1, &[obj(pyro)]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(pyro).phased_out);
    t.advance_to(P1, Step::PrecombatMain);
    assert!(!t.obj_now(pyro).phased_out);
    assert!(crate::r_s02_common::can_activate(&mut t, P1, pyro));
    t.set_step(P1, Step::BeginningOfCombat);
    assert!(can_attack(&mut t, pyro));
}

#[test]
fn weeping_angel_stops_being_a_creature_and_its_prevented_damage_triggers_nothing() {
    cr!("205.1b", "615.1", "510.2");
    ruling!(
        "Weeping Angel",
        "As the first triggered ability of Weeping Angel resolves, it continues to be an artifact but will stop being both an Alien and an Angel until it becomes a creature again. It's just a statue, how dangerous could it be?"
    );
    supported("Weeping Angel");
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P0, "Weeping Angel");
    t.set_step(P1, Step::PrecombatMain);
    cast_new(&mut t, P1, "Grizzly Bears", &[]);
    t.resolve_all();
    let o = t.obj_now(angel);
    assert!(!o.is(CardType::Creature));
    assert!(o.is(CardType::Artifact));
    assert!(!o.chars.subtypes.iter().any(|s| s == "Angel" || s == "Alien"));

    ruling!(
        "Weeping Angel",
        "Damage that is prevented never occurs, so any triggered abilities that trigger due to combat damage that would have been dealt by Weeping Angel won't trigger."
    );
    // Spirit Link: "Whenever enchanted creature deals damage, you gain that much life."
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P0, "Weeping Angel");
    let link = t.battlefield(P0, "Spirit Link");
    t.g.attach(link, Entity::Object(angel));
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.attack(&[(angel, Entity::Player(P1))], &[(bears, angel)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.zone(t.g.current(bears)), Zone::Library(P1));
    assert_eq!(count_of(&t, P1, CardType::Creature), 0);
}

#[test]
fn the_grim_captain_chooses_what_the_creature_attacks() {
    cr!("508.4", "506.3");
    ruling!(
        "Throne of the Grim Captain // The Grim Captain",
        "You choose which player, planeswalker, or battle the creature you put onto the battlefield is attacking as it enters the battlefield. It doesn't have to be the same player, planeswalker, or battle that The Grim Captain is attacking."
    );
    const THRONE: &str = "Throne of the Grim Captain // The Grim Captain";
    supported(THRONE);
    let mut t = TestGame::new(2);
    let throne = t.battlefield(P0, THRONE);
    let mats = [
        t.graveyard(P0, "Colossal Dreadmaw"),
        t.graveyard(P0, "Merfolk of the Pearl Trident"),
        t.graveyard(P0, "Vampire Interloper"),
        t.graveyard(P0, "Spectral Sailor"),
    ];
    t.lands(P0, "Wastes", 4);
    let es: Vec<Entity> = mats.iter().map(|o| obj(*o)).collect();
    t.answer_choose(P0, &es);
    let uid = t
        .g
        .obj(throne)
        .chars
        .abilities
        .iter()
        .find(|a| matches!(a.kind, AbilityKind::Activated(_)) && a.text.starts_with("Craft"))
        .map(|a| a.uid)
        .unwrap();
    t.g.turn.priority = Some(P0);
    t.g.activate_ability(P0, throne, uid).expect("craft");
    t.clear_answers();
    t.resolve_all();
    let captain = t.named_on_battlefield("The Grim Captain")[0];
    t.g.objects[captain.0 as usize].summoning_sick = false;
    let walker = t.battlefield(P1, "Liliana of the Veil");
    let ring = t.battlefield(P1, "Sol Ring");
    t.answer_choose(P1, &[obj(ring)]);
    // The Dreadmaw enters attacking Liliana; the Captain attacks P1.
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(mats[0])]);
    t.answer_choose(P0, &[obj(walker)]);
    attack_with(&mut t, &[(captain, Entity::Player(P1))]);
    t.resolve_all();
    let maw = t.named_on_battlefield("Colossal Dreadmaw");
    assert_eq!(maw.len(), 1);
    assert_eq!(
        crate::r_s12_common::attack_target(&t, maw[0]),
        Some(obj(walker))
    );
}
