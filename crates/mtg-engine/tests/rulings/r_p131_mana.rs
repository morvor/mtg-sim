//! Rulings batch P131 — mana: triggered mana abilities and abilities that add mana but
//! aren't mana abilities (CR 605), mana replacement effects, what "could produce" checks,
//! tapping artifacts to pay costs, X values fixed on resolution, and cast triggers that
//! resolve before their spell.

use crate::r_p131_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::*;

fn pool_of(t: &TestGame, p: PlayerId, ty: ManaType) -> usize {
    t.g.player(p).mana_pool.count(ty)
}

// --- Triggered mana abilities and mana replacement ------------------------------------------

#[test]
fn overgrowth_is_a_triggered_mana_ability() {
    cr!("605.1b", "605.4a");
    ruling!("Overgrowth", "This is a triggered mana ability.");
    supported("Overgrowth");
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    attach_new(&mut t, P0, "Overgrowth", forest);
    t.activate(P0, forest, 0, &[]).unwrap();
    // No trigger on the stack: the extra mana is there at once.
    assert_eq!(t.stack_len(), 0);
    assert_eq!(pool_of(&t, P0, ManaType::G), 3);
}

#[test]
fn quirion_sentinels_mana_trigger_uses_the_stack() {
    cr!("605.5a");
    ruling!(
        "Quirion Sentinel",
        "This is not a mana ability. It uses the stack, and can be responded to."
    );
    supported("Quirion Sentinel");
    let mut t = TestGame::new(2);
    t.enter(P0, "Quirion Sentinel");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    assert!(is_trigger(&t, top(&t)));
    assert_eq!(pool_total(&t, P0), 0);
    t.resolve_all();
    assert_eq!(pool_total(&t, P0), 1);
}

#[test]
fn virtue_of_strength_triples_a_basic_lands_intrinsic_mana_ability() {
    cr!("305.6", "106.6a");
    ruling!(
        "Virtue of Strength // Garenbrig Growth",
        "An object with the land card type and a basic land type has the intrinsic ability \"{T}: Add [mana symbol],\" even if the text box doesn't actually contain that text or the object has no text box."
    );
    supported("Virtue of Strength // Garenbrig Growth");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Virtue of Strength // Garenbrig Growth");
    let forest = t.battlefield(P0, "Forest");
    let wastes = t.battlefield(P0, "Wastes");
    t.activate(P0, forest, 0, &[]).unwrap();
    t.activate(P0, wastes, 0, &[]).unwrap();
    assert_eq!(pool_of(&t, P0, ManaType::G), 3);
    assert_eq!(pool_of(&t, P0, ManaType::C), 3);
}

#[test]
fn virtue_of_strength_doesnt_triple_a_triggered_mana_ability() {
    cr!("605.1b", "605.4a");
    ruling!(
        "Virtue of Strength // Garenbrig Growth",
        "If an ability triggers \"whenever you tap\" a basic land for mana and produces mana, that triggered mana ability won't be affected by Virtue of Strength."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Virtue of Strength // Garenbrig Growth");
    let forest = t.battlefield(P0, "Forest");
    attach_new(&mut t, P0, "Overgrowth", forest);
    t.activate(P0, forest, 0, &[]).unwrap();
    // {G}{G}{G} from the Forest, plus {G}{G} from Overgrowth.
    assert_eq!(pool_of(&t, P0, ManaType::G), 5);
}

// --- Fellwar Stone ------------------------------------------------------------------------

#[test]
fn fellwar_stone_works_with_the_opponents_lands_tapped() {
    cr!("106.7");
    ruling!(
        "Fellwar Stone",
        "This works even if the opponent's lands are tapped. It only checks what kinds of mana can be produced, not if the abilities that produce them are usable right now."
    );
    let mut t = TestGame::new(2);
    let stone = t.battlefield(P0, "Fellwar Stone");
    let forest = t.battlefield(P1, "Forest");
    t.g.tap(forest);
    t.activate(P0, stone, 0, &[]).unwrap();
    assert_eq!(pool_of(&t, P0, ManaType::G), 1);
}

#[test]
fn fellwar_stone_considers_replacement_effects_on_the_opponents_lands() {
    cr!("106.7");
    ruling!(
        "Fellwar Stone",
        "When determining what colors of mana your opponents' lands could produce, take into account any applicable replacement effects that would apply to those lands' mana abilities"
    );
    supported("Contamination");
    let mut t = TestGame::new(2);
    let stone = t.battlefield(P0, "Fellwar Stone");
    t.battlefield(P1, "Forest");
    t.battlefield(P1, "Contamination");
    t.activate(P0, stone, 0, &[]).unwrap();
    // The Forest could produce only {B}; Fellwar Stone isn't a land, so it isn't affected.
    assert_eq!(pool_of(&t, P0, ManaType::B), 1);
    assert_eq!(pool_of(&t, P0, ManaType::G), 0);
}

// --- Drain Power --------------------------------------------------------------------------

#[test]
fn drain_power_lets_the_target_choose_how_its_lands_make_mana() {
    cr!("106.13", "605.3a");
    ruling!(
        "Drain Power",
        "This card forces the target player to draw mana from lands if they are untapped, but that player can choose how to draw the mana if a land they control has multiple mana abilities, or mana abilities with choices."
    );
    supported("Drain Power");
    for (i, ty) in [(0, ManaType::R), (1, ManaType::G)] {
        let mut t = TestGame::new(2);
        let taiga = t.battlefield(P1, "Taiga");
        let tapped = t.battlefield(P1, "Mountain");
        t.g.tap(tapped);
        t.answer(P1, DecisionKind::Option, Answer::Index(i));
        let from = t.asked().len();
        cast(&mut t, P0, "Drain Power", &[Entity::Player(P1)]);
        t.resolve_all();
        assert!(t.obj_now(taiga).tapped);
        // P1 was asked; P0 got the one mana P1 chose (the tapped Mountain gave none).
        assert!(t.asked()[from..]
            .iter()
            .any(|(p, d)| *p == P1 && matches!(d, Decision::ChooseOption { .. })));
        assert_eq!(pool_total(&t, P0), 1);
        assert_eq!(pool_of(&t, P0, ty), 1);
    }
}

#[test]
fn drain_power_can_be_responded_to_by_spending_the_mana() {
    cr!("117.1a", "608.2c");
    ruling!(
        "Drain Power",
        "Your opponent may use instants and abilities of permanents in response to this spell before you get the mana."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Forest");
    t.battlefield(P1, "Forest");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast(&mut t, P0, "Drain Power", &[Entity::Player(P1)]);
    // In response, P1 taps a Forest to cast Giant Growth.
    let gg = t.hand(P1, "Giant Growth");
    t.cast(P1, gg).target(bears).go();
    t.resolve();
    assert_eq!(t.pt(bears), (5, 5));
    t.resolve_all();
    assert_eq!(pool_of(&t, P0, ManaType::G), 1);
}

// --- Traitorous Greed --------------------------------------------------------------------

#[test]
fn traitorous_greed_can_target_your_own_untapped_creature() {
    cr!("115.1");
    ruling!(
        "Traitorous Greed",
        "Traitorous Greed can target any creature, even one that's untapped or one you already control."
    );
    supported("Traitorous Greed");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    cast(&mut t, P0, "Traitorous Greed", &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P0);
    assert!(!t.obj_now(bears).tapped);
    assert_eq!(pool_total(&t, P0), 2);
}

// --- Tapping artifacts for costs --------------------------------------------------------------

#[test]
fn urza_and_meria_tap_summoning_sick_artifact_creatures() {
    cr!("302.6", "602.5a");
    ruling!(
        "Urza, Lord High Artificer",
        "You can tap any untapped artifact you control to pay the cost of the mana ability, including an artifact creature you haven't controlled continuously since the beginning of your most recent turn."
    );
    ruling!(
        "Meria, Scholar of Antiquity",
        "You may tap any untapped nontoken artifact to activate Meria’s abilities, including artifact creatures that entered the battlefield this turn, even if they don’t have haste."
    );
    supported("Urza, Lord High Artificer");
    supported("Meria, Scholar of Antiquity");
    for (name, ty) in [
        ("Urza, Lord High Artificer", ManaType::U),
        ("Meria, Scholar of Antiquity", ManaType::G),
    ] {
        let mut t = TestGame::new(2);
        let lord = t.battlefield(P0, name);
        let thopter = t.battlefield_sick(P0, "Ornithopter");
        t.answer_choose(P0, &[obj(thopter)]);
        activate_containing(&mut t, P0, lord, "Add").unwrap();
        assert!(t.obj(thopter).tapped, "{name}");
        assert_eq!(pool_of(&t, P0, ty), 1, "{name}");
    }
}

#[test]
fn tapping_an_equipment_for_urza_doesnt_affect_it() {
    cr!("701.26a");
    ruling!(
        "Urza, Lord High Artificer",
        "Tapping an Equipment this way won't affect its abilities or the equipped creature."
    );
    let mut t = TestGame::new(2);
    let urza = t.battlefield(P0, "Urza, Lord High Artificer");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let blade = attach_new(&mut t, P0, "Bonesplitter", bears);
    assert_eq!(t.pt(bears), (4, 2));
    t.answer_choose(P0, &[obj(blade)]);
    activate_containing(&mut t, P0, urza, "Add").unwrap();
    assert!(t.obj(blade).tapped);
    assert_eq!(pool_of(&t, P0, ManaType::U), 1);
    t.g.recompute();
    assert_eq!(t.pt(bears), (4, 2));
}

// --- X fixed on resolution -----------------------------------------------------------------

#[test]
fn geode_grottos_x_is_locked_in_as_it_resolves() {
    cr!("608.2h", "611.2c");
    ruling!(
        "Dowsing Device // Geode Grotto",
        "The value of X for Geode Grotto's last ability is determined only as it resolves. Once that happens, the value of X won't change later in the turn even if the number of artifacts you control changes."
    );
    supported("Dowsing Device // Geode Grotto");
    let mut t = TestGame::new(2);
    let grotto =
        crate::r_s17_common::enter_transformed(&mut t, P0, "Dowsing Device // Geode Grotto");
    assert_eq!(t.obj(grotto).chars.name, "Geode Grotto");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Sol Ring");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 2);
    act(&mut t, P0, grotto, "haste", &[obj(bears)]).unwrap();
    // A second artifact before it resolves counts; a third afterwards doesn't.
    t.battlefield(P0, "Sol Ring");
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 2));
    t.battlefield(P0, "Sol Ring");
    t.g.recompute();
    assert_eq!(t.pt(bears), (4, 2));
}

/// P0 taps Brass Infiniscope (its delayed trigger is set up), then casts `spell` with X =
/// `x`, paying from the pool; everything resolves.
fn infiniscope(spell: &str, x: i64, pool: &[(ManaType, u32)]) -> TestGame {
    supported("Brass Infiniscope");
    let mut t = TestGame::new(2);
    let scope = t.battlefield(P0, "Brass Infiniscope");
    t.activate(P0, scope, 0, &[]).unwrap();
    t.resolve_all();
    for (ty, n) in pool {
        mana(&mut t, P0, *ty, *n);
    }
    let c = t.hand(P0, spell);
    t.cast(P0, c).x(x).target(Entity::Player(P1)).go();
    t.resolve_all();
    t
}

#[test]
fn brass_infiniscope_uses_the_spells_x() {
    cr!("107.3e", "603.7");
    ruling!(
        "Brass Infiniscope",
        "The value of X in the delayed triggered ability created by Brass Infiniscope's ability is the value of X of the spell you cast as it exists on the stack when that ability resolves."
    );
    // Blaze, X = 5: {5}{R} (two of it from the Infiniscope's {C}{C}).
    let t = infiniscope("Blaze", 5, &[(ManaType::R, 1), (ManaType::C, 3)]);
    assert_eq!(t.life(P1), 15);
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn brass_infiniscope_gains_half_x_once_however_many_x_symbols() {
    cr!("107.3e", "603.7");
    ruling!(
        "Brass Infiniscope",
        "You'll only gain half X life no matter how many instances of {X} are in the mana cost of the spell you cast."
    );
    supported("Crackle with Power");
    // Crackle with Power, X = 2: {X}{X}{X}{R}{R} = {6}{R}{R}; 10 damage to P1.
    let t = infiniscope(
        "Crackle with Power",
        2,
        &[(ManaType::R, 2), (ManaType::C, 4)],
    );
    assert_eq!(t.life(P1), 10);
    assert_eq!(t.life(P0), 21);
}

// --- Resolution-time checks and cast triggers ------------------------------------------------

/// `p` creates a 0/1 colorless Eldrazi Spawn creature token (as a resolving effect would).
fn eldrazi_spawn(t: &mut TestGame, p: PlayerId) {
    use mtg_engine::ability::{Effect, PlayerRef, TokenSpec, Value};
    use mtg_engine::types::{CardType, ColorSet};
    let spec = TokenSpec {
        name: "Eldrazi Spawn".into(),
        colors: ColorSet::NONE,
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: vec!["Eldrazi".into(), "Spawn".into()],
        power: Some(0),
        toughness: Some(1),
        abilities: vec![],
        scryfall_name: None,
        pt_values: None,
    };
    let mut ctx = mtg_engine::eval::Ctx::new(None, p);
    t.g.exec(
        &Effect::CreateToken {
            spec,
            count: Value::c(1),
            controller: PlayerRef::You,
            tapped: false,
            attacking: false,
        },
        &mut ctx,
    );
    t.g.recompute();
}

#[test]
fn brood_birthing_checks_for_a_spawn_as_it_resolves() {
    cr!("608.2h");
    ruling!(
        "Brood Birthing",
        "Whether you control an Eldrazi Spawn is checked as Brood Birthing resolves."
    );
    supported("Brood Birthing");
    for spawn in [false, true] {
        let mut t = TestGame::new(2);
        cast(&mut t, P0, "Brood Birthing", &[]);
        if spawn {
            eldrazi_spawn(&mut t, P0);
        }
        t.resolve_all();
        let n = tokens_with(&t, P0, "Spawn");
        assert_eq!(n, if spawn { 4 } else { 1 });
    }
}

#[test]
fn writhing_chrysalis_cast_trigger_resolves_even_if_its_countered() {
    cr!("603.3", "601.2i", "701.6a");
    ruling!(
        "Writhing Chrysalis",
        "Writhing Chrysalis's second ability will resolve before Writhing Chrysalis does. If Writhing Chrysalis is countered or otherwise leaves the stack in response to that triggered ability, the triggered ability will still resolve as normal."
    );
    supported("Writhing Chrysalis");
    let mut t = TestGame::new(2);
    let spell = cast(&mut t, P0, "Writhing Chrysalis", &[]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    assert!(t.g.counter(spell, None));
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Writhing Chrysalis"));
    assert_eq!(tokens_with(&t, P0, "Spawn"), 2);
}
