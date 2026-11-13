//! Rulings batch P146 — repeatable sacrifice outlets ("plunder"): a permanent can be
//! sacrificed to pay the cost of its own ability (CR 602.2b, 601.2h), activation
//! restrictions that look at the turn's history (CR 602.5), sacrifices made while an
//! ability resolves (CR 608.2c), Peregrin Took's token replacement (CR 614.1a), and
//! abilities that look at the source as the ability resolves (CR 113.7a, 608.2h).

use crate::r_p146_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

/// P0 activates `name`'s first ability, sacrificing the creature itself (the only
/// creature P0 controls), with `lands` for the mana. The creature is in the graveyard
/// as soon as the ability has been activated, and P0 draws a card when it resolves.
fn sacrifice_itself(name: &str, lands: &[(&str, usize)]) -> TestGame {
    supported(name);
    let mut t = TestGame::new(2);
    for (land, n) in lands {
        t.lands(P0, land, *n);
    }
    let c = t.battlefield(P0, name);
    t.answer_choose(P0, &[obj(c)]);
    t.activate(P0, c, 0, &[]).unwrap();
    assert!(t.in_graveyard(P0, name), "{name}");
    assert_eq!(t.stack_len(), 1, "{name}");
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1, "{name}");
    t
}

#[test]
fn wall_of_mulch_can_be_sacrificed_for_its_own_ability() {
    cr!("602.2b", "601.2h", "701.21a");
    ruling!("Wall of Mulch", "It can be sacrificed for its own ability.");
    // "{G}, Sacrifice a Wall: Draw a card."
    sacrifice_itself("Wall of Mulch", &[("Forest", 1)]);
}

#[test]
fn portcullis_vine_can_be_sacrificed_for_its_own_ability() {
    cr!("602.2b", "601.2h", "701.21a");
    ruling!(
        "Portcullis Vine",
        "Portcullis Vine can be sacrificed to pay the cost of its last ability."
    );
    // "{2}, {T}, Sacrifice a creature with defender: Draw a card." Tapping and sacrificing
    // itself are both paid.
    sacrifice_itself("Portcullis Vine", &[("Wastes", 2)]);
}

#[test]
fn soulreaper_thallid_and_barricade_can_be_sacrificed_for_their_own_abilities() {
    cr!("602.2b", "601.2h", "701.21a");
    ruling!(
        "Soulreaper of Mogis",
        "Soulreaper of Mogis can be sacrificed to pay the cost of its own ability."
    );
    ruling!(
        "Thallid Soothsayer",
        "You can sacrifice Thallid Soothsayer to pay the cost for its own ability."
    );
    ruling!(
        "Gibbering Barricade",
        "You may sacrifice Gibbering Barricade to its own ability."
    );
    // "{2}{B}, Sacrifice a creature: Draw a card."
    sacrifice_itself("Soulreaper of Mogis", &[("Swamp", 1), ("Wastes", 2)]);
    // "{2}, Sacrifice a creature: Draw a card."
    sacrifice_itself("Thallid Soothsayer", &[("Wastes", 2)]);
    // "{2}{B}, Sacrifice a creature: You gain 1 life and draw a card."
    let t = sacrifice_itself("Gibbering Barricade", &[("Swamp", 1), ("Wastes", 2)]);
    assert_eq!(t.life(P0), 21);
}

#[test]
fn falkenrath_pit_fighter_needs_only_some_life_loss_by_an_opponent() {
    cr!("602.5", "119.3");
    ruling!(
        "Falkenrath Pit Fighter",
        "Falkenrath Pit Fighter's ability can be activated if any opponent lost any amount of life this turn, even if they also gained life at some point."
    );
    supported("Falkenrath Pit Fighter");
    // "{1}{R}, Discard a card, Sacrifice a Vampire: Draw two cards. Activate only if an
    // opponent lost life this turn."
    let setup = || {
        let mut t = TestGame::new(2);
        t.lands(P0, "Mountain", 2);
        t.hand(P0, "Grizzly Bears");
        let f = t.battlefield(P0, "Falkenrath Pit Fighter");
        (t, f)
    };
    // No opponent lost life: it can't be activated.
    let (mut t, f) = setup();
    assert!(!can_activate(&mut t, P0, f));
    // Only P0 lost life: still not.
    lose_life(&mut t, P0, 2);
    assert!(!can_activate(&mut t, P0, f));
    // P1 lost 1 life and gained 3: their life total is higher than at the start of the
    // turn, but they lost life this turn.
    let (mut t, f) = setup();
    lose_life(&mut t, P1, 1);
    gain_life(&mut t, P1, 3);
    assert_eq!(t.life(P1), 22);
    assert!(can_activate(&mut t, P0, f));
    let hand = t.hand_size(P0);
    t.answer_choose(P0, &[obj(f)]);
    t.activate(P0, f, 0, &[]).unwrap();
    t.resolve_all();
    // Discarded one, drew two; the Pit Fighter sacrificed itself.
    assert_eq!(t.hand_size(P0), hand + 1);
    assert!(t.in_graveyard(P0, "Falkenrath Pit Fighter"));
}

#[test]
fn baron_bertram_graywater_token_copy_and_original_both_trigger() {
    cr!("603.2", "603.6a", "704.5j");
    ruling!(
        "Baron Bertram Graywater",
        "If Baron Bertram Graywater enters under your control and is itself a token, its own ability will trigger and you’ll create a Vampire Rogue token."
    );
    supported("Baron Bertram Graywater");
    // "Whenever one or more tokens you control enter, create a 1/1 black Vampire Rogue
    // creature token with lifelink. This ability triggers only once each turn."
    let mut t = TestGame::new(2);
    let baron = t.battlefield(P0, "Baron Bertram Graywater");
    // (The legend rule is applied before the triggers are put on the stack, so the
    // token may already be gone.)
    crate::r_s17_common::token_copy(&mut t, P0, baron);
    assert_eq!(triggers_on_stack(&t, "Vampire Rogue"), 2);
    // Both Barons' abilities triggered (the token's own and the nontoken one's); the
    // legend rule then made P0 keep only one of them.
    t.resolve_all();
    let rogues =
        t.g.permanents()
            .filter(|o| o.controller == P0 && o.chars.has_subtype("Rogue"))
            .count();
    assert_eq!(rogues, 2);
    assert_eq!(t.named_on_battlefield("Baron Bertram Graywater").len(), 1);
}

#[test]
fn tail_the_suspects_extra_land_plays_are_cumulative() {
    cr!("305.2", "305.2a", "715.3");
    ruling!(
        "Kellan, Inquisitive Prodigy // Tail the Suspect",
        "The effect of Tail the Suspect that allows you to play an additional land is cumulative with similar effects."
    );
    // The Adventure: "Investigate. You may play an additional land this turn."
    let c = mtg_engine::card::card("Kellan, Inquisitive Prodigy // Tail the Suspect");
    assert!(c.faces[1].unsupported.is_empty());
    let mut t = TestGame::new(2);
    for _ in 0..2 {
        t.lands(P0, "Forest", 1);
        t.lands(P0, "Island", 1);
        let k = t.hand(P0, "Kellan, Inquisitive Prodigy // Tail the Suspect");
        t.cast(P0, k).method(CastMethod::Half(1)).go();
        t.resolve_all();
    }
    // Two Clues, and three land plays this turn.
    let clues =
        t.g.permanents()
            .filter(|o| o.controller == P0 && o.chars.has_subtype("Clue"))
            .count();
    assert_eq!(clues, 2);
    let mut played = 0;
    for _ in 0..5 {
        let land = t.hand(P0, "Forest");
        if t.play_land(P0, land).is_ok() {
            played += 1;
        }
    }
    assert_eq!(played, 3);
}

#[test]
fn vraska_sacrifice_is_chosen_and_made_while_the_ability_resolves() {
    cr!("608.2c");
    ruling!(
        "Vraska, Golgari Queen",
        "You choose whether to sacrifice a permanent (and which one to sacrifice) while Vraska's first ability is resolving."
    );
    supported("Vraska, Golgari Queen");
    // "+2: You may sacrifice another permanent. If you do, you gain 1 life and draw a
    // card."
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Vraska, Golgari Queen");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let from = n_asked(&t);
    t.activate(P0, v, 0, &[]).unwrap();
    // Nothing was chosen or sacrificed while activating.
    assert!(t.on_battlefield(bears));
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseEntities { .. })));
    // As it resolves: P0 chooses the Bears, which is sacrificed with no priority in
    // between.
    yes(&mut t, P0);
    t.answer_choose(P0, &[obj(bears)]);
    let hand = t.hand_size(P0);
    let from = n_asked(&t);
    t.g.resolve_top();
    assert!(t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseEntities { .. })));
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::Priority { .. })));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.life(P0), 21);
    assert_eq!(t.hand_size(P0), hand + 1);
    // Choosing not to sacrifice: nothing happens.
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Vraska, Golgari Queen");
    t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, v, 0, &[]).unwrap();
    t.answer_yes(P0, false);
    t.resolve_all();
    assert!(!t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.life(P0), 20);
}

#[test]
fn peregrin_took_applies_to_tokens_you_create_whoever_controls_the_effect() {
    cr!("614.1a", "111.2");
    ruling!(
        "Peregrin Took",
        "You don't need to control the spell or ability that creates the tokens, but you do have to be the one creating the tokens for Peregrin Took's ability to apply."
    );
    supported("Peregrin Took");
    // "If one or more tokens would be created under your control, those tokens plus an
    // additional Food token are created instead."
    let create = |t: &mut TestGame, by: PlayerId, under: PlayerId| {
        run_from(
            t,
            by,
            None,
            mtg_engine::ability::Effect::CreateToken {
                spec: mtg_engine::tokens::predefined("Treasure").unwrap(),
                count: mtg_engine::ability::Value::c(1),
                controller: mtg_engine::ability::PlayerRef::Player(under),
                tapped: false,
                attacking: false,
            },
            &[],
        );
    };
    let foods = |t: &TestGame, p: PlayerId| {
        t.g.permanents()
            .filter(|o| o.controller == p && o.chars.has_subtype("Food"))
            .count()
    };
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Peregrin Took");
    // An opponent's effect has P0 create a Treasure: P0 also creates a Food.
    create(&mut t, P1, P0);
    assert_eq!((treasures(&t, P0), foods(&t, P0)), (1, 1));
    // P0's effect has the opponent create a Treasure: no Food for anyone.
    create(&mut t, P0, P1);
    assert_eq!((treasures(&t, P1), foods(&t, P1), foods(&t, P0)), (1, 0, 1));
}

#[test]
fn commissar_severina_raine_counts_teammates_attackers() {
    cr!("805.10a", "810.9");
    ruling!(
        "Commissar Severina Raine",
        "In Two-Headed Giant and other formats where multiple players can attack at the same time, Commissar Severina Raine's triggered ability counts all attacking creatures, even those controlled by other players."
    );
    supported("Commissar Severina Raine");
    // "Whenever Commissar Severina Raine attacks, each opponent loses X life, where X is
    // the number of other attacking creatures."
    let mut t = two_headed_giant();
    let s = t.battlefield(P0, "Commissar Severina Raine");
    let b = t.battlefield(P1, "Grizzly Bears");
    let life = t.life(P2);
    attack_with(&mut t, &[(s, Entity::Player(P2)), (b, Entity::Player(P2))]);
    t.resolve_all();
    // X = 1 (the teammate's Bears); each of the two opponents loses 1 life from the
    // shared team life total.
    assert_eq!(life - t.life(P2), 2);
}

#[test]
fn blood_host_gains_life_even_if_it_left() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Blood Host",
        "If Blood Host isn’t on the battlefield when its ability resolves, you won’t put a +1/+1 counter on it but you’ll still gain 2 life."
    );
    supported("Blood Host");
    // "{1}{B}, Sacrifice another creature: Put a +1/+1 counter on this creature and you
    // gain 2 life."
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    let h = t.battlefield(P0, "Blood Host");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(bears)]);
    t.activate(P0, h, 0, &[]).unwrap();
    destroy(&mut t, h);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    assert!(t.in_graveyard(P0, "Blood Host"));
    // With Blood Host still there: the counter too.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    let h = t.battlefield(P0, "Blood Host");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(bears)]);
    t.activate(P0, h, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!((t.life(P0), t.pt(h)), (22, (4, 4)));
}

/// Brawl-Bash Ogre attacks with Grizzly Bears and Hill Giant beside it; P0 answers yes
/// and chooses `victims` when the trigger resolves. Returns the game, the Ogre and the
/// decisions asked before the trigger resolved.
fn brawl_bash(victims: &[&str]) -> (TestGame, ObjectId, usize) {
    supported("Brawl-Bash Ogre");
    let mut t = TestGame::new(2);
    let o = t.battlefield(P0, "Brawl-Bash Ogre");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    attack_with(&mut t, &[(o, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 1);
    yes(&mut t, P0);
    let vs: Vec<Entity> = victims
        .iter()
        .map(|n| obj(if *n == "Grizzly Bears" { a } else { b }))
        .collect();
    t.answer_choose(P0, &vs);
    let from = n_asked(&t);
    t.g.resolve_top();
    (t, o, from)
}

#[test]
fn brawl_bash_ogre_no_one_acts_between_the_sacrifice_and_the_pump() {
    cr!("608.2c", "117.1");
    ruling!(
        "Brawl-Bash Ogre",
        "Once Brawl-Bash Ogre’s triggered ability begins to resolve, no player may take actions until it’s done."
    );
    // "Whenever this creature attacks, you may sacrifice another creature. If you do, this
    // creature gets +2/+2 until end of turn."
    let (t, o, from) = brawl_bash(&["Grizzly Bears"]);
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::Priority { .. })));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.pt(o), (5, 5));
}

#[test]
fn brawl_bash_ogre_sacrifices_only_one_creature() {
    cr!("608.2c", "701.21a");
    ruling!(
        "Brawl-Bash Ogre",
        "While resolving the triggered ability of Brawl-Bash Ogre, you can’t sacrifice multiple creatures to give it +2/+2 more than once."
    );
    // Trying to sacrifice both: at most one creature is sacrificed, for one +2/+2.
    let (t, o, from) = brawl_bash(&["Grizzly Bears", "Hill Giant"]);
    let max = t.asked()[from..].iter().find_map(|(_, d)| match d {
        Decision::ChooseEntities { max, .. } => Some(*max),
        _ => None,
    });
    assert_eq!(max, Some(1));
    let gone = [
        t.in_graveyard(P0, "Grizzly Bears"),
        t.in_graveyard(P0, "Hill Giant"),
    ];
    assert_eq!(gone.iter().filter(|g| **g).count(), 1);
    assert_eq!(t.pt(o), (5, 5));
}

#[test]
fn master_transmuter_can_return_itself_and_put_itself_back() {
    cr!("602.2b", "601.2h", "400.7");
    ruling!(
        "Master Transmuter",
        "Master Transmuter can be returned to its owner's hand to pay the cost of its activated ability."
    );
    ruling!(
        "Master Transmuter",
        "The artifact card you put onto the battlefield when the ability resolves may be the same card that you returned to your hand when you paid the cost."
    );
    supported("Master Transmuter");
    // "{U}, {T}, Return an artifact you control to its owner's hand: You may put an
    // artifact card from your hand onto the battlefield."
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 1);
    let m = t.battlefield(P0, "Master Transmuter");
    put_counters(&mut t, m, "+1/+1", 1);
    t.answer_choose(P0, &[obj(m)]);
    t.activate(P0, m, 0, &[]).unwrap();
    // Paid: it's in P0's hand.
    let in_hand = t.g.find_in_zone(Zone::Hand(P0), "Master Transmuter");
    assert_eq!(in_hand.len(), 1);
    yes(&mut t, P0);
    t.answer_choose(P0, &[obj(in_hand[0])]);
    t.resolve_all();
    // It's back as a new object: untapped, without the counter, summoning sick.
    let back = t.named_on_battlefield("Master Transmuter");
    assert_eq!(back.len(), 1);
    assert_ne!(back[0], m);
    let o = t.obj(back[0]);
    assert!(!o.tapped && o.summoning_sick);
    assert_eq!(t.counters(back[0], "+1/+1"), 0);
}
