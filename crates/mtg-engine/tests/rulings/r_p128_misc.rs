//! Rulings batch P128 — the rest: blocking restrictions checked only as blockers are
//! declared (CR 509.1b), "attacks alone", "whenever a creature attacks", token copies and
//! their enters abilities (CR 707.2, 603.6a), phasing and durations (CR 702.26), players
//! choosing in turn order (CR 101.4), decayed (CR 702.147), naming cards, copies of cards
//! in exile, cards drawn this way, restricted mana that stays in the pool (CR 106.6,
//! 500.4), and returning exiled cards with flashback.

use crate::r_p128_common::*;
use crate::r_s17_common::enter_transformed;
use crate::r_s25_common::name_now;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// The players asked to choose entities (in order) since decision `from`.
fn choosers_since(t: &TestGame, from: usize) -> Vec<PlayerId> {
    t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
        .map(|(p, _)| *p)
        .collect()
}

/// Wraps `p`'s agent so that it chooses the first candidate whenever it's asked for
/// targets (other decisions are answered from the script).
fn first_target_candidate(t: &mut TestGame, p: PlayerId) {
    struct First(Box<dyn mtg_engine::decision::Agent>);
    impl mtg_engine::decision::Agent for First {
        fn decide(&mut self, g: &mtg_engine::game::Game, p: PlayerId, d: &Decision) -> Answer {
            match d {
                Decision::ChooseTargets { candidates, .. } if !candidates.is_empty() => {
                    Answer::Entities(vec![candidates[0]])
                }
                _ => self.0.decide(g, p, d),
            }
        }
    }
    let mut agents = t.g.agents.0.lock().unwrap();
    let inner = std::mem::replace(
        &mut agents[p.idx()],
        Box::new(mtg_engine::decision::PassiveAgent),
    );
    agents[p.idx()] = Box::new(First(inner));
}

#[test]
fn blocking_restrictions_apply_only_as_blockers_are_declared() {
    cr!("509.1b", "506.4");
    ruling!(
        "Sandman, Shifting Scoundrel",
        "Sandman's second ability applies only at the time blockers are declared. For example, if Sandman is blocked by a creature with 4 power, and then an effect gives that creature -3/-0, it won't stop blocking Sandman."
    );
    ruling!(
        "Glissa's Retriever",
        "After Glissa's Retriever has become blocked by a creature with power 3 or greater, reducing the blocking creature's power won't cause Glissa's Retriever to become unblocked."
    );
    supported("Glissa's Retriever");
    for card in ["Sandman, Shifting Scoundrel", "Glissa's Retriever"] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Forest", 3);
        let a = t.battlefield(P0, card);
        let giant = t.battlefield(P1, "Hill Giant");
        let (ap, at) = t.pt(a);
        attack_with(&mut t, &[(a, Entity::Player(P1))]);
        t.answer(
            P1,
            DecisionKind::Blockers,
            Answer::Blockers(vec![(giant, a)]),
        );
        t.advance_to(P0, Step::DeclareBlockers);
        assert!(t.g.combat.as_ref().unwrap().is_blocked(a), "{card}");
        cast_new(&mut t, P1, "Startle", &[obj(giant)]);
        t.resolve_all();
        assert_eq!(t.pt(giant).0, 1);
        t.advance_to(P0, Step::EndOfCombat);
        assert_eq!(t.life(P1), 20, "{card} stayed blocked");
        // It dealt its combat damage to the Giant.
        assert!(t.in_graveyard(P1, "Hill Giant"), "{card} {ap}/{at}");
    }
}

#[test]
fn ancestral_katana_needs_a_lone_attacker() {
    cr!("506.5", "603.2");
    ruling!(
        "Ancestral Katana",
        "A creature attacks alone if it's the only creature declared as an attacker during the declare attackers step."
    );
    supported("Ancestral Katana");
    for n in [2, 1] {
        let mut t = TestGame::new(2);
        let katana = t.battlefield(P0, "Ancestral Katana");
        t.lands(P0, "Wastes", 1);
        let ws: Vec<_> = (0..n)
            .map(|_| t.battlefield(P0, "Bloodsoaked Champion"))
            .collect();
        t.answer_yes(P0, true);
        let decl: Vec<_> = ws.iter().map(|w| (*w, Entity::Player(P1))).collect();
        attack_with(&mut t, &decl);
        t.resolve_all();
        let attached = attached_to(&t, katana);
        if n == 1 {
            assert_eq!(attached, Some(obj(ws[0])));
        } else {
            assert_eq!(attached, None, "two attackers");
        }
    }
}

#[test]
fn anointer_of_valor_sees_any_creature_attacking() {
    cr!("508.1", "603.2");
    ruling!(
        "Anointer of Valor",
        "Anointer of Valor's triggered ability triggers when any creature attacks any player or planeswalker."
    );
    supported("Anointer of Valor");
    // An opponent's creature attacking P0, during P1's turn.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Anointer of Valor");
    t.lands(P0, "Wastes", 3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(bears, Entity::Player(P0))]);
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
}

#[test]
fn saheeli_token_copy_gets_enters_abilities() {
    cr!("707.2", "603.6a", "614.1c");
    ruling!(
        "Saheeli, Radiant Creator",
        "Any abilities of the copied permanent that trigger when that permanent enters will trigger. Any “as [this permanent] enters” or “[this permanent] enters with” abilities of the chosen permanent will also work."
    );
    supported("Saheeli, Radiant Creator");
    for (copied, draws, counters_on) in [("Elvish Visionary", 1, 0), ("Spike Feeder", 0, 2)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Saheeli, Radiant Creator");
        let target = t.enter(P0, copied);
        t.resolve_all();
        t.g.players[0].counters.insert(counters::ENERGY.into(), 3);
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[obj(target)]);
        let hand = t.hand_size(P0);
        t.advance_to(P0, Step::BeginningOfCombat);
        t.resolve_all();
        let tok = tokens(&t, P0);
        assert_eq!(tok.len(), 1, "{copied}");
        assert_eq!(t.hand_size(P0), hand + draws, "{copied}");
        assert_eq!(t.counters(tok[0], counters::PLUS1), counters_on, "{copied}");
        assert!(t.obj(tok[0]).is(CardType::Artifact));
    }
}

#[test]
fn phasing_out_ends_for_as_long_as_durations_and_takes_attachments() {
    cr!("702.26d", "702.26f", "702.26g", "611.2b");
    ruling!(
        "Spectral Adversary",
        "Any continuous effects with a \"for as long as\" duration ignore phased-out objects. If ignoring those objects causes the effect's conditions to no longer be met, the duration will expire."
    );
    ruling!(
        "Spectral Adversary",
        "As a permanent phases out, Auras and Equipment attached to it also phase out at the same time. Those Auras and Equipment will phase in at the same time that permanent does, and they'll phase in still attached to that permanent."
    );
    supported("Mind Flayer");
    // P1's Mind Flayer took P0's Grizzly Bears; phasing the Flayer out ends that.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P1, &[obj(bears)]);
    let flayer = t.enter(P1, "Mind Flayer");
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P1);
    t.lands(P0, "Island", 2);
    for yes in [true, true, false] {
        t.answer_yes(P0, yes);
    }
    t.answer_targets(P0, &[obj(flayer)]);
    t.enter(P0, "Spectral Adversary");
    t.resolve_all();
    assert!(t.obj_now(flayer).phased_out);
    assert_eq!(t.obj_now(bears).controller, P0);
    // P1's Grizzly Bears with P1's Rancor: both phase out, and in together, attached.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let rancor = attach_new(&mut t, P1, "Rancor", obj(bears));
    t.lands(P0, "Island", 2);
    for yes in [true, true, false] {
        t.answer_yes(P0, yes);
    }
    t.answer_targets(P0, &[obj(bears)]);
    t.enter(P0, "Spectral Adversary");
    t.resolve_all();
    assert!(t.obj_now(bears).phased_out && t.obj_now(rancor).phased_out);
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.obj_now(bears).phased_out && !t.obj_now(rancor).phased_out);
    assert_eq!(attached_to(&t, rancor), Some(obj(bears)));
    assert_eq!(t.pt(bears), (4, 2));
}

#[test]
fn augusta_each_player_exiles_in_turn_order() {
    cr!("101.4", "603.12");
    ruling!(
        "Augusta, Order Returned",
        "As Augusta's last ability resolves, the player whose turn it is chooses a card to exile from their graveyard, then each other player in turn order does the same, then all of the chosen cards are exiled at the same time."
    );
    supported("Augusta, Order Returned");
    let mut t = TestGame::new(3);
    let aug = t.battlefield(P0, "Augusta, Order Returned");
    let picks = [
        t.graveyard(P0, "Grizzly Bears"),
        t.graveyard(P1, "Opt"),
        t.graveyard(P2, "Forest"),
    ];
    t.graveyard(P0, "Forest");
    t.graveyard(P1, "Forest");
    t.graveyard(P2, "Island");
    for (i, p) in [P0, P1, P2].into_iter().enumerate() {
        t.answer_choose(p, &[obj(picks[i])]);
    }
    t.answer_targets(P0, &[obj(aug)]);
    // When P2 chooses, nothing has been exiled yet.
    let seen = watch(
        &mut t,
        P2,
        |d| matches!(d, Decision::ChooseEntities { .. }),
        |g| g.exile.len(),
    );
    let from = t.asked().len();
    attack_with(&mut t, &[(aug, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(choosers_since(&t, from), vec![P0, P1, P2]);
    assert_eq!(seen.lock().unwrap().as_slice(), &[0]);
    for c in picks {
        assert!(is_in(&t, c, Zone::Exile));
    }
    assert_eq!(t.counters(aug, counters::PLUS1), 2, "two nonland cards");
}

#[test]
fn kardurs_vicious_return_discards_in_turn_order_at_once() {
    cr!("101.4", "714.2b");
    ruling!(
        "Kardur's Vicious Return",
        "As the chapter II ability resolves, first the player whose turn it is chooses a card in hand and sets it aside without revealing it, then each other player in turn order does the same. Then all the chosen cards are discarded at the same time."
    );
    supported("Kardur's Vicious Return");
    let mut t = TestGame::new(3);
    let saga = t.battlefield(P0, "Kardur's Vicious Return");
    put_counters(&mut t, saga, "lore", 1);
    t.resolve_all();
    let mut chosen = vec![];
    for p in [P0, P1, P2] {
        let c = t.hand(p, "Grizzly Bears");
        t.hand(p, "Forest");
        t.answer_choose(p, &[obj(c)]);
        chosen.push(c);
    }
    // When P2 chooses, nothing has been discarded yet.
    let seen = watch(
        &mut t,
        P2,
        |d| matches!(d, Decision::ChooseEntities { .. }),
        |g| g.players.iter().map(|p| p.graveyard.len()).sum::<usize>(),
    );
    let from = t.asked().len();
    put_counters(&mut t, saga, "lore", 1);
    t.resolve_all();
    assert_eq!(choosers_since(&t, from), vec![P0, P1, P2]);
    assert_eq!(seen.lock().unwrap().as_slice(), &[0]);
    for c in chosen {
        let owner = t.obj(c).owner;
        assert!(is_in(&t, c, Zone::Graveyard(owner)));
    }
}

#[test]
fn decayed_cant_block_and_is_sacrificed_after_attacking() {
    cr!("702.147a", "509.1b");
    ruling!(
        "Falcon Abomination",
        "Decayed represents a static ability and a triggered ability. \"Decayed\" means \"This creature can't block\" and \"When this creature attacks, sacrifice it at end of combat.\""
    );
    ruling!(
        "Gisa, Glorious Resurrector",
        "Decayed represents a static ability and a triggered ability. \"Decayed\" means \"This creature can't block\" and \"When this creature attacks, sacrifice it at end of combat.\""
    );
    supported("Falcon Abomination");
    supported("Gisa, Glorious Resurrector");
    // Falcon Abomination's token, and a creature Gisa returns: they can't block...
    let mut t = TestGame::new(2);
    t.enter(P0, "Falcon Abomination");
    t.resolve_all();
    let zombie = crate::r_s01_common::with_subtype(&t, P0, "Zombie")
        .into_iter()
        .find(|z| t.obj(*z).is_token())
        .unwrap();
    t.battlefield(P0, "Gisa, Glorious Resurrector");
    let giant = t.battlefield(P1, "Hill Giant");
    destroy(&mut t, giant);
    assert!(is_in(&t, giant, Zone::Exile));
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.obj_now(giant).controller, P0);
    let g = t.g.current(giant);
    // P1 attacks on their next turn: neither can block.
    let attacker = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(attacker, Entity::Player(P0))]);
    t.answer(
        P0,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(zombie, attacker), (g, attacker)]),
    );
    let life = t.life(P0);
    t.advance_to(P1, Step::EndOfCombat);
    assert_eq!(t.life(P0), life - 2, "unblocked");
    // ... and when they attack, they're sacrificed at end of combat.
    t.advance_to(P0, Step::BeginningOfCombat);
    attack_with(
        &mut t,
        &[(zombie, Entity::Player(P1)), (g, Entity::Player(P1))],
    );
    assert!(t.on_battlefield(zombie) && t.on_battlefield(g), "attacking");
    t.advance_to(P0, Step::PostcombatMain);
    assert!(!t.on_battlefield(zombie));
    assert!(!t.on_battlefield(g));
}

#[test]
fn the_rise_of_sozin_exiles_up_to_four_named_cards() {
    cr!("201.4", "701.23a");
    ruling!(
        "The Rise of Sozin // Fire Lord Sozin",
        "Even if you choose a card name (for example, a basic land) that appears more than four times in your opponent's deck, you may only choose up to four copies of the card. You can also always choose fewer than four copies of the card to exile, even if there are more in the deck."
    );
    supported("The Rise of Sozin // Fire Lord Sozin");
    for n in [4usize, 2] {
        let mut t = TestGame::new(2);
        let saga = t.battlefield(P0, "The Rise of Sozin // Fire Lord Sozin");
        put_counters(&mut t, saga, "lore", 1);
        t.resolve_all();
        let mut forests = vec![];
        for _ in 0..3 {
            forests.push(t.library_top(P1, "Forest"));
        }
        forests.push(t.hand(P1, "Forest"));
        forests.push(t.hand(P1, "Forest"));
        forests.push(t.graveyard(P1, "Forest"));
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.answer(P0, DecisionKind::Name, Answer::Text("Forest".into()));
        let pick: Vec<Entity> = forests[..n].iter().map(|f| obj(*f)).collect();
        t.answer_choose(P0, &pick);
        put_counters(&mut t, saga, "lore", 1);
        t.resolve_all();
        let exiled =
            t.g.exile
                .iter()
                .filter(|e| t.obj(**e).chars.name == "Forest")
                .count();
        assert_eq!(exiled, n);
    }
}

#[test]
fn fire_lord_sozin_targets_total_mana_value_and_any_zeros() {
    cr!("601.2c", "115.1", "202.3");
    ruling!(
        "The Rise of Sozin // Fire Lord Sozin",
        "Fire Lord Sozin's triggered ability can target any combination of creature cards whose mana values total X or less. Regardless of what other creature cards you choose as targets, you can also choose any number of creature cards with mana value 0 as targets."
    );
    let mut t = TestGame::new(2);
    let sozin = enter_transformed(&mut t, P0, "The Rise of Sozin // Fire Lord Sozin");
    t.g.objects[sozin.0 as usize].summoning_sick = false;
    let cards = [
        t.graveyard(P1, "Grizzly Bears"),
        t.graveyard(P1, "Llanowar Elves"),
        t.graveyard(P1, "Ornithopter"),
        t.graveyard(P1, "Memnite"),
    ];
    t.graveyard(P1, "Hill Giant");
    t.lands(P0, "Wastes", 3);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.answer(P0, DecisionKind::Number, Answer::Number(3));
    let ts: Vec<Entity> = cards.iter().map(|c| obj(*c)).collect();
    t.answer_targets(P0, &ts);
    attack_with(&mut t, &[(sozin, Entity::Player(P1))]);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    for c in cards {
        assert!(t.on_battlefield(c), "{}", t.obj(c).chars.name);
        assert_eq!(t.obj_now(c).controller, P0);
    }
    assert!(t.in_graveyard(P1, "Hill Giant"));
}

#[test]
fn espers_to_magicite_copies_only_the_printed_card() {
    cr!("707.2", "400.7", "707.9b");
    ruling!(
        "Espers to Magicite",
        "Except for the listed exceptions, the token copies exactly what was printed on the original card and nothing else. It doesn't copy any information about the object the card may have been at any point before it was exiled."
    );
    supported("Espers to Magicite");
    let mut t = TestGame::new(2);
    // P1's Clone entered as a copy of Hill Giant and died.
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[obj(giant)]);
    let clone = t.enter(P1, "Clone");
    t.settle();
    assert_eq!(name_now(&t, clone), "Hill Giant");
    destroy(&mut t, clone);
    t.clear_answers();
    // P0 targets the first candidate (the Clone card, a new object in exile); "As this
    // enters, you may have it enter as a copy" is declined.
    first_target_candidate(&mut t, P0);
    t.answer_yes(P0, false);
    cast_new(&mut t, P0, "Espers to Magicite", &[]);
    t.resolve_all();
    let tok = tokens(&t, P0);
    assert_eq!(tok.len(), 1);
    let o = t.obj(tok[0]);
    assert_eq!(o.chars.name, "Clone");
    assert!(o.is(CardType::Artifact) && !o.is(CardType::Creature));
}

#[test]
fn explosion_of_riches_triggers_for_each_card_drawn() {
    cr!("603.12", "101.4", "115.1");
    ruling!(
        "Explosion of Riches",
        "Explosion of Riches goes on the stack without a target. While the spell is resolving, you draw a card, then each other player in turn order chooses whether to draw a card. After that, each other player who chose to draw a card does so. Finally, Explosion of Riches's reflexive triggered ability triggers for each card drawn while it was resolving, and those abilities are put on the stack with random targets chosen."
    );
    supported("Explosion of Riches");
    let mut t = TestGame::new(3);
    t.answer_yes(P1, true);
    t.answer_yes(P2, false);
    let hands = [t.hand_size(P0), t.hand_size(P1), t.hand_size(P2)];
    let spell = cast_new(&mut t, P0, "Explosion of Riches", &[]);
    assert!(crate::r_s25_common::targets_of(&t, spell).is_empty());
    t.resolve();
    t.settle();
    assert_eq!(t.stack_len(), 2, "two cards drawn: two triggers");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hands[0] + 1);
    assert_eq!(t.hand_size(P1), hands[1] + 1);
    assert_eq!(t.hand_size(P2), hands[2]);
    assert_eq!(40 - t.life(P1) - t.life(P2), 10);
}

#[test]
fn restricted_red_mana_keeps_its_restriction_while_it_stays() {
    cr!("106.6", "106.4", "500.4");
    ruling!(
        "Leyline Tyrant",
        "If a red mana you add has certain restrictions or riders associated with it (for example, if it was produced by Base Camp), they'll apply to that mana no matter when you spend it."
    );
    ruling!(
        "Ashling, Flame Dancer",
        "If a red mana you add has certain restrictions or riders associated with it (for example, if it was produced by the last ability of Arena of Glory), they'll apply to that mana no matter when you spend it."
    );
    ruling!(
        "Electro, Assaulting Battery",
        "If a red mana you add has certain restrictions or riders associated with it (for example, if it was produced by the last ability of Arena of Glory), they'll apply to that mana no matter when you spend it."
    );
    supported("Leyline Tyrant");
    supported("Base Camp");
    supported("Arena of Glory");
    // Leyline Tyrant + Base Camp's red mana: kept into combat, still only for Clerics,
    // Rogues, Warriors and Wizards (not Lightning Bolt; Grim Lavamancer's ability yes).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Leyline Tyrant");
    let camp = t.battlefield(P0, "Base Camp");
    let lava = t.battlefield(P0, "Grim Lavamancer");
    t.graveyard(P0, "Forest");
    t.graveyard(P0, "Forest");
    let bolt = t.hand(P0, "Lightning Bolt");
    t.answer(P0, DecisionKind::Option, Answer::Index(3));
    activate_containing(&mut t, P0, camp, "Add one mana").unwrap();
    assert_eq!(t.g.players[0].mana_pool.total(), 1);
    t.advance_to(P0, Step::BeginningOfCombat);
    assert_eq!(t.g.players[0].mana_pool.total(), 1, "the red mana stayed");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    assert!(
        t.cast(P0, bolt).try_go().is_err(),
        "Lightning Bolt can't use it"
    );
    t.clear_answers();
    assert_eq!(t.g.players[0].mana_pool.total(), 1);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_containing(&mut t, P0, lava, "damage").unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // Ashling / Electro + Arena of Glory's {R}{R}: kept into the second main phase, and
    // the creature spell it's spent on still gains haste.
    for keeper in ["Ashling, Flame Dancer", "Electro, Assaulting Battery"] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, keeper);
        let arena = t.battlefield(P0, "Arena of Glory");
        t.lands(P0, "Mountain", 1);
        activate_containing(&mut t, P0, arena, "Exert").unwrap();
        t.resolve_all();
        assert_eq!(t.g.players[0].mana_pool.total(), 2, "{keeper}");
        t.advance_to(P0, Step::PostcombatMain);
        assert_eq!(t.g.players[0].mana_pool.total(), 2, "{keeper}: kept");
        let piker = t.hand(P0, "Goblin Piker");
        t.cast(P0, piker).go();
        t.resolve_all();
        assert!(
            t.obj_now(piker)
                .has_keyword(mtg_engine::keywords::KeywordKind::Haste),
            "{keeper}"
        );
    }
}

#[test]
fn runic_repetition_targets() {
    cr!("115.1", "702.34a", "708.2", "400.7");
    ruling!(
        "Runic Repetition",
        "The card could have been exiled for any reason, not just because it was cast using flashback."
    );
    ruling!(
        "Runic Repetition",
        "A card that’s exiled face down doesn’t have any characteristics or abilities, so it can’t be the target of Runic Repetition."
    );
    ruling!(
        "Runic Repetition",
        "An effect that gives flashback to an instant or sorcery card in your graveyard stops applying once that card has left the stack. The card won’t have flashback while exiled and can’t be the target of Runic Repetition (unless it naturally has flashback)."
    );
    supported("Runic Repetition");
    // Think Twice exiled by an effect: returned.
    let mut t = TestGame::new(2);
    let tt = t.graveyard(P0, "Think Twice");
    exile_now(&mut t, tt);
    let rr = t.hand(P0, "Runic Repetition");
    t.lands(P0, "Island", 3);
    assert!(castable(&mut t, P0, rr));
    t.answer_targets(P0, &[obj(t.g.current(tt))]);
    t.cast(P0, rr).go();
    t.resolve_all();
    assert!(is_in(&t, tt, Zone::Hand(P0)));
    // Face down in exile: no legal target.
    let mut t = TestGame::new(2);
    let tt = t.exile(P0, "Think Twice");
    t.g.objects[tt.0 as usize].face_down = true;
    t.g.recompute();
    let rr = t.hand(P0, "Runic Repetition");
    t.lands(P0, "Island", 3);
    assert!(!castable(&mut t, P0, rr), "face down");
    // Lightning Bolt cast with Snapcaster Mage's flashback, then exiled: no flashback.
    let mut t = TestGame::new(2);
    let bolt = t.graveyard(P0, "Lightning Bolt");
    t.answer_targets(P0, &[obj(bolt)]);
    t.enter(P0, "Snapcaster Mage");
    t.resolve_all();
    t.lands(P0, "Mountain", 1);
    let fb = legal_cast_methods(&mut t, P0, bolt);
    assert!(!fb.is_empty(), "flashback from Snapcaster Mage");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, bolt).method(fb[0].clone()).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(is_in(&t, bolt, Zone::Exile));
    let rr = t.hand(P0, "Runic Repetition");
    t.lands(P0, "Island", 3);
    assert!(!castable(&mut t, P0, rr), "no flashback in exile");
}

#[test]
fn jayas_phoenix_copy_resolves_first_even_if_the_ability_is_countered() {
    cr!("707.10", "603.7a", "608.1");
    ruling!(
        "Jaya's Phoenix",
        "The copy of the loyalty ability resolves before the loyalty ability it's copying. It resolves even if that loyalty ability is countered."
    );
    supported("Jaya's Phoenix");
    supported("Ajani Goldmane");
    supported("Stifle");
    for counter in [false, true] {
        let mut t = TestGame::new(2);
        let ph = t.battlefield(P0, "Jaya's Phoenix");
        let ajani = t.enter(P0, "Ajani Goldmane");
        t.resolve_all();
        attack_with(&mut t, &[(ph, Entity::Player(P1))]);
        t.advance_to(P0, Step::PostcombatMain);
        t.resolve_all();
        assert_eq!(t.life(P1), 17);
        // "+1: You gain 2 life." The Phoenix's delayed ability triggers and copies it.
        activate_containing(&mut t, P0, ajani, "gain 2 life").unwrap();
        t.settle();
        assert_eq!(t.stack_len(), 2, "the loyalty ability and the Phoenix's trigger");
        t.resolve();
        t.settle();
        assert_eq!(t.stack_len(), 2, "the loyalty ability and its copy");
        let original = t.g.stack[0];
        if counter {
            cast_new(&mut t, P1, "Stifle", &[obj(original)]);
            t.resolve();
            assert_eq!(t.stack_len(), 1, "only the copy is left");
        }
        // The copy is on top: it resolves first.
        t.resolve();
        assert_eq!(t.life(P0), 22, "counter: {counter}");
        let left: Vec<ObjectId> = if counter { vec![] } else { vec![original] };
        assert_eq!(t.g.stack, left, "counter: {counter}");
        t.resolve_all();
        assert_eq!(t.life(P0), if counter { 22 } else { 24 });
    }
}
