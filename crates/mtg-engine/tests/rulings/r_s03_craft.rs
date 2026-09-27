//! Rulings batch S03 — craft (CR 702.167): "Craft with [materials] [cost]" means "[Cost],
//! Exile this permanent, Exile [materials] from among permanents you control and/or cards
//! in your graveyard: Return this card to the battlefield transformed under its owner's
//! control. Activate only as a sorcery."

use crate::r_s01_common::*;
use crate::r_s02_common::{can_activate, create_token};
use crate::r_s03_common::{choice_candidates, run_effect};
use mtg_engine::ability::{Destination, Effect, Filter, Modification, PlayerRef, Sel};
use mtg_engine::types::{Color, ColorSet};
use mtg_engine::object::{FaceState, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const VISAGE: &str = "Visage of Dread // Dread Osseosaur";
const LATTICE: &str = "Saheeli's Lattice // Mastercraft Raptor";
const SEEDSTONES: &str = "Jade Seedstones // Jadeheart Attendant";
const BLADE: &str = "Tithing Blade // Consuming Sepulcher";

/// Activates the craft ability (the only activated ability) of `src`, exiling `materials`.
fn craft(t: &mut TestGame, src: ObjectId, materials: &[ObjectId]) {
    let es: Vec<Entity> = materials.iter().map(|o| Entity::Object(*o)).collect();
    t.answer_choose(P0, &es);
    t.activate(P0, src, 0, &[]).expect("craft");
}

/// The permanent named `name` on the battlefield (exactly one).
fn one(t: &TestGame, name: &str) -> ObjectId {
    let v = t.named_on_battlefield(name);
    assert_eq!(v.len(), 1, "{name}");
    v[0]
}

/// Moves `id` from exile to its owner's graveyard (as an effect would).
fn exile_to_graveyard(t: &mut TestGame, id: ObjectId) {
    run_effect(
        t,
        None,
        P0,
        Effect::Move {
            what: Sel::Target(0),
            to: Destination::zone(mtg_engine::ability::ZoneKind::Graveyard),
        },
        &[Entity::Object(id)],
    );
    assert!(!t.g.is_live(id));
}

#[test]
fn materials_can_come_partly_from_the_battlefield_and_partly_from_the_graveyard() {
    cr!("702.167a", "702.167b", "712.14a");
    ruling!(
        "Visage of Dread // Dread Osseosaur",
        "If the materials required include multiple objects, you may exile some of them from among permanents you control and the rest from among cards in your graveyard. You don't have to choose all permanents or all cards from your graveyard."
    );
    supported(VISAGE);
    // Visage of Dread: "Craft with two creatures {5}{B}". P0 has Grizzly Bears on the
    // battlefield and Hill Giant in the graveyard (and a creature card in hand, which
    // can't be used).
    let mut t = TestGame::new(2);
    let visage = t.battlefield(P0, VISAGE);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.graveyard(P0, "Hill Giant");
    t.hand(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 6);
    let from = t.asked().len();
    craft(&mut t, visage, &[bears, giant]);
    // The materials offered: the creature permanent and the creature card in the
    // graveyard (not Visage itself, nor the card in hand).
    let offered = choice_candidates(&t, from, "craft");
    assert_eq!(offered.len(), 1);
    let mut offered = offered[0].clone();
    offered.sort_by_key(|e| format!("{e:?}"));
    let mut expected = vec![Entity::Object(bears), Entity::Object(giant)];
    expected.sort_by_key(|e| format!("{e:?}"));
    assert_eq!(offered, expected);
    assert!(t.in_exile("Grizzly Bears") && t.in_exile("Hill Giant"));
    t.resolve_all();
    // Visage of Dread returned transformed: Dread Osseosaur.
    let osseosaur = one(&t, "Dread Osseosaur");
    assert_eq!(t.obj_now(osseosaur).face, FaceState::Back);
    assert_eq!(t.obj_now(osseosaur).owner, P0);
    assert_eq!(t.pt(osseosaur), (5, 4));
    assert!(t.named_on_battlefield("Visage of Dread").is_empty());

    // Both from the graveyard works too; with only one creature available it can't be
    // activated.
    let mut t = TestGame::new(2);
    let visage = t.battlefield(P0, VISAGE);
    let a = t.graveyard(P0, "Hill Giant");
    let b = t.graveyard(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 6);
    craft(&mut t, visage, &[a, b]);
    t.resolve_all();
    one(&t, "Dread Osseosaur");
    let mut t = TestGame::new(2);
    let visage = t.battlefield(P0, VISAGE);
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 6);
    assert!(!can_activate(&mut t, P0, visage));
    // Nor at instant speed.
    let mut t = TestGame::new(2);
    let visage = t.battlefield(P0, VISAGE);
    t.battlefield(P0, "Grizzly Bears");
    t.graveyard(P0, "Hill Giant");
    t.lands(P0, "Swamp", 6);
    assert!(can_activate(&mut t, P0, visage));
    t.set_step(P0, Step::Upkeep);
    assert!(!can_activate(&mut t, P0, visage));
}

#[test]
fn cards_used_to_craft_stay_used_while_exiled_and_it_stays_on_the_battlefield() {
    cr!("702.167c", "613.4a");
    ruling!(
        "Saheeli's Lattice // Mastercraft Raptor",
        "The back faces of some cards with craft refer to cards \"used to craft\" it. This refers to the cards exiled as part of the cost of the craft ability of the front face. Those cards are considered to be \"used to craft\" that permanent as long as they remain exiled and the permanent remains on the battlefield, even if the permanent's controller changes or some of its characteristics change"
    );
    supported(LATTICE);
    // Saheeli's Lattice: "Craft with one or more Dinosaurs {4}{R}" // Mastercraft Raptor:
    // "Mastercraft Raptor's power is equal to the total power of the exiled cards used to
    // craft it." (*/4). P0 crafts with Colossal Dreadmaw (6/6) and Raging Regisaur (4/4).
    let mut t = TestGame::new(2);
    let lattice = t.battlefield(P0, LATTICE);
    let dreadmaw = t.graveyard(P0, "Colossal Dreadmaw");
    let regisaur = t.battlefield(P0, "Raging Regisaur");
    t.graveyard(P0, "Carnage Tyrant");
    t.lands(P0, "Mountain", 5);
    craft(&mut t, lattice, &[dreadmaw, regisaur]);
    t.resolve_all();
    let raptor = one(&t, "Mastercraft Raptor");
    assert_eq!(t.pt(raptor), (10, 4));
    // Another player gains control of it: the same cards were used to craft it.
    let mut ctx = mtg_engine::eval::Ctx::new(None, P1);
    ctx.targets = vec![vec![Entity::Object(raptor)]];
    t.g.exec(
        &Effect::GainControl {
            what: Sel::Target(0),
            who: PlayerRef::You,
            duration: mtg_engine::ability::Duration::Permanent,
        },
        &mut ctx,
    );
    t.g.recompute();
    assert_eq!(t.obj_now(raptor).controller, P1);
    assert_eq!(t.pt(raptor), (10, 4));
    // Its characteristics change (it becomes a blue Wall in addition to its other types):
    // the same cards were used to craft it.
    run_effect(
        &mut t,
        None,
        P1,
        Effect::Modify {
            what: Sel::All(Filter::Objects(vec![raptor])),
            mods: vec![
                Modification::SetColors(ColorSet::single(Color::Blue)),
                Modification::AddSubtypes(vec!["Wall".into()]),
            ],
            duration: mtg_engine::ability::Duration::EndOfTurn,
        },
        &[],
    );
    assert!(t.obj_now(raptor).chars.has_subtype("Wall"));
    assert_eq!(t.pt(raptor), (10, 4));
    // A card that leaves exile is no longer one used to craft it.
    let dreadmaw_now = t.g.find_in_zone(Zone::Exile, "Colossal Dreadmaw")[0];
    exile_to_graveyard(&mut t, dreadmaw_now);
    t.g.recompute();
    assert_eq!(t.pt(raptor), (4, 4));
    // A new object: once the Raptor leaves the battlefield, nothing was used to craft it.
    let regisaur_now = t.g.find_in_zone(Zone::Exile, "Raging Regisaur")[0];
    assert!(mtg_engine::kw::craft::used_to_craft(
        &t.g,
        Some(raptor),
        regisaur_now
    ));
    crate::r_s02_common::destroy(&mut t, raptor);
    assert!(!mtg_engine::kw::craft::used_to_craft(
        &t.g,
        Some(raptor),
        regisaur_now
    ));
}

#[test]
fn tokens_can_be_materials_but_arent_cards_used_to_craft_it() {
    cr!("702.167a", "702.167c", "111.7", "704.5d");
    ruling!(
        "Jade Seedstones // Jadeheart Attendant",
        "You may exile tokens you control as part of the materials required. However, because they aren't cards and won't stay in exile, any abilities that refer to what you \"used to craft\" the back faces won't refer to anything."
    );
    supported(SEEDSTONES);
    // Jade Seedstones: "Craft with creature {5}{G}{G}" // Jadeheart Attendant: "When this
    // creature enters, you gain life equal to the mana value of the exiled card used to
    // craft it." Crafted with a creature card (Hill Giant, mana value 4): P0 gains 4.
    let mut t = TestGame::new(2);
    let seed = t.battlefield(P0, SEEDSTONES);
    let giant = t.graveyard(P0, "Hill Giant");
    t.lands(P0, "Forest", 7);
    craft(&mut t, seed, &[giant]);
    t.resolve_all();
    one(&t, "Jadeheart Attendant");
    assert_eq!(t.life(P0), 24);
    // Crafted with a creature token: it's exiled (and ceases to exist), and P0 gains no
    // life.
    let mut t = TestGame::new(2);
    let seed = t.battlefield(P0, SEEDSTONES);
    let token = create_token(&mut t, P0, "Soldier");
    t.lands(P0, "Forest", 7);
    craft(&mut t, seed, &[token]);
    t.resolve_all();
    let attendant = one(&t, "Jadeheart Attendant");
    assert_eq!(t.pt(attendant), (7, 7));
    assert!(!t.g.is_live(token) || t.obj_now(token).zone != Zone::Battlefield);
    assert!(t.g.exile.iter().all(|o| !t.obj(*o).is_token()));
    assert_eq!(t.life(P0), 20);

    // Saheeli's Lattice crafted with a Dinosaur token and Colossal Dreadmaw: only the card
    // counts toward Mastercraft Raptor's power.
    supported(LATTICE);
    let mut t = TestGame::new(2);
    let lattice = t.battlefield(P0, LATTICE);
    let dino = create_token(&mut t, P0, "Dinosaur");
    let dreadmaw = t.graveyard(P0, "Colossal Dreadmaw");
    t.lands(P0, "Mountain", 5);
    craft(&mut t, lattice, &[dino, dreadmaw]);
    t.resolve_all();
    let raptor = one(&t, "Mastercraft Raptor");
    assert_eq!(t.pt(raptor), (6, 4));
}

#[test]
fn a_copy_of_a_craft_card_that_isnt_double_faced_stays_in_exile() {
    cr!("702.167a", "712.14a", "707.2");
    ruling!(
        "Tithing Blade // Consuming Sepulcher",
        "If a card that isn't a transforming double-faced card becomes a copy of a card with craft, it'll stay in exile if you activate the craft ability. It won't return to the battlefield."
    );
    supported(BLADE);
    supported("Phyrexian Metamorph");
    // Tithing Blade: "Craft with creature {4}{B}". Phyrexian Metamorph enters as a copy of
    // it and activates its craft ability, exiling Grizzly Bears.
    let mut t = TestGame::new(2);
    let blade = t.battlefield(P0, BLADE);
    t.answer_choose(P0, &[Entity::Object(blade)]);
    let m = t.enter(P0, "Phyrexian Metamorph");
    t.resolve_all();
    let m = t.g.current(m);
    assert_eq!(t.obj_now(m).chars.name, "Tithing Blade");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 5);
    craft(&mut t, m, &[bears]);
    t.resolve_all();
    // The Metamorph card stays in exile; the Bears were exiled too.
    assert!(t.in_exile("Phyrexian Metamorph"));
    assert!(t.in_exile("Grizzly Bears"));
    assert!(t.named_on_battlefield("Consuming Sepulcher").is_empty());
    assert!(t.named_on_battlefield("Phyrexian Metamorph").is_empty());
    // The real Tithing Blade crafts normally.
    let giant = t.graveyard(P0, "Hill Giant");
    t.lands(P0, "Swamp", 5);
    craft(&mut t, blade, &[giant]);
    t.resolve_all();
    let sepulcher = one(&t, "Consuming Sepulcher");
    assert_eq!(t.obj_now(sepulcher).face, FaceState::Back);
}
