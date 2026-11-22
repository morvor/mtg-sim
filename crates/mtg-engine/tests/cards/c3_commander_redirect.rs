//! "Your commander" terms in mana abilities and costs (CR 903.3d, 903.4, 903.8;
//! cards3-costs-mana-replacements, `kw/your_commander_terms.rs`), and "the next time it
//! would deal combat damage this turn, it deals that damage to you instead".

use mtg_engine::decision::{Answer, Decision};
use mtg_engine::game::Variant;
use mtg_engine::mana::ManaType;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

fn commander_game() -> TestGame {
    TestGame::with_config(
        2,
        GameConfig {
            variant: Variant::Commander,
            ..Default::default()
        },
    )
}

/// Makes `id` one of `p`'s commanders (CR 903.3).
fn make_commander(t: &mut TestGame, p: PlayerId, id: ObjectId) {
    t.g.objects[id.0 as usize].is_commander = true;
    let name = t.g.obj(id).chars.name.clone();
    t.g.players[p.idx()].commander_names.push(name);
    t.g.recompute();
}

fn scries(t: &TestGame) -> usize {
    t.asked()
        .iter()
        .filter(|(_, d)| matches!(d, Decision::Scry { .. }))
        .count()
}

fn color(t: &mut TestGame, p: PlayerId, ty: ManaType) {
    let i = [
        ManaType::W,
        ManaType::U,
        ManaType::B,
        ManaType::R,
        ManaType::G,
    ]
    .iter()
    .position(|x| *x == ty)
    .unwrap();
    t.answer(p, DecisionKind::Option, Answer::Index(i));
}

#[test]
fn war_room_costs_life_for_each_color_of_your_commanders() {
    cr!("903.4", "903.4f", "119.4");
    ruling!(
        "War Room",
        "you pay no life to activate War Room's last ability"
    );
    ruling!(
        "War Room",
        "If you don't have a commander, you can't activate War Room's last ability at all."
    );
    assert_supported("War Room");
    // Two colors (Kalain is black and red): 2 life.
    let mut t = commander_game();
    t.set_step(P0, Step::PrecombatMain);
    let k = t.command(P0, "Kalain, Reclusive Painter");
    make_commander(&mut t, P0, k);
    let room = t.battlefield(P0, "War Room");
    t.lands(P0, "Wastes", 3);
    let hand = t.hand_size(P0);
    t.activate(P0, room, 1, &[]).unwrap();
    t.resolve();
    assert_eq!(t.life(P0), 38);
    assert_eq!(t.hand_size(P0), hand + 1);
    // A colorless commander: no life.
    let mut t = commander_game();
    t.set_step(P0, Step::PrecombatMain);
    let k = t.command(P0, "Memnite");
    make_commander(&mut t, P0, k);
    let room = t.battlefield(P0, "War Room");
    t.lands(P0, "Wastes", 3);
    t.activate(P0, room, 1, &[]).unwrap();
    t.resolve();
    assert_eq!(t.life(P0), 40);
    // No commander: unpayable.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let room = t.battlefield(P0, "War Room");
    t.lands(P0, "Wastes", 3);
    assert!(t.activate(P0, room, 1, &[]).is_err());
    assert_eq!(t.life(P0), 20);
}

#[test]
fn path_of_ancestry_scries_for_a_creature_sharing_a_type_with_your_commander() {
    cr!("106.6", "903.3d");
    ruling!(
        "Path of Ancestry",
        "If your commander has no creature types, it can't share a creature type with any spell that you cast."
    );
    assert_supported("Path of Ancestry");
    for (spell, scry) in [("Elvish Visionary", 1), ("Grizzly Bears", 0)] {
        let mut t = commander_game();
        t.set_step(P0, Step::PrecombatMain);
        // An Elf commander (Kalain is a Human Elf Bard).
        let k = t.command(P0, "Kalain, Reclusive Painter");
        make_commander(&mut t, P0, k);
        let path = t.battlefield(P0, "Path of Ancestry");
        color(&mut t, P0, ManaType::B);
        t.activate(P0, path, 0, &[]).unwrap();
        // A {1}{G} creature: the Path's {B} and a Forest pay it.
        t.lands(P0, "Forest", 1);
        let c = t.hand(P0, spell);
        t.cast(P0, c).go();
        t.resolve_all();
        assert_eq!(scries(&t), scry, "{spell}");
    }
}

#[test]
fn study_hall_scries_for_each_time_your_commander_was_cast() {
    cr!("106.6", "903.8");
    assert_supported("Study Hall");
    let mut t = commander_game();
    t.set_step(P0, Step::PrecombatMain);
    let cmd = t.command(P0, "Grizzly Bears");
    make_commander(&mut t, P0, cmd);
    let hall = t.battlefield(P0, "Study Hall");
    t.lands(P0, "Forest", 2);
    color(&mut t, P0, ManaType::G);
    t.activate(P0, hall, 1, &[]).unwrap();
    t.cast(P0, cmd).go();
    t.resolve_all();
    // Cast once from the command zone: scry 1.
    let n = t.asked().iter().find_map(|(_, d)| match d {
        Decision::Scry { cards, .. } => Some(cards.len()),
        _ => None,
    });
    assert_eq!(n, Some(1));
    // Mana spent on another spell: nothing.
    let mut t = commander_game();
    t.set_step(P0, Step::PrecombatMain);
    let cmd = t.command(P0, "Grizzly Bears");
    make_commander(&mut t, P0, cmd);
    let hall = t.battlefield(P0, "Study Hall");
    t.lands(P0, "Forest", 2);
    color(&mut t, P0, ManaType::G);
    t.activate(P0, hall, 1, &[]).unwrap();
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.resolve_all();
    assert_eq!(scries(&t), 0);
}

#[test]
fn opal_palace_gives_your_commander_counters_for_its_casts() {
    cr!("106.6", "903.8", "614.1c");
    ruling!("Opal Palace", "includes the most recent time");
    assert_supported("Opal Palace");
    let mut t = commander_game();
    t.set_step(P0, Step::PrecombatMain);
    let cmd = t.command(P0, "Grizzly Bears");
    make_commander(&mut t, P0, cmd);
    let palace = t.battlefield(P0, "Opal Palace");
    t.lands(P0, "Forest", 2);
    color(&mut t, P0, ManaType::G);
    t.activate(P0, palace, 1, &[]).unwrap();
    t.cast(P0, cmd).go();
    t.resolve_all();
    let bears = t.g.current(cmd);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    // A spell that isn't your commander gets none.
    let mut t = commander_game();
    t.set_step(P0, Step::PrecombatMain);
    let cmd = t.command(P0, "Grizzly Bears");
    make_commander(&mut t, P0, cmd);
    let palace = t.battlefield(P0, "Opal Palace");
    t.lands(P0, "Forest", 2);
    color(&mut t, P0, ManaType::G);
    t.activate(P0, palace, 1, &[]).unwrap();
    let other = t.hand(P0, "Grizzly Bears");
    t.cast(P0, other).method(CastMethod::Normal).go();
    t.resolve_all();
    assert_eq!(t.counters(t.g.current(other), "+1/+1"), 0);
}

#[test]
fn goblin_psychopath_deals_its_next_combat_damage_to_you_if_you_lose_the_flip() {
    cr!("614.9", "705.2");
    ruling!(
        "Goblin Psychopath",
        "resolves before combat damage is assigned"
    );
    assert_supported("Goblin Psychopath");
    for win in [true, false] {
        let mut t = TestGame::new(2);
        let gp = t.battlefield(P0, "Goblin Psychopath");
        t.g.dice.loaded_coins.push_back(win);
        t.set_step(P0, Step::PrecombatMain);
        t.attack(&[(gp, Entity::Player(P1))], &[]);
        if win {
            assert_eq!((t.life(P0), t.life(P1)), (20, 15));
        } else {
            assert_eq!((t.life(P0), t.life(P1)), (15, 20));
        }
    }
}

#[test]
fn opal_palace_doubled_mana_gives_two_counters_for_each_cast() {
    cr!("106.6a", "903.8");
    ruling!(
        "Opal Palace",
        "that commander enters with two counters for each time it's been cast from the command zone this game"
    );
    let mut t = commander_game();
    t.set_step(P0, Step::PrecombatMain);
    let cmd = t.command(P0, "Kalonian Tusker");
    make_commander(&mut t, P0, cmd);
    t.battlefield(P0, "Mana Reflection");
    let palace = t.battlefield(P0, "Opal Palace");
    // {1}: a Wastes (its doubled {C}{C} pays it, one {C} left over).
    t.lands(P0, "Wastes", 1);
    t.activate(P0, palace, 1, &[]).unwrap();
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::G), 2);
    // {G}{G}: the Palace's two green mana.
    t.cast(P0, cmd).go();
    t.resolve_all();
    let tusker = t.g.current(cmd);
    assert!(t.on_battlefield(tusker));
    assert_eq!(t.counters(tusker, "+1/+1"), 2);
}

#[test]
fn path_of_ancestry_doubled_mana_scries_once_for_each_mana() {
    cr!("106.6a", "903.3d");
    ruling!(
        "Path of Ancestry",
        "Each of those abilities will cause you to scry 1. You won't scry 2."
    );
    let mut t = commander_game();
    t.set_step(P0, Step::PrecombatMain);
    for _ in 0..3 {
        t.library_top(P0, "Forest");
    }
    let k = t.command(P0, "Llanowar Elves");
    make_commander(&mut t, P0, k);
    t.battlefield(P0, "Mana Reflection");
    let path = t.battlefield(P0, "Path of Ancestry");
    t.activate(P0, path, 0, &[]).unwrap();
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::G), 2);
    // Elvish Visionary ({1}{G}, an Elf) with both.
    let c = t.hand(P0, "Elvish Visionary");
    t.cast(P0, c).go();
    t.resolve_all();
    let sizes: Vec<usize> = t
        .asked()
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::Scry { cards, .. } => Some(cards.len()),
            _ => None,
        })
        .collect();
    assert_eq!(sizes, vec![1, 1]);
}
