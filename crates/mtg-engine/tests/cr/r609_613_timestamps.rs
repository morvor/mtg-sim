//! CR 613.7 (timestamp order) and CR 613.9 (one effect overriding another).

use crate::r609_common::*;
use mtg_engine::ability::*;
use mtg_engine::card::{FaceDef, Layout};
use mtg_engine::events::MoveCause;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::object::*;
use mtg_engine::replacement::{EtbInfo, MoveEv};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn gains_flying() -> Vec<Modification> {
    vec![Modification::AddKeyword(Keyword::new(KeywordKind::Flying))]
}

fn loses_flying() -> Vec<Modification> {
    vec![Modification::RemoveKeyword(KeywordKind::Flying)]
}

/// An enchantment: "All creatures have flying" / "All creatures lose flying".
fn flying_anthem(give: bool) -> CardDef {
    permanent(
        if give {
            "Sky Anthem"
        } else {
            "Grounding Field"
        },
        &[CardType::Enchantment],
        vec![continuous(
            Filter::creature(),
            if give { gains_flying() } else { loses_flying() },
        )],
    )
}

fn aura(name: &str, mods: Vec<Modification>) -> CardDef {
    let mut enchant = Keyword::new(KeywordKind::Enchant);
    enchant.filter = Some(Filter::Permanent);
    let mut d = permanent(
        name,
        &[CardType::Enchantment],
        vec![
            AbilityDef::new(AbilityKind::Keyword(enchant), "Enchant permanent"),
            continuous(Filter::AttachedToSource, mods),
        ],
    );
    d.faces[0].chars.subtypes.push("Aura".into());
    d
}

fn equipment(name: &str, mods: Vec<Modification>) -> CardDef {
    let mut d = permanent(
        name,
        &[CardType::Artifact],
        vec![continuous(Filter::AttachedToSource, mods)],
    );
    d.faces[0].chars.subtypes.push("Equipment".into());
    d
}

fn move_to(t: &mut TestGame, id: ObjectId, to: Zone, controller: PlayerId) -> ObjectId {
    t.g.move_object_ev(MoveEv {
        obj: id,
        to,
        pos: LibraryPosition::Top,
        cause: MoveCause::Effect,
        by: Some(controller),
        etb: EtbInfo {
            controller: Some(controller),
            ..Default::default()
        },
        source: None,
    })
    .expect("moved")
}

#[test]
fn rune_of_flight_and_colossus_hammer() {
    // CR 613.7a example: an ability granted to the Equipment by the Aura has the Aura's
    // (later) timestamp; when the Equipment gets a new timestamp by becoming attached,
    // its abilities keep their relative order, so the granted "has flying" still applies
    // after the Equipment's own "loses flying".
    cr!("613.7a", "613.7e");
    let mut t = TestGame::new(2);
    let bird = t.custom(
        P0,
        creature_with("Bird", 1, 1, &[], vec![keyword(KeywordKind::Flying)]),
        Zone::Battlefield,
    );
    let other = t.custom(P0, creature("Walker", 2, 2, &[]), Zone::Battlefield);
    let mut hammer_mods = vec![pt(10, 10)];
    hammer_mods.extend(loses_flying());
    let hammer = t.custom(
        P0,
        equipment("Colossus Hammer", hammer_mods),
        Zone::Battlefield,
    );
    assert!(t.g.attach(hammer, Entity::Object(bird)));
    t.recompute();
    assert_eq!(t.pt(bird), (11, 11));
    assert!(!has_kw(&t, bird, KeywordKind::Flying));
    // Rune of Flight: enchanted Equipment has "Equipped creature has flying."
    let granted = continuous(Filter::AttachedToSource, gains_flying());
    let rune = t.custom(
        P0,
        aura("Rune of Flight", vec![Modification::AddAbility(granted)]),
        Zone::Battlefield,
    );
    assert!(t.g.attach(rune, Entity::Object(hammer)));
    t.recompute();
    assert!(has_kw(&t, bird, KeywordKind::Flying));
    // The Hammer becomes attached to another creature: new timestamp for the Hammer
    // (613.7e), but the granted ability still applies after "loses flying".
    assert!(t.g.attach(hammer, Entity::Object(other)));
    t.recompute();
    assert!(t.obj_now(hammer).timestamp > t.obj_now(rune).timestamp);
    assert_eq!(t.pt(other), (12, 12));
    assert!(has_kw(&t, other, KeywordKind::Flying));
    assert!(has_kw(&t, bird, KeywordKind::Flying));
    assert_eq!(t.pt(bird), (1, 1));
}

#[test]
fn resolved_effects_are_ordered_by_creation() {
    // CR 613.7b: a continuous effect from a resolving spell gets its timestamp when it's
    // created; CR 613.9: of "has flying" and "loses flying", the later one wins.
    cr!("613.7b", "613.9", "613.7");
    let mut t = TestGame::new(2);
    let c = t.custom(P0, creature("Bear", 2, 2, &[]), Zone::Battlefield);
    modify_target(&mut t, P0, c, gains_flying());
    modify_target(&mut t, P0, c, loses_flying());
    assert!(!has_kw(&t, c, KeywordKind::Flying));
    modify_target(&mut t, P0, c, gains_flying());
    assert!(has_kw(&t, c, KeywordKind::Flying));
}

#[test]
fn keyword_counters_get_new_timestamps() {
    // CR 613.7c: each counter gets a timestamp as it's put on; putting another counter of
    // the same kind gives all of them the new timestamp. Keyword counters apply in layer 6
    // in timestamp order with other effects (CR 613.1f).
    cr!("613.7c", "613.1f");
    let mut t = TestGame::new(2);
    let c = t.custom(P0, creature("Bear", 2, 2, &[]), Zone::Battlefield);
    put_counters(&mut t, c, "flying", 1);
    assert!(has_kw(&t, c, KeywordKind::Flying));
    modify_target(&mut t, P0, c, loses_flying());
    assert!(!has_kw(&t, c, KeywordKind::Flying));
    put_counters(&mut t, c, "flying", 1);
    assert_eq!(t.counters(c, "flying"), 2);
    assert!(has_kw(&t, c, KeywordKind::Flying));
}

#[test]
fn objects_get_timestamps_when_entering_a_zone() {
    // CR 613.7d: an object receives a timestamp when it enters a zone. Two conflicting
    // enchantments apply in the order they entered; when the older one leaves and comes
    // back, it's the newer one.
    cr!("613.7d", "613.9");
    let mut t = TestGame::new(2);
    let bear = t.custom(P0, creature("Bear", 2, 2, &[]), Zone::Battlefield);
    let give = t.custom(P0, flying_anthem(true), Zone::Battlefield);
    t.custom(P0, flying_anthem(false), Zone::Battlefield);
    t.recompute();
    assert!(!has_kw(&t, bear, KeywordKind::Flying));
    let exiled = move_to(&mut t, give, Zone::Exile, P0);
    move_to(&mut t, exiled, Zone::Battlefield, P0);
    assert!(has_kw(&t, bear, KeywordKind::Flying));
}

#[test]
fn auras_get_new_timestamps_when_attached() {
    // CR 613.7e: an Aura receives a new timestamp each time it becomes attached; CR 613.9
    // example: "Enchanted creature has flying" vs. "Enchanted creature loses flying".
    cr!("613.7e", "613.9");
    let mut t = TestGame::new(2);
    let a = t.custom(P0, creature("Bear", 2, 2, &[]), Zone::Battlefield);
    let b = t.custom(P0, creature("Other Bear", 2, 2, &[]), Zone::Battlefield);
    let wings = t.custom(P0, aura("Wings", gains_flying()), Zone::Battlefield);
    let clip = t.custom(P0, aura("Clipped", loses_flying()), Zone::Battlefield);
    assert!(t.g.attach(wings, Entity::Object(a)));
    assert!(t.g.attach(clip, Entity::Object(a)));
    t.recompute();
    assert!(!has_kw(&t, a, KeywordKind::Flying));
    // Move "Wings" away and back: it's now the later effect.
    assert!(t.g.attach(wings, Entity::Object(b)));
    assert!(t.g.attach(wings, Entity::Object(a)));
    t.recompute();
    assert!(has_kw(&t, a, KeywordKind::Flying));
}

#[test]
fn turning_face_up_or_down_gives_a_new_timestamp() {
    // CR 613.7f: a permanent receives a new timestamp each time it turns face up or face
    // down, so its static abilities' effects become the newest.
    cr!("613.7f", "613.2b");
    let mut t = TestGame::new(2);
    let bear = t.custom(P0, creature("Bear", 2, 2, &[]), Zone::Battlefield);
    let wizard = t.custom(
        P0,
        creature_with(
            "Sky Wizard",
            1,
            1,
            &[Color::Blue],
            vec![continuous(Filter::creature().you_control(), gains_flying())],
        ),
        Zone::Battlefield,
    );
    t.custom(P1, flying_anthem(false), Zone::Battlefield);
    t.recompute();
    assert!(!has_kw(&t, bear, KeywordKind::Flying));
    let ts0 = t.obj_now(wizard).timestamp;
    assert!(mtg_engine::facedown::turn_face_down(&mut t.g, wizard));
    t.recompute();
    // Face down (layer 1b): a nameless 2/2 without abilities.
    assert_eq!(t.pt(wizard), (2, 2));
    assert_eq!(t.obj_now(wizard).chars.name, "");
    let ts1 = t.obj_now(wizard).timestamp;
    assert!(ts1 > ts0);
    assert!(mtg_engine::facedown::turn_face_up(&mut t.g, wizard, false));
    t.recompute();
    assert!(t.obj_now(wizard).timestamp > ts1);
    assert!(has_kw(&t, bear, KeywordKind::Flying));
}

#[test]
fn transforming_gives_a_new_timestamp() {
    // CR 613.7g: a double-faced permanent receives a new timestamp each time it
    // transforms.
    cr!("613.7g");
    let mut t = TestGame::new(2);
    let bear = t.custom(P0, creature("Bear", 2, 2, &[]), Zone::Battlefield);
    let mut front = chars("Sky Initiate");
    front.card_types = CardTypeSet::single(CardType::Creature);
    front.power = Some(1);
    front.toughness = Some(1);
    front.abilities = vec![continuous(Filter::creature().you_control(), gains_flying())];
    let mut back = front.clone();
    back.name = "Sky Master".into();
    back.power = Some(3);
    back.toughness = Some(3);
    let mut def = CardDef::custom(front);
    def.layout = Layout::Transform;
    def.faces.push(FaceDef {
        chars: back,
        unsupported: vec![],
        star_power: false,
        star_toughness: false,
    });
    let dfc = t.custom(P0, def, Zone::Battlefield);
    t.custom(P1, flying_anthem(false), Zone::Battlefield);
    t.recompute();
    assert!(!has_kw(&t, bear, KeywordKind::Flying));
    assert!(mtg_engine::dfc::transform(&mut t.g, dfc));
    t.recompute();
    assert_eq!(t.pt(dfc), (3, 3));
    assert!(has_kw(&t, bear, KeywordKind::Flying));
}

#[test]
fn simultaneous_entries_are_timestamped_in_apnap_order() {
    // CR 613.7m: objects entering simultaneously get timestamps in APNAP order: the
    // active player's first (in the order of their choice), then each other player's.
    cr!("613.7m");
    for active in [P0, P1] {
        let mut t = TestGame::new(2);
        t.set_step(active, mtg_engine::turn::Step::PrecombatMain);
        let bear = t.custom(P0, creature("Bear", 2, 2, &[]), Zone::Battlefield);
        let give = t.custom(P0, flying_anthem(true), Zone::Graveyard(P0));
        let take = t.custom(P1, flying_anthem(false), Zone::Graveyard(P1));
        let moves = vec![(take, P1), (give, P0)]
            .into_iter()
            .map(|(o, p)| MoveEv {
                obj: o,
                to: Zone::Battlefield,
                pos: LibraryPosition::Top,
                cause: MoveCause::Effect,
                by: Some(p),
                etb: EtbInfo {
                    controller: Some(p),
                    ..Default::default()
                },
                source: None,
            })
            .collect();
        t.g.move_objects(moves);
        t.recompute();
        // The non-active player's enchantment is newer and wins.
        let expect_flying = active == P1;
        assert_eq!(has_kw(&t, bear, KeywordKind::Flying), expect_flying);
    }
    // The active player orders their own objects.
    for order in [vec![0usize, 1], vec![1, 0]] {
        let mut t = TestGame::new(2);
        let bear = t.custom(P0, creature("Bear", 2, 2, &[]), Zone::Battlefield);
        let give = t.custom(P0, flying_anthem(true), Zone::Graveyard(P0));
        let take = t.custom(P0, flying_anthem(false), Zone::Graveyard(P0));
        t.answer(P0, DecisionKind::Order, Answer::Indices(order.clone()));
        let moves = vec![give, take]
            .into_iter()
            .map(|o| MoveEv {
                obj: o,
                to: Zone::Battlefield,
                pos: LibraryPosition::Top,
                cause: MoveCause::Effect,
                by: Some(P0),
                etb: EtbInfo {
                    controller: Some(P0),
                    ..Default::default()
                },
                source: None,
            })
            .collect();
        t.g.move_objects(moves);
        t.recompute();
        // The object ordered last is newest.
        let give_last = order == vec![1, 0];
        assert_eq!(has_kw(&t, bear, KeywordKind::Flying), give_last);
    }
}

#[test]
fn enchanted_creature_is_white_gets_white_bonus() {
    // CR 613.9 example: "White creatures get +1/+1" and "Enchanted creature is white":
    // the creature gets +1/+1 regardless of its previous color.
    cr!("613.9");
    let mut t = TestGame::new(2);
    let c = t.custom(
        P0,
        creature("Green Bear", 2, 2, &[Color::Green]),
        Zone::Battlefield,
    );
    let aura_id = t.custom(
        P0,
        aura(
            "Whitewash",
            vec![Modification::SetColors(colors(&[Color::White]))],
        ),
        Zone::Battlefield,
    );
    // The anthem is older than the Aura.
    let _ = aura_id;
    t.custom(
        P0,
        permanent(
            "White Anthem",
            &[CardType::Enchantment],
            vec![continuous(
                Filter::and(vec![Filter::creature(), Filter::Color(Color::White)]),
                vec![pt(1, 1)],
            )],
        ),
        Zone::Battlefield,
    );
    assert!(t.g.attach(aura_id, Entity::Object(c)));
    t.recompute();
    assert_eq!(t.pt(c), (3, 3));
}
