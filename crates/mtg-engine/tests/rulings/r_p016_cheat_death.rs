//! Rulings batch P016 — effects that return other creatures (and cards) to the battlefield
//! after they die: Abnormal Endurance, Demonic Gifts, Graceful Reprieve, Saffi
//! Eriksdotter (a returned creature is a new object, CR 400.7), Gerrard's Hourglass
//! Pendant, Tezzeret's Touch, Nissa's Zendikon and Oathkeeper, Takeno's Daisho.

use crate::r_p016_common::*;
use crate::r_s01_common::supported;
use crate::r_s02_common::{can_attack, create_token};
use crate::r_s05_common::move_to;
use crate::r_s26_common::modify_until_eot;
use mtg_engine::ability::{Modification, Value};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn destroy_and_resolve(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.destroy(id, None);
    t.resolve_all();
}

/// The creature (given the effect by `give`) dies and returns; the returned creature is a
/// new object, so when it dies again it stays in the graveyard. Returns whether it came
/// back tapped.
fn works_only_once(name: &str, give: fn(&mut TestGame, ObjectId)) -> bool {
    supported(name);
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    give(&mut t, bears);
    destroy_and_resolve(&mut t, bears);
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1, "{name}: didn't return");
    assert_ne!(back[0], bears, "{name}");
    let tapped = t.obj(back[0]).tapped;
    destroy_and_resolve(&mut t, back[0]);
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty(), "{name}");
    assert!(t.in_graveyard(P0, "Grizzly Bears"), "{name}");
    tapped
}

fn cast_on(name: &'static str) -> impl Fn(&mut TestGame, ObjectId) {
    move |t, id| {
        cast_at(t, P0, name, &[o(id)]);
        t.resolve_all();
    }
}

#[test]
fn granted_and_delayed_return_abilities_work_only_once() {
    cr!("400.7", "603.7c");
    ruling!(
        "Abnormal Endurance",
        "Abnormal Endurance's effect works only once. If the targeted creature dies and is then returned to the battlefield, it's considered to be a new creature. If that new creature dies, it won't come back."
    );
    ruling!(
        "Demonic Gifts",
        "The reanimating effect of Demonic Gifts works only once. Once the creature dies and returns to the battlefield, it’s a new object with no relation to the creature it was. If that new creature dies, it won’t come back."
    );
    ruling!(
        "Graceful Reprieve",
        "Graceful Reprieve’s effect works only once. If the targeted creature leaves the battlefield and is then returned to the battlefield, it’s considered to be a new creature. If that new creature is put into a graveyard, it won’t come back a second time."
    );
    ruling!(
        "Saffi Eriksdotter",
        "Saffi's effect works only once. If the target creature dies and is then returned to the battlefield, it's considered to be a new creature. If that new creature dies, it won't come back."
    );
    assert!(works_only_once("Abnormal Endurance", |t, id| cast_on(
        "Abnormal Endurance"
    )(t, id)));
    assert!(!works_only_once("Demonic Gifts", |t, id| cast_on(
        "Demonic Gifts"
    )(t, id)));
    works_only_once("Graceful Reprieve", |t, id| {
        cast_on("Graceful Reprieve")(t, id)
    });
    works_only_once("Saffi Eriksdotter", |t, id| {
        let saffi = t.battlefield(P0, "Saffi Eriksdotter");
        t.activate(P0, saffi, 0, &[o(id)]).unwrap();
        t.resolve_all();
    });
}

#[test]
fn graceful_reprieve_on_a_token_returns_nothing() {
    cr!("111.7", "111.8");
    ruling!(
        "Graceful Reprieve",
        "Graceful Reprieve can target a token creature, but since tokens cease to exist after they leave the battlefield, it won’t be returned to the battlefield."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let token = create_token(&mut t, P0, "Soldier");
    cast_at(&mut t, P0, "Graceful Reprieve", &[o(token)]);
    t.resolve_all();
    destroy_and_resolve(&mut t, token);
    assert!(t.g.permanents().all(|p| !p.is_token()));
}

#[test]
fn saffi_targeting_itself_dies_but_doesnt_return() {
    cr!("608.2b", "700.4");
    ruling!(
        "Saffi Eriksdotter",
        "Saffi can be the target of its own ability. In this case, the ability won't resolve because its target is illegal, so Saffi won't be returned to the battlefield, but any abilities that check when or if a creature dies will see that it died."
    );
    supported("Blood Artist");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Blood Artist");
    let saffi = t.battlefield(P0, "Saffi Eriksdotter");
    let ab = t.activate(P0, saffi, 0, &[o(saffi)]).unwrap().unwrap();
    assert_eq!(stack_targets(&t, ab), vec![o(saffi)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Saffi Eriksdotter"));
    assert!(t.named_on_battlefield("Saffi Eriksdotter").is_empty());
    // Blood Artist saw it die.
    assert_eq!(t.life(P1), 19);
}

// ---------------------------------------------------------------------------
// Tezzeret's Touch
// ---------------------------------------------------------------------------

fn touch(t: &mut TestGame, artifact: ObjectId) -> ObjectId {
    supported("Tezzeret's Touch");
    let a = t.battlefield(P0, "Tezzeret's Touch");
    t.g.attach(a, Entity::Object(artifact));
    t.settle();
    a
}

#[test]
fn tezzerets_touch_artifact_needs_continuous_control_to_attack() {
    cr!("302.6", "301.7");
    ruling!(
        "Tezzeret's Touch",
        "An artifact that becomes a creature due to Tezzeret’s Touch can’t attack unless you’ve controlled it continuously since your turn began. It doesn’t matter whether or not it was a creature at that time."
    );
    // Controlled since the turn began (as a noncreature artifact): it can attack.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::BeginningOfCombat);
    let ring = t.battlefield(P0, "Sol Ring");
    touch(&mut t, ring);
    assert_eq!(t.pt(ring), (5, 5));
    assert!(can_attack(&mut t, ring));
    // It came under P0's control this turn: it can't.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::BeginningOfCombat);
    let ring = t.battlefield_sick(P0, "Sol Ring");
    touch(&mut t, ring);
    assert!(t.obj_now(ring).is(CardType::Creature));
    assert!(!can_attack(&mut t, ring));
}

#[test]
fn tezzerets_touch_leaving_removes_a_noncreature_artifact_from_combat() {
    cr!("506.4", "302.1");
    ruling!(
        "Tezzeret's Touch",
        "If Tezzeret’s Touch becomes unattached from an artifact that’s attacking or blocking, that artifact will be removed from combat unless it’s normally a creature"
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::BeginningOfCombat);
    let ring = t.battlefield(P0, "Sol Ring");
    let thopter = t.battlefield(P0, "Ornithopter");
    let a = touch(&mut t, ring);
    let b = touch(&mut t, thopter);
    crate::r_s01_common::attack_with(
        &mut t,
        &[(ring, Entity::Player(P1)), (thopter, Entity::Player(P1))],
    );
    assert!(t.g.is_attacking(ring) && t.g.is_attacking(thopter));
    move_to(&mut t, a, Zone::Graveyard(P0));
    move_to(&mut t, b, Zone::Graveyard(P0));
    t.resolve_all();
    assert!(!t.g.is_attacking(ring));
    assert!(t.g.is_attacking(thopter));
}

// ---------------------------------------------------------------------------
// Gerrard's Hourglass Pendant
// ---------------------------------------------------------------------------

#[test]
fn pendant_skips_extra_turns_created_by_spells() {
    cr!("500.7", "614.10");
    ruling!(
        "Gerrard's Hourglass Pendant",
        "An “extra turn” is any turn created by a spell or ability."
    );
    ruling!(
        "Gerrard's Hourglass Pendant",
        "Extra turns can still be created while Gerrard’s Hourglass Pendant is on the battlefield. They’re not skipped until they would begin"
    );
    supported("Time Walk");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Gerrard's Hourglass Pendant");
    cast_at(&mut t, P0, "Time Walk", &[]);
    t.resolve_all();
    t.advance_to_step(Step::Upkeep);
    assert_eq!(t.g.turn.active, P1);
    assert!(!t.g.turn.extra);
}

/// Activates P0's Pendant (it has plenty of mana) and resolves.
fn activate_pendant(t: &mut TestGame, pendant: ObjectId) {
    lots(t, P0);
    t.activate(P0, pendant, 0, &[]).unwrap();
    t.resolve_all();
}

#[test]
fn pendant_returns_only_cards_put_into_your_graveyard_from_the_battlefield() {
    cr!("400.7", "404.2");
    ruling!(
        "Gerrard's Hourglass Pendant",
        "Permanent cards that were put into your graveyard from anywhere else, such as a card that was discarded, stay in the graveyard. Permanent cards that you gained control of and went to another player’s graveyard from the battlefield will likewise stay in the graveyard."
    );
    ruling!(
        "Gerrard's Hourglass Pendant",
        "Permanent spells that were countered earlier in the turn never entered the battlefield, so they won’t return to the battlefield because of the last ability."
    );
    supported("Gerrard's Hourglass Pendant");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let pendant = t.battlefield(P0, "Gerrard's Hourglass Pendant");
    // Dies from the battlefield: returns.
    let bears = t.battlefield(P0, "Grizzly Bears");
    destroy_and_resolve(&mut t, bears);
    // Discarded: stays.
    let wall = t.hand(P0, "Wall of Omens");
    move_to(&mut t, wall, Zone::Graveyard(P0));
    // P1's Hill Giant under P0's control dies: it goes to P1's graveyard and stays.
    let giant = t.battlefield(P1, "Hill Giant");
    cast_at(&mut t, P0, "Act of Treason", &[o(giant)]);
    t.resolve_all();
    assert_eq!(t.obj_now(giant).controller, P0);
    destroy_and_resolve(&mut t, giant);
    // A countered creature spell: stays.
    let spell = cast_at(&mut t, P0, "Ornithopter", &[]);
    cast_at(&mut t, P1, "Counterspell", &[o(spell)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Ornithopter"));
    activate_pendant(&mut t, pendant);
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1);
    assert!(t.obj(back[0]).tapped);
    assert!(t.in_graveyard(P0, "Wall of Omens"));
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert!(t.in_graveyard(P0, "Ornithopter"));
}

#[test]
fn pendant_aura_cant_enchant_what_returns_with_it() {
    cr!("303.4f", "303.4g");
    ruling!(
        "Gerrard's Hourglass Pendant",
        "You choose what an Aura card put onto the battlefield this way will enchant. You can’t choose any permanent cards entering the battlefield at the same time as that Aura. If there’s nothing legal for the Aura to enchant, it stays in the graveyard."
    );
    // Pacifism on P0's Bears; the Bears die and Pacifism follows.
    let setup = |with_giant: bool| {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let pendant = t.battlefield(P0, "Gerrard's Hourglass Pendant");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let giant = with_giant.then(|| t.battlefield(P1, "Hill Giant"));
        let pac = t.battlefield(P0, "Pacifism");
        t.g.attach(pac, o(bears));
        destroy_and_resolve(&mut t, bears);
        assert!(t.in_graveyard(P0, "Pacifism"));
        if let Some(g) = giant {
            t.answer_choose(P0, &[o(g)]);
        }
        activate_pendant(&mut t, pendant);
        (t, giant)
    };
    // Nothing else to enchant: Pacifism stays in the graveyard; the Bears return.
    let (t, _) = setup(false);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert!(t.in_graveyard(P0, "Pacifism"));
    // With P1's Giant: Pacifism enchants the Giant, not the returning Bears.
    let (t, giant) = setup(true);
    let pac = t.named_on_battlefield("Pacifism");
    assert_eq!(pac.len(), 1);
    assert_eq!(t.obj(pac[0]).attached_to, Some(o(giant.unwrap())));
}

// ---------------------------------------------------------------------------
// Nissa's Zendikon, Oathkeeper
// ---------------------------------------------------------------------------

fn zendikon(t: &mut TestGame, land: ObjectId) {
    supported("Nissa's Zendikon");
    let z = t.battlefield(P0, "Nissa's Zendikon");
    t.g.attach(z, o(land));
    t.settle();
}

#[test]
fn nissas_zendikon_keeps_abilities_and_sets_pt_in_timestamp_order() {
    cr!("613.4b", "613.7");
    ruling!(
        "Nissa's Zendikon",
        "Nissa's Zendikon doesn't remove any abilities the land creature has."
    );
    ruling!(
        "Nissa's Zendikon",
        "Nissa's Zendikon overwrites any previous effects that set the land creature's power and/or toughness to specific values. Other effects that set these characteristics to specific values that start to apply after the ability resolves will overwrite that part of the effect."
    );
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    let abilities = t.obj_now(forest).chars.abilities.len();
    // An earlier effect made it a 1/1 creature.
    modify_until_eot(
        &mut t,
        forest,
        vec![
            Modification::AddTypes(vec![CardType::Creature]),
            Modification::SetPT(Some(Value::c(1)), Some(Value::c(1))),
        ],
    );
    assert_eq!(t.pt(forest), (1, 1));
    zendikon(&mut t, forest);
    assert_eq!(t.pt(forest), (4, 4));
    assert!(t.obj_now(forest).chars.abilities.len() > abilities);
    assert!(t.obj_now(forest).is(CardType::Land));
    // Its mana ability is still there.
    assert!(t
        .obj_now(forest)
        .chars
        .abilities
        .iter()
        .any(|a| a.text.contains("Add {G}")));
    // A later effect setting power and toughness wins.
    modify_until_eot(
        &mut t,
        forest,
        vec![Modification::SetPT(Some(Value::c(2)), Some(Value::c(2)))],
    );
    assert_eq!(t.pt(forest), (2, 2));
}

#[test]
fn oathkeeper_checks_the_card_for_samurai() {
    cr!("400.7", "608.2h");
    ruling!(
        "Oathkeeper, Takeno's Daisho",
        "Oathkeeper's second ability checks whether the card's creature type is Samurai, not whether the creature was a Samurai when it left the battlefield."
    );
    supported("Oathkeeper, Takeno's Daisho");
    let equipped_dies = |name: &str, make_samurai: bool| -> TestGame {
        let mut t = TestGame::new(2);
        let c = t.battlefield(P1, name);
        let ok = t.battlefield(P0, "Oathkeeper, Takeno's Daisho");
        t.g.attach(ok, o(c));
        if make_samurai {
            modify_until_eot(
                &mut t,
                c,
                vec![Modification::AddSubtypes(vec!["Samurai".into()])],
            );
        }
        destroy_and_resolve(&mut t, c);
        t
    };
    // A Samurai card returns, under Oathkeeper's controller's control.
    let t = equipped_dies("Devoted Retainer", false);
    let back = t.named_on_battlefield("Devoted Retainer");
    assert_eq!(back.len(), 1);
    assert_eq!(t.obj(back[0]).controller, P0);
    // A creature that was a Samurai only on the battlefield doesn't.
    let t = equipped_dies("Grizzly Bears", true);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}
