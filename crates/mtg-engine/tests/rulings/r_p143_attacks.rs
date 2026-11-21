//! Rulings batch P143 — attack triggers and tokens that enter attacking (CR 508.3, 508.4),
//! attacking alone, exalted from Clues, intervening "if" clauses, and "you may pay"
//! triggers paid once.

use crate::r_p143_common::*;
use crate::r_p108_common::put_counters;
use crate::r_s01_common::{block_and_finish, tokens, with_subtype};
use crate::r_s02_common::create_token;
use crate::r_s09_common::{attacks_now, declare};
use mtg_engine::combat::remove_from_combat;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

/// "Whenever a creature you control attacks, you gain 1 life." — counts declared attackers.
fn attack_watcher(t: &mut TestGame) {
    let c = custom_card(
        "Attack Watcher",
        "Enchantment",
        "{1}",
        None,
        "Whenever a creature you control attacks, you gain 1 life.",
    );
    t.custom(P0, c, Zone::Battlefield);
}

fn attacking(t: &TestGame, id: ObjectId) -> Option<Entity> {
    let id = t.g.current(id);
    attacks_now(t).into_iter().find(|(a, _)| *a == id).map(|x| x.1)
}

/// The tokens (other than Clues) `p` controls that are attacking, with what they attack.
fn attacking_tokens(t: &TestGame, p: PlayerId) -> Vec<Entity> {
    tokens(t, p)
        .into_iter()
        .filter_map(|id| attacking(t, id))
        .collect()
}

/// The three cards whose attack trigger creates a token that's tapped and attacking (the
/// Swinemaster's needs {1}{G}).
const TOKEN_ATTACKERS: &[&str] = &["Falconer Adept", "Carefree Swinemaster", "Skyknight Vanguard"];

fn attack_with_token_maker(t: &mut TestGame, name: &str, at: PlayerId) -> ObjectId {
    let c = t.battlefield(P0, name);
    if name == "Carefree Swinemaster" {
        t.lands(P0, "Forest", 2);
        t.answer_yes(P0, true);
    }
    declare(t, P0, &[(c, Entity::Player(at))]);
    t.resolve_all();
    c
}

#[test]
fn tokens_entering_attacking_were_never_declared_as_attackers() {
    cr!("508.4", "508.3a");
    ruling!(
        "Falconer Adept",
        "Although the Bird is an attacking creature, it was never declared as an attacking creature. This means that abilities that trigger whenever a creature attacks won't trigger when it enters the battlefield attacking."
    );
    ruling!(
        "Carefree Swinemaster",
        "Although the token created is an attacking creature, it was never declared as an attacking creature. This means that abilities that trigger whenever a creature attacks won't trigger when it enters the battlefield attacking."
    );
    ruling!(
        "Skyknight Vanguard",
        "Although the token is an attacking creature, it was never declared as an attacking creature. This means that abilities that trigger whenever a creature attacks won’t trigger when the token enters the battlefield attacking."
    );
    for name in TOKEN_ATTACKERS {
        supported(name);
        let mut t = TestGame::new(2);
        attack_watcher(&mut t);
        attack_with_token_maker(&mut t, name, P1);
        assert_eq!(attacking_tokens(&t, P0), vec![Entity::Player(P1)], "{name}");
        assert_eq!(t.life(P0), 21, "{name}");
    }
}

#[test]
fn cant_attack_effects_dont_stop_a_token_entering_attacking() {
    cr!("508.4");
    ruling!(
        "Falconer Adept",
        "Any effects that say that the Bird can't attack (such as that of Propaganda) affect only the declaration of attackers. They won't stop the Bird token from entering the battlefield attacking."
    );
    // "Creature tokens can't attack." would stop the Bird from being declared.
    let mut t = TestGame::new(2);
    let c = custom_card("Token Ban", "Enchantment", "{1}", None, "Creature tokens can't attack.");
    t.custom(P1, c, Zone::Battlefield);
    let soldier = create_token(&mut t, P0, "Soldier");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!crate::r_s09_common::legal_attack(&mut t, &[(soldier, Entity::Player(P1))]));
    attack_with_token_maker(&mut t, "Falconer Adept", P1);
    let bird = with_subtype(&t, P0, "Bird");
    assert_eq!(bird.len(), 1);
    assert_eq!(attacking(&t, bird[0]), Some(Entity::Player(P1)));
}

#[test]
fn the_token_may_attack_a_different_player() {
    cr!("508.4", "508.4a");
    ruling!(
        "Falconer Adept",
        "You choose which player or planeswalker the Bird token is attacking. It doesn't have to be attacking the same player or planeswalker that Falconer Adept is attacking."
    );
    ruling!(
        "Carefree Swinemaster",
        "You choose which player or planeswalker is being attacked by the Boar token as it enters the battlefield. It doesn't have to be the same player or planeswalker that Carefree Swinemaster is attacking."
    );
    ruling!(
        "Skyknight Vanguard",
        "You choose which player or planeswalker the token is attacking. It doesn’t have to be attacking the same player or planeswalker that Skyknight Vanguard is attacking."
    );
    for name in TOKEN_ATTACKERS {
        for to_walker in [false, true] {
            let mut t = TestGame::new(3);
            let walker = t.battlefield(P2, "Narset, Parter of Veils");
            let want = if to_walker { obj(walker) } else { Entity::Player(P2) };
            t.answer_choose(P0, &[want]);
            let c = attack_with_token_maker(&mut t, name, P1);
            assert_eq!(attacking(&t, c), Some(Entity::Player(P1)));
            assert_eq!(attacking_tokens(&t, P0), vec![want], "{name}");
        }
    }
}

#[test]
fn arthur_puts_a_creature_onto_the_battlefield_attacking_anyone() {
    cr!("508.4");
    supported("Arthur, Marigold Knight");
    ruling!(
        "Arthur, Marigold Knight",
        "You choose the player, planeswalker, or battle the creature you put onto the battlefield is attacking. It doesn't have to be the same player, planeswalker, or battle that Arthur or any other attacking creatures are attacking."
    );
    let mut t = TestGame::new(3);
    let arthur = t.battlefield(P0, "Arthur, Marigold Knight");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lib = set_library(&mut t, P0, &["Hill Giant", "Island"]);
    t.answer_choose(P0, &[obj(lib[0])]);
    t.answer_choose(P0, &[Entity::Player(P2)]);
    declare(
        &mut t,
        P0,
        &[(arthur, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    t.resolve_all();
    assert!(t.on_battlefield(lib[0]));
    assert_eq!(attacking(&t, lib[0]), Some(Entity::Player(P2)));
    assert!(t.obj_now(lib[0]).tapped);
}

#[test]
fn attacking_alone_means_declared_alone() {
    cr!("506.5", "702.83b");
    // Tempered in Solitude: "Whenever a creature you control attacks alone, exile the top
    // card of your library. You may play that card this turn."
    supported("Tempered in Solitude");
    ruling!(
        "Tempered in Solitude",
        "A creature attacks alone if it's the only creature declared as an attacker during the declare attackers step. For example, Tempered in Solitude's ability won't trigger if you attack with multiple creatures and all but one of them are removed from combat."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tempered in Solitude");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    declare(&mut t, P0, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    let b_now = t.g.current(b);
    remove_from_combat(&mut t.g, b_now);
    t.settle();
    t.resolve_all();
    assert!(t.g.find_in_zone(Zone::Exile, "Filler").is_empty());
    // Alone, it triggers.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tempered in Solitude");
    let a = t.battlefield(P0, "Grizzly Bears");
    declare(&mut t, P0, &[(a, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.g.find_in_zone(Zone::Exile, "Filler").len(), 1);

    supported("Merchant of Truth");
    ruling!(
        "Merchant of Truth",
        "A creature attacks alone if it’s the only creature declared as an attacker during the declare attackers step (including creatures controlled by your teammates, if applicable). For example, exalted won’t trigger if you attack with multiple creatures and all but one of them are removed from combat. Similarly, creatures that enter the battlefield attacking later in combat won’t affect exalted abilities that have already triggered or resolved."
    );
    // Clues you control have exalted.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Merchant of Truth");
    create_token(&mut t, P0, "Clue");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    declare(&mut t, P0, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    let b_now = t.g.current(b);
    remove_from_combat(&mut t.g, b_now);
    t.settle();
    t.resolve_all();
    assert_eq!(t.pt(a), (2, 2));
    // Falconer Adept attacks alone: exalted triggers, and the Bird entering attacking
    // doesn't undo it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Merchant of Truth");
    create_token(&mut t, P0, "Clue");
    let f = attack_with_token_maker(&mut t, "Falconer Adept", P1);
    assert_eq!(attacking_tokens(&t, P0).len(), 1);
    assert_eq!(t.pt(f), (3, 4));
}

#[test]
fn clues_have_an_instance_of_exalted_per_merchant() {
    cr!("702.83a", "702.83b");
    ruling!(
        "Merchant of Truth",
        "If you control multiple Merchants of Truth, Clues you control will have multiple instances of exalted."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Merchant of Truth");
    t.battlefield(P0, "Merchant of Truth");
    create_token(&mut t, P0, "Clue");
    let a = t.battlefield(P0, "Grizzly Bears");
    declare(&mut t, P0, &[(a, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.pt(a), (4, 4));
}

fn clues(t: &TestGame, p: PlayerId) -> usize {
    with_subtype(t, p, "Clue").len()
}

#[test]
fn leaves_the_battlefield_triggers_look_back_with_their_source() {
    cr!("603.10a", "603.6c");
    ruling!(
        "Merchant of Truth",
        "If Merchant of Truth dies at the same time as one or more other nontoken creatures you control, its second ability will trigger for each of those creatures."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Merchant of Truth");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    create_token(&mut t, P0, "Soldier");
    cast_new(&mut t, P0, "Day of Judgment", &[]);
    t.resolve_all();
    // The Bears, the Giant and the Merchant itself; not the token.
    assert_eq!(clues(&t, P0), 3);

    supported("Sally Sparrow");
    ruling!(
        "Sally Sparrow",
        "If Sally Sparrow leaves the battlefield at the same time as the other creatures, its last ability will trigger."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sally Sparrow");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    cast_new(&mut t, P0, "Day of Judgment", &[]);
    t.resolve_all();
    assert_eq!(clues(&t, P0), 1);
}

#[test]
fn chalk_outline_triggers_once_for_cards_leaving_together() {
    cr!("603.2c");
    supported("Chalk Outline");
    ruling!(
        "Chalk Outline",
        "If multiple creature cards leave your graveyard at the same time, Chalk Outline's ability will trigger only once."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Chalk Outline");
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Hill Giant");
    t.graveyard(P0, "Memnite");
    // Bojuka Bog: "When this land enters, exile target player's graveyard."
    t.answer_targets(P1, &[Entity::Player(P0)]);
    t.enter(P1, "Bojuka Bog");
    t.resolve_all();
    assert_eq!(t.graveyard_size(P0), 0);
    assert_eq!(with_subtype(&t, P0, "Detective").len(), 1);
    assert_eq!(clues(&t, P0), 1);
}

#[test]
fn nick_valentine_stays_blocked() {
    cr!("509.1h", "506.4");
    supported("Nick Valentine, Private Eye");
    ruling!(
        "Nick Valentine, Private Eye",
        "Once Nick Valentine has been blocked, making the blocking creature stop being an artifact won't cause Nick Valentine to become unblocked."
    );
    use mtg_engine::ability::Modification;
    use mtg_engine::types::CardType;
    let mut t = TestGame::new(2);
    let nick = t.battlefield(P0, "Nick Valentine, Private Eye");
    let wall = t.battlefield(P1, "Memnite");
    declare(&mut t, P0, &[(nick, Entity::Player(P1))]);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![(wall, nick)]));
    t.advance_to(P0, Step::DeclareBlockers);
    crate::r_p117_common::modify(&mut t, wall, vec![Modification::RemoveTypes(vec![CardType::Artifact])]);
    assert!(!t.obj_now(wall).is(CardType::Artifact));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    // The Memnite (1/1, no longer an artifact) took the damage and died.
    assert!(!t.on_battlefield(wall));
}

#[test]
fn second_spell_each_turn_counts_spells_cast_before() {
    cr!("700.15");
    ruling!(
        "Lavinia, Foil to Conspiracy",
        "Spells that were cast before Lavinia, Foil to Conspiracy count. If Lavinia was the first spell you cast this turn, the next spell you cast this turn is your second spell."
    );
    ruling!(
        "Malcolm, the Eyes",
        "Spells that were cast before Malcolm entered the battlefield count. If Malcolm was the first spell you cast this turn, the next spell you cast this turn is your second spell."
    );
    for name in ["Lavinia, Foil to Conspiracy", "Malcolm, the Eyes"] {
        supported(name);
        // It's the first spell: the next one is the second.
        let mut t = TestGame::new(2);
        cast_new(&mut t, P0, name, &[]);
        t.resolve_all();
        cast_new(&mut t, P0, "Ornithopter", &[]);
        t.resolve_all();
        assert_eq!(clues(&t, P0), 1, "{name}");
        // A spell cast before it, then it (the second spell, no trigger): a third doesn't
        // trigger.
        let mut t = TestGame::new(2);
        cast_new(&mut t, P0, "Ornithopter", &[]);
        t.resolve_all();
        cast_new(&mut t, P0, name, &[]);
        t.resolve_all();
        cast_new(&mut t, P0, "Memnite", &[]);
        t.resolve_all();
        assert_eq!(clues(&t, P0), 0, "{name}");
    }
}

#[test]
fn maestros_diabolist_checks_for_a_devil_on_resolution() {
    cr!("603.4");
    supported("Maestros Diabolist");
    ruling!(
        "Maestros Diabolist",
        "Maestros Diabolist's triggered ability has an \"intervening 'if'\" clause. This means that it checks to see if you control a Devil token both when the ability triggers and when it resolves. Notably, this means that attacking with two Maestros Diabolists will normally cause you to create only one Devil token."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Maestros Diabolist");
    let b = t.battlefield(P0, "Maestros Diabolist");
    declare(&mut t, P0, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Devil").len(), 1);
}

#[test]
fn a_may_pay_trigger_cost_is_paid_at_most_once() {
    cr!("603.5", "118.8");
    ruling!(
        "Carefree Swinemaster",
        "While resolving Carefree Swinemaster's triggered ability, you can't pay the cost multiple times to create more than one token."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 6);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    let c = t.battlefield(P0, "Carefree Swinemaster");
    let from = n_asked(&t);
    declare(&mut t, P0, &[(c, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Boar").len(), 1);
    let asks = asked_of(&t, P0, from)
        .iter()
        .filter(|d| matches!(d, Decision::YesNo { .. }))
        .count();
    assert_eq!(asks, 1);

    supported("Descendant of Storms");
    ruling!(
        "Descendant of Storms",
        "You can’t pay the cost for the triggered ability more than once each time it triggers."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 6);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    let d = t.battlefield(P0, "Descendant of Storms");
    declare(&mut t, P0, &[(d, Entity::Player(P1))]);
    t.resolve_all();
    let spirits = with_subtype(&t, P0, "Spirit").len() as u32;
    assert_eq!(spirits + t.counters(d, counters::PLUS1), 1);
}

#[test]
fn beginning_of_combat_triggers_each_combat_before_attackers() {
    cr!("507.1", "505.1a");
    supported("Battle-Rattle Shaman");
    ruling!(
        "Battle-Rattle Shaman",
        "Battle-Rattle Shaman's ability triggers when the beginning of combat step starts during each of your turns. It resolves before you declare attackers."
    );
    let mut t = TestGame::new(2);
    let shaman = t.battlefield(P0, "Battle-Rattle Shaman");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(shaman)]);
    t.set_step(P0, Step::PrecombatMain);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.pt(shaman), (4, 2));
    // Attacking with it: it attacks as a 4/2.
    declare(&mut t, P0, &[(shaman, Entity::Player(P1))]);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 16);

    ruling!(
        "Battle-Rattle Shaman",
        "If a spell or ability causes your turn to have multiple combat phases, Battle Rattle Shaman's ability triggers during each of them."
    );
    let mut t = TestGame::new(2);
    let shaman = t.battlefield(P0, "Battle-Rattle Shaman");
    for _ in 0..2 {
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[obj(shaman)]);
    }
    t.set_step(P0, Step::PrecombatMain);
    cast_new(&mut t, P0, "Relentless Assault", &[]);
    t.resolve_all();
    t.advance_to(P0, Step::End);
    // +2/+0 twice.
    assert_eq!(t.pt(shaman), (6, 2));
}

#[test]
fn soaring_lightbringer_triggers_for_each_player_attacked() {
    cr!("508.1", "508.3a");
    supported("Soaring Lightbringer");
    ruling!(
        "Soaring Lightbringer",
        "Soaring Lightbringer's last ability will trigger once for each player you attack."
    );
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Soaring Lightbringer");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let c = t.battlefield(P0, "Memnite");
    declare(
        &mut t,
        P0,
        &[
            (a, Entity::Player(P1)),
            (b, Entity::Player(P2)),
            (c, Entity::Player(P2)),
        ],
    );
    t.resolve_all();
    let mut glimmers = attacking_tokens(&t, P0);
    glimmers.sort_by_key(|e| format!("{e:?}"));
    assert_eq!(glimmers, vec![Entity::Player(P1), Entity::Player(P2)]);
}

#[test]
fn attacking_a_player_doesnt_include_their_planeswalkers() {
    cr!("508.1b", "506.2");
    supported("Meriadoc Brandybuck");
    ruling!(
        "Meriadoc Brandybuck",
        "Halflings attacking a planeswalker or battle won't cause Meriadoc Brandybuck's ability to trigger."
    );
    let mut t = TestGame::new(2);
    let walker = t.battlefield(P1, "Narset, Parter of Veils");
    let m = t.battlefield(P0, "Meriadoc Brandybuck");
    declare(&mut t, P0, &[(m, obj(walker))]);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Food").len(), 0);
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Meriadoc Brandybuck");
    declare(&mut t, P0, &[(m, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Food").len(), 1);

    supported("Horn of the Mark");
    ruling!(
        "Horn of the Mark",
        "Two or more creatures you control must attack the same player in order for Horn of the Mark's ability to trigger. Creatures you control that attack planeswalkers that player controls or battles that player is protecting won't count toward Horn of the Mark's trigger condition."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Horn of the Mark");
    let walker = t.battlefield(P1, "Narset, Parter of Veils");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let lib = set_library(&mut t, P0, &["Island"; 6]);
    declare(&mut t, P0, &[(a, Entity::Player(P1)), (b, obj(walker))]);
    t.resolve_all();
    assert_eq!(library_top_first(&t, P0), lib);
    // Both attacking the player: it triggers (the top five go to the bottom).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Horn of the Mark");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let lib = set_library(&mut t, P0, &["Island"; 6]);
    declare(&mut t, P0, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(library_top_first(&t, P0)[0], lib[5]);
}

#[test]
fn horn_of_the_mark_triggers_for_each_player_attacked_by_two() {
    cr!("508.1", "802.2a");
    ruling!(
        "Horn of the Mark",
        "In a multiplayer game, Horn of the Mark's triggered ability triggers once for each player you attack with at least two creatures."
    );
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Horn of the Mark");
    let ids: Vec<ObjectId> = (0..5).map(|_| t.battlefield(P0, "Grizzly Bears")).collect();
    let lib = set_library(&mut t, P0, &["Island"; 12]);
    declare(
        &mut t,
        P0,
        &[
            (ids[0], Entity::Player(P1)),
            (ids[1], Entity::Player(P1)),
            (ids[2], Entity::Player(P2)),
            (ids[3], Entity::Player(P2)),
            (ids[4], Entity::Player(P1)),
        ],
    );
    t.resolve_all();
    // Two looks at five cards each went to the bottom: the eleventh card is on top.
    assert_eq!(library_top_first(&t, P0)[0], lib[10]);
}
