//! CR 702.141 Encore.

use crate::common_k702_011_017::assert_supported;
use crate::common_k702_052_066::run_effect;
use crate::common_k702_140_152::*;
use mtg_engine::ability::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Activates the encore ability of the card in P0's graveyard, paying for it.
fn encore(t: &mut TestGame, card: ObjectId, mana: &[(ManaType, u32)]) -> ObjectId {
    for (ty, n) in mana {
        add_mana(t, P0, *ty, *n);
    }
    let uid = ability_uid(t, card, "Encore");
    activate_uid(t, P0, card, uid)
        .expect("encore")
        .expect("on the stack")
}

/// The attack declared for each token (after declaring with the engine's default).
fn attacked(t: &TestGame, token: ObjectId) -> Option<Entity> {
    t.g.combat
        .as_ref()?
        .attackers
        .iter()
        .find(|a| a.id == token)
        .and_then(|a| a.target)
}

#[test]
fn encore_creates_a_hasty_token_copy_attacking_each_opponent() {
    cr!("702.141", "702.141a");
    assert_supported("Impulsive Pilferer");
    ruling!("Impulsivity", "Each token must attack the appropriate player if able.");
    let mut t = TestGame::new(3);
    // Impulsive Pilferer: {R} 1/1, "When this creature dies, create a Treasure token."
    // Encore {3}{R}.
    let card = t.graveyard(P0, "Impulsive Pilferer");
    let ab = encore(&mut t, card, &[(ManaType::R, 1), (ManaType::C, 3)]);
    // Exiling the card is part of the cost.
    assert!(t.g.is_live(ab));
    assert!(t.in_exile("Impulsive Pilferer"));
    t.resolve_all();
    let tokens = creature_tokens(&t, P0);
    assert_eq!(tokens.len(), 2);
    for tok in &tokens {
        let o = t.obj(*tok);
        assert_eq!(o.chars.name, "Impulsive Pilferer");
        assert!(o.chars.has_keyword(KeywordKind::Haste));
        assert!(o.summoning_sick);
    }
    // Each token must attack the opponent it was created for; a declaration swapping them
    // disobeys both requirements.
    t.set_step(P0, Step::BeginningOfCombat);
    let options = mtg_engine::combat::attack_options(&t.g);
    let (a, b) = (tokens[0], tokens[1]);
    assert!(mtg_engine::combat::attack_declaration_legal(
        &t.g,
        &options,
        &[(a, Entity::Player(P1)), (b, Entity::Player(P2))]
    ));
    assert!(!mtg_engine::combat::attack_declaration_legal(
        &t.g,
        &options,
        &[(a, Entity::Player(P2)), (b, Entity::Player(P1))]
    ));
    assert!(!mtg_engine::combat::attack_declaration_legal(
        &t.g, &options, &[]
    ));
    // No attack declared: the engine declares the legal one.
    declare_attackers(&mut t, &[]);
    assert_eq!(attacked(&t, a), Some(Entity::Player(P1)));
    assert_eq!(attacked(&t, b), Some(Entity::Player(P2)));
    to_step(&mut t, Step::EndOfCombat);
    assert_eq!((t.life(P1), t.life(P2)), (19, 19));
    // At the beginning of the next end step, they're sacrificed (and their own dies
    // triggers make Treasures).
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(creature_tokens(&t, P0).is_empty());
    assert_eq!(t.named_on_battlefield("Treasure Token").len(), 2);
    // The card stays in exile.
    assert_eq!(t.zone(card), Zone::Exile);
}

#[test]
fn encore_is_activated_only_from_the_graveyard_as_a_sorcery() {
    cr!("702.141a");
    let mut t = TestGame::new(2);
    let card = t.hand(P0, "Impulsive Pilferer");
    add_mana(&mut t, P0, ManaType::R, 4);
    let uid = ability_uid(&mut t, card, "Encore");
    assert!(!activatable(&mut t, P0, card, uid));
    let card = t.graveyard(P0, "Impulsive Pilferer");
    add_mana(&mut t, P0, ManaType::R, 4);
    let uid = ability_uid(&mut t, card, "Encore");
    assert!(activatable(&mut t, P0, card, uid));
    // Not at instant speed.
    t.set_step(P0, Step::BeginningOfCombat);
    add_mana(&mut t, P0, ManaType::R, 4);
    assert!(!activatable(&mut t, P0, card, uid));
    t.set_step(P1, Step::PrecombatMain);
    add_mana(&mut t, P0, ManaType::R, 4);
    assert!(!activatable(&mut t, P0, card, uid));
}

#[test]
fn the_tokens_copy_only_the_card() {
    cr!("702.141a");
    ruling!(
        "Impulsivity",
        "The tokens copy only what's on the original card. Effects that modified that creature when it was previously on the battlefield won't be copied."
    );
    let mut t = TestGame::new(2);
    // Fin-Clade Fugitives: 7/4. Encore {4}{G}.
    let card = t.graveyard(P0, "Fin-Clade Fugitives");
    encore(&mut t, card, &[(ManaType::G, 5)]);
    t.resolve_all();
    let tokens = creature_tokens(&t, P0);
    assert_eq!(tokens.len(), 1);
    assert_eq!(t.pt(tokens[0]), (7, 4));
    assert_eq!(t.obj(tokens[0]).chars.name, "Fin-Clade Fugitives");
}

#[test]
fn opponents_who_left_the_game_get_no_token() {
    cr!("702.141a");
    ruling!(
        "Impulsivity",
        "Opponents who have left the game aren't counted when determining how many tokens to create."
    );
    let mut t = TestGame::new(3);
    t.g.players[P2.idx()].has_lost = true;
    let card = t.graveyard(P0, "Impulsive Pilferer");
    encore(&mut t, card, &[(ManaType::R, 4)]);
    t.resolve_all();
    assert_eq!(creature_tokens(&t, P0).len(), 1);
}

#[test]
fn a_token_that_cant_attack_doesnt_have_to() {
    cr!("702.141a", "508.1d");
    ruling!(
        "Impulsivity",
        "If one of the tokens can't attack for any reason (such as being tapped), then it doesn't attack."
    );
    let mut t = TestGame::new(3);
    let card = t.graveyard(P0, "Impulsive Pilferer");
    encore(&mut t, card, &[(ManaType::R, 4)]);
    t.resolve_all();
    let tokens = creature_tokens(&t, P0);
    t.g.objects[tokens[0].0 as usize].tapped = true;
    t.set_step(P0, Step::BeginningOfCombat);
    declare_attackers(&mut t, &[]);
    assert_eq!(attacked(&t, tokens[0]), None);
    assert_eq!(attacked(&t, tokens[1]), Some(Entity::Player(P2)));
}

#[test]
fn a_token_that_cant_attack_its_opponent_may_attack_anyone_or_not_at_all() {
    cr!("702.141a", "508.1d");
    ruling!(
        "Impulsivity",
        "If an effect stops a token from attacking a specific player, that token can attack any player, planeswalker, or battle, or not attack at all."
    );
    let mut t = TestGame::new(2);
    let card = t.graveyard(P0, "Impulsive Pilferer");
    encore(&mut t, card, &[(ManaType::R, 4)]);
    t.resolve_all();
    let token = creature_tokens(&t, P0)[0];
    // "Creatures can't attack P1."
    run_effect(
        &mut t,
        None,
        P1,
        Effect::AddRestriction {
            restriction: Restriction::CantAttackPlayer {
                attackers: Filter::Any,
                defender: PlayerFilter::You,
                planeswalkers: false,
                battles: false,
            },
            duration: Duration::EndOfTurn,
        },
        &[],
    );
    t.set_step(P0, Step::BeginningOfCombat);
    let options = mtg_engine::combat::attack_options(&t.g);
    assert!(mtg_engine::combat::attack_declaration_legal(
        &t.g, &options, &[]
    ));
    declare_attackers(&mut t, &[]);
    assert_eq!(attacked(&t, token), None);
}

#[test]
fn a_token_another_player_controls_isnt_sacrificed() {
    cr!("702.141a");
    ruling!(
        "Impulsivity",
        "If one of the tokens somehow is under another player's control as the delayed triggered ability resolves, you can't sacrifice that token."
    );
    let mut t = TestGame::new(2);
    let card = t.graveyard(P0, "Fin-Clade Fugitives");
    encore(&mut t, card, &[(ManaType::G, 5)]);
    t.resolve_all();
    let token = creature_tokens(&t, P0)[0];
    run_effect(
        &mut t,
        None,
        P1,
        Effect::GainControl {
            what: Sel::Target(0),
            who: PlayerRef::You,
            duration: Duration::Permanent,
        },
        &[Entity::Object(token)],
    );
    t.answer(P0, DecisionKind::Attackers, Answer::Attackers(vec![]));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.g.is_live(token));
    assert_eq!(t.obj(token).controller, P1);
}

#[test]
fn cards_in_a_graveyard_can_be_given_encore_with_a_cost_of_their_own() {
    cr!("702.141a");
    assert_supported("Wire Surgeons");
    assert_supported("Graywater's Fixer");
    // Wire Surgeons: "Each artifact creature card in your graveyard has encore. Its encore
    // cost is equal to its mana cost."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wire Surgeons");
    let sable = t.graveyard(P0, "Bronze Sable");
    let bears = t.graveyard(P0, "Grizzly Bears");
    assert!(has_kw(&t, sable, KeywordKind::Encore));
    assert!(!has_kw(&t, bears, KeywordKind::Encore));
    // Bronze Sable's mana cost is {2}.
    let uid = ability_uid(&mut t, sable, "Encore");
    add_mana(&mut t, P0, ManaType::C, 1);
    assert!(!activatable(&mut t, P0, sable, uid));
    add_mana(&mut t, P0, ManaType::C, 1);
    activate_uid(&mut t, P0, sable, uid).unwrap();
    assert_eq!(pool(&t, P0), 0);
    t.resolve_all();
    let tokens = creature_tokens(&t, P0);
    assert_eq!(tokens.len(), 1);
    assert_eq!(t.obj(tokens[0]).chars.name, "Bronze Sable");
    // Graywater's Fixer: "Each outlaw creature card in your graveyard has encore {X},
    // where X is its mana value."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Graywater's Fixer");
    let pirate = t.graveyard(P0, "Fathom Fleet Swordjack");
    let bears = t.graveyard(P0, "Grizzly Bears");
    assert!(!has_kw(&t, bears, KeywordKind::Encore));
    // Its own encore {5}{R} and the granted encore {X} = {4} (its mana value).
    let costs: Vec<String> = t
        .obj(pirate)
        .chars
        .keywords()
        .filter(|k| k.kind == KeywordKind::Encore)
        .map(|k| format!("{:?}", k.cost.as_ref().and_then(|c| c.mana.clone())))
        .collect();
    assert_eq!(costs.len(), 2, "{costs:?}");
    assert!(costs.iter().any(|c| c.contains("{4}")), "{costs:?}");
}
