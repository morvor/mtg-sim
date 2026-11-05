//! Rulings batch P027 — a token put onto the battlefield attacking: its controller
//! chooses which player, planeswalker or battle it attacks as it enters, which needn't be
//! what the creature that made it is attacking, nor what other such tokens attack (CR
//! 508.4, 508.4a). Played in a three-player game: P0 attacks P1; P1 controls a
//! planeswalker; P0 controls a battle P2 protects.

use crate::r_p027_common::*;
use crate::r_s01_common::{attack_with, supported, tokens};
use crate::r_s06_common::activate_containing;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// The things P0 can attack other than P1: P2, P1's Chandra Nalaar, and the Invasion of
/// Azgol battle P2 protects.
struct Arena {
    t: TestGame,
    pw: ObjectId,
    battle: ObjectId,
}

fn arena() -> Arena {
    let mut t = TestGame::new(3);
    let pw = t.battlefield(P1, "Chandra Nalaar");
    let battle = t.battlefield(P0, "Invasion of Azgol");
    t.g.objects[battle.0 as usize].choices.player = Some(P2);
    t.set_step(P0, Step::PrecombatMain);
    Arena { t, pw, battle }
}

/// What the creature (followed across zone changes) is attacking.
fn attacking(t: &TestGame, id: ObjectId) -> Option<Entity> {
    t.g.combat
        .as_ref()
        .and_then(|c| c.attack_target(t.g.current(id)))
}

/// The "choose what the token is attacking" decisions asked since `from`: each one's
/// options.
fn attack_choices(t: &TestGame, from: usize) -> Vec<Vec<Entity>> {
    t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            mtg_engine::decision::Decision::ChooseEntities {
                candidates, prompt, ..
            } if prompt.contains("attacking") => Some(candidates.clone()),
            _ => None,
        })
        .collect()
}

/// Runs one attack in a fresh arena: `setup` puts the attacking creatures onto the
/// battlefield, queues the answers the attack triggers need (other than what the tokens
/// attack), and returns the attackers (all attacking P1). Each new token attacks the next
/// of `choose` (indices into [P2, planeswalker, battle, P1]). Returns the arena, the
/// attackers and the new tokens.
fn run(
    setup: impl FnOnce(&mut TestGame) -> Vec<ObjectId>,
    choose: &[usize],
) -> (Arena, Vec<ObjectId>) {
    run_choosing(setup, choose, |_| None)
}

/// [`run`], where `each` (given the game) names an object P0 chooses before each token
/// is created (Calamity's "choose a nonlegendary creature that saddled it").
fn run_choosing(
    setup: impl FnOnce(&mut TestGame) -> Vec<ObjectId>,
    choose: &[usize],
    each: impl Fn(&TestGame) -> Option<ObjectId>,
) -> (Arena, Vec<ObjectId>) {
    let mut a = arena();
    let options = [
        Entity::Player(P2),
        obj(a.pw),
        obj(a.battle),
        Entity::Player(P1),
    ];
    let attackers = setup(&mut a.t);
    for i in choose {
        if let Some(e) = each(&a.t) {
            a.t.answer_choose(P0, &[obj(e)]);
        }
        a.t.answer_choose(P0, &[options[*i]]);
    }
    let before = tokens(&a.t, P0);
    let from = a.t.asked().len();
    let decl: Vec<(ObjectId, Entity)> = attackers
        .iter()
        .map(|id| (*id, Entity::Player(P1)))
        .collect();
    attack_with(&mut a.t, &decl);
    a.t.resolve_all();
    let toks: Vec<ObjectId> = tokens(&a.t, P0)
        .into_iter()
        .filter(|x| !before.contains(x))
        .collect();
    assert_eq!(toks.len(), choose.len(), "tokens created");
    // Each choice offered every opponent, planeswalker and battle P0 could attack.
    let offered = attack_choices(&a.t, from);
    assert_eq!(offered.len(), choose.len());
    for o in &offered {
        for e in options {
            assert!(o.contains(&e), "{e:?} wasn't offered: {o:?}");
        }
    }
    for (tok, i) in toks.iter().zip(choose) {
        assert_eq!(attacking(&a.t, *tok), Some(options[*i]));
        assert!(a.t.obj_now(*tok).tapped);
    }
    for at in &attackers {
        assert_eq!(attacking(&a.t, *at), Some(Entity::Player(P1)));
    }
    (a, toks)
}

#[test]
fn flamerush_rider_token_attacks_another_opponent_or_planeswalker() {
    cr!("508.4", "508.4a");
    ruling!("Flamerush Rider", "You choose which opponent or opposing planeswalker the token is attacking as you put it onto the battlefield. It doesn't have to be the same player or planeswalker Flamerush Rider is attacking.");
    supported("Flamerush Rider");
    for choice in [0, 1] {
        run(
            |t| {
                let rider = t.battlefield(P0, "Flamerush Rider");
                let bears = t.battlefield(P0, "Grizzly Bears");
                t.answer_targets(P0, &[obj(bears)]);
                vec![rider, bears]
            },
            &[choice],
        );
    }
}

#[test]
fn gyrus_token_attacks_another_player_or_planeswalker() {
    cr!("508.4", "508.4a");
    ruling!("Gyrus, Waker of Corpses", "You choose which player or planeswalker the token is attacking. It doesn't have to be attacking the same player or planeswalker that Gyrus is attacking.");
    // "Whenever Gyrus attacks, you may exile target creature card with lesser power from
    // your graveyard. If you do, create a token that's a copy of that card and that's
    // tapped and attacking."
    supported("Gyrus, Waker of Corpses");
    for choice in [0, 1] {
        run(
            |t| {
                let gyrus = t.battlefield(P0, "Gyrus, Waker of Corpses");
                plus1(t, gyrus, 5);
                let giant = t.graveyard(P0, "Hill Giant");
                t.answer_targets(P0, &[obj(giant)]);
                t.answer_yes(P0, true);
                vec![gyrus]
            },
            &[choice],
        );
    }
}

#[test]
fn delina_token_attacks_another_player_or_planeswalker() {
    cr!("508.4", "508.4a", "706.2");
    ruling!("Delina, Wild Mage", "You declare which player or planeswalker the token is attacking as you put it onto the battlefield. It doesn't have to be the same player or planeswalker Delina, Wild Mage is attacking.");
    supported("Delina, Wild Mage");
    for choice in [0, 1] {
        run(
            |t| {
                let delina = t.battlefield(P0, "Delina, Wild Mage");
                let bears = t.battlefield(P0, "Grizzly Bears");
                t.answer_targets(P0, &[obj(bears)]);
                t.g.dice.loaded.extend([5]);
                vec![delina]
            },
            &[choice],
        );
    }
}

#[test]
fn phantom_steed_token_attacks_another_player_or_planeswalker() {
    cr!("508.4", "508.4a");
    ruling!("Phantom Steed", "You declare which player or planeswalker the token is attacking as you put it onto the battlefield. It doesn't have to be the same player or planeswalker Phantom Steed is attacking.");
    // "When this creature enters, exile another target creature you control until this
    // creature leaves the battlefield. Whenever this creature attacks, create a tapped and
    // attacking token that's a copy of the exiled card, except it's an Illusion ..."
    supported("Phantom Steed");
    for choice in [0, 1] {
        run(
            |t| {
                let bears = t.battlefield(P0, "Grizzly Bears");
                t.answer_targets(P0, &[obj(bears)]);
                let steed = t.enter(P0, "Phantom Steed");
                t.resolve_all();
                assert!(t.in_exile("Grizzly Bears"));
                crate::r_p023_common::unsick(t, steed);
                vec![steed]
            },
            &[choice],
        );
    }
}

#[test]
fn shaun_token_attacks_another_player_planeswalker_or_battle() {
    cr!("508.4", "508.4a", "310.5");
    ruling!("Shaun, Father of Synths", "You declare which player, planeswalker, or battle the token is attacking as you put it onto the battlefield. It doesn’t have to be attacking the same player, planeswalker, or battle that Shaun is attacking.");
    // "Whenever you attack, you may create a tapped and attacking token that's a copy of
    // target attacking legendary creature you control other than Shaun, except ..."
    supported("Shaun, Father of Synths");
    for choice in [0, 1, 2] {
        run(
            |t| {
                let shaun = t.battlefield(P0, "Shaun, Father of Synths");
                let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
                t.answer_yes(P0, true);
                t.answer_targets(P0, &[obj(isamaru)]);
                vec![shaun, isamaru]
            },
            &[choice],
        );
    }
}

#[test]
fn calamity_tokens_attack_whatever_each_chooses() {
    cr!("508.4", "508.4a", "310.5", "702.171a");
    ruling!("Calamity, Galloping Inferno", "You choose which player, planeswalker, or battle each of the tokens is attacking as it enters the battlefield. They don't have to attack the same player, planeswalker, or battle that Calamity is attacking, nor do they have to attack the same player, planeswalker, or battle as each other.");
    supported("Calamity, Galloping Inferno");
    for choice in [[0, 1], [2, 0], [3, 2]] {
        run_choosing(
            |t| {
                let calamity = t.battlefield(P0, "Calamity, Galloping Inferno");
                let bears = t.battlefield(P0, "Grizzly Bears");
                t.answer_choose(P0, &[obj(bears)]);
                activate_containing(t, P0, calamity, "Saddle").expect("saddle");
                t.resolve_all();
                vec![calamity]
            },
            &choice,
            // The creature to copy, each time ("Repeat this process once").
            |t| t.named_on_battlefield("Grizzly Bears").first().copied(),
        );
    }
}

#[test]
fn altair_tokens_attack_whatever_each_chooses() {
    cr!("508.4", "508.4a", "310.5");
    ruling!("Altaïr Ibn-La'Ahad", "You choose which player, planeswalker, or battle the tokens are attacking. Each token can enter attacking a different player, planeswalker, or battle, and they don't need to be the same one that Altaïr is attacking.");
    // "Whenever Altaïr attacks, exile up to one target Assassin creature card from your
    // graveyard with a memory counter on it. Then for each creature card you own in exile
    // with a memory counter on it, create a tapped and attacking token that's a copy of
    // it."
    supported("Altaïr Ibn-La'Ahad");
    for choice in [[0, 1], [2, 3]] {
        run(
            |t| {
                let altair = t.battlefield(P0, "Altaïr Ibn-La'Ahad");
                for name in ["Grizzly Bears", "Hill Giant"] {
                    let e = t.exile(P0, name);
                    t.g.add_counters(obj(e), "memory", 1, None);
                }
                t.answer_targets(P0, &[]);
                vec![altair]
            },
            &choice,
        );
    }
}
