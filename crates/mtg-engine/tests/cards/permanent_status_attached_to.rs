//! Objects described by what they're attached to: "destroy all Equipment attached to that
//! creature", "target Aura attached to a creature", "target creature and all Equipment
//! attached to it" (patterns in `src/oracle/patterns/attach_control_grammar.rs`).

use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn assert_compiles(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

#[test]
fn attached_to_cards_compile() {
    assert_compiles(&[
        "Turn to Slag",
        "Eaten by Spiders",
        "Soul Nova",
        "Strip Bare",
        "End Hostilities",
        "Light of Judgment",
        "Fiery Annihilation",
        "Devout Harpist",
        "Miracle Worker",
        "Savaen Elves",
        "Piety Charm",
        "Treefolk Mystic",
        "Corrosive Ooze",
        "Street Sweeper",
        "Awaken the Sleeper",
        "Shackles of Treachery",
    ]);
}

#[test]
fn turn_to_slag_destroys_only_the_equipment_on_that_creature() {
    cr!("608.2c", "301.5a");
    ruling!(
        "Turn to Slag",
        "Turn to Slag can target a creature that has no Equipment attached to it."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 5);
    let ogre = t.battlefield(P1, "Colossal Dreadmaw");
    let blade = t.battlefield(P1, "Bonesplitter");
    assert!(t.g.attach(blade, Entity::Object(ogre)));
    let bears = t.battlefield(P1, "Grizzly Bears");
    let other = t.battlefield(P1, "Leonin Scimitar");
    assert!(t.g.attach(other, Entity::Object(bears)));
    let loose = t.battlefield(P1, "Short Sword");
    let slag = t.hand(P0, "Turn to Slag");
    t.cast(P0, slag).target(ogre).go();
    t.resolve_all();
    // The 6/6 Dreadmaw survives 5 damage; its Equipment does not.
    assert!(t.on_battlefield(ogre));
    assert!(!t.on_battlefield(blade));
    assert!(t.on_battlefield(other));
    assert!(t.on_battlefield(loose));
}

#[test]
fn turn_to_slag_with_an_illegal_target_destroys_nothing() {
    cr!("608.2b");
    ruling!(
        "Turn to Slag",
        "If the target creature is an illegal target by the time Turn to Slag tries to resolve, the spell won't resolve. You won't destroy any Equipment."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 5);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let blade = t.battlefield(P1, "Bonesplitter");
    assert!(t.g.attach(blade, Entity::Object(bears)));
    let slag = t.hand(P0, "Turn to Slag");
    t.cast(P0, slag).target(bears).go();
    // The creature leaves before the spell resolves: the Equipment stays (unattached).
    t.g.destroy(bears, None);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(t.on_battlefield(blade));
    assert_eq!(t.obj_now(blade).attached_to, None);
}

#[test]
fn target_creature_and_all_equipment_attached_to_it() {
    cr!("608.2c");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 3);
    let bird = t.battlefield(P1, "Wind Drake");
    let blade = t.battlefield(P1, "Bonesplitter");
    assert!(t.g.attach(blade, Entity::Object(bird)));
    let bears = t.battlefield(P1, "Grizzly Bears");
    let other = t.battlefield(P1, "Leonin Scimitar");
    assert!(t.g.attach(other, Entity::Object(bears)));
    let spiders = t.hand(P0, "Eaten by Spiders");
    t.cast(P0, spiders).target(bird).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Wind Drake"));
    assert!(t.in_graveyard(P1, "Bonesplitter"));
    assert!(t.on_battlefield(other));
}

#[test]
fn strip_bare_destroys_auras_and_equipment_on_target_creature() {
    cr!("303.4b", "301.5a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 1);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let blade = t.battlefield(P1, "Bonesplitter");
    assert!(t.g.attach(blade, Entity::Object(bears)));
    let strength = t.battlefield(P1, "Holy Strength");
    assert!(t.g.attach(strength, Entity::Object(bears)));
    let elf = t.battlefield(P1, "Llanowar Elves");
    let other = t.battlefield(P1, "Holy Strength");
    assert!(t.g.attach(other, Entity::Object(elf)));
    let strip = t.hand(P0, "Strip Bare");
    t.cast(P0, strip).target(bears).go();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert!(!t.on_battlefield(blade));
    assert!(!t.on_battlefield(strength));
    assert!(t.on_battlefield(other));
}

#[test]
fn end_hostilities_destroys_what_is_attached_to_creatures() {
    cr!("608.2c");
    ruling!(
        "End Hostilities",
        "Permanents attached to creatures may include Auras and Equipment. All the affected permanents are destroyed at the same time."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 5);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let blade = t.battlefield(P1, "Bonesplitter");
    assert!(t.g.attach(blade, Entity::Object(bears)));
    let loose = t.battlefield(P1, "Leonin Scimitar");
    let forest = t.battlefield(P1, "Forest");
    let growth = t.battlefield(P1, "Wild Growth");
    assert!(t.g.attach(growth, Entity::Object(forest)));
    let eh = t.hand(P0, "End Hostilities");
    t.cast(P0, eh).go();
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(t.in_graveyard(P1, "Bonesplitter"));
    assert!(t.on_battlefield(loose));
    assert!(t.on_battlefield(growth));
}

#[test]
fn light_of_judgment_chooses_the_equipment_as_it_resolves() {
    cr!("608.2c", "608.2d");
    ruling!(
        "Light of Judgment",
        "You don't choose the Equipment to destroy until Light of Judgment is resolving."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 5);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let a = t.battlefield(P1, "Bonesplitter");
    let b = t.battlefield(P1, "Leonin Scimitar");
    assert!(t.g.attach(a, Entity::Object(wurm)));
    assert!(t.g.attach(b, Entity::Object(wurm)));
    let elf = t.battlefield(P1, "Llanowar Elves");
    let c = t.battlefield(P1, "Short Sword");
    assert!(t.g.attach(c, Entity::Object(elf)));
    let light = t.hand(P0, "Light of Judgment");
    t.answer_choose(P0, &[Entity::Object(b)]);
    t.cast(P0, light).target(wurm).go();
    t.resolve_all();
    assert!(t.on_battlefield(a));
    assert!(!t.on_battlefield(b));
    assert!(t.on_battlefield(c));
    // Only the Equipment attached to that creature were offered.
    let offered = t
        .asked()
        .into_iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates),
            _ => None,
        })
        .expect("a choice");
    assert!(offered.contains(&Entity::Object(a)));
    assert!(!offered.contains(&Entity::Object(c)));
}

#[test]
fn fiery_annihilation_equipment_no_longer_attached_isnt_exiled() {
    cr!("608.2b", "115.1");
    ruling!(
        "Fiery Annihilation",
        "If the target Equipment is no longer attached to the target creature when Fiery Annihilation resolves, it won't be exiled."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let elf = t.battlefield(P1, "Llanowar Elves");
    let blade = t.battlefield(P1, "Bonesplitter");
    assert!(t.g.attach(blade, Entity::Object(bears)));
    let fa = t.hand(P0, "Fiery Annihilation");
    t.cast(P0, fa).target(bears).target(blade).go();
    // In response, the Equipment moves to another creature.
    assert!(t.g.attach(blade, Entity::Object(elf)));
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert!(t.on_battlefield(blade));

    // Still attached: exiled with the creature.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let blade = t.battlefield(P1, "Bonesplitter");
    assert!(t.g.attach(blade, Entity::Object(bears)));
    let fa = t.hand(P0, "Fiery Annihilation");
    t.cast(P0, fa).target(bears).target(blade).go();
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert!(t.in_exile("Bonesplitter"));
}

#[test]
fn devout_harpist_targets_only_an_aura_attached_to_a_creature() {
    cr!("115.1", "303.4b");
    let mut t = TestGame::new(2);
    let harpist = t.battlefield(P0, "Devout Harpist");
    let forest = t.battlefield(P1, "Forest");
    let growth = t.battlefield(P1, "Wild Growth");
    assert!(t.g.attach(growth, Entity::Object(forest)));
    // The only Aura is on a land: no legal target, so it can't be activated.
    assert!(t
        .activate(P0, harpist, 0, &[Entity::Object(growth)])
        .is_err());
    t.clear_answers();
    let bears = t.battlefield(P1, "Grizzly Bears");
    let strength = t.battlefield(P1, "Holy Strength");
    assert!(t.g.attach(strength, Entity::Object(bears)));
    t.activate(P0, harpist, 0, &[Entity::Object(strength)])
        .unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(strength));
    assert!(t.on_battlefield(growth));
}

#[test]
fn treefolk_mystic_destroys_the_auras_on_the_creature_it_blocks() {
    cr!("509.3b", "603.2");
    let mut t = TestGame::new(2);
    let mystic = t.battlefield(P0, "Treefolk Mystic");
    t.set_step(P1, Step::BeginningOfCombat);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let strength = t.battlefield(P1, "Holy Strength");
    assert!(t.g.attach(strength, Entity::Object(bears)));
    t.attack(&[(bears, Entity::Player(P0))], &[(mystic, bears)]);
    t.resolve_all();
    assert!(!t.on_battlefield(strength));
}

#[test]
fn corrosive_ooze_destroys_the_equipment_even_after_the_creature_died() {
    cr!("603.7c", "608.2h", "704.5n");
    ruling!(
        "Corrosive Ooze",
        "If the creature Corrosive Ooze blocks or is blocking leaves the battlefield, the Equipment that was attached to that creature immediately before it left the battlefield will be destroyed"
    );
    ruling!(
        "Corrosive Ooze",
        "The Equipment will be destroyed even if Corrosive Ooze leaves the battlefield before that time."
    );
    let mut t = TestGame::new(2);
    let ooze = t.battlefield(P0, "Corrosive Ooze");
    t.set_step(P1, Step::BeginningOfCombat);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let blade = t.battlefield(P1, "Bonesplitter");
    assert!(t.g.attach(blade, Entity::Object(bears)));
    let other = t.battlefield(P1, "Leonin Scimitar");
    // The equipped Bears (4/2) and the Ooze (2/2) destroy each other in combat.
    t.attack(&[(bears, Entity::Player(P0))], &[(ooze, bears)]);
    assert!(!t.on_battlefield(bears));
    assert!(!t.on_battlefield(ooze));
    assert!(t.on_battlefield(blade), "unattached, not yet destroyed");
    t.resolve_all();
    assert!(!t.on_battlefield(blade));
    assert!(t.on_battlefield(other));
}

#[test]
fn corrosive_ooze_spares_equipment_unattached_before_the_creature_left() {
    cr!("608.2h", "701.3d");
    ruling!(
        "Corrosive Ooze",
        "If the creature Corrosive Ooze blocks or is blocking leaves the battlefield, the Equipment that was attached to that creature immediately before it left the battlefield will be destroyed"
    );
    let mut t = TestGame::new(2);
    let ooze = t.battlefield(P0, "Corrosive Ooze");
    t.set_step(P1, Step::BeginningOfCombat);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let blade = t.battlefield(P1, "Bonesplitter");
    let scimitar = t.battlefield(P1, "Leonin Scimitar");
    assert!(t.g.attach(blade, Entity::Object(bears)));
    assert!(t.g.attach(scimitar, Entity::Object(bears)));
    t.answer(
        P1,
        DecisionKind::Attackers,
        mtg_engine::decision::Answer::Attackers(vec![(bears, Entity::Player(P0))]),
    );
    t.answer(
        P0,
        DecisionKind::Blockers,
        mtg_engine::decision::Answer::Blockers(vec![(ooze, bears)]),
    );
    t.advance_to(P1, Step::DeclareBlockers);
    t.resolve_all();
    // The Bonesplitter is unattached while the Bears are still on the battlefield; then
    // the Bears leave with only the Scimitar attached.
    t.g.unattach(blade);
    t.g.destroy(bears, None);
    t.g.flush_events();
    t.settle();
    assert!(!t.on_battlefield(bears));
    t.advance_to(P1, Step::EndOfCombat);
    t.resolve_all();
    assert!(t.on_battlefield(blade));
    assert!(!t.on_battlefield(scimitar));
}

#[test]
fn balan_has_double_strike_with_two_equipment_attached() {
    cr!("611.3a", "613.1f");
    assert_compiles(&[
        "Balan, Wandering Knight",
        "Face of Divinity",
        "Daybreak Coronet",
    ]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 2);
    let balan = t.battlefield(P0, "Balan, Wandering Knight");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let a = t.battlefield(P0, "Bonesplitter");
    let b = t.battlefield(P0, "Leonin Scimitar");
    assert!(t.g.attach(a, Entity::Object(balan)));
    assert!(t.g.attach(b, Entity::Object(bears)));
    t.g.recompute();
    assert!(!t.obj_now(balan).has_keyword(KeywordKind::DoubleStrike));
    // "{1}{W}: Attach all Equipment you control to Balan."
    t.activate(P0, balan, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(b).attached_to, Some(Entity::Object(balan)));
    assert!(t.obj_now(balan).has_keyword(KeywordKind::DoubleStrike));
    assert_eq!(t.pt(balan), (6, 4));
}

#[test]
fn face_of_divinity_needs_another_aura_on_the_creature() {
    cr!("611.3a");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let face = t.battlefield(P0, "Face of Divinity");
    assert!(t.g.attach(face, Entity::Object(bears)));
    t.g.recompute();
    assert_eq!(t.pt(bears), (4, 4));
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::FirstStrike));
    let strength = t.battlefield(P0, "Holy Strength");
    assert!(t.g.attach(strength, Entity::Object(bears)));
    t.g.recompute();
    assert!(t.obj_now(bears).has_keyword(KeywordKind::FirstStrike));
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Lifelink));
}

#[test]
fn daybreak_coronet_enchants_only_a_creature_with_another_aura() {
    cr!("303.4a", "303.4c", "704.5m");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elf = t.battlefield(P0, "Llanowar Elves");
    let strength = t.battlefield(P0, "Holy Strength");
    assert!(t.g.attach(strength, Entity::Object(bears)));
    let coronet = t.hand(P0, "Daybreak Coronet");
    t.cast(P0, coronet).target(bears).go();
    // The Elves have no Aura: not a legal target.
    let candidates = t
        .asked()
        .into_iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseTargets { candidates, .. } => Some(candidates),
            _ => None,
        })
        .expect("targets");
    assert!(candidates.contains(&Entity::Object(bears)));
    assert!(!candidates.contains(&Entity::Object(elf)));
    t.resolve_all();
    let coronet = t.g.current(coronet);
    assert_eq!(t.obj_now(coronet).attached_to, Some(Entity::Object(bears)));
    assert_eq!(t.pt(bears), (6, 7));
    // Once the other Aura is gone, the Coronet is put into the graveyard.
    t.g.destroy(strength, None);
    t.settle();
    assert!(t.in_graveyard(P0, "Daybreak Coronet"));
}

#[test]
fn silence_the_believers_exiles_the_targets_and_the_auras_attached_to_them() {
    cr!("608.2c", "601.2f");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let other = t.battlefield(P1, "Llanowar Elves");
    let on_a = t.battlefield(P1, "Holy Strength");
    let on_other = t.battlefield(P1, "Pacifism");
    assert!(t.g.attach(on_a, Entity::Object(a)));
    assert!(t.g.attach(on_other, Entity::Object(other)));
    let spell = t.hand(P0, "Silence the Believers");
    let swamps = t.lands(P0, "Swamp", 7);
    t.set_step(P0, Step::PrecombatMain);
    t.cast(P0, spell)
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .go();
    t.resolve_all();
    // {2}{B}{B} plus {2}{B} for the second target (strive).
    assert!(swamps.iter().all(|l| t.obj_now(*l).tapped));
    assert!(t.in_exile("Grizzly Bears"));
    assert!(t.in_exile("Hill Giant"));
    assert!(t.in_exile("Holy Strength"));
    // An Aura attached to another creature stays.
    assert!(t.on_battlefield(on_other));
}

#[test]
fn steam_vines_that_player_attaches_it_to_a_land_of_their_choice() {
    cr!("701.3a", "303.4");
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P1, "Forest");
    let island = t.battlefield(P1, "Island");
    let vines = t.battlefield(P0, "Steam Vines");
    assert!(t.g.attach(vines, Entity::Object(forest)));
    t.answer_choose(P1, &[Entity::Object(island)]);
    t.g.tap(forest);
    t.g.flush_events();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Forest"));
    assert_eq!(t.life(P1), 19);
    assert!(t.on_battlefield(vines));
    assert_eq!(t.obj_now(vines).attached_to, Some(Entity::Object(island)));
    // P1 chose.
    assert!(t.asked().iter().any(|(p, d)| *p == P1
        && matches!(d, mtg_engine::decision::Decision::ChooseEntities { .. })));
}
