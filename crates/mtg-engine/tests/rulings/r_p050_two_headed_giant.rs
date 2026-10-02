//! Rulings batch P050 — "each opponent loses N life and you gain N life" in a
//! Two-Headed Giant game: damage, life loss and life gain happen to each player
//! individually and the results are applied to the team's shared life total (CR 810.9).
//! So each opponent losing N costs the opposing team 2N, while you gain N only once.

use crate::r_p050_common::*;
use crate::r_p057_common::into_upkeep;
use crate::r_s01_common::{attack_with, give_mana_for, supported};
use crate::r_s02_common::destroy;
use crate::r_s04_common::add_mana;
use crate::r_s05_common::enter;
use crate::r_s28_common::energy;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Runs `go` in a fresh 2HG game and checks P0's team gained `gain` and the opposing team
/// lost `loss`.
fn drain_2hg(name: &str, gain: i32, loss: i32, go: impl FnOnce(&mut TestGame)) {
    supported(name);
    let mut t = two_headed_giant();
    let before = team_lives(&t);
    assert_eq!(before, (30, 30));
    go(&mut t);
    assert_team_change(&t, before, gain, loss, name);
}

fn etb(t: &mut TestGame, name: &str) {
    enter(t, P0, name);
    t.resolve_all();
}

fn dies(t: &mut TestGame, name: &str) {
    let id = t.battlefield(P0, name);
    destroy(t, id);
    t.resolve_all();
}

fn cast(t: &mut TestGame, name: &str) {
    give_mana_for(t, P0, name);
    let c = t.hand(P0, name);
    t.cast(P0, c).go();
    t.resolve_all();
}

#[test]
fn enters_triggers() {
    cr!("810.9");
    ruling!(
        "Ayara, First of Locthwain",
        "In a Two-Headed Giant game, Ayara's first ability causes the opposing team to lose 1 life twice, and you gain 1 life once."
    );
    ruling!(
        "Cauldron Familiar",
        "In a Two-Headed Giant game, Cauldron Familiar's first ability causes the opposing team to lose 1 life twice, and you gain 1 life once."
    );
    ruling!(
        "Grasping Thrull",
        "In a Two-Headed Giant game, Grasping Thrull’s second ability causes the opposing team to lose 4 life and you gain 2 life."
    );
    ruling!(
        "Malakir Blood-Priest",
        "In a Two-Headed Giant game, Malakir Blood-Priest's ability causes the opposing team to lose twice X life and you to gain X life."
    );
    ruling!(
        "Vampire Spawn",
        "In a Two-Headed Giant game, Vampire Spawn's ability causes the opposing team to lose 4 life and you to gain 2 life."
    );
    drain_2hg("Ayara, First of Locthwain", 1, 2, |t| {
        etb(t, "Ayara, First of Locthwain")
    });
    drain_2hg("Cauldron Familiar", 1, 2, |t| etb(t, "Cauldron Familiar"));
    // "it deals 2 damage to each opponent and you gain 2 life"
    drain_2hg("Grasping Thrull", 2, 4, |t| etb(t, "Grasping Thrull"));
    // X = 2: Malakir Blood-Priest (a Cleric) and a Rogue in your party.
    drain_2hg("Malakir Blood-Priest", 2, 4, |t| {
        t.battlefield(P0, "Night Market Lookout");
        etb(t, "Malakir Blood-Priest")
    });
    drain_2hg("Vampire Spawn", 2, 4, |t| etb(t, "Vampire Spawn"));
}

#[test]
fn dies_triggers() {
    cr!("810.9");
    ruling!(
        "Nocturnal Feeder",
        "In a Two-Headed Giant game, Nocturnal Feeder’s ability causes the opposing team to lose 4 life and you to gain 2 life."
    );
    ruling!(
        "Serrated Scorpion",
        "In a Two-Headed Giant game, Serrated Scorpion’s ability causes the opposing team to lose 4 life and you to gain 2 life."
    );
    ruling!(
        "Spirit of Malevolence",
        "In a Two-Headed Giant game, Spirit of Malevolence's ability causes the opposing team to lose 2 life and you to gain 1 life."
    );
    drain_2hg("Nocturnal Feeder", 2, 4, |t| dies(t, "Nocturnal Feeder"));
    drain_2hg("Serrated Scorpion", 2, 4, |t| dies(t, "Serrated Scorpion"));
    drain_2hg("Spirit of Malevolence", 1, 2, |t| {
        dies(t, "Spirit of Malevolence")
    });
}

#[test]
fn spells() {
    cr!("810.9");
    ruling!(
        "Creeping Chill",
        "In a Two-Headed Giant game, Creeping Chill causes the opposing team to lose 6 life and you gain 3 life."
    );
    ruling!(
        "Zof Consumption // Zof Bloodbog",
        "In a Two-Headed Giant game, Zof Consumption causes the opposing team to lose 8 life and you to gain 4 life."
    );
    drain_2hg("Creeping Chill", 3, 6, |t| cast(t, "Creeping Chill"));
    drain_2hg("Zof Consumption // Zof Bloodbog", 4, 8, |t| {
        t.lands(P0, "Swamp", 6);
        let c = t.hand(P0, "Zof Consumption // Zof Bloodbog");
        t.cast(P0, c).go();
        t.resolve_all();
    });
}

#[test]
fn arterial_flow_each_opponent_discards_and_loses() {
    cr!("810.9");
    ruling!(
        "Arterial Flow",
        "In a Two-Headed Giant game, Arterial Flow causes the opposing team to lose 4 life, and each player on that team discards two cards. You gain 2 life."
    );
    drain_2hg("Arterial Flow", 2, 4, |t| {
        t.battlefield(P0, "Vampire Neonate");
        for p in [P2, P3] {
            for _ in 0..3 {
                t.hand(p, "Grizzly Bears");
            }
        }
        cast(t, "Arterial Flow");
        assert_eq!(t.hand_size(P2), 1);
        assert_eq!(t.hand_size(P3), 1);
    });
}

#[test]
fn vicious_rumors_each_opponent_is_affected() {
    cr!("810.9");
    ruling!(
        "Vicious Rumors",
        "In a Two-Headed Giant game, Vicious Rumors causes the opposing team to lose 2 life, each of those players discards a card and puts the top card of their library into their graveyard, and you gain 1 life."
    );
    drain_2hg("Vicious Rumors", 1, 2, |t| {
        t.hand(P2, "Grizzly Bears");
        t.hand(P3, "Grizzly Bears");
        cast(t, "Vicious Rumors");
        for p in [P2, P3] {
            assert_eq!(t.hand_size(p), 0);
            assert_eq!(t.graveyard_size(p), 2, "discarded and milled");
        }
    });
}

#[test]
fn curry_favor() {
    cr!("810.9");
    ruling!(
        "Smitten Swordmaster // Curry Favor",
        "In a Two-Headed Giant game, Curry Favor causes the opposing team to lose X life twice, after you gain X life once."
    );
    // X = 2 Knights you control.
    drain_2hg("Smitten Swordmaster // Curry Favor", 2, 4, |t| {
        t.battlefield(P0, "Smitten Swordmaster // Curry Favor");
        t.battlefield(P0, "Smitten Swordmaster // Curry Favor");
        t.lands(P0, "Swamp", 1);
        let c = t.hand(P0, "Smitten Swordmaster // Curry Favor");
        t.cast(P0, c).method(CastMethod::Half(1)).go();
        t.resolve_all();
    });
}

#[test]
fn activated_abilities() {
    cr!("810.9");
    ruling!(
        "Bontu the Glorified",
        "In a Two-Headed Giant game, Bontu's activated ability causes the opposing team to lose 2 life and you gain 1 life."
    );
    ruling!(
        "Vampire Neonate",
        "In a Two-Headed Giant game, Vampire Neonate's ability causes the opposing team to lose 2 life and you to gain 1 life."
    );
    ruling!(
        "Vampire Opportunist",
        "In a Two-Headed Giant game, Vampire Opportunist’s ability causes the opposing team to lose 4 life and you gain 2 life."
    );
    ruling!(
        "Lampad of Death's Vigil",
        "In a Two-Headed Giant game, each opposing team loses 2 life and you gain 1 life."
    );
    drain_2hg("Bontu the Glorified", 1, 2, |t| {
        t.lands(P0, "Swamp", 2);
        let b = t.battlefield(P0, "Bontu the Glorified");
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.answer_choose(P0, &[Entity::Object(bears)]);
        t.activate(P0, b, 0, &[]).unwrap();
        t.resolve_all();
        assert!(t.in_graveyard(P0, "Grizzly Bears"));
    });
    drain_2hg("Vampire Neonate", 1, 2, |t| {
        t.lands(P0, "Swamp", 2);
        let v = t.battlefield(P0, "Vampire Neonate");
        t.activate(P0, v, 0, &[]).unwrap();
        t.resolve_all();
    });
    drain_2hg("Vampire Opportunist", 2, 4, |t| {
        t.lands(P0, "Swamp", 7);
        let v = t.battlefield(P0, "Vampire Opportunist");
        t.activate(P0, v, 0, &[]).unwrap();
        t.resolve_all();
    });
    drain_2hg("Lampad of Death's Vigil", 1, 2, |t| {
        t.lands(P0, "Swamp", 1);
        let l = t.battlefield(P0, "Lampad of Death's Vigil");
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.answer_choose(P0, &[Entity::Object(bears)]);
        t.activate(P0, l, 0, &[]).unwrap();
        t.resolve_all();
    });
}

#[test]
fn gontis_machinations_drain() {
    cr!("810.9");
    ruling!(
        "Gonti's Machinations",
        "In a Two-Headed Giant game, Gonti's Machinations's second ability causes the opposing team to lose a total of 6 life."
    );
    // "Each opponent loses 3 life. You gain life equal to the life lost this way."
    drain_2hg("Gonti's Machinations", 6, 6, |t| {
        let g = t.battlefield(P0, "Gonti's Machinations");
        t.g.players[0].counters.insert("energy".into(), 2);
        t.activate(P0, g, 0, &[]).unwrap();
        t.resolve_all();
        assert_eq!(energy(t, P0), 0);
    });
}

#[test]
fn gontis_machinations_doesnt_trigger_on_teammate_life_loss() {
    cr!("810.9");
    ruling!(
        "Gonti's Machinations",
        "If your teammate is dealt damage or otherwise loses life, Gonti's Machinations doesn't trigger, even though your life total went down."
    );
    supported("Gonti's Machinations");
    let mut t = two_headed_giant();
    t.battlefield(P0, "Gonti's Machinations");
    t.g.lose_life(P1, 2);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.life(P0), 28, "the shared total went down");
    assert_eq!(energy(&t, P0), 0, "no trigger");
    t.g.lose_life(P0, 1);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(energy(&t, P0), 1, "your own life loss triggers it");
}

#[test]
fn upkeep_and_main_phase_triggers() {
    cr!("810.9");
    ruling!(
        "Ill-Gotten Inheritance",
        "In a Two-Headed Giant game, Ill-Gotten Inheritance’s first ability causes the opposing team to lose 2 life and you gain 1 life."
    );
    ruling!(
        "Twilight Prophet",
        "In a Two-Headed Giant game, Twilight Prophet's last ability causes the opposing team to lose twice X life and you gain X life."
    );
    ruling!(
        "Sanctum of Stone Fangs",
        "In a Two-Headed Giant game, Sanctum of Stone Fangs causes the opposing team to lose twice X life and you to gain X life."
    );
    drain_2hg("Ill-Gotten Inheritance", 1, 2, |t| {
        t.battlefield(P0, "Ill-Gotten Inheritance");
        into_upkeep(t, P0);
        t.resolve_all();
    });
    // X = 2 (Grizzly Bears' mana value).
    drain_2hg("Twilight Prophet", 2, 4, |t| {
        t.battlefield(P0, "Twilight Prophet");
        t.g.players[0].has_citys_blessing = true;
        t.library_top(P0, "Grizzly Bears");
        into_upkeep(t, P0);
        t.resolve_all();
        assert!(t.in_hand(P0, "Grizzly Bears"));
    });
    // X = 1 Shrine.
    drain_2hg("Sanctum of Stone Fangs", 1, 2, |t| {
        t.battlefield(P0, "Sanctum of Stone Fangs");
        t.set_step(P0, Step::Draw);
        t.advance_to(P0, Step::PrecombatMain);
        t.resolve_all();
    });
}

#[test]
fn chainers_torment_chapter_abilities() {
    cr!("810.9");
    ruling!(
        "Chainer's Torment",
        "In a Two-Headed Giant game, the first chapter abilities of Chainer's Torment each cause the opposing team to lose 4 life and you to gain 2 life."
    );
    drain_2hg("Chainer's Torment", 2, 4, |t| etb(t, "Chainer's Torment"));
}

#[test]
fn other_triggers() {
    cr!("810.9");
    ruling!(
        "Bontu's Monument",
        "In a Two-Headed Giant game, the triggered ability of Bontu's Monument causes the opposing team to lose 2 life and you gain 1 life."
    );
    ruling!(
        "Wayward Servant",
        "In a Two-Headed Giant game, the triggered ability of Wayward Servant causes the opposing team to lose 2 life and you gain 1 life."
    );
    ruling!(
        "Night Market Lookout",
        "In a Two-Headed Giant game, Night Market Lookout's ability causes the opposing team to lose a total of 2 life."
    );
    ruling!(
        "Whispering Snitch",
        "In a Two-Headed Giant game, Whispering Snitch's ability causes the opposing team to lose 2 life and you gain 1 life."
    );
    ruling!(
        "Underhanded Designs",
        "In a Two-Headed Giant game, Underhanded Designs's first ability causes the opposing team to lose a total of 2 life. You still gain only 1 life."
    );
    ruling!(
        "Faith of the Devoted",
        "In a Two-Headed Giant game, the triggered ability of Faith of the Devoted causes the opposing team to lose 4 life and you gain 2 life."
    );
    drain_2hg("Bontu's Monument", 1, 2, |t| {
        t.battlefield(P0, "Bontu's Monument");
        cast(t, "Memnite");
    });
    drain_2hg("Wayward Servant", 1, 2, |t| {
        t.battlefield(P0, "Wayward Servant");
        etb(t, "Walking Corpse");
    });
    drain_2hg("Night Market Lookout", 1, 2, |t| {
        let n = t.battlefield(P0, "Night Market Lookout");
        t.g.tap(n);
        t.g.flush_events();
        t.resolve_all();
    });
    drain_2hg("Whispering Snitch", 1, 2, |t| {
        t.battlefield(P0, "Whispering Snitch");
        cast(t, "Consider");
    });
    drain_2hg("Underhanded Designs", 1, 2, |t| {
        t.battlefield(P0, "Underhanded Designs");
        t.lands(P0, "Wastes", 1);
        t.answer_yes(P0, true);
        etb(t, "Ornithopter");
    });
    drain_2hg("Faith of the Devoted", 2, 4, |t| {
        t.battlefield(P0, "Faith of the Devoted");
        t.lands(P0, "Wastes", 1);
        t.answer_yes(P0, true);
        let c = t.hand(P0, "Grizzly Bears");
        t.g.discard(P0, c, None);
        t.g.flush_events();
        t.resolve_all();
    });
}

#[test]
fn insatiable_hemophage_mutate_trigger() {
    cr!("810.9");
    ruling!(
        "Insatiable Hemophage",
        "In a Two-Headed Giant game, Insatiable Hemophage’s last ability causes the opposing team to lose life equal to twice the value of X and you to gain X life."
    );
    // Mutated once: X = 1.
    drain_2hg("Insatiable Hemophage", 1, 2, |t| {
        let bears = t.battlefield(P0, "Grizzly Bears");
        add_mana(t, P0, ManaType::B, 1);
        add_mana(t, P0, ManaType::C, 2);
        let h = t.hand(P0, "Insatiable Hemophage");
        t.cast(P0, h)
            .method(CastMethod::Keyword(KeywordKind::Mutate))
            .target(bears)
            .go();
        t.resolve_all();
    });
}

#[test]
fn attack_triggers() {
    cr!("810.9");
    ruling!(
        "Knights' Charge",
        "In a Two-Headed Giant game, Knights' Charge's first ability causes the opposing team to lose 1 life twice, and you gain 1 life once."
    );
    ruling!(
        "Sanctum Seeker",
        "In a Two-Headed Giant game, Sanctum Seeker's ability causes the opposing team to lose 2 life and you to gain 1 life."
    );
    ruling!(
        "Resolute Survivors",
        "In a Two-Headed Giant game, Resolute Survivors’s last ability causes it to deal a total of 2 damage to the opposing team and you gain 1 life."
    );
    drain_2hg("Knights' Charge", 1, 2, |t| {
        t.battlefield(P0, "Knights' Charge");
        let k = t.battlefield(P0, "Smitten Swordmaster // Curry Favor");
        attack_with(t, &[(k, Entity::Player(P2))]);
        t.resolve_all();
    });
    drain_2hg("Sanctum Seeker", 1, 2, |t| {
        let s = t.battlefield(P0, "Sanctum Seeker");
        attack_with(t, &[(s, Entity::Player(P2))]);
        t.resolve_all();
    });
    drain_2hg("Resolute Survivors", 1, 2, |t| {
        let s = t.battlefield(P0, "Resolute Survivors");
        t.answer_yes(P0, true);
        attack_with(t, &[(s, Entity::Player(P2))]);
        t.resolve_all();
    });
}

#[test]
fn campaign_of_vengeance_one_defending_player() {
    cr!("810.9", "805.10e");
    ruling!(
        "Campaign of Vengeance",
        "In a Two-Headed Giant game, Campaign of Vengeance’s triggered ability has one defending player of your choice lose 1 life. The team doesn’t lose 2 life."
    );
    drain_2hg("Campaign of Vengeance", 1, 1, |t| {
        t.battlefield(P0, "Campaign of Vengeance");
        let b = t.battlefield(P0, "Grizzly Bears");
        attack_with(t, &[(b, Entity::Player(P2))]);
        t.resolve_all();
    });
}

#[test]
fn campaign_of_vengeance_multiplayer() {
    cr!("802.2a");
    ruling!(
        "Campaign of Vengeance",
        "In a multiplayer game, each instance of the ability affects only one defending player. For example, if you attack player A with two creatures and player B with two creatures, each of those players will lose 2 life, not 4."
    );
    supported("Campaign of Vengeance");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Campaign of Vengeance");
    let a: Vec<ObjectId> = (0..4).map(|_| t.battlefield(P0, "Memnite")).collect();
    attack_with(
        &mut t,
        &[
            (a[0], Entity::Player(P1)),
            (a[1], Entity::Player(P1)),
            (a[2], Entity::Player(P2)),
            (a[3], Entity::Player(P2)),
        ],
    );
    t.resolve_all();
    assert_eq!((t.life(P1), t.life(P2)), (18, 18));
    assert_eq!(t.life(P0), 24);
}

#[test]
fn sorin_markov_sets_the_targeted_players_life_total() {
    cr!("810.9c");
    ruling!(
        "Sorin Markov",
        "In a Two-Headed Giant game, Sorin’s second ability causes the targeted opponent’s team’s life total to become 10. Only the targeted player is actually considered to have actually gained or lost life."
    );
    supported("Sorin Markov");
    let mut t = two_headed_giant();
    let s = t.battlefield(P0, "Sorin Markov");
    t.activate(P0, s, 1, &[Entity::Player(P2)]).unwrap();
    t.resolve_all();
    assert_eq!((t.life(P2), t.life(P3)), (10, 10));
    let lost = |p: PlayerId| t.g.history.life_lost.get(&p).copied().unwrap_or(0);
    assert_eq!(lost(P2), 20, "the targeted player lost the life");
    assert_eq!(lost(P3), 0, "the teammate didn't");
    // Gaining: the opposing team at 4 goes up to 10.
    let mut t = two_headed_giant();
    t.g.lose_life(P3, 26);
    t.g.flush_events();
    let s = t.battlefield(P0, "Sorin Markov");
    t.activate(P0, s, 1, &[Entity::Player(P2)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P2), 10);
    let gained = |p: PlayerId| t.g.history.life_gained.get(&p).copied().unwrap_or(0);
    assert_eq!((gained(P2), gained(P3)), (6, 0));
}
