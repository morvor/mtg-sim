//! CR 307: sorceries — casting at sorcery speed, resolving, never entering the
//! battlefield, and "only as a sorcery" / "any time a sorcery couldn't have been cast".

use crate::r300_common::*;
use mtg_engine::ability::*;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn sorcery_spells_are_cast_at_sorcery_speed_and_use_the_stack() {
    cr!("307.1");
    check_sorcery_timing("Divination", "{2}{U}");
}

#[test]
fn a_resolving_sorcery_does_what_it_says_then_goes_to_its_owners_graveyard() {
    cr!("307.2");
    let mut t = TestGame::new(2);
    let hand = t.hand_size(P0);
    cast_others_card(&mut t, P0, P1, "Divination", "{2}{U}", &[]);
    assert_eq!(t.hand_size(P0), hand + 2);
    assert!(t.in_graveyard(P1, "Divination"));
    assert!(!t.in_graveyard(P0, "Divination"));
}

#[test]
fn a_sorcery_cant_enter_the_battlefield() {
    cr!("307.4");
    let mut t = TestGame::new(2);
    let div = t.hand(P0, "Divination");
    run_effect(
        &mut t,
        P0,
        None,
        Effect::Move {
            what: Sel::Target(0),
            to: Destination::battlefield(),
        },
        &[Entity::Object(div)],
    );
    // It remains in its previous zone.
    assert_eq!(t.zone(div), Zone::Hand(P0));
    assert!(t.named_on_battlefield("Divination").is_empty());
}

fn equip_legal(t: &mut TestGame, p: PlayerId, splitter: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    t.g.legal_actions(p)
        .iter()
        .any(|a| matches!(a, mtg_engine::decision::Action::Activate { source, .. } if *source == splitter))
}

#[test]
fn only_as_a_sorcery_means_priority_main_phase_own_turn_and_empty_stack() {
    cr!("307.5");
    let mut t = TestGame::new(2);
    let splitter = t.battlefield(P0, "Bonesplitter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 2);
    // An effect stopping P0 from casting sorcery spells (P0 has no sorcery card anyway)
    // doesn't affect activating an ability "only as a sorcery".
    t.custom(
        P1,
        CB::new("Sorcery Ban")
            .enchantment()
            .ability(stat(StaticEffect::Restriction(Restriction::CantCast {
                who: PlayerFilter::Any,
                what: Filter::Type(CardType::Sorcery),
            })))
            .build(),
        Zone::Battlefield,
    );
    // Equip: activate only as a sorcery.
    t.set_step(P1, Step::PrecombatMain);
    assert!(!equip_legal(&mut t, P0, splitter), "opponent's turn");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!equip_legal(&mut t, P0, splitter), "not a main phase");
    t.set_step(P0, Step::PostcombatMain);
    hold_stack(&mut t, P0);
    assert!(!equip_legal(&mut t, P0, splitter), "nonempty stack");
    t.resolve_all();
    assert!(equip_legal(&mut t, P0, splitter));
    t.activate(P0, splitter, 0, &[Entity::Object(bears)])
        .unwrap();
    t.resolve();
    assert_eq!(t.pt(bears), (4, 2));
}

/// Spider Climb's "as though it had flash" casting method.
fn own_flash(t: &TestGame, card: ObjectId) -> CastMethod {
    t.g.cast_options(P0, card)
        .into_iter()
        .find(|o| o.flash)
        .expect("flash option")
        .method
}

#[test]
fn cast_any_time_a_sorcery_couldnt_have_been_cast() {
    cr!("307.5a");
    supported("Spider Climb");
    ruling!(
        "Spider Climb",
        "The sacrifice occurs only if you cast it using its own ability"
    );
    // Cast with its own ability during the opponent's turn: sacrificed at the beginning of
    // the next cleanup step.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    let climb = t.hand(P0, "Spider Climb");
    t.set_step(P1, Step::DeclareAttackers);
    let m = own_flash(&t, climb);
    t.cast(P0, climb).method(m).target(bears).go();
    t.resolve();
    let aura = t.named_on_battlefield("Spider Climb");
    assert_eq!(aura.len(), 1);
    assert_eq!(t.pt(bears), (2, 5));
    t.advance_to(P0, Step::Upkeep);
    assert!(t.named_on_battlefield("Spider Climb").is_empty());
    assert!(t.in_graveyard(P0, "Spider Climb"));
    // Also cast while another spell was on the stack in P0's own main phase.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    let climb = t.hand(P0, "Spider Climb");
    hold_stack(&mut t, P0);
    let m = own_flash(&t, climb);
    t.cast(P0, climb).method(m).target(bears).go();
    t.resolve_all();
    t.advance_to(P1, Step::Upkeep);
    assert!(t.in_graveyard(P0, "Spider Climb"));
    // Cast the same way when a sorcery could have been cast: it stays.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    let climb = t.hand(P0, "Spider Climb");
    let m = own_flash(&t, climb);
    t.cast(P0, climb).method(m).target(bears).go();
    t.resolve();
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.named_on_battlefield("Spider Climb").len(), 1);
    // Cast at instant speed because of another effect: it stays.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Leyline of Anticipation");
    t.lands(P0, "Forest", 1);
    let climb = t.hand(P0, "Spider Climb");
    t.set_step(P1, Step::DeclareAttackers);
    t.cast(P0, climb).target(bears).go();
    t.resolve();
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(t.named_on_battlefield("Spider Climb").len(), 1);
}
