//! Rulings batch P032 — myriad (CR 702.116) and other tokens created tapped and
//! attacking: the tokens of one myriad instance enter at the same time; each token's
//! controller chooses as it's created whether it attacks that opponent or a planeswalker
//! they control, never a battle (CR 702.116a, 508.4); tokens put onto the battlefield
//! attacking weren't declared as attackers, so attack restrictions don't apply (CR 508.4).

use crate::r_p032_common::*;
use crate::r_s01_common::{asked_since, attack_with, supported};
use crate::r_s26_common::new_tokens;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// A four-player game in P0's beginning of combat.
fn four_players() -> TestGame {
    let mut t = TestGame::new(4);
    t.set_step(P0, Step::BeginningOfCombat);
    t
}

/// `attacker` attacks P1 (with myriad, creating tokens for P2 and P3 when the questions
/// are answered yes); resolves the triggers. Returns the new tokens.
fn attack_p1(t: &mut TestGame, attacker: ObjectId) -> Vec<ObjectId> {
    let before = t.g.battlefield.clone();
    attack_with(t, &[(attacker, Entity::Player(P1))]);
    t.resolve_all();
    new_tokens(t, P0, &before)
}

#[test]
fn myriad_tokens_of_one_instance_enter_at_the_same_time() {
    cr!("702.116a", "603.2c");
    supported("Cloakwood Swarmkeeper");
    // Cloakwood Swarmkeeper: "Whenever one or more tokens you control enter, put a +1/+1
    // counter on this creature." It triggers once for the tokens of one myriad trigger.
    ruling!(
        "Goldlust Triad",
        "The tokens created by a single instance of myriad all enter at the same time."
    );
    supported("Goldlust Triad");
    let mut t = four_players();
    let keeper = t.battlefield(P0, "Cloakwood Swarmkeeper");
    let triad = t.battlefield(P0, "Goldlust Triad");
    let tok = attack_p1(&mut t, triad);
    assert_eq!(tok.len(), 2);
    assert_eq!(t.counters(keeper, counters::PLUS1), 1);
}

/// `p`'s Invasion of Segovia, protected by `protector`.
fn siege(t: &mut TestGame, p: PlayerId, protector: PlayerId) -> ObjectId {
    t.answer_choose(p, &[Entity::Player(protector)]);
    let id = t.enter(p, "Invasion of Segovia");
    assert_eq!(mtg_engine::battle::protector(&t.g, id), Some(protector));
    id
}

/// What the attacking creature is attacking.
fn attack_target(t: &TestGame, id: ObjectId) -> Option<Entity> {
    crate::r_s12_common::attack_target(t, id)
}

/// The candidates of each "what the token is attacking" choice since `from`.
fn attack_choices(t: &TestGame, from: usize) -> Vec<Vec<Entity>> {
    asked_since(t, from)
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities {
                prompt, candidates, ..
            } if prompt.contains("token is attacking") => Some(candidates),
            _ => None,
        })
        .collect()
}

/// Three players: P2 controls Jace Beleren, and P2 protects P1's battle. `attacker` (with
/// myriad) attacks P1; the token for P2 is sent at Jace. Asserts each token's choice was
/// offered P2 and Jace only (never the battle), and returns the tokens.
fn myriad_token_attacks_jace(
    t: &mut TestGame,
    attacker: ObjectId,
    n_tokens: usize,
) -> Vec<ObjectId> {
    let jace = t.battlefield(P2, "Jace Beleren");
    let battle = siege(t, P1, P2);
    let from = t.asked().len();
    for _ in 0..n_tokens {
        t.answer_choose(P0, &[obj(jace)]);
    }
    let tok = attack_p1(t, attacker);
    assert_eq!(tok.len(), n_tokens);
    let offered = attack_choices(t, from);
    assert_eq!(offered.len(), n_tokens, "one choice per token, as it's created");
    for o in offered {
        let mut o = o.clone();
        o.sort();
        let mut want = vec![Entity::Player(P2), obj(jace)];
        want.sort();
        assert_eq!(o, want);
        assert!(!o.contains(&obj(battle)));
    }
    for id in &tok {
        assert_eq!(attack_target(t, *id), Some(obj(jace)));
    }
    tok
}

fn three_players() -> TestGame {
    let mut t = TestGame::new(3);
    t.set_step(P0, Step::BeginningOfCombat);
    t
}

#[test]
fn myriad_tokens_attack_the_player_or_a_planeswalker_never_a_battle() {
    cr!("702.116a", "508.4", "310.5");
    ruling!(
        "Scion of Calamity",
        "You choose whether each token is attacking the player or a planeswalker they control as the token is created. If it's attacking a planeswalker, you choose which one. You can't have any of the tokens attack a battle."
    );
    ruling!(
        "Goldlust Triad",
        "You choose whether each token is attacking the player or a planeswalker they control as the token is created. If it’s attacking a planeswalker, you choose which one. You can’t have any of the tokens attack a battle."
    );
    ruling!(
        "Scurry of Squirrels",
        "You choose whether each token is attacking the player or a planeswalker they control as the token is created. If it's attacking a planeswalker, you choose which one. You can't have any of the tokens attack a battle."
    );
    for (name, n) in [
        ("Scion of Calamity", 1),
        ("Goldlust Triad", 1),
        // Two instances of myriad: two tokens, each chosen separately.
        ("Scurry of Squirrels", 2),
    ] {
        supported(name);
        let mut t = three_players();
        let c = t.battlefield(P0, name);
        myriad_token_attacks_jace(&mut t, c, n);
    }
}

#[test]
fn granted_myriad_tokens_attack_the_player_or_a_planeswalker_never_a_battle() {
    cr!("702.116a", "508.4", "310.5");
    ruling!(
        "Mass of Mysteries",
        "You choose whether each token is attacking the player or a planeswalker they control as the token is created. If it's attacking a planeswalker, you choose which one. You can't have any of the tokens attack a battle."
    );
    ruling!(
        "Muddle, the Ever-Changing",
        "You choose whether each token is attacking the player or a planeswalker they control as the token is created. If it's attacking a planeswalker, you choose which one. You can't have any of the tokens attack a battle."
    );
    ruling!(
        "Ironwill Forger",
        "You choose whether each token is attacking the player or a planeswalker they control as the token is created. If it’s attacking a planeswalker, you choose which one. You can’t have any of the tokens attack a battle."
    );
    // Mass of Mysteries: "At the beginning of combat on your turn, another target
    // Elemental you control gains myriad until end of turn."
    supported("Mass of Mysteries");
    let mut t = TestGame::new(3);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Mass of Mysteries");
    let elemental = t.battlefield(P0, "Air Elemental");
    t.answer_targets(P0, &[obj(elemental)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    let tok = myriad_token_attacks_jace(&mut t, elemental, 1);
    assert_eq!(t.obj_now(tok[0]).chars.name, "Air Elemental");

    // Muddle: "Whenever you cast an instant or sorcery spell, Muddle becomes a copy of up
    // to one target nonlegendary creature you control until end of turn, except it has
    // myriad."
    supported("Muddle, the Ever-Changing");
    let mut t = TestGame::new(3);
    t.set_step(P0, Step::PrecombatMain);
    let muddle = t.battlefield(P0, "Muddle, the Ever-Changing");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    let growth = t.hand(P0, "Giant Growth");
    t.answer_targets(P0, &[obj(bears)]);
    t.cast(P0, growth).target(obj(bears)).go();
    t.resolve_all();
    assert_eq!(t.obj_now(muddle).chars.name, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    let tok = myriad_token_attacks_jace(&mut t, muddle, 1);
    assert_eq!(t.obj_now(tok[0]).chars.name, "Grizzly Bears");

    // Ironwill Forger: "Lieutenant — At the beginning of combat on your turn, if you
    // control your commander, target nonlegendary creature you control gains myriad until
    // end of turn."
    supported("Ironwill Forger");
    let mut t = TestGame::with_config(
        3,
        mtg_engine::game::GameConfig {
            variant: mtg_engine::game::Variant::Commander,
            ..Default::default()
        },
    );
    t.set_step(P0, Step::PrecombatMain);
    let cmd = t.battlefield(P0, "Mass of Mysteries");
    t.g.objects[cmd.0 as usize].is_commander = true;
    t.g.players[0].commander_names.push("Mass of Mysteries".into());
    let forger = t.battlefield(P0, "Ironwill Forger");
    t.answer_targets(P0, &[obj(forger)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    let tok = myriad_token_attacks_jace(&mut t, forger, 1);
    assert_eq!(t.obj_now(tok[0]).chars.name, "Ironwill Forger");
}

#[test]
fn nacatl_war_pride_tokens_enter_attacking_despite_attack_restrictions() {
    cr!("508.4", "508.1c");
    ruling!(
        "Nacatl War-Pride",
        "The tokens enter attacking, even if the attack couldn't legally be declared."
    );
    supported("Nacatl War-Pride");
    supported("Silent Arbiter");
    // P1's Silent Arbiter: "No more than one creature can attack each combat." Nacatl
    // War-Pride attacks alone; P1 controls two creatures: two tokens enter attacking.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::BeginningOfCombat);
    t.battlefield(P1, "Silent Arbiter");
    t.battlefield(P1, "Grizzly Bears");
    let pride = t.battlefield(P0, "Nacatl War-Pride");
    let before = t.g.battlefield.clone();
    attack_with(&mut t, &[(pride, Entity::Player(P1))]);
    t.resolve_all();
    let tok = new_tokens(&t, P0, &before);
    assert_eq!(tok.len(), 2);
    for id in tok {
        assert_eq!(attack_target(&t, id), Some(Entity::Player(P1)));
        assert!(t.obj_now(id).tapped);
    }
    assert_eq!(t.g.combat.as_ref().unwrap().attackers.len(), 3);
}

#[test]
fn warren_warleader_rabbit_can_attack_something_else() {
    cr!("508.4", "506.3");
    ruling!(
        "Warren Warleader",
        "You choose the player, planeswalker, or battle the Rabbit token is attacking. It doesn’t have to be the same player, planeswalker, or battle that any other attacking creature is attacking."
    );
    supported("Warren Warleader");
    // Three players: Warren Warleader attacks P1; the Rabbit attacks P1's battle (which
    // P2 protects), Jace Beleren (P2's), or a player.
    for pick in 0..3 {
        let mut t = three_players();
        let jace = t.battlefield(P2, "Jace Beleren");
        let battle = siege(&mut t, P1, P2);
        let lord = t.battlefield(P0, "Warren Warleader");
        let target = [Entity::Player(P2), obj(jace), obj(battle)][pick];
        t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
        let from = t.asked().len();
        t.answer_choose(P0, &[target]);
        let before = t.g.battlefield.clone();
        attack_with(&mut t, &[(lord, Entity::Player(P1))]);
        t.resolve_all();
        let rabbits = new_tokens(&t, P0, &before);
        assert_eq!(rabbits.len(), 1);
        assert_eq!(attack_target(&t, rabbits[0]), Some(target));
        let offered = crate::r_s03_common::choice_candidates(&t, from, "attacking");
        assert!(offered[0].contains(&obj(battle)));
        assert!(offered[0].contains(&Entity::Player(P1)));
    }
}
