//! Token creation variants (CR 111, 303.4f, 508.4, 607.2d): Aura tokens created attached
//! to each of several creatures, tokens that must attack a given opponent, "otherwise,
//! create one of those tokens", tokens attacking a player or a planeswalker they control,
//! tokens of the chosen color and type, a token with power and toughness from a value, and
//! "for each of those tokens".

use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::Color;
use mtg_engine::*;

fn supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

#[test]
fn smoke_spirits_aid_attaches_a_smoke_blessing_to_each_target() {
    cr!("303.4f", "111.10");
    supported("Smoke Spirits' Aid");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 3);
    let s = t.hand(P0, "Smoke Spirits' Aid");
    t.cast(P0, s).x(2).targets(&[obj(a), obj(b)]).go();
    t.resolve_all();
    let blessings = t.named_on_battlefield("Smoke Blessing");
    assert_eq!(blessings.len(), 2);
    let hosts: Vec<Option<ObjectId>> = blessings
        .iter()
        .map(|x| t.obj_now(*x).attached_to.and_then(|e| e.object()))
        .collect();
    assert!(hosts.contains(&Some(a)) && hosts.contains(&Some(b)));
    // When the enchanted creature dies, 1 damage to its controller and a Treasure for me.
    t.lands(P0, "Swamp", 3);
    let kill = t.hand(P0, "Murder");
    t.cast(P0, kill).target(b).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.named_on_battlefield("Treasure Token").len(), 1);
}

#[test]
fn twisted_sewer_witch_gives_each_rat_a_wicked_role() {
    cr!("303.4f", "111.10j");
    supported("Twisted Sewer-Witch");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Typhoid Rats");
    t.enter(P0, "Twisted Sewer-Witch");
    t.resolve_all();
    let roles: Vec<ObjectId> =
        t.g.permanents_controlled_by(P0)
            .into_iter()
            .filter(|o| t.g.obj(*o).chars.has_subtype("Role"))
            .collect();
    // The existing Rat and the new Rat token.
    assert_eq!(roles.len(), 2);
}

#[test]
fn scriv_attaches_a_contract_that_punishes_attacking_others() {
    cr!("303.4f", "508.1");
    supported("Scriv, the Obligator");
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(theirs)]);
    t.enter(P0, "Scriv, the Obligator");
    t.resolve_all();
    let contract = t.named_on_battlefield("Contract");
    assert_eq!(contract.len(), 1);
    assert_eq!(t.obj_now(contract[0]).controller, P0);
    // The Bears attack the Contract's controller, who isn't one of their own opponents:
    // no bonus, and the Bears' controller loses 2 life.
    t.set_step(P1, Step::PrecombatMain);
    t.attack(&[(theirs, Entity::Player(P0))], &[]);
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.life(P1), 18);
    // With three players, attacking another opponent of the Contract's controller: +2/+0.
    let mut t = TestGame::new(3);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(theirs)]);
    t.enter(P0, "Scriv, the Obligator");
    t.resolve_all();
    t.set_step(P1, Step::PrecombatMain);
    t.attack(&[(theirs, Entity::Player(P2))], &[]);
    assert_eq!(t.life(P2), 16);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn furygale_flocking_tokens_must_attack_that_opponent() {
    cr!("508.1d", "111.1");
    ruling!(
        "Furygale Flocking",
        "Each of the tokens must attack the appropriate player if able"
    );
    supported("Furygale Flocking");
    let mut t = TestGame::new(3);
    t.lands(P0, "Mountain", 10);
    let s = t.hand(P0, "Furygale Flocking");
    t.cast(P0, s).go();
    t.resolve_all();
    let els = t.named_on_battlefield("Elemental Token");
    assert_eq!(els.len(), 4);
    assert!(els.iter().all(|e| t
        .obj_now(*e)
        .has_keyword(mtg_engine::keywords::KeywordKind::Haste)));
    // Declaring no attackers is corrected to each attacking its opponent.
    t.answer(P0, DecisionKind::Attackers, Answer::Attackers(vec![]));
    t.advance_to(P0, Step::DeclareBlockers);
    let combat = t.g.combat.as_ref().expect("combat");
    let mut by_player = [0, 0, 0];
    for e in &els {
        match combat.attack_target(*e) {
            Some(Entity::Player(p)) => by_player[p.idx()] += 1,
            other => panic!("not attacking a player: {other:?}"),
        }
    }
    assert_eq!(by_player, [0, 2, 2]);
}

#[test]
fn brood_birthing_makes_three_with_a_spawn_and_one_without() {
    cr!("111.1");
    ruling!(
        "Brood Birthing",
        "If you put a single token onto the battlefield this way, it's exactly like the ones specified earlier"
    );
    supported("Brood Birthing");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let s = t.hand(P0, "Brood Birthing");
    t.cast(P0, s).go();
    t.resolve_all();
    let spawn = t.named_on_battlefield("Eldrazi Spawn Token");
    assert_eq!(spawn.len(), 1);
    assert!(!t.obj_now(spawn[0]).chars.abilities.is_empty());
    t.lands(P0, "Mountain", 2);
    let s = t.hand(P0, "Brood Birthing");
    t.cast(P0, s).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Eldrazi Spawn Token").len(), 4);
}

#[test]
fn adeline_tokens_may_attack_a_planeswalker_of_that_opponent() {
    cr!("508.4", "111.1");
    ruling!(
        "Adeline, Resplendent Cathar",
        "You choose whether each token is attacking that opponent or a planeswalker they control"
    );
    supported("Adeline, Resplendent Cathar");
    let mut t = TestGame::new(2);
    let adeline = t.battlefield(P0, "Adeline, Resplendent Cathar");
    let pw = t.battlefield(P1, "Jace, Mirror Mage");
    t.answer(P0, DecisionKind::Any, Answer::Default);
    t.answer_choose(P0, &[obj(pw)]);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(adeline, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    let humans = t.named_on_battlefield("Human Token");
    assert_eq!(humans.len(), 1);
    let combat = t.g.combat.as_ref().expect("combat");
    let target = combat.attack_target(humans[0]);
    assert!(
        target == Some(Entity::Player(P1)) || target == Some(obj(pw)),
        "{target:?}"
    );
    assert!(t.obj_now(humans[0]).tapped);
}

#[test]
fn sarpadian_empires_creates_a_token_of_the_chosen_color_and_type() {
    cr!("607.2d", "111.1");
    supported("Sarpadian Empires, Vol. VII");
    let mut t = TestGame::new(2);
    // white Citizen, blue Camarid, black Thrull, red Goblin, green Saproling: Thrull.
    t.answer(P0, DecisionKind::Option, Answer::Index(2));
    let book = t.enter(P0, "Sarpadian Empires, Vol. VII");
    t.lands(P0, "Wastes", 3);
    t.activate(P0, book, 0, &[]).unwrap();
    t.resolve_all();
    let thrulls = t.named_on_battlefield("Thrull Token");
    assert_eq!(thrulls.len(), 1);
    let c = &t.obj_now(thrulls[0]).chars;
    assert!(c.colors.contains(Color::Black) && c.colors.count() == 1);
    assert!(c.has_subtype("Thrull"));
    assert_eq!(t.pt(thrulls[0]), (1, 1));
}

#[test]
fn riptide_replicator_creates_an_x_x_token_of_the_chosen_color_and_type() {
    cr!("607.2d", "111.1", "107.3c");
    supported("Riptide Replicator");
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Riptide Replicator");
    {
        let ch = &mut t.g.objects[r.0 as usize].choices;
        ch.color = Some(Color::Blue);
        ch.creature_type = Some("Merfolk".into());
    }
    t.g.add_counters(obj(r), "charge", 3, None);
    t.lands(P0, "Wastes", 4);
    t.activate(P0, r, 0, &[]).unwrap();
    t.resolve_all();
    let toks = t.named_on_battlefield("Merfolk Token");
    assert_eq!(toks.len(), 1);
    assert_eq!(t.pt(toks[0]), (3, 3));
    assert!(t.obj_now(toks[0]).chars.colors.contains(Color::Blue));
}

#[test]
fn the_skullspore_nexus_token_has_the_total_power_of_those_creatures() {
    cr!("111.3", "603.10a");
    ruling!(
        "The Skullspore Nexus",
        "Use the power of the nontoken creatures that died as they last existed on the battlefield"
    );
    supported("The Skullspore Nexus");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Skullspore Nexus");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Plains", 6);
    let w = t.hand(P0, "Wrath of God");
    t.cast(P0, w).go();
    t.resolve_all();
    let d = t.named_on_battlefield("Fungus Dinosaur Token");
    assert_eq!(d.len(), 1);
    assert_eq!(t.pt(d[0]), (5, 5));
}

#[test]
fn liberated_livestock_may_attach_an_aura_to_each_token() {
    cr!("303.4f", "111.1");
    ruling!(
        "Liberated Livestock",
        "If you attempt to put an Aura onto the battlefield this way and that Aura can't legally be attached to the token"
    );
    supported("Liberated Livestock");
    let mut t = TestGame::new(2);
    let ll = t.battlefield(P0, "Liberated Livestock");
    t.graveyard(P0, "Holy Strength");
    // The Cat: yes (Holy Strength); the Bird and the Ox: no.
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    t.answer_yes(P0, false);
    t.lands(P0, "Swamp", 3);
    let kill = t.hand(P0, "Murder");
    t.cast(P0, kill).target(ll).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Cat Token").len(), 1);
    assert_eq!(t.named_on_battlefield("Bird Token").len(), 1);
    assert_eq!(t.named_on_battlefield("Ox Token").len(), 1);
    let hs = t.named_on_battlefield("Holy Strength");
    assert_eq!(hs.len(), 1);
    let host = t
        .obj_now(hs[0])
        .attached_to
        .and_then(|e| e.object())
        .unwrap();
    assert!(t.obj_now(host).is_token());
}

#[test]
fn lucy_maclean_gives_another_player_a_copy_of_a_token_once_each_turn() {
    cr!("115.1", "111.1", "603.2");
    supported("Lucy MacLean, Positively Armed");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lucy MacLean, Positively Armed");
    t.lands(P0, "Plains", 2);
    let r = t.hand(P0, "Raise the Alarm");
    let hand = t.hand_size(P0);
    t.cast(P0, r).go();
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_yes(P0, true);
    t.resolve_all();
    let soldiers = t.named_on_battlefield("Soldier Token");
    let p1: Vec<_> = soldiers
        .iter()
        .filter(|o| t.obj_now(**o).controller == P1)
        .collect();
    // Only once each turn: one copy for P1, and P0 drew a card (P1 is an opponent).
    assert_eq!(soldiers.len(), 3);
    assert_eq!(p1.len(), 1);
    assert_eq!(t.hand_size(P0), hand - 1 + 1);
}
