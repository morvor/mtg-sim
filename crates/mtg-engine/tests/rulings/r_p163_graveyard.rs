//! Rulings batch P163 — graveyard sweepers: "target player shuffles their graveyard into
//! their library" (CR 701.24a, 608.2n), targets in graveyards and illegal targets (CR
//! 115.1, 608.2b), Rest in Peace and discarding (CR 614.1a, 702.35a, 701.9), choices made
//! on resolution (CR 608.2d), and costs that exile the source (CR 602.2b).

use crate::r_p163_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn shuffle_graveyard_spells_with_an_empty_graveyard_still_shuffle_and_draw() {
    cr!("701.24a", "608.2c");
    ruling!(
        "Clear the Mind",
        "Clear the Mind can target a player with no cards in their graveyard. That player shuffles their library and then you draw a card."
    );
    ruling!(
        "Learn from the Past",
        "If the player's graveyard is empty, the player will shuffle their library, then you'll draw a card."
    );
    for name in ["Clear the Mind", "Learn from the Past"] {
        supported(name);
        let mut t = TestGame::new(2);
        assert_eq!(t.graveyard_size(P1), 0);
        let hand = t.hand_size(P0);
        cast_resolve(&mut t, P0, name, &[Entity::Player(P1)]);
        assert_eq!(shuffles(&t, P1), 1, "{name}");
        assert_eq!(t.hand_size(P0), hand + 1, "{name}");
    }
}

#[test]
fn shuffle_graveyard_spell_targeting_yourself_ends_up_alone_in_the_graveyard() {
    cr!("608.2n", "701.24a");
    ruling!(
        "Clear the Mind",
        "If you choose yourself as the target for Clear the Mind, Clear the Mind will end up as the only card in your graveyard after it’s done resolving."
    );
    ruling!(
        "Reminisce",
        "This card won't be put into your graveyard until after it's finished resolving, which means it won't be shuffled into your library as part of its own effect."
    );
    supported("Reminisce");
    for name in ["Clear the Mind", "Reminisce"] {
        let mut t = TestGame::new(2);
        t.graveyard(P0, "Grizzly Bears");
        t.graveyard(P0, "Lightning Bolt");
        cast_resolve(&mut t, P0, name, &[Entity::Player(P0)]);
        assert_eq!(graveyard_names(&t, P0), vec![name.to_string()], "{name}");
    }
}

/// Puts Loaming Shaman onto the battlefield for P0; its trigger targets `player` and
/// `cards` (in `player`'s graveyard) and is put on the stack.
fn loaming(t: &mut TestGame, player: PlayerId, cards: &[ObjectId]) {
    t.answer_targets(P0, &[Entity::Player(player)]);
    let cards: Vec<Entity> = cards.iter().map(|c| obj(*c)).collect();
    t.answer_targets(P0, &cards);
    enter(t, P0, "Loaming Shaman");
    assert_eq!(stacked_triggers(t), 1);
}

#[test]
fn loaming_shaman_with_no_target_cards_just_shuffles() {
    cr!("701.24a", "115.1d");
    ruling!(
        "Loaming Shaman",
        "If Loaming Shaman’s triggered ability targets a player but doesn’t target any cards in that player’s graveyard, that player just shuffles their library."
    );
    let mut t = TestGame::new(2);
    t.graveyard(P1, "Grizzly Bears");
    let lib = t.library_size(P1);
    loaming(&mut t, P1, &[]);
    t.resolve_all();
    assert_eq!(shuffles(&t, P1), 1);
    assert_eq!(t.library_size(P1), lib);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn loaming_shaman_illegal_cards_or_player() {
    cr!("608.2b", "701.24a");
    ruling!(
        "Loaming Shaman",
        "If the target cards become illegal targets, the target player shuffles their library. If the target player becomes an illegal target, Loaming Shaman’s ability has no effect."
    );
    // The target cards leave the graveyard: the player still shuffles.
    let mut t = TestGame::new(2);
    let a = t.graveyard(P1, "Grizzly Bears");
    let b = t.graveyard(P1, "Hill Giant");
    let lib = t.library_size(P1);
    loaming(&mut t, P1, &[a, b]);
    move_to(&mut t, a, Zone::Exile);
    move_to(&mut t, b, Zone::Exile);
    t.resolve_all();
    assert_eq!(shuffles(&t, P1), 1);
    assert_eq!(t.library_size(P1), lib);
    // The player gains hexproof: nothing happens.
    let mut t = TestGame::new(2);
    let a = t.graveyard(P1, "Grizzly Bears");
    loaming(&mut t, P1, &[a]);
    t.battlefield(P1, "Leyline of Sanctity");
    t.resolve_all();
    assert_eq!(shuffles(&t, P1), 0);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn rest_in_peace_discarded_cards_still_have_madness_and_are_found_in_exile() {
    cr!("614.1a", "702.35a", "608.2h");
    ruling!(
        "Rest in Peace",
        "If a card is discarded while Rest in Peace is on the battlefield, abilities that function when a card is discarded (such as madness) still work, even though that card never reaches a graveyard. In addition, spells or abilities that check the characteristics of a discarded card (such as Chandra Ablaze's first ability) can find that card in exile."
    );
    supported("Rest in Peace");
    supported("Chandra Ablaze");
    supported("Fiery Temper");
    // Madness: Fiery Temper discarded to a Chandra Ablaze +1 is cast for {R}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rest in Peace");
    let ch = t.battlefield(P0, "Chandra Ablaze");
    empty_hand(&mut t, P0);
    t.lands(P0, "Mountain", 1);
    let temper = t.hand(P0, "Fiery Temper");
    t.answer_choose(P0, &[obj(temper)]);
    t.activate(P0, ch, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve();
    // Chandra found the red card in exile: 4 damage.
    assert_eq!(t.life(P1), 16);
    assert!(t.in_exile("Fiery Temper"));
    yes(&mut t, P0);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve();
    assert_eq!(t.zone(temper), Zone::Stack);
    t.resolve_all();
    assert_eq!(t.life(P1), 13);
    assert!(t.in_exile("Fiery Temper"));
    // A nonred card: no damage.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rest in Peace");
    let ch = t.battlefield(P0, "Chandra Ablaze");
    empty_hand(&mut t, P0);
    let bears = t.hand(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(bears)]);
    t.activate(P0, ch, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert!(t.in_exile("Grizzly Bears"));
}

#[test]
fn canoptek_scarab_swarm_artifact_land_makes_one_token() {
    cr!("205.2b", "603.2");
    ruling!(
        "Canoptek Scarab Swarm",
        "If an exiled card is both an artifact card and a land card, you will create one token, not two."
    );
    supported("Canoptek Scarab Swarm");
    let mut t = TestGame::new(2);
    t.graveyard(P1, "Seat of the Synod");
    t.graveyard(P1, "Ornithopter");
    t.graveyard(P1, "Forest");
    t.graveyard(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    enter(&mut t, P0, "Canoptek Scarab Swarm");
    t.resolve_all();
    assert_eq!(t.graveyard_size(P1), 0);
    assert_eq!(tokens(&t, P0).len(), 3);
}

#[test]
fn go_blank_discards_then_exiles_the_graveyard() {
    cr!("701.9a", "608.2c");
    ruling!(
        "Go Blank",
        "If the target player has one card in hand as Go Blank resolves, they’ll discard it and the cards in their graveyard will be exiled. If their hand is empty, you’ll just exile the cards in their graveyard."
    );
    supported("Go Blank");
    for in_hand in [1, 0] {
        let mut t = TestGame::new(2);
        empty_hand(&mut t, P1);
        if in_hand == 1 {
            t.hand(P1, "Grizzly Bears");
        }
        t.graveyard(P1, "Hill Giant");
        cast_resolve(&mut t, P0, "Go Blank", &[Entity::Player(P1)]);
        assert_eq!(t.hand_size(P1), 0);
        assert_eq!(t.graveyard_size(P1), 0);
        assert!(t.in_exile("Hill Giant"));
        assert_eq!(t.in_exile("Grizzly Bears"), in_hand == 1);
    }
}

#[test]
fn relic_of_progenitus_target_player_chooses_on_resolution() {
    cr!("608.2d", "115.1");
    ruling!(
        "Relic of Progenitus",
        "If you activate Relic of Progenitus's first ability, the targeted player chooses which card to exile. The choice is made as the ability resolves."
    );
    supported("Relic of Progenitus");
    let mut t = TestGame::new(2);
    let relic = t.battlefield(P0, "Relic of Progenitus");
    t.graveyard(P1, "Grizzly Bears");
    let giant = t.graveyard(P1, "Hill Giant");
    t.activate(P0, relic, 0, &[Entity::Player(P1)]).unwrap();
    // A card put into the graveyard in response can be chosen.
    let late = t.graveyard(P1, "Craw Wurm");
    let from = n_asked(&t);
    t.answer_choose(P1, &[obj(late)]);
    t.resolve_all();
    let asked: Vec<_> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(asked, vec![P1]);
    assert!(t.in_exile("Craw Wurm"));
    assert_eq!(t.zone(giant), Zone::Graveyard(P1));
    assert_eq!(t.graveyard_size(P1), 2);
}

#[test]
fn relic_of_progenitus_second_ability_draws_with_empty_graveyards() {
    cr!("602.2b", "608.2c");
    ruling!(
        "Relic of Progenitus",
        "You can activate Relic of Progenitus's second ability even if no players have any cards in their graveyards. You'll still draw a card."
    );
    let mut t = TestGame::new(2);
    let relic = t.battlefield(P0, "Relic of Progenitus");
    t.lands(P0, "Wastes", 1);
    let hand = t.hand_size(P0);
    activate_resolve(&mut t, P0, relic, 1, &[]);
    assert_eq!(t.hand_size(P0), hand + 1);
    assert!(t.in_exile("Relic of Progenitus"));
}

#[test]
fn silent_gravestone_stops_only_targeting() {
    cr!("115.10", "115.10a", "601.2c");
    ruling!(
        "Silent Gravestone",
        "Only spells and abilities that target cards in graveyards will be affected. Spells and abilities that affect cards in graveyards without targeting them (such as Extract from Darkness) can still affect those cards."
    );
    supported("Silent Gravestone");
    supported("Extract from Darkness");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Silent Gravestone");
    let wurm = t.graveyard(P1, "Craw Wurm");
    // Raise Dead can't target the card.
    t.lands(P0, "Swamp", 1);
    let raise = t.hand(P0, "Raise Dead");
    assert!(!can_cast(&mut t, P0, raise, CastMethod::Normal));
    // Extract from Darkness puts it onto the battlefield.
    t.answer_choose(P0, &[obj(wurm)]);
    cast_resolve(&mut t, P0, "Extract from Darkness", &[]);
    let w = t.named_on_battlefield("Craw Wurm");
    assert_eq!(w.len(), 1);
    assert_eq!(t.g.obj(w[0]).controller, P0);
}

#[test]
fn silent_gravestone_is_exiled_on_resolution() {
    cr!("602.2b", "608.2c");
    ruling!(
        "Silent Gravestone",
        "Silent Gravestone isn’t exiled until its activated ability resolves."
    );
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Silent Gravestone");
    t.lands(P0, "Wastes", 4);
    t.graveyard(P1, "Grizzly Bears");
    let hand = t.hand_size(P0);
    t.activate(P0, g, 0, &[]).unwrap();
    // Still on the battlefield (tapped) while the ability is on the stack.
    assert!(t.on_battlefield(g));
    assert!(t.obj(g).tapped);
    // Its static ability still applies: a card in a graveyard can't be targeted.
    t.lands(P0, "Swamp", 1);
    let raise = t.hand(P0, "Raise Dead");
    assert!(!can_cast(&mut t, P0, raise, CastMethod::Normal));
    t.resolve_all();
    assert!(t.in_exile("Silent Gravestone"));
    assert!(t.in_exile("Grizzly Bears"));
    // It drew a card (Raise Dead was added to the hand too).
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn abstergo_entertainment_illegal_target_means_no_graveyards_are_exiled() {
    cr!("608.2b", "115.1");
    ruling!(
        "Abstergo Entertainment",
        "You don’t have to choose a target historic card to activate Abstergo Entertainment’s last ability. However, if you do, and that card is an illegal target at the time the ability tries to resolve, it won’t resolve and none of its effects will happen. No graveyards will be exiled."
    );
    supported("Abstergo Entertainment");
    // No target: graveyards are exiled.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Abstergo Entertainment");
    t.lands(P0, "Wastes", 3);
    t.graveyard(P1, "Grizzly Bears");
    activate_resolve(&mut t, P0, a, 2, &[]);
    assert!(t.in_exile("Abstergo Entertainment"));
    assert_eq!(t.graveyard_size(P1), 0);
    // The target leaves the graveyard: nothing happens.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Abstergo Entertainment");
    t.lands(P0, "Wastes", 3);
    let thopter = t.graveyard(P0, "Ornithopter");
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P1, "Hill Giant");
    t.activate(P0, a, 2, &[obj(thopter)]).unwrap();
    move_to(&mut t, thopter, Zone::Exile);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P1, "Hill Giant"));
    // A legal target: it returns, then the graveyards are exiled.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Abstergo Entertainment");
    t.lands(P0, "Wastes", 3);
    let thopter = t.graveyard(P0, "Ornithopter");
    t.graveyard(P1, "Hill Giant");
    activate_resolve(&mut t, P0, a, 2, &[obj(thopter)]);
    assert!(t.in_hand(P0, "Ornithopter"));
    assert_eq!(t.graveyard_size(P1), 0);
}

/// Casts the real card `name` for P0 (with lands for its cost) answering each target slot
/// with the given entities (and the given modes, if any). Returns the spell.
fn cast_slots(t: &mut TestGame, name: &str, modes: &[usize], slots: &[Vec<Entity>]) {
    lands_for_cost(t, P0, name);
    let c = t.hand(P0, name);
    let mut b = t.cast(P0, c);
    if !modes.is_empty() {
        b = b.modes(modes);
    }
    for s in slots {
        b = b.targets(s);
    }
    b.go();
}

#[test]
fn target_player_shuffles_target_cards_does_nothing_if_the_player_is_illegal() {
    cr!("608.2b", "701.24a");
    // The other "target player shuffles up to N target cards from their graveyard into
    // their library" cards: with the player an illegal target, the cards stay put, and
    // the rest of the spell still happens.
    for name in [
        "Dwell on the Past",
        "Memory's Journey",
        "Stream of Consciousness",
        "Krosan Reclamation",
        "Gaea's Blessing",
        "Witness the Future",
        "Rite of Renewal",
        "Quandrix Command",
    ] {
        supported(name);
        for illegal in [false, true] {
            let mut t = TestGame::new(2);
            let bears = t.graveyard(P1, "Grizzly Bears");
            let mine = t.graveyard(P0, "Hill Giant");
            let target = t.battlefield(P0, "Craw Wurm");
            let lib = t.library_size(P1);
            let hand = t.hand_size(P0);
            let player = vec![Entity::Player(P1)];
            let cards = vec![obj(bears)];
            match name {
                "Rite of Renewal" => cast_slots(&mut t, name, &[], &[vec![obj(mine)], player, cards]),
                "Quandrix Command" => {
                    cast_slots(&mut t, name, &[2, 3], &[vec![obj(target)], player, cards])
                }
                _ => cast_slots(&mut t, name, &[], &[player, cards]),
            }
            if illegal {
                t.battlefield(P1, "Leyline of Sanctity");
            }
            t.resolve_all();
            assert_eq!(t.in_graveyard(P1, "Grizzly Bears"), illegal, "{name}");
            assert_eq!(shuffles(&t, P1), usize::from(!illegal), "{name}");
            assert_eq!(t.library_size(P1), lib + usize::from(!illegal), "{name}");
            // The rest of the spell happens either way.
            match name {
                "Gaea's Blessing" | "Witness the Future" => {
                    assert_eq!(t.hand_size(P0), hand + 1, "{name}")
                }
                "Rite of Renewal" => assert!(t.in_hand(P0, "Hill Giant"), "{name}"),
                "Quandrix Command" => assert_eq!(t.pt(target), (8, 6), "{name}"),
                _ => {}
            }
        }
    }
}
