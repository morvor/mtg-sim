//! Attaching and unattaching (CR 701.3): "attach [attachments] to [recipient]" and
//! "unattach [object]" (patterns in `src/oracle/patterns/attach_control_grammar.rs`).

use mtg_engine::decision::Decision;
use mtg_engine::object::Zone;
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

fn attached(t: &TestGame, id: ObjectId) -> Option<Entity> {
    t.obj_now(id).attached_to
}

/// The candidates of the last "choose" decision `p` was asked.
fn last_choice(t: &TestGame, p: PlayerId) -> Vec<Entity> {
    t.asked()
        .into_iter()
        .rev()
        .find_map(|(q, d)| match d {
            Decision::ChooseEntities { candidates, .. } if q == p => Some(candidates),
            _ => None,
        })
        .unwrap_or_default()
}

#[test]
fn attach_cards_compile() {
    assert_compiles(&[
        "Auriok Windwalker",
        "Kor Outfitter",
        "Magnetic Theft",
        "Aura Finesse",
        "Codsworth, Handy Helper",
        "Nahiri, the Lithomancer",
        "Nahiri, Heir of the Ancients",
        "Resolute Strike",
        "Vow to Erebor",
        "Battlefield Improvisation",
        "Ardenn, Intrepid Archaeologist",
        "Beatrix, Loyal General",
        "Heavenly Blademaster",
        "Armory Automaton",
        "Super-Soldier Serum",
        "Iron Hills Stalwart",
        "Cloud, Ex-SOLDIER",
        "Sokka, Swordmaster",
        "Barret, Avalanche Leader",
        "Raubahn, Bull of Ala Mhigo",
        "Sokka and Suki",
        "Kemba, Kha Enduring",
        "Swordsman, Sharp Scoundrel",
        "Sigarda's Aid",
        "Hammer of Nazahn",
        "Amy Rose",
        "Shagrat, Loot Bearer",
        "Kazuul's Toll Collector",
        "Vulshok Battlemaster",
        "Goblin Plate Mail",
        "Fractal Harness",
        "U.S.Agent, John Walker",
        "Tony Stark // The Invincible Iron Man",
        "Scythe of the Wretched",
        "Deathrender",
        "Auriok Survivors",
        "Yuffie, Materia Hunter",
        "Halvar, God of Battle // Sword of the Realms",
        "Archnemesis",
        "Stonehewer Giant",
        "Quest for the Holy Relic",
        "Carry Away",
        "Tamiyo's Compleation",
    ]);
}

#[test]
fn attach_target_equipment_to_target_creature() {
    cr!("701.3a", "701.3c", "301.5b");
    let mut t = TestGame::new(2);
    let walker = t.battlefield(P0, "Auriok Windwalker");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elf = t.battlefield(P0, "Llanowar Elves");
    let blade = t.battlefield(P0, "Bonesplitter");
    assert!(t.g.attach(blade, Entity::Object(bears)));
    let ts = t.obj_now(blade).timestamp;
    t.activate(P0, walker, 0, &[Entity::Object(blade), Entity::Object(elf)])
        .unwrap();
    t.resolve_all();
    assert_eq!(attached(&t, blade), Some(Entity::Object(elf)));
    // CR 701.3c: a new timestamp.
    assert!(t.obj_now(blade).timestamp > ts);
    assert_eq!(t.pt(elf), (3, 1));
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn equipment_that_cant_equip_the_creature_doesnt_move() {
    cr!("701.3b", "301.5b");
    // Protection from artifacts: it can't be equipped by an artifact (CR 702.16).
    let mut t = TestGame::new(2);
    let walker = t.battlefield(P0, "Auriok Windwalker");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let chosen = t.battlefield(P0, "Tel-Jilad Chosen");
    let blade = t.battlefield(P0, "Bonesplitter");
    assert!(t.g.attach(blade, Entity::Object(bears)));
    t.activate(
        P0,
        walker,
        0,
        &[Entity::Object(blade), Entity::Object(chosen)],
    )
    .unwrap();
    t.resolve_all();
    assert_eq!(attached(&t, blade), Some(Entity::Object(bears)));
}

#[test]
fn kor_outfitter_does_nothing_if_a_target_is_illegal() {
    cr!("608.2b", "701.3b");
    ruling!(
        "Kor Outfitter",
        "If either target is illegal by the time the ability resolves, the ability won’t do anything."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let blade = t.battlefield(P0, "Bonesplitter");
    let elf = t.battlefield(P0, "Llanowar Elves");
    assert!(t.g.attach(blade, Entity::Object(bears)));
    let outfitter = t.hand(P0, "Kor Outfitter");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(blade)]);
    t.answer_targets(P0, &[Entity::Object(elf)]);
    t.cast(P0, outfitter).go();
    t.resolve(); // the creature spell
    t.settle(); // the trigger is put on the stack
    assert_eq!(t.stack_len(), 1);
    // The creature target leaves before the ability resolves.
    let elf_now = t.g.current(elf);
    t.g.destroy(elf_now, None);
    t.resolve_all();
    assert_eq!(attached(&t, blade), Some(Entity::Object(bears)));

    // With both targets legal, the Equipment moves.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let blade = t.battlefield(P0, "Bonesplitter");
    let elf = t.battlefield(P0, "Llanowar Elves");
    assert!(t.g.attach(blade, Entity::Object(bears)));
    let outfitter = t.hand(P0, "Kor Outfitter");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(blade)]);
    t.answer_targets(P0, &[Entity::Object(elf)]);
    t.cast(P0, outfitter).go();
    t.resolve_all();
    assert_eq!(attached(&t, blade), Some(Entity::Object(elf)));
}

#[test]
fn magnetic_theft_doesnt_change_control() {
    cr!("301.5d", "701.3a");
    ruling!(
        "Magnetic Theft",
        "Magnetic Theft can cause an Equipment one player controls to be attached to a creature another player controls."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let their_bears = t.battlefield(P1, "Grizzly Bears");
    let blade = t.battlefield(P1, "Bonesplitter");
    assert!(t.g.attach(blade, Entity::Object(their_bears)));
    let elf = t.battlefield(P0, "Llanowar Elves");
    let theft = t.hand(P0, "Magnetic Theft");
    t.cast(P0, theft).target(blade).target(elf).go();
    t.resolve_all();
    assert_eq!(attached(&t, blade), Some(Entity::Object(elf)));
    assert_eq!(t.obj_now(blade).controller, P1);
    assert_eq!(t.pt(elf), (3, 1));
}

#[test]
fn aura_finesse_aura_that_cant_enchant_doesnt_move_but_you_draw() {
    cr!("303.4j", "701.3b");
    ruling!(
        "Aura Finesse",
        "As Aura Finesse resolves, if the targeted Aura can’t legally enchant the targeted creature"
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let strength = t.battlefield(P0, "Holy Strength");
    assert!(t.g.attach(strength, Entity::Object(bears)));
    // A white Aura can't enchant a creature with protection from white.
    let knight = t.battlefield(P1, "White Knight");
    let _ = knight;
    let black_knight = t.battlefield(P1, "Black Knight");
    let finesse = t.hand(P0, "Aura Finesse");
    let hand = t.hand_size(P0);
    t.cast(P0, finesse)
        .target(strength)
        .target(black_knight)
        .go();
    t.resolve_all();
    assert_eq!(attached(&t, strength), Some(Entity::Object(bears)));
    assert_eq!(t.hand_size(P0), hand); // cast one, drew one
                                       // A creature it can enchant (one an opponent controls): it moves; you still control it.
    let finesse = t.hand(P0, "Aura Finesse");
    t.lands(P0, "Island", 1);
    t.cast(P0, finesse).target(strength).target(knight).go();
    t.resolve_all();
    assert_eq!(attached(&t, strength), Some(Entity::Object(knight)));
    assert_eq!(t.obj_now(strength).controller, P0);
}

#[test]
fn nahiri_attaches_a_chosen_equipment_to_the_token() {
    cr!("701.3a", "608.2c");
    let mut t = TestGame::new(2);
    let nahiri = t.battlefield(P0, "Nahiri, the Lithomancer");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let blade = t.battlefield(P0, "Bonesplitter");
    assert!(t.g.attach(blade, Entity::Object(bears)));
    // An opponent's Equipment can't be chosen.
    let theirs = t.battlefield(P1, "Leonin Scimitar");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(blade)]);
    t.activate(P0, nahiri, 0, &[]).unwrap();
    t.resolve_all();
    let token =
        t.g.battlefield
            .iter()
            .copied()
            .find(|o| t.obj_now(*o).is_token() && t.obj_now(*o).chars.has_subtype("Kor"))
            .expect("token");
    assert_eq!(attached(&t, blade), Some(Entity::Object(token)));
    assert_eq!(t.pt(token), (3, 1));
    assert!(!last_choice(&t, P0).contains(&Entity::Object(theirs)));
}

#[test]
fn only_attachments_that_could_be_attached_are_chosen() {
    cr!("701.3a", "303.4j");
    ruling!(
        "Heavenly Blademaster",
        "You can’t try to attach an Aura or Equipment to Heavenly Blademaster if that Aura or Equipment can’t legally be attached to it."
    );
    ruling!(
        "Heavenly Blademaster",
        "You don’t move any Auras or Equipment until Heavenly Blademaster’s triggered ability is resolving."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 6);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let blade = t.battlefield(P0, "Bonesplitter");
    assert!(t.g.attach(blade, Entity::Object(bears)));
    let strength = t.battlefield(P0, "Holy Strength");
    assert!(t.g.attach(strength, Entity::Object(bears)));
    let forest = t.battlefield(P0, "Forest");
    let growth = t.battlefield(P0, "Wild Growth");
    assert!(t.g.attach(growth, Entity::Object(forest)));
    let blademaster = t.hand(P0, "Heavenly Blademaster");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(blade), Entity::Object(strength)]);
    t.cast(P0, blademaster).go();
    t.resolve_all();
    let bm = t.g.current(blademaster);
    assert_eq!(attached(&t, blade), Some(Entity::Object(bm)));
    assert_eq!(attached(&t, strength), Some(Entity::Object(bm)));
    // The Aura with "enchant land" wasn't offered and stays on the Forest.
    let offered = last_choice(&t, P0);
    assert!(offered.contains(&Entity::Object(blade)));
    assert!(!offered.contains(&Entity::Object(growth)));
    assert_eq!(attached(&t, growth), Some(Entity::Object(forest)));
    // "Other creatures you control get +1/+1 for each Aura and Equipment attached to this
    // creature."
    assert_eq!(t.pt(bears), (4, 4));
}

#[test]
fn ardenn_attaches_a_curse_to_target_player() {
    cr!("701.3a", "303.4b");
    ruling!(
        "Ardenn, Intrepid Archaeologist",
        "You choose which Auras and Equipment to move as Ardenn’s ability resolves."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ardenn, Intrepid Archaeologist");
    let curse = t.battlefield(P0, "Curse of the Pierced Heart");
    assert!(t.g.attach(curse, Entity::Player(P0)));
    let bears = t.battlefield(P0, "Grizzly Bears");
    let blade = t.battlefield(P0, "Bonesplitter");
    assert!(t.g.attach(blade, Entity::Object(bears)));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(curse)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(attached(&t, curse), Some(Entity::Player(P1)));
    // An Equipment can't be attached to a player: it wasn't offered.
    assert!(!last_choice(&t, P0).contains(&Entity::Object(blade)));
    assert_eq!(attached(&t, blade), Some(Entity::Object(bears)));
}

#[test]
fn stonehewer_giant_attaches_the_found_equipment_to_a_creature_it_can_equip() {
    cr!("701.3a", "400.7", "701.23a");
    ruling!(
        "Stonehewer Giant",
        "Stonehewer Giant's ability doesn't target a creature. However, the creature must be able to be legally equipped by the Equipment."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 2);
    let giant = t.battlefield(P0, "Stonehewer Giant");
    let chosen = t.battlefield(P0, "Tel-Jilad Chosen");
    let blade = t.library_top(P0, "Bonesplitter");
    t.answer_choose(P0, &[Entity::Object(blade)]);
    t.activate(P0, giant, 0, &[]).unwrap();
    t.resolve_all();
    let on_bf = t.g.current(blade);
    assert_eq!(t.zone(on_bf), Zone::Battlefield);
    // The creature with protection from artifacts wasn't a choice; the Giant was.
    let offered = last_choice(&t, P0);
    assert!(offered.contains(&Entity::Object(giant)));
    assert!(!offered.contains(&Entity::Object(chosen)));
    assert_eq!(attached(&t, on_bf), Some(Entity::Object(giant)));
    assert_eq!(t.pt(giant), (6, 4));
}

#[test]
fn vulshok_battlemaster_takes_every_equipment() {
    cr!("701.3a", "301.5d");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 5);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Bonesplitter");
    assert!(t.g.attach(theirs, Entity::Object(bears)));
    let mine = t.battlefield(P0, "Leonin Scimitar");
    let bm = t.hand(P0, "Vulshok Battlemaster");
    t.cast(P0, bm).go();
    t.resolve_all();
    let bm = t.g.current(bm);
    assert_eq!(attached(&t, theirs), Some(Entity::Object(bm)));
    assert_eq!(attached(&t, mine), Some(Entity::Object(bm)));
    assert_eq!(t.obj_now(theirs).controller, P1);
    assert_eq!(t.pt(bm), (5, 3));
}

#[test]
fn goblin_plate_mail_attaches_to_the_amassed_army() {
    cr!("701.47a", "701.47c", "701.3a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let mail = t.hand(P0, "Goblin Plate Mail");
    t.cast(P0, mail).go();
    t.resolve_all();
    let mail = t.g.current(mail);
    let army =
        t.g.battlefield
            .iter()
            .copied()
            .find(|o| t.obj_now(*o).chars.has_subtype("Army"))
            .expect("army");
    assert_eq!(attached(&t, mail), Some(Entity::Object(army)));
    // A 0/0 Army with a +1/+1 counter, +1/+0 from the Equipment.
    assert_eq!(t.pt(army), (2, 1));
}

#[test]
fn armory_automaton_attach_all_or_none_of_the_targets() {
    cr!("701.3a", "603.5");
    ruling!(
        "Armory Automaton",
        "you must either attach all of the target Equipment or attach none of them"
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let a = t.battlefield(P0, "Bonesplitter");
    let b = t.battlefield(P1, "Leonin Scimitar");
    assert!(t.g.attach(a, Entity::Object(bears)));
    let auto = t.hand(P0, "Armory Automaton");
    t.answer_targets(P0, &[Entity::Object(a), Entity::Object(b)]);
    t.answer_yes(P0, true);
    t.cast(P0, auto).go();
    t.resolve_all();
    let auto = t.g.current(auto);
    assert_eq!(attached(&t, a), Some(Entity::Object(auto)));
    assert_eq!(attached(&t, b), Some(Entity::Object(auto)));
    assert_eq!(t.obj_now(b).controller, P1);
    assert_eq!(t.pt(auto), (5, 3));
}

#[test]
fn sigarda_s_aid_attaches_the_equipment_that_entered() {
    cr!("701.3a", "603.2");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 1);
    t.battlefield(P0, "Sigarda's Aid");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let blade = t.hand(P0, "Bonesplitter");
    t.cast(P0, blade).go();
    t.resolve(); // the Equipment spell
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_yes(P0, true);
    t.resolve_all();
    let blade = t.g.current(blade);
    assert_eq!(attached(&t, blade), Some(Entity::Object(bears)));
}

#[test]
fn carry_away_unattaches_the_enchanted_equipment() {
    cr!("701.3d", "303.4e");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let blade = t.battlefield(P1, "Bonesplitter");
    assert!(t.g.attach(blade, Entity::Object(bears)));
    let carry = t.hand(P0, "Carry Away");
    t.cast(P0, carry).target(blade).go();
    t.resolve_all();
    assert_eq!(attached(&t, blade), None);
    assert_eq!(t.obj_now(blade).controller, P0);
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn tamiyos_compleation_unattaches_only_an_equipment() {
    cr!("701.3d");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 4);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let blade = t.battlefield(P1, "Bonesplitter");
    assert!(t.g.attach(blade, Entity::Object(bears)));
    let tc = t.hand(P0, "Tamiyo's Compleation");
    t.cast(P0, tc).target(blade).go();
    t.resolve_all();
    assert_eq!(attached(&t, blade), None);
    assert!(t.obj_now(blade).tapped);
    // On a creature: it's tapped, nothing is unattached.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 4);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let strength = t.battlefield(P1, "Holy Strength");
    assert!(t.g.attach(strength, Entity::Object(bears)));
    let tc = t.hand(P0, "Tamiyo's Compleation");
    t.cast(P0, tc).target(bears).go();
    t.resolve_all();
    assert!(t.obj_now(bears).tapped);
    assert_eq!(attached(&t, strength), Some(Entity::Object(bears)));
}

#[test]
fn lynde_attaches_a_curse_attached_to_you_to_an_opponent() {
    cr!("701.3a", "303.4");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lynde, Cheerful Tormentor");
    let curse = t.battlefield(P1, "Curse of the Pierced Heart");
    assert!(t.g.attach(curse, Entity::Player(P0)));
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(curse)]);
    let hand = t.hand_size(P0);
    t.set_step(P0, Step::Untap);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(attached(&t, curse), Some(Entity::Player(P1)));
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn thorin_attached_equipment_lets_the_creature_deal_damage() {
    cr!("603.12", "701.3a");
    assert_compiles(&["Thorin, Mountain-king"]);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let blade = t.battlefield(P0, "Bonesplitter");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(blade)]);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.enter(P0, "Thorin, Mountain-king");
    t.resolve_all();
    assert_eq!(attached(&t, blade), Some(Entity::Object(bears)));
    // The equipped Bears (4/2) dealt 4 damage to the Hill Giant.
    assert!(!t.on_battlefield(giant));
}

#[test]
fn thorin_no_trigger_if_nothing_became_attached() {
    cr!("603.12", "701.3b");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let blade = t.battlefield(P0, "Bonesplitter");
    assert!(t.g.attach(blade, Entity::Object(bears)));
    let giant = t.battlefield(P1, "Hill Giant");
    // The Bonesplitter is already attached to the Bears: nothing becomes attached.
    t.answer_targets(P0, &[Entity::Object(blade)]);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.enter(P0, "Thorin, Mountain-king");
    t.resolve_all();
    assert_eq!(attached(&t, blade), Some(Entity::Object(bears)));
    assert!(t.on_battlefield(giant));
    assert_eq!(t.obj_now(giant).damage, 0);
}

#[test]
fn frodo_attaches_an_equipment_with_mana_value_2_or_3() {
    cr!("701.3a", "202.3");
    assert_compiles(&["Frodo, Determined Hero"]);
    let mut t = TestGame::new(2);
    let star = t.battlefield(P0, "Vulshok Morningstar");
    let blade = t.battlefield(P0, "Bonesplitter");
    t.answer_targets(P0, &[Entity::Object(star)]);
    t.answer_yes(P0, true);
    let frodo = t.enter(P0, "Frodo, Determined Hero");
    t.resolve_all();
    assert_eq!(attached(&t, star), Some(Entity::Object(frodo)));
    assert_eq!(t.pt(frodo), (4, 4));
    // Bonesplitter (mana value 1) wasn't a legal target.
    let offered = t
        .asked()
        .into_iter()
        .find_map(|(q, d)| match d {
            Decision::ChooseTargets { candidates, .. } if q == P0 => Some(candidates),
            _ => None,
        })
        .expect("no target choice");
    assert!(offered.contains(&Entity::Object(star)));
    assert!(!offered.contains(&Entity::Object(blade)));
}
