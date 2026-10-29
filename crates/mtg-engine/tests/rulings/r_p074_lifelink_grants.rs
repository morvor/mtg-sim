//! Rulings batch P074 — granting lifelink: combat-conditional statics (Throatseeker,
//! Windbrisk Raptor), abilities that target attacking or blocked creatures (Unraveling
//! Mummy, Apocalypse Runner), "creatures you control" sets fixed on resolution (Tenacity,
//! Vault of the Archangel, Rush of Battle, Midnight Mayhem, Book of Mazarbul, Bond of
//! Discipline), statics covering every matching permanent (Necromancer's Covenant, The
//! Broken Sky), and equipment and Auras (Resurrection Orb, Felidar Umbra).

use crate::r_s01_common::{attack_with, give_mana_for, supported};
use crate::r_s02_common::{create_token, destroy};
use crate::r_s03_common::to_blockers;
use crate::r_s05_common::{enter, tokens_with_subtype};
use crate::r_s06_common::{activate_containing, attach_new, has_kw};

use crate::r_s17_common::enter_transformed;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const LIFELINK: KeywordKind = KeywordKind::Lifelink;

fn pl(p: PlayerId) -> Entity {
    Entity::Player(p)
}

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

fn cast_resolve(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) {
    give_mana_for(t, p, name);
    let c = t.hand(p, name);
    let mut cb = t.cast(p, c);
    for e in targets {
        cb = cb.target(*e);
    }
    cb.go();
    t.resolve_all();
}

#[test]
fn throatseeker_applies_only_to_unblocked_attackers_after_blocks() {
    cr!("509.1h", "506.4", "511.3");
    ruling!(
        "Throatseeker",
        "Throatseeker’s effect applies only after blockers have been declared. Before then, attacking creatures are neither blocked nor unblocked."
    );
    ruling!(
        "Throatseeker",
        "If an unblocked attacking Ninja you control leaves combat (most likely because the combat phase has ended), it no longer has lifelink."
    );
    supported("Throatseeker");
    let mut t = TestGame::new(2);
    let ts = t.battlefield(P0, "Throatseeker");
    attack_with(&mut t, &[(ts, pl(P1))]);
    assert!(!has_kw(&t, ts, LIFELINK), "not yet unblocked");
    t.advance_to(P0, Step::DeclareBlockers);
    assert!(has_kw(&t, ts, LIFELINK));
    t.advance_to(P0, Step::PostcombatMain);
    assert_eq!((t.life(P0), t.life(P1)), (23, 17));
    assert!(!has_kw(&t, ts, LIFELINK), "left combat");
}

#[test]
fn windbrisk_raptor_lifelink_lasts_while_attacking_and_raptor_remains() {
    cr!("506.4", "508.1", "611.3a");
    ruling!(
        "Windbrisk Raptor",
        "This is a static ability. Your creatures will have lifelink from the moment they’re declared as attackers until the moment the combat phase ends, they’re removed from combat, or Windbrisk Raptor leaves the battlefield, whichever comes first."
    );
    supported("Windbrisk Raptor");
    let mut t = TestGame::new(2);
    let raptor = t.battlefield(P0, "Windbrisk Raptor");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    assert!(!has_kw(&t, bears, LIFELINK));
    attack_with(&mut t, &[(bears, pl(P1)), (elves, pl(P1))]);
    assert!(has_kw(&t, bears, LIFELINK) && has_kw(&t, elves, LIFELINK));
    assert!(!has_kw(&t, raptor, LIFELINK), "not attacking");
    t.advance_to(P0, Step::PostcombatMain);
    assert_eq!(t.life(P0), 23);
    assert!(!has_kw(&t, bears, LIFELINK), "combat ended");
    // Raptor leaving the battlefield ends it too.
    let mut t = TestGame::new(2);
    let raptor = t.battlefield(P0, "Windbrisk Raptor");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, pl(P1))]);
    assert!(has_kw(&t, bears, LIFELINK));
    destroy(&mut t, raptor);
    assert!(!has_kw(&t, bears, LIFELINK));
}

#[test]
fn unraveling_mummy_targets_itself_redundantly_and_keeps_it_after_combat() {
    cr!("702.15f", "702.2f", "611.2a");
    ruling!(
        "Unraveling Mummy",
        "Unraveling Mummy’s abilities can target itself while it’s attacking."
    );
    ruling!(
        "Unraveling Mummy",
        "Multiple instances of lifelink and deathtouch are redundant."
    );
    ruling!(
        "Unraveling Mummy",
        "The target Zombie will still have lifelink or deathtouch even after combat when it stops being an attacking creature."
    );
    supported("Unraveling Mummy");
    let mut t = TestGame::new(2);
    let mummy = t.battlefield(P0, "Unraveling Mummy");
    t.lands(P0, "Plains", 4);
    t.lands(P0, "Swamp", 2);
    attack_with(&mut t, &[(mummy, pl(P1))]);
    for _ in 0..2 {
        t.answer_targets(P0, &[obj(mummy)]);
        activate_containing(&mut t, P0, mummy, "lifelink").unwrap();
        t.resolve_all();
    }
    t.answer_targets(P0, &[obj(mummy)]);
    activate_containing(&mut t, P0, mummy, "deathtouch").unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(mummy).chars.keywords_of(LIFELINK).count(), 2);
    t.advance_to(P0, Step::PostcombatMain);
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 22, "two instances of lifelink gain 2, not 4");
    assert!(has_kw(&t, mummy, LIFELINK) && has_kw(&t, mummy, KeywordKind::Deathtouch));
}

#[test]
fn apocalypse_runner_after_blocks_doesnt_unblock() {
    cr!("509.1h", "702.15b");
    ruling!(
        "Apocalypse Runner",
        "Once a creature has been blocked, activating this Vehicle’s first ability targeting that creature won’t cause it to become unblocked."
    );
    supported("Apocalypse Runner");
    let mut t = TestGame::new(2);
    let runner = t.battlefield(P0, "Apocalypse Runner");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wall = t.battlefield(P1, "Wall of Stone");
    to_blockers(&mut t, &[(bears, pl(P1))], &[(wall, bears)]);
    t.answer_targets(P0, &[obj(bears)]);
    activate_containing(&mut t, P0, runner, "lifelink").unwrap();
    t.resolve_all();
    assert!(has_kw(&t, bears, LIFELINK));
    t.advance_to(P0, Step::PostcombatMain);
    assert_eq!(t.life(P1), 20, "still blocked");
    assert_eq!(t.obj_now(wall).damage, 2);
    assert_eq!(t.life(P0), 22);
}

#[test]
fn creatures_you_control_are_fixed_as_the_effect_resolves() {
    cr!("611.2c");
    ruling!(
        "Tenacity",
        "The set of creatures affected by Tenacity is determined as the spell resolves. Creatures you begin to control later in the turn won't get +1/+1 or gain lifelink."
    );
    ruling!(
        "Vault of the Archangel",
        "Vault of the Archangel's last ability affects only creatures you control when that ability resolves. It won't affect creatures that come under your control later in the turn."
    );
    ruling!(
        "Rush of Battle",
        "Warrior creatures you control as Rush of Battle resolves gain lifelink in addition to getting +2/+1."
    );
    ruling!(
        "Book of Mazarbul",
        "The last chapter ability affects only creatures you control at the time it resolves. Any creatures that come under your control later in the turn won't be affected."
    );
    ruling!(
        "Midnight Mayhem",
        "Midnight Mayhem affects only Gremlins you control at the time it resolves, including the Gremlin tokens it creates. Gremlins you begin to control later in the turn won't gain menace, lifelink, or haste."
    );
    for name in ["Tenacity", "Rush of Battle", "Bond of Discipline"] {
        supported(name);
    }
    // Tenacity: +1/+1, lifelink, untap.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.tap(bears);
    cast_resolve(&mut t, P0, "Tenacity", &[]);
    let later = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.pt(bears), (3, 3));
    assert!(has_kw(&t, bears, LIFELINK) && !t.obj_now(bears).tapped);
    assert_eq!(t.pt(later), (2, 2));
    assert!(!has_kw(&t, later, LIFELINK));
    // Vault of the Archangel.
    let mut t = TestGame::new(2);
    let vault = t.battlefield(P0, "Vault of the Archangel");
    t.lands(P0, "Plains", 3);
    t.lands(P0, "Swamp", 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    activate_containing(&mut t, P0, vault, "lifelink").unwrap();
    t.resolve_all();
    let later = t.battlefield(P0, "Grizzly Bears");
    assert!(has_kw(&t, bears, LIFELINK) && has_kw(&t, bears, KeywordKind::Deathtouch));
    assert!(!has_kw(&t, later, LIFELINK) && !has_kw(&t, later, KeywordKind::Deathtouch));
    // Rush of Battle: Warriors gain lifelink as well as +2/+1; non-Warriors only get
    // +2/+1.
    let mut t = TestGame::new(2);
    let khenra = t.battlefield(P0, "Defiant Khenra");
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Rush of Battle", &[]);
    let later = t.battlefield(P0, "Defiant Khenra");
    assert_eq!(t.pt(khenra), (4, 3));
    assert!(has_kw(&t, khenra, LIFELINK));
    assert_eq!(t.pt(bears), (4, 3));
    assert!(!has_kw(&t, bears, LIFELINK));
    assert_eq!(t.pt(later), (2, 2));
    assert!(!has_kw(&t, later, LIFELINK));
    // Book of Mazarbul's chapter III.
    let mut t = TestGame::new(2);
    let book = t.battlefield(P0, "Book of Mazarbul");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.add_counters(obj(book), "lore", 3, None);
    t.g.flush_events();
    t.resolve_all();
    let later = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.pt(bears), (3, 2));
    assert!(has_kw(&t, bears, KeywordKind::Menace));
    assert_eq!(t.pt(later), (2, 2));
    assert!(!has_kw(&t, later, KeywordKind::Menace));
    // Midnight Mayhem: an earlier Gremlin and the three it creates; not a later one.
    let mut t = TestGame::new(2);
    let gremlin = create_token(&mut t, P0, "Gremlin");
    cast_resolve(&mut t, P0, "Midnight Mayhem", &[]);
    let gremlins = tokens_with_subtype(&t, P0, "Gremlin");
    assert_eq!(gremlins.len(), 4);
    assert!(gremlins.contains(&gremlin));
    for g in &gremlins {
        for kw in [KeywordKind::Menace, LIFELINK, KeywordKind::Haste] {
            assert!(has_kw(&t, *g, kw), "{kw:?}");
        }
    }
    let later = create_token(&mut t, P0, "Gremlin");
    assert!(!has_kw(&t, later, LIFELINK) && !has_kw(&t, later, KeywordKind::Menace));

}

#[test]
fn bond_of_discipline_can_be_cast_with_no_creatures() {
    cr!("601.2c", "608.2b");
    ruling!(
        "Bond of Discipline",
        "You can cast Bond of Discipline even if your opponents control no creatures or you control no creatures. The creatures that are on the battlefield will be affected as appropriate."
    );
    let mut t = TestGame::new(2);
    cast_resolve(&mut t, P0, "Bond of Discipline", &[]);
    assert!(t.in_graveyard(P0, "Bond of Discipline"));
    // With only an opponent's creature: it's tapped.
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P1, "Llanowar Elves");
    cast_resolve(&mut t, P0, "Bond of Discipline", &[]);
    assert!(t.obj_now(elves).tapped);
    // With only your creature: it gains lifelink.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Bond of Discipline", &[]);
    assert!(has_kw(&t, bears, LIFELINK) && !t.obj_now(bears).tapped);
}

#[test]
fn necromancers_covenant_zombies_are_yours_and_all_your_zombies_have_lifelink() {
    cr!("111.2", "613.1f");
    ruling!(
        "Necromancer's Covenant",
        "The Zombie tokens are put onto the battlefield under your control."
    );
    ruling!(
        "Necromancer's Covenant",
        "Necromancer’s Covenant grants lifelink to all Zombies you control, not just the tokens put onto the battlefield with its first ability."
    );
    supported("Necromancer's Covenant");
    let mut t = TestGame::new(2);
    t.graveyard(P1, "Grizzly Bears");
    t.graveyard(P1, "Llanowar Elves");
    let corpse = t.battlefield(P0, "Undead Minotaur");
    let their_zombie = t.battlefield(P1, "Undead Minotaur");
    t.answer_targets(P0, &[pl(P1)]);
    enter(&mut t, P0, "Necromancer's Covenant");
    t.resolve_all();
    let zombies = tokens_with_subtype(&t, P0, "Zombie");
    assert_eq!(zombies.len(), 2);
    assert!(tokens_with_subtype(&t, P1, "Zombie").is_empty());
    for z in zombies {
        assert_eq!(t.obj_now(z).controller, P0);
        assert!(has_kw(&t, z, LIFELINK));
    }
    assert!(has_kw(&t, corpse, LIFELINK));
    assert!(!has_kw(&t, their_zombie, LIFELINK));
}

#[test]
fn the_broken_sky_applies_to_all_creature_tokens_you_control() {
    cr!("613.1f", "111.1");
    ruling!(
        "Invasion of Tolvada // The Broken Sky",
        "The Broken Sky’s first ability applies to all creature tokens you control, not only the ones created by its other ability."
    );
    let mut t = TestGame::new(2);
    enter_transformed(&mut t, P0, "Invasion of Tolvada // The Broken Sky");
    let soldier = create_token(&mut t, P0, "Soldier");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.recompute();
    assert_eq!(t.pt(soldier), (2, 1));
    assert!(has_kw(&t, soldier, LIFELINK));
    assert_eq!(t.pt(bears), (2, 2));
    assert!(!has_kw(&t, bears, LIFELINK));
}

#[test]
fn resurrection_orb_delayed_trigger_works_after_the_orb_is_gone() {
    cr!("603.7c", "603.7a");
    ruling!(
        "Resurrection Orb",
        "Resurrection Orb's triggered ability creates a delayed triggered ability that will trigger at the beginning of the next end step. It triggers even if Resurrection Orb is no longer on the battlefield at that time."
    );
    supported("Resurrection Orb");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let orb = attach_new(&mut t, P0, "Resurrection Orb", obj(bears));
    assert!(has_kw(&t, bears, LIFELINK));
    destroy(&mut t, bears);
    t.resolve_all();
    destroy(&mut t, orb);
    assert!(t.in_graveyard(P0, "Resurrection Orb"));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn felidar_umbra_saves_only_the_creature_it_enchants() {
    cr!("702.89a", "614.6");
    ruling!(
        "Felidar Umbra",
        "It's not possible for Felidar Umbra to save two creatures that are being destroyed simultaneously."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    attach_new(&mut t, P0, "Felidar Umbra", obj(bears));
    cast_resolve(&mut t, P0, "Wrath of God", &[]);
    assert!(t.on_battlefield(bears));
    assert!(!t.on_battlefield(elves));
    assert!(t.in_graveyard(P0, "Felidar Umbra"));
    assert!(t.in_graveyard(P0, "Llanowar Elves"));
}
