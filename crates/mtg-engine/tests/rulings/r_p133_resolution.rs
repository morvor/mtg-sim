//! Rulings batch P133 — how bounce effects resolve: who chooses and when (Run Ashore,
//! Dispersal, Blizzard Specter, Blood Clock, Umbilicus, Withdraw, Consuming Tide, Turtles
//! in Time, Nimbus Champion, Icefeather Aven), what "all" covers, and who is affected
//! (Call to Heel, Undo, Sea God's Revenge).

use crate::r_p130_common::loyalty;
use crate::r_p133_common::*;
use crate::r_s01_common::supported;
use crate::r_s11_common::turn_face_up;
use crate::r_s12_common::morph;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn bounce_all_only_affects_the_battlefield() {
    cr!("611.2c", "400.1");
    ruling!(
        "Rebuild",
        "Artifact cards not on the battlefield won't be returned by Rebuild."
    );
    ruling!(
        "Hurkyl's Recall",
        "It only returns artifacts which are on the battlefield."
    );
    ruling!("Sunder", "Only affects lands on the battlefield.");
    for (name, kind, target) in [
        ("Rebuild", "Ornithopter", false),
        ("Hurkyl's Recall", "Ornithopter", true),
        ("Sunder", "Forest", false),
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        let on_bf = t.battlefield(P1, kind);
        let in_gy = t.graveyard(P1, kind);
        let in_exile = t.exile(P1, kind);
        let card = spell(&mut t, P0, name);
        let b = t.cast(P0, card);
        if target {
            b.target(Entity::Player(P1)).go();
        } else {
            b.go();
        }
        t.resolve_all();
        assert!(in_hand_of(&t, on_bf, P1), "{name}");
        assert_eq!(t.zone(in_gy), Zone::Graveyard(P1), "{name}");
        assert_eq!(t.zone(in_exile), Zone::Exile, "{name}");
    }
}

#[test]
fn displacement_wave_x_in_permanent_costs_is_zero() {
    cr!("202.3e");
    ruling!(
        "Displacement Wave",
        "If a permanent has an {X} in its mana cost, that X is 0."
    );
    supported("Displacement Wave");
    let mut t = TestGame::new(2);
    let endless = t.battlefield(P1, "Endless One");
    t.g.add_counters(Entity::Object(endless), "+1/+1", 4, None);
    let memnite = t.battlefield(P1, "Memnite");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let wave = spell(&mut t, P0, "Displacement Wave");
    t.cast(P0, wave).x(0).go();
    t.resolve_all();
    assert!(in_hand_of(&t, endless, P1));
    assert!(in_hand_of(&t, memnite, P1));
    assert!(t.on_battlefield(bears));
}

#[test]
fn run_ashore_owner_chooses_top_or_bottom() {
    cr!("608.2d");
    ruling!(
        "Run Ashore",
        "For the first mode, the owner of the card chooses whether it goes on the top or bottom of that library."
    );
    supported("Run Ashore");
    // P0 controls P1's creature: P1 (the owner) chooses.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    crate::r_s06_common::give_control(&mut t, bears, P0);
    let card = spell(&mut t, P0, "Run Ashore");
    t.cast(P0, card).modes(&[0]).target(bears).go();
    let from = t.asked().len();
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Library(P1));
    let choosers: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| !matches!(d, Decision::Priority { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(choosers, vec![P1]);
}

#[test]
fn run_ashore_both_modes_aura_returned_before_sba() {
    cr!("608.2c", "704.3", "704.5m");
    ruling!(
        "Run Ashore",
        "If you choose both modes, they happen in the listed order and nothing happens in between them. Notably, if the second mode targets an Aura currently enchanting the target of the first mode, that Aura will be returned to its owner's hand."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let aura = crate::r_s06_common::attach_new(&mut t, P0, "Pacifism", bears);
    let card = spell(&mut t, P0, "Run Ashore");
    t.cast(P0, card).modes(&[0, 1]).target(bears).target(aura).go();
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Library(P1));
    assert!(in_hand_of(&t, aura, P0));
}

#[test]
fn dispersal_opponents_choose_then_discard() {
    cr!("101.4", "709.3");
    ruling!(
        "Discovery // Dispersal",
        "As Dispersal resolves, first the next opponent in turn order (or, if it's an opponent's turn, that opponent) chooses a nonland permanent they control with the highest mana cost among their permanents, then each other opponent in turn order does the same, knowing the choices made before them. Then all the chosen permanents are returned at the same time. Next, each opponent in the same order chooses a card in hand without revealing it, then the chosen cards are discarded at the same time."
    );
    ruling!(
        "Discovery // Dispersal",
        "If an opponent controls only lands as Dispersal resolves, that player won't return any permanent to their hand. They'll still discard a card."
    );
    supported("Discovery // Dispersal");
    let mut t = TestGame::new(3);
    let g1 = t.battlefield(P1, "Hill Giant");
    let g2 = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let p2_land = t.battlefield(P2, "Forest");
    t.hand(P1, "Island");
    t.hand(P2, "Island");
    t.hand(P2, "Mountain");
    let card = t.hand(P0, "Discovery // Dispersal");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Wastes", 3);
    t.answer_choose(P1, &[Entity::Object(g2)]);
    t.cast(P0, card).method(CastMethod::Half(1)).go();
    let from = t.asked().len();
    t.resolve_all();
    // P1 returned the chosen greatest-mana-value permanent, then discarded a card: the
    // returned Giant or the Island (two cards in hand, one discarded).
    assert!(t.on_battlefield(g1) && t.on_battlefield(bears));
    assert_ne!(t.zone(g2), Zone::Battlefield);
    assert_eq!(t.hand_size(P1), 1);
    // P2 controls only a land: nothing returned, one card discarded.
    assert!(t.on_battlefield(p2_land));
    assert_eq!(t.hand_size(P2), 1);
    assert_eq!(t.graveyard_size(P2), 1);
    // P1 (next in turn order) chose before P2 was asked anything.
    let order: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| !matches!(d, Decision::Priority { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(order.first(), Some(&P1));
}

#[test]
fn blizzard_specter_damaged_player_chooses_untargeted() {
    cr!("700.2", "702.11b", "115.10");
    ruling!(
        "Blizzard Specter",
        "If Blizzard Specter’s first mode is chosen, the player who was dealt damage chooses the permanent to return to hand when the ability resolves. A permanent with hexproof or protection from black may be chosen this way."
    );
    supported("Blizzard Specter");
    let mut t = TestGame::new(2);
    let specter = t.battlefield(P0, "Blizzard Specter");
    let scout = t.battlefield(P1, "Gladecover Scout");
    let knight = t.battlefield(P1, "White Knight");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    t.answer_choose(P1, &[Entity::Object(knight)]);
    t.attack(&[(specter, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert!(in_hand_of(&t, knight, P1));
    assert!(t.on_battlefield(scout));
}

#[test]
fn upkeep_bounce_choices_on_resolution() {
    cr!("603.3d", "608.2d", "115.10");
    ruling!(
        "Blood Clock",
        "The choice of which permanent to return or whether to pay life is made as the triggered ability resolves."
    );
    ruling!(
        "Umbilicus",
        "The player makes all choices as the triggered ability resolves. They permanent they choose to return to its owner’s hand, if any, isn’t targeted by the ability."
    );
    for name in ["Blood Clock", "Umbilicus"] {
        supported(name);
        let mut t = TestGame::new(2);
        t.battlefield(P0, name);
        t.advance_to(P1, Step::Upkeep);
        t.settle();
        assert_eq!(t.stack_len(), 1, "{name}");
        // P1 gets a hexproof creature with the trigger on the stack and returns it.
        let scout = t.battlefield(P1, "Gladecover Scout");
        t.answer_yes(P1, false);
        t.answer_choose(P1, &[Entity::Object(scout)]);
        t.resolve_all();
        assert!(in_hand_of(&t, scout, P1), "{name}");
        assert_eq!(t.life(P1), 20, "{name}");
    }
}

#[test]
fn withdraw_targets_and_payment() {
    cr!("115.3", "608.2d", "118.12");
    ruling!(
        "Withdraw",
        "The creature’s controller gets the option to pay when the spell resolves."
    );
    ruling!("Withdraw", "The two creatures may have different controllers.");
    supported("Withdraw");
    for pay in [true, false] {
        let mut t = TestGame::new(2);
        let mine = t.battlefield(P0, "Grizzly Bears");
        let theirs = t.battlefield(P1, "Grizzly Bears");
        t.lands(P1, "Wastes", 1);
        let card = spell(&mut t, P0, "Withdraw");
        t.cast(P0, card).target(mine).target(theirs).go();
        t.answer_yes(P1, pay);
        let from = t.asked().len();
        t.resolve_all();
        assert!(in_hand_of(&t, mine, P0));
        assert_eq!(t.on_battlefield(theirs), pay);
        assert!(t.asked()[from..]
            .iter()
            .any(|(p, d)| *p == P1 && !matches!(d, Decision::Priority { .. })));
    }
}

#[test]
fn two_target_spells_need_two_different_creatures() {
    cr!("115.3");
    ruling!(
        "Hope and Glory",
        "Must target two different creatures."
    );
    ruling!(
        "Sick and Tired",
        "Must target two different creatures."
    );
    ruling!("Withdraw", "Must target two different creatures.");
    for (name, two_slots) in [
        ("Hope and Glory", false),
        ("Sick and Tired", false),
        ("Withdraw", true),
    ] {
        supported(name);
        // Only one creature: it can't be cast.
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let card = spell(&mut t, P0, name);
        let r = if two_slots {
            t.cast(P0, card).target(bears).target(bears).try_go()
        } else {
            t.cast(P0, card)
                .targets(&[Entity::Object(bears), Entity::Object(bears)])
                .try_go()
        };
        assert!(r.is_err(), "{name}");
        // Two creatures: fine.
        let mut t = TestGame::new(2);
        let a = t.battlefield(P1, "Grizzly Bears");
        let b = t.battlefield(P1, "Grizzly Bears");
        let card = spell(&mut t, P0, name);
        let r = if two_slots {
            t.cast(P0, card).target(a).target(b).try_go()
        } else {
            t.cast(P0, card)
                .targets(&[Entity::Object(a), Entity::Object(b)])
                .try_go()
        };
        assert!(r.is_ok(), "{name}");
    }
}

#[test]
fn undo_returns_to_each_owner() {
    cr!("400.3", "108.3");
    ruling!(
        "Undo",
        "The two creatures may have different owners and return to their respective owner’s hands."
    );
    supported("Undo");
    let mut t = TestGame::new(2);
    let stolen = t.battlefield(P1, "Grizzly Bears");
    crate::r_s06_common::give_control(&mut t, stolen, P0);
    let mine = t.battlefield(P0, "Hill Giant");
    let card = spell(&mut t, P0, "Undo");
    t.cast(P0, card)
        .targets(&[Entity::Object(stolen), Entity::Object(mine)])
        .go();
    t.resolve_all();
    assert!(in_hand_of(&t, stolen, P1));
    assert!(in_hand_of(&t, mine, P0));
}

#[test]
fn call_to_heel_controller_draws() {
    cr!("608.2h", "110.2");
    ruling!(
        "Call to Heel",
        "The player who draws a card is the player who controls the creature when Call to Heel resolves. This may not be the player whose hand the creature is put into."
    );
    supported("Call to Heel");
    let mut t = TestGame::new(2);
    let stolen = t.battlefield(P1, "Grizzly Bears");
    crate::r_s06_common::give_control(&mut t, stolen, P0);
    let card = spell(&mut t, P0, "Call to Heel");
    t.cast(P0, card).target(stolen).go();
    let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
    t.resolve_all();
    assert!(in_hand_of(&t, stolen, P1));
    assert_eq!(t.hand_size(P0), h0 + 1);
    assert_eq!(t.hand_size(P1), h1 + 1);
}

#[test]
fn sea_gods_revenge_targets_of_different_opponents() {
    cr!("115.1", "601.2c");
    ruling!(
        "Sea God's Revenge",
        "In multiplayer games, the three target creatures can be controlled by different opponents."
    );
    supported("Sea God's Revenge");
    let mut t = TestGame::new(4);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P2, "Grizzly Bears");
    let c = t.battlefield(P3, "Grizzly Bears");
    let card = spell(&mut t, P0, "Sea God's Revenge");
    t.cast(P0, card)
        .targets(&[Entity::Object(a), Entity::Object(b), Entity::Object(c)])
        .go();
    t.resolve_all();
    assert!(in_hand_of(&t, a, P1) && in_hand_of(&t, b, P2) && in_hand_of(&t, c, P3));
}

#[test]
fn consuming_tide_choices_in_turn_order() {
    cr!("101.4", "608.2c");
    ruling!(
        "Consuming Tide",
        "Starting with the player whose turn it is, each player chooses a nonland permanent they control. Players get to know what permanents have been chose by players that chose before them as they make their choice. Then all nonland permanents that weren't chosen by any player are returned to their owners' hands at the same time."
    );
    supported("Consuming Tide");
    let mut t = TestGame::new(2);
    let a0 = t.battlefield(P0, "Grizzly Bears");
    let b0 = t.battlefield(P0, "Hill Giant");
    let a1 = t.battlefield(P1, "Grizzly Bears");
    let b1 = t.battlefield(P1, "Hill Giant");
    let land = t.battlefield(P1, "Forest");
    let card = spell(&mut t, P0, "Consuming Tide");
    t.cast(P0, card).go();
    t.answer_choose(P0, &[Entity::Object(b0)]);
    t.answer_choose(P1, &[Entity::Object(a1)]);
    let from = t.asked().len();
    t.resolve_all();
    assert!(t.on_battlefield(b0) && t.on_battlefield(a1) && t.on_battlefield(land));
    assert!(in_hand_of(&t, a0, P0) && in_hand_of(&t, b1, P1));
    let order: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(order, vec![P0, P1]);
}

#[test]
fn turtles_in_time_decisions_in_turn_order() {
    cr!("101.4");
    ruling!(
        "Turtles in Time",
        "Players decide and announce in turn order, starting with the active player, whether or not they want to shuffle their hand and graveyard into their library. Then, all players who chose to do so shuffle their hands and graveyards into their libraries and draw seven cards. Players later in the turn order will know the choices made by players before them."
    );
    supported("Turtles in Time");
    let mut t = TestGame::new(3);
    t.hand(P1, "Island");
    t.hand(P2, "Island");
    t.graveyard(P1, "Island");
    let card = spell(&mut t, P0, "Turtles in Time");
    t.cast(P0, card).go();
    t.answer_yes(P0, false);
    t.answer_yes(P1, true);
    t.answer_yes(P2, false);
    let from = t.asked().len();
    t.resolve_all();
    let order: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::YesNo { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(order, vec![P0, P1, P2]);
    assert_eq!(t.hand_size(P1), 7);
    assert_eq!(t.graveyard_size(P1), 0);
    assert_eq!(t.hand_size(P2), 1);
}

#[test]
fn nimbus_champion_checks_power_on_resolution() {
    cr!("608.2c", "603.3d");
    ruling!(
        "Nimbus Champion",
        "Nimbus Champion’s ability can target any creature. Whether that creature is returned to its owner’s hand is determined as the ability resolves."
    );
    supported("Nimbus Champion");
    // A 3-power creature can be targeted though it's too big: it stays.
    let mut t = TestGame::new(2);
    let champ = t.battlefield(P0, "Nimbus Champion");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.answer_yes(P0, true);
    crate::r_s01_common::attack_with(&mut t, &[(champ, Entity::Player(P1))]);
    t.resolve_all();
    assert!(t.on_battlefield(giant));
    // A 2-power creature with two Warriors when the trigger is put on the stack; one
    // Warrior leaves before it resolves: it stays.
    let mut t = TestGame::new(2);
    let champ = t.battlefield(P0, "Nimbus Champion");
    let warrior = t.battlefield(P0, "Elvish Warrior");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_yes(P0, true);
    crate::r_s01_common::attack_with(&mut t, &[(champ, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 1);
    kill(&mut t, warrior);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    // With the second Warrior still there: returned.
    let mut t = TestGame::new(2);
    let champ = t.battlefield(P0, "Nimbus Champion");
    t.battlefield(P0, "Elvish Warrior");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_yes(P0, true);
    crate::r_s01_common::attack_with(&mut t, &[(champ, Entity::Player(P1))]);
    t.resolve_all();
    assert!(in_hand_of(&t, bears, P1));
}

#[test]
fn icefeather_aven_must_target_your_own_creature() {
    cr!("603.3d", "702.37e");
    ruling!(
        "Icefeather Aven",
        "If you control the only other creatures when Icefeather Aven is turned face up, you must target one of them. You choose whether that creature is returned to its owner's hand as the ability resolves."
    );
    supported("Icefeather Aven");
    for yes in [false, true] {
        let mut t = TestGame::new(2);
        let aven = morph(&mut t, P0, "Icefeather Aven");
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.lands(P0, "Forest", 1);
        t.lands(P0, "Island", 1);
        t.lands(P0, "Wastes", 1);
        assert!(turn_face_up(&mut t, P0, aven));
        t.settle();
        assert_eq!(t.stack_len(), 1);
        t.answer_yes(P0, yes);
        t.resolve_all();
        assert_eq!(in_hand_of(&t, bears, P0), yes);
    }
}

#[test]
fn thousand_winds_returns_other_tapped_creatures() {
    cr!("608.2h", "702.37e");
    ruling!(
        "Thousand Winds",
        "It doesn't matter if Thousand Winds is tapped or untapped (or even on the battlefield) when its last ability resolves. Tapped creatures except Thousand Winds will be returned to their owners' hands."
    );
    supported("Thousand Winds");
    let mut t = TestGame::new(2);
    let winds = morph(&mut t, P0, "Thousand Winds");
    t.g.objects[winds.0 as usize].tapped = true;
    let tapped = t.battlefield(P1, "Grizzly Bears");
    t.g.objects[tapped.0 as usize].tapped = true;
    let untapped = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    t.lands(P0, "Wastes", 5);
    assert!(turn_face_up(&mut t, P0, winds));
    t.resolve_all();
    assert!(t.on_battlefield(winds));
    assert!(in_hand_of(&t, tapped, P1));
    assert!(t.on_battlefield(untapped));
    // It left the battlefield before its ability resolved: still only "other" ones.
    let mut t = TestGame::new(2);
    let winds = morph(&mut t, P0, "Thousand Winds");
    let tapped = t.battlefield(P1, "Grizzly Bears");
    t.g.objects[tapped.0 as usize].tapped = true;
    t.lands(P0, "Island", 2);
    t.lands(P0, "Wastes", 5);
    assert!(turn_face_up(&mut t, P0, winds));
    t.settle();
    kill(&mut t, winds);
    t.resolve_all();
    assert!(in_hand_of(&t, tapped, P1));
}

#[test]
fn stern_proctor_must_return_your_own_artifact() {
    cr!("603.3d", "115.1");
    ruling!(
        "Stern Proctor",
        "If there is an artifact or enchantment on the battlefield (even your own) when this card enters, you must return one to owner's hand. If not, then simply ignore the \"enters\" ability."
    );
    supported("Stern Proctor");
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    t.enter(P0, "Stern Proctor");
    t.resolve_all();
    assert!(in_hand_of(&t, thopter, P0));
    // Nothing to target: the ability is ignored.
    let mut t = TestGame::new(2);
    t.enter(P0, "Stern Proctor");
    t.settle();
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn mu_yanling_effect_lasts_until_your_next_turn_would_have_begun() {
    cr!("800.4m");
    ruling!(
        "Mu Yanling, Celestial Wind",
        "In a multiplayer game, if you leave the game after Mu Yanling’s first ability resolves but before your next turn begins, its effect lasts until your next turn would have begun. It neither expires immediately nor lasts indefinitely."
    );
    supported("Mu Yanling, Celestial Wind");
    let mut t = TestGame::new(3);
    let mu = t.battlefield(P0, "Mu Yanling, Celestial Wind");
    loyalty(&mut t, mu, 4);
    let giant = t.battlefield(P1, "Hill Giant");
    t.activate(P0, mu, 0, &[Entity::Object(giant)]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(giant).0, -2);
    t.g.lose_game(P0);
    t.settle();
    t.advance_to(P1, Step::PrecombatMain);
    assert_eq!(t.pt(giant).0, -2, "it expired immediately");
    t.advance_to(P2, Step::PrecombatMain);
    assert_eq!(t.pt(giant).0, -2);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.pt(giant).0, 3, "it lasted past P0's next turn");
}
