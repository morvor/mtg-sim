//! "Untap up to N lands" (CR 701.26b): the lands are chosen as the effect happens, aren't
//! targeted, and may be any player's. "Add one mana of any color in your commander's color
//! identity" (CR 903.4, 903.4f, 702.124c).

use mtg_engine::decision::Decision;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let c = card(n);
        assert!(
            c.unsupported_text().is_empty(),
            "{n} has unsupported text: {:?}",
            c.unsupported_text()
        );
    }
}

fn pool(t: &TestGame, p: PlayerId) -> Vec<ManaType> {
    let mut v: Vec<ManaType> = t.g.player(p).mana_pool.mana.iter().map(|m| m.ty).collect();
    v.sort();
    v
}

#[test]
fn untap_and_commander_mana_compile() {
    assert_supported(&[
        "Peregrine Drake",
        "Palinchron",
        "Frantic Search",
        "Rewind",
        "Unwind",
        "Snap",
        "Time Spiral",
        "Command Tower",
        "Arcane Signet",
        "Commander's Sphere",
    ]);
}

#[test]
fn untap_up_to_five_lands_as_the_trigger_resolves() {
    cr!("701.26b", "603.3");
    ruling!(
        "Peregrine Drake",
        "They aren't targeted, and they don't have to be lands that you control."
    );
    let mut t = TestGame::new(2);
    let islands = t.lands(P0, "Island", 5);
    let theirs = t.battlefield(P1, "Forest");
    t.g.tap(theirs);
    let drake = t.hand(P0, "Peregrine Drake");
    t.cast(P0, drake).go();
    for i in &islands {
        assert!(t.obj_now(*i).tapped);
    }
    // Choose four of the Islands and the opponent's Forest.
    let mut chosen: Vec<Entity> = islands[..4].iter().map(|i| Entity::Object(*i)).collect();
    chosen.push(Entity::Object(theirs));
    t.answer_choose(P0, &chosen);
    t.resolve(); // the Drake
    t.resolve(); // its trigger
    assert!(t.on_battlefield(t.named_on_battlefield("Peregrine Drake")[0]));
    for i in &islands[..4] {
        assert!(!t.obj_now(*i).tapped);
    }
    assert!(t.obj_now(islands[4]).tapped);
    assert!(!t.obj_now(theirs).tapped);
}

#[test]
fn untap_up_to_three_lands_after_the_rest_of_the_spell() {
    cr!("701.26b");
    ruling!(
        "Frantic Search",
        "You choose which lands to untap as the spell resolves."
    );
    let mut t = TestGame::new(2);
    let islands = t.lands(P0, "Island", 5);
    let search = t.hand(P0, "Frantic Search");
    t.cast(P0, search).go();
    let tapped: Vec<Entity> = islands
        .iter()
        .filter(|i| t.obj_now(**i).tapped)
        .map(|i| Entity::Object(*i))
        .collect();
    assert_eq!(tapped.len(), 3);
    // The discard (the default choice), then the lands: the three tapped Islands.
    t.answer(P0, DecisionKind::Entities, Answer::Default);
    t.answer_choose(P0, &tapped);
    t.resolve();
    assert!(islands.iter().all(|i| !t.obj_now(*i).tapped));
    let asked = t
        .asked()
        .into_iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
        .count();
    assert_eq!(asked, 2);
}

/// Makes `name` a commander `p` owns, in the command zone.
fn commander(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = t.command(p, name);
    t.g.objects[id.0 as usize].is_commander = true;
    id
}

#[test]
fn commander_color_identity_mana() {
    cr!("903.4", "106.1a");
    let mut t = TestGame::new(2);
    commander(&mut t, P0, "Niv-Mizzet, Parun");
    let tower = t.battlefield(P0, "Command Tower");
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.activate(P0, tower, 0, &[]).unwrap();
    assert_eq!(pool(&t, P0), vec![ManaType::R]);
    let options: Vec<Vec<String>> = t
        .asked()
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseOption { options, .. } => Some(options),
            _ => None,
        })
        .collect();
    assert_eq!(options, vec![vec!["U".to_string(), "R".to_string()]]);
}

#[test]
fn no_commander_no_mana() {
    cr!("903.4f");
    ruling!(
        "Command Tower",
        "If you don't have a commander, Command Tower's ability produces no mana."
    );
    let mut t = TestGame::new(2);
    // The opponent's commander doesn't count.
    commander(&mut t, P1, "Niv-Mizzet, Parun");
    let tower = t.battlefield(P0, "Command Tower");
    t.activate(P0, tower, 0, &[]).unwrap();
    assert!(pool(&t, P0).is_empty());
    // Nor is it tapped for mana by automatic payment.
    let bolt = t.hand(P0, "Lightning Bolt");
    assert!(t.cast(P0, bolt).target(P1).try_go().is_err());
}

#[test]
fn colorless_identity_produces_no_mana() {
    cr!("903.4f");
    ruling!(
        "Arcane Signet",
        "If your commander is a card that has no colors in its color identity, Arcane Signet's ability produces no mana."
    );
    let mut t = TestGame::new(2);
    commander(&mut t, P0, "Karn, Legacy Reforged");
    let signet = t.battlefield(P0, "Arcane Signet");
    t.activate(P0, signet, 0, &[]).unwrap();
    assert!(pool(&t, P0).is_empty());
}

#[test]
fn two_commanders_combine_identities() {
    cr!("702.124c", "903.4");
    ruling!(
        "Command Tower",
        "If you have two commanders, the ability adds one mana of any color in their combined color identities."
    );
    let mut t = TestGame::new(2);
    commander(&mut t, P0, "Thrasios, Triton Hero");
    let tymna = commander(&mut t, P0, "Tymna the Weaver");
    // A commander on the battlefield is still the commander.
    t.g.move_object(
        tymna,
        Zone::Battlefield,
        mtg_engine::events::MoveCause::Effect,
        Some(P0),
    )
    .unwrap();
    let tower = t.battlefield(P0, "Command Tower");
    // White, blue, black, green: the fourth option is green.
    t.answer(P0, DecisionKind::Option, Answer::Index(3));
    t.activate(P0, tower, 0, &[]).unwrap();
    assert_eq!(pool(&t, P0), vec![ManaType::G]);
    let options: Vec<Vec<String>> = t
        .asked()
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseOption { options, .. } => Some(options),
            _ => None,
        })
        .collect();
    assert_eq!(
        options,
        vec![vec![
            "W".to_string(),
            "U".to_string(),
            "B".to_string(),
            "G".to_string()
        ]]
    );
}

#[test]
fn commander_identity_mana_pays_automatically() {
    cr!("903.4", "601.2g");
    let mut t = TestGame::new(2);
    commander(&mut t, P0, "Niv-Mizzet, Parun");
    let tower = t.battlefield(P0, "Command Tower");
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    assert!(t.obj_now(tower).tapped);
    t.resolve();
    assert_eq!(t.life(P1), 17);
}
