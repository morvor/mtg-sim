//! CR 712: double-faced cards (nonmodal and modal; meld cards are in r712_meld_cards.rs).

use super::r709_common::*;
use mtg_engine::ability::*;
use mtg_engine::card::{card, Layout};
use mtg_engine::events::MoveCause;
use mtg_engine::facedown;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Kessig Prowler ({G} 2/1 Werewolf Horror, "{4}{G}: Transform this creature.") //
/// Sinuous Predator (4/4 colorless Eldrazi Werewolf).
const PROWLER: &str = "Kessig Prowler // Sinuous Predator";
/// Lunarch Veteran ({W} 1/1, disturb {1}{W}) // Luminous Phantom (1/1 Spirit Cleric).
const VETERAN: &str = "Lunarch Veteran // Luminous Phantom";
/// Kazandu Mammoth ({1}{G}{G} 3/3 creature) // Kazandu Valley (land).
const MAMMOTH: &str = "Kazandu Mammoth // Kazandu Valley";
/// Halvar, God of Battle ({2}{W}{W} 4/4 God) // Sword of the Realms ({1}{W} Equipment).
const HALVAR: &str = "Halvar, God of Battle // Sword of the Realms";
/// Sea Gate Restoration ({4}{U}{U}{U} sorcery) // Sea Gate, Reborn (land).
const SEA_GATE: &str = "Sea Gate Restoration // Sea Gate, Reborn";

fn transform(t: &mut TestGame, id: ObjectId) {
    run_effect(
        t,
        P0,
        None,
        &[Entity::Object(id)],
        Effect::Transform {
            what: Sel::Target(0),
        },
    );
}

fn value(t: &mut TestGame, p: PlayerId, v: Value) -> i64 {
    t.g.recompute();
    t.g.eval_value(&v, &mtg_engine::eval::Ctx::new(None, p))
}

#[test]
fn there_are_three_kinds_of_double_faced_cards() {
    cr!("712.1");
    for (name, layout) in [
        (PROWLER, Layout::Transform),
        (MAMMOTH, Layout::ModalDfc),
        ("Graf Rats", Layout::Meld),
    ] {
        let d = card(name);
        assert_eq!(d.layout, layout, "{name}");
        assert!(d.layout.is_double_faced(), "{name}");
    }
    // Split, flip and adventurer cards have a normal card back.
    for name in [
        "Fire // Ice",
        "Akki Lavarunner // Tok-Tok, Volcano Born",
        "Bonecrusher Giant // Stomp",
    ] {
        assert!(!card(name).layout.is_double_faced(), "{name}");
    }
    // Each kind behaves differently: a nonmodal one transforms, a meld card doesn't.
    let mut t = TestGame::new(2);
    let prowler = t.battlefield(P0, PROWLER);
    let rats = t.battlefield(P0, "Graf Rats");
    transform(&mut t, prowler);
    transform(&mut t, rats);
    assert_eq!(t.obj(prowler).face, FaceState::Back);
    assert_eq!(t.obj(rats).face, FaceState::Front);
}

#[test]
fn a_nonmodal_double_faced_card_transforms_with_its_own_abilities() {
    cr!("712.2", "712.8", "712.18");
    supported(PROWLER);
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let prowler = t.battlefield(P0, PROWLER);
    // An effect that applied to the front face: +2/+2 until end of turn.
    run_effect(
        &mut t,
        P0,
        None,
        &[Entity::Object(prowler)],
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::ModifyPT(Value::c(2), Value::c(2))],
            duration: Duration::EndOfTurn,
        },
    );
    assert_eq!(t.pt(prowler), (4, 3));
    let front = t.obj(prowler).chars.clone();
    // "{4}{G}: Transform this creature."
    t.lands(P0, "Forest", 5);
    t.activate(P0, prowler, 0, &[]).unwrap();
    t.resolve_all();
    // It turned over: each face has its own characteristics.
    let back = t.obj(prowler).chars.clone();
    assert_eq!(front.name, "Kessig Prowler");
    assert_eq!(back.name, "Sinuous Predator");
    assert_eq!(front.colors, ColorSet::single(Color::Green));
    assert_eq!(back.colors, ColorSet::NONE);
    assert!(front.has_subtype("Horror") && !back.has_subtype("Horror"));
    assert!(back.has_subtype("Eldrazi"));
    // It's the same object, and the +2/+2 still applies: 4/4 + 2/2.
    assert!(t.g.is_live(prowler));
    assert_eq!(t.pt(prowler), (6, 6));
}

#[test]
fn a_front_face_doesnt_have_its_back_faces_power_and_toughness() {
    cr!("712.2c");
    // Westvale Abbey is a land; the 9/7 printed in gray is Ormendahl's.
    let mut t = TestGame::new(2);
    let hand = t.hand(P0, "Westvale Abbey // Ormendahl, Profane Prince");
    let abbey = t.battlefield(P0, "Westvale Abbey // Ormendahl, Profane Prince");
    t.g.recompute();
    for id in [hand, abbey] {
        let c = &t.obj(id).chars;
        assert_eq!(c.name, "Westvale Abbey");
        assert_eq!((c.power, c.toughness), (None, None));
        assert!(!c.is_creature());
    }
}

#[test]
fn a_modal_double_faced_cards_faces_are_independent() {
    cr!("712.3", "712.3c");
    let mut t = TestGame::new(2);
    let mammoth = t.hand(P0, MAMMOTH);
    t.g.recompute();
    // In the hand it's only a creature card; the hint bar about the land face isn't a
    // characteristic.
    let c = t.obj(mammoth).chars.clone();
    assert!(c.is_creature() && !c.is_land());
    assert!(!c.has_name("Kazandu Valley"));
    let ctx = mtg_engine::eval::Ctx::new(None, P0);
    assert!(!t.g.matches(mammoth, &Filter::Type(CardType::Land), &ctx));
    // Played as its land face, it's only that land.
    t.set_step(P0, Step::PrecombatMain);
    t.play_land(P0, mammoth).unwrap();
    let valley = t.g.current(mammoth);
    let c = &t.obj(valley).chars;
    assert_eq!(c.name, "Kazandu Valley");
    assert!(c.is_land() && !c.is_creature());
    assert_eq!(t.pt(valley), (0, 0));
    // An ability of a modal double-faced card may transform it (Monica Rambeau's).
    let monica = t.battlefield(P0, "Monica Rambeau // Photon, Living Light");
    transform(&mut t, monica);
    assert_eq!(t.obj(monica).chars.name, "Photon, Living Light");
}

#[test]
fn outside_the_battlefield_and_stack_a_double_faced_card_has_only_its_front_face() {
    cr!("712.8a");
    ruling!(
        "Azusa's Many Journeys // Likeness of the Seeker",
        "In every zone other than the battlefield, consider only the characteristics of its front face"
    );
    let mut t = TestGame::new(2);
    let gy = t.graveyard(P0, SEA_GATE);
    let ex = t.exile(P0, PROWLER);
    let lib = t.library_top(P0, PROWLER);
    t.g.recompute();
    // The sorcery face: no land card in the graveyard.
    assert!(t.obj(gy).chars.is(CardType::Sorcery));
    assert_eq!(
        value(
            &mut t,
            P0,
            Value::CardsInGraveyard(PlayerRef::You, Filter::Type(CardType::Land))
        ),
        0
    );
    for id in [ex, lib] {
        assert_eq!(t.obj(id).chars.name, "Kessig Prowler");
        assert_eq!(
            (t.obj(id).chars.power, t.obj(id).chars.toughness),
            (Some(2), Some(1))
        );
    }
    // Outside the game too (a sideboard).
    let side = t.g.add_to_sideboard(P0, vec![card(PROWLER)]);
    t.g.recompute();
    assert_eq!(t.obj(side[0]).chars.name, "Kessig Prowler");
}

#[test]
fn a_modal_double_faced_spell_or_permanent_has_the_face_that_is_up() {
    cr!("712.8f", "712.11b", "712.13");
    ruling!(
        "Kazandu Mammoth // Kazandu Valley",
        "On the stack and battlefield, consider whichever face is up"
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let halvar = t.hand(P0, HALVAR);
    // The player chooses which face to cast: the front or the back.
    let faces: Vec<FaceState> =
        t.g.cast_options(P0, halvar)
            .into_iter()
            .map(|o| o.face)
            .collect();
    assert_eq!(faces, vec![FaceState::Front, FaceState::Back]);
    // Sword of the Realms ({1}{W}): only that face is evaluated and put on the stack.
    t.lands(P0, "Plains", 2);
    let spell = t.cast(P0, halvar).method(CastMethod::Half(1)).go();
    let c = t.obj(spell).chars.clone();
    assert_eq!(c.name, "Sword of the Realms");
    assert!(c.is(CardType::Artifact) && !c.is_creature());
    assert_eq!(mv(&mut t, spell), 2);
    // It resolves and enters with the same face up.
    t.resolve_all();
    let sword = t.g.current(spell);
    assert_eq!(t.obj(sword).face, FaceState::Back);
    assert_eq!(t.obj(sword).chars.name, "Sword of the Realms");
    assert_eq!(mv(&mut t, sword), 2);
}

#[test]
fn a_permanent_doesnt_transform_or_convert_into_an_instant_or_sorcery_face() {
    cr!("712.10");
    let mut t = TestGame::new(2);
    // Invasion of Kylem's back face, Valor's Reach Tag Team, is a sorcery.
    let kylem = t.battlefield(P0, "Invasion of Kylem // Valor's Reach Tag Team");
    transform(&mut t, kylem);
    assert_eq!(t.obj(kylem).face, FaceState::Front);
    assert_eq!(t.obj(kylem).chars.name, "Invasion of Kylem");
    run_effect(
        &mut t,
        P0,
        None,
        &[Entity::Object(kylem)],
        Effect::KeywordAction {
            action: KeywordAction::Convert,
            who: PlayerRef::You,
            what: Sel::Target(0),
            n: Value::c(1),
        },
    );
    assert_eq!(t.obj(kylem).face, FaceState::Front);
    assert_eq!(t.zone(kylem), Zone::Battlefield);
    // A creature back face: it transforms.
    let prowler = t.battlefield(P0, PROWLER);
    transform(&mut t, prowler);
    assert_eq!(t.obj(prowler).face, FaceState::Back);
}

#[test]
fn a_double_faced_spell_is_cast_with_its_front_face_up_by_default() {
    cr!("712.11", "712.13");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Forest", 1);
    let prowler = t.hand(P0, PROWLER);
    let spell = t.cast(P0, prowler).go();
    assert_eq!(t.obj(spell).face, FaceState::Front);
    assert_eq!(t.obj(spell).chars.name, "Kessig Prowler");
    t.resolve_all();
    let perm = t.g.current(spell);
    assert_eq!(t.obj(perm).face, FaceState::Front);
    assert_eq!(t.obj(perm).chars.name, "Kessig Prowler");
}

#[test]
fn a_spell_cast_transformed_has_its_back_face_up() {
    cr!("712.11a", "712.11d", "712.13", "712.8c");
    ruling!(
        "Lunarch Veteran // Luminous Phantom",
        "the card is put onto the stack with its back face up"
    );
    ruling!(
        "Lunarch Veteran // Luminous Phantom",
        "A spell cast this way enters the battlefield with its back face up"
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Plains", 2);
    let vet = t.graveyard(P0, VETERAN);
    // Disturb is an ability of the front face; the back face doesn't have it, yet it's
    // considered to determine whether the spell can be cast.
    let back = &card(VETERAN).faces[1].chars;
    assert!(!back.has_keyword(KeywordKind::Disturb));
    let spell = t
        .cast(P0, vet)
        .method(CastMethod::Keyword(KeywordKind::Disturb))
        .go();
    assert_eq!(t.obj(spell).face, FaceState::Back);
    assert_eq!(t.obj(spell).chars.name, "Luminous Phantom");
    // Its mana value is its front face's.
    assert_eq!(mv(&mut t, spell), 1);
    t.resolve_all();
    let phantom = t.g.current(spell);
    assert_eq!(t.obj(phantom).face, FaceState::Back);
    assert_eq!(t.obj(phantom).chars.name, "Luminous Phantom");
}

/// An enchantment: "Battles and creatures you control enter transformed."
fn enters_transformed() -> CardDef {
    CB::new("Warp Gate")
        .enchantment()
        .cost("{0}")
        .ability(stat(StaticEffect::Replacement(ReplacementDef {
            event: ReplacementEvent::EntersBattlefield(Filter::and(vec![
                Filter::Or(vec![
                    Filter::Type(CardType::Battle),
                    Filter::Type(CardType::Creature),
                ]),
                Filter::ControlledBy(PlayerRel::You),
            ])),
            action: ReplacementAction::EnterTransformed,
            self_replacement: false,
            optional: false,
        })))
        .build()
}

#[test]
fn a_spell_entering_transformed_with_an_instant_or_sorcery_back_face_goes_to_the_graveyard() {
    cr!("712.13a");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.custom(P0, enters_transformed(), Zone::Battlefield);
    // A creature spell with its front face up on the stack enters transformed.
    t.lands(P0, "Forest", 1);
    let prowler = t.hand(P0, PROWLER);
    let spell = t.cast(P0, prowler).go();
    t.resolve_all();
    let perm = t.g.current(spell);
    assert_eq!(t.obj(perm).zone, Zone::Battlefield);
    assert_eq!(t.obj(perm).chars.name, "Sinuous Predator");
    // A creature spell that isn't represented by a double-faced card (nor a meld card,
    // which can't be transformed) can't enter transformed: it just enters.
    for (name, land) in [("Grizzly Bears", "Forest"), ("Graf Rats", "Swamp")] {
        t.lands(P0, land, 2);
        let c = t.hand(P0, name);
        let spell = t.cast(P0, c).go();
        t.resolve_all();
        let perm = t.g.current(spell);
        assert_eq!(t.obj(perm).zone, Zone::Battlefield, "{name}");
        assert_eq!(t.obj(perm).face, FaceState::Front, "{name}");
        assert_eq!(t.obj(perm).chars.name, name);
    }
    // Invasion of Kylem's back face is a sorcery: it doesn't enter the battlefield and is
    // put into its owner's graveyard instead.
    t.lands(P0, "Mountain", 3);
    t.lands(P0, "Plains", 1);
    let kylem = t.hand(P0, "Invasion of Kylem // Valor's Reach Tag Team");
    t.cast(P0, kylem).go();
    t.resolve_all();
    assert!(t.named_on_battlefield("Invasion of Kylem").is_empty());
    assert!(t.named_on_battlefield("Valor's Reach Tag Team").is_empty());
    assert!(t.in_graveyard(P0, "Invasion of Kylem"));
    // Without that effect, it enters as a battle.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 3);
    t.lands(P0, "Plains", 1);
    let kylem = t.hand(P0, "Invasion of Kylem // Valor's Reach Tag Team");
    t.cast(P0, kylem).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Invasion of Kylem").len(), 1);
}

#[test]
fn a_double_faced_card_put_onto_the_battlefield_enters_front_face_up() {
    cr!("712.14");
    let mut t = TestGame::new(2);
    let prowler = t.graveyard(P0, PROWLER);
    run_effect(
        &mut t,
        P0,
        None,
        &[Entity::Object(prowler)],
        Effect::Move {
            what: Sel::Target(0),
            to: Destination::battlefield(),
        },
    );
    let perm = t.g.current(prowler);
    assert_eq!(t.obj(perm).zone, Zone::Battlefield);
    assert_eq!(t.obj(perm).face, FaceState::Front);
    assert_eq!(t.obj(perm).chars.name, "Kessig Prowler");
}

#[test]
fn a_card_put_onto_the_battlefield_transformed() {
    cr!("712.14a");
    ruling!(
        "Azusa's Many Journeys // Likeness of the Seeker",
        "if a single-faced card is a copy of Azusa's Many Journeys, the chapter III ability will cause it to be exiled and then remain in exile"
    );
    supported("Azusa's Many Journeys // Likeness of the Seeker");
    // Chapter III: "Exile this Saga, then return it to the battlefield transformed under
    // your control."
    let mut t = TestGame::new(2);
    let saga = t.enter(P0, "Azusa's Many Journeys // Likeness of the Seeker");
    t.resolve_all();
    t.g.add_counters(Entity::Object(saga), counters::LORE, 2, None);
    t.g.flush_events();
    t.resolve_all();
    let seeker = t.named_on_battlefield("Likeness of the Seeker");
    assert_eq!(seeker.len(), 1);
    assert_eq!(t.obj(seeker[0]).face, FaceState::Back);
    // A single-faced copy of it (Copy Enchantment) is exiled and stays in exile.
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P1, "Azusa's Many Journeys // Likeness of the Seeker");
    t.answer(P0, DecisionKind::YesNo, Answer::Bool(true));
    t.answer_choose(P0, &[Entity::Object(saga)]);
    let copy = t.enter(P0, "Copy Enchantment");
    t.resolve_all();
    assert_eq!(t.obj(copy).chars.name, "Azusa's Many Journeys");
    t.g.add_counters(Entity::Object(copy), counters::LORE, 2, None);
    t.g.flush_events();
    t.resolve_all();
    assert!(t.named_on_battlefield("Likeness of the Seeker").is_empty());
    assert!(t.in_exile("Copy Enchantment"));
}

#[test]
fn a_modal_double_faced_card_whose_front_face_isnt_a_permanent_stays_where_it_is() {
    cr!("712.14b");
    ruling!(
        "Kazandu Mammoth // Kazandu Valley",
        "If that front face can't be put onto the battlefield, it doesn't enter the battlefield"
    );
    let mut t = TestGame::new(2);
    let gate = t.graveyard(P0, SEA_GATE);
    run_effect(
        &mut t,
        P0,
        None,
        &[Entity::Object(gate)],
        Effect::Move {
            what: Sel::Target(0),
            to: Destination::battlefield(),
        },
    );
    assert!(t.g.is_live(gate));
    assert_eq!(t.zone(gate), Zone::Graveyard(P0));
    assert!(t.g.battlefield.is_empty());
}

#[test]
fn a_face_down_double_faced_card_has_the_face_down_characteristics() {
    cr!("712.15", "712.15a");
    supported("Soul Summons");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    // Soul Summons: "Manifest the top card of your library."
    let card_id = t.library_top(P0, PROWLER);
    t.lands(P0, "Plains", 2);
    let spell = t.hand(P0, "Soul Summons");
    t.cast(P0, spell).go();
    t.resolve_all();
    let m = t.g.current(card_id);
    let c = t.obj(m).chars.clone();
    assert!(t.obj(m).face_down);
    assert!(c.name.is_empty());
    assert_eq!((c.power, c.toughness), (Some(2), Some(2)));
    // While face down it can't transform.
    transform(&mut t, m);
    assert!(t.obj(m).face_down);
    assert_eq!(t.obj(m).face, FaceState::Front);
    assert_eq!(t.pt(m), (2, 2));
    // Turned face up, it has its front face up.
    assert!(facedown::turn_face_up(&mut t.g, m, false));
    t.g.recompute();
    assert_eq!(t.obj(m).face, FaceState::Front);
    assert_eq!(t.obj(m).chars.name, "Kessig Prowler");
}

#[test]
fn double_faced_permanents_cant_be_turned_face_down() {
    cr!("712.16");
    let mut t = TestGame::new(2);
    let prowler = t.battlefield(P0, PROWLER);
    let bears = t.battlefield(P0, "Grizzly Bears");
    run_effect(
        &mut t,
        P0,
        None,
        &[Entity::Object(prowler), Entity::Object(bears)],
        Effect::TurnFaceDown {
            what: Sel::Target(0),
        },
    );
    // Nothing happens to the double-faced permanent; the single-faced one turns face down.
    assert!(!t.obj(prowler).face_down);
    assert_eq!(t.obj(prowler).chars.name, "Kessig Prowler");
    assert_eq!(t.pt(prowler), (2, 1));
    assert!(t.obj(bears).face_down);
    // Also after it transformed, and a meld card.
    transform(&mut t, prowler);
    let rats = t.battlefield(P0, "Graf Rats");
    run_effect(
        &mut t,
        P0,
        None,
        &[Entity::Object(prowler), Entity::Object(rats)],
        Effect::TurnFaceDown {
            what: Sel::Target(0),
        },
    );
    assert!(!t.obj(prowler).face_down);
    assert_eq!(t.obj(prowler).chars.name, "Sinuous Predator");
    assert!(!t.obj(rats).face_down);
}

#[test]
fn a_double_faced_card_exiled_face_down_stays_hidden() {
    cr!("712.17");
    let mut t = TestGame::new(2);
    let prowler = t.hand(P0, PROWLER);
    run_effect(
        &mut t,
        P0,
        None,
        &[Entity::Object(prowler)],
        Effect::Exile {
            what: Sel::Target(0),
            face_down: true,
            link: false,
        },
    );
    let ex = t.g.current(prowler);
    assert_eq!(t.obj(ex).zone, Zone::Exile);
    assert!(t.obj(ex).face_down);
    assert!(t.obj(ex).chars.name.is_empty());
    // Its owner doesn't get to see it just because it's double-faced, nor its opponent.
    assert!(!facedown::can_look_at(&t.g, P1, ex));
}

#[test]
fn either_face_name_may_be_chosen_but_not_both() {
    cr!("712.19");
    ruling!(
        "Kazandu Mammoth // Kazandu Valley",
        "If an effect instructs a player to choose a card name, the name of either face may be chosen"
    );
    let mut t = TestGame::new(2);
    // Meddling Mage: "Choose a nonland card name. Spells with the chosen name can't be
    // cast."
    for (named, ok) in [
        ("Sword of the Realms", true),
        ("Halvar, God of Battle", true),
        ("Halvar, God of Battle // Sword of the Realms", false),
        ("Chittering Host", true),
    ] {
        name_card(&mut t, P1, named);
        let mage = t.enter(P1, "Meddling Mage");
        let chosen = t
            .obj_now(mage)
            .choices
            .card_name
            .clone()
            .unwrap_or_default();
        assert_eq!(chosen == named, ok, "{named}");
    }
    // Naming the back face stops casting that face, not the other.
    let mut t = TestGame::new(2);
    name_card(&mut t, P1, "Sword of the Realms");
    t.enter(P1, "Meddling Mage");
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Plains", 4);
    let halvar = t.hand(P0, HALVAR);
    assert!(t
        .cast(P0, halvar)
        .method(CastMethod::Half(1))
        .try_go()
        .is_err());
    let halvar = t.g.current(halvar);
    assert!(t.cast(P0, halvar).try_go().is_ok());
}

#[test]
fn an_as_this_transforms_ability_applies_while_it_transforms() {
    cr!("712.20");
    // Sephiroth, One-Winged Angel: "Super Nova — As this creature transforms into
    // Sephiroth, One-Winged Angel, you get an emblem with 'Whenever a creature dies,
    // target opponent loses 1 life and you gain 1 life.'"
    let mut t = TestGame::new(2);
    let seph = t.battlefield(
        P0,
        "Sephiroth, Fabled SOLDIER // Sephiroth, One-Winged Angel",
    );
    let emblems = |t: &TestGame| {
        t.g.command
            .iter()
            .filter(|id| t.obj(**id).kind == ObjKind::Emblem && t.obj(**id).controller == P0)
            .count()
    };
    assert_eq!(emblems(&t), 0);
    transform(&mut t, seph);
    assert_eq!(t.obj(seph).chars.name, "Sephiroth, One-Winged Angel");
    assert_eq!(emblems(&t), 1);
    // Transforming back to the front face (which doesn't have it) does nothing more.
    transform(&mut t, seph);
    assert_eq!(emblems(&t), 1);
    // The emblem works.
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.g.move_object(bears, Zone::Graveyard(P1), MoveCause::Destroy, None);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.life(P0), 21);
}
