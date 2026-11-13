//! A spell's own cost changes (CR 601.2f, 118.7): "This spell costs {1} less to cast for
//! each ...", "... if it targets ...", "If [condition], this spell costs ...", "This
//! spell costs {X} less to cast, where X is ...", "This spell costs {2} more to cast if
//! it targets a Dragon."

use mtg_engine::decision::Action;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

fn castable(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    let saved = t.g.turn.priority;
    t.g.turn.priority = Some(p);
    let ok =
        t.g.legal_actions(p)
            .iter()
            .any(|a| matches!(a, Action::Cast { card: c, .. } if *c == card));
    t.g.turn.priority = saved;
    ok
}

fn tapped(t: &TestGame, lands: &[ObjectId]) -> usize {
    lands.iter().filter(|l| t.obj_now(**l).tapped).count()
}

#[test]
fn costs_less_for_each_card_but_not_its_colored_mana() {
    cr!("601.2f", "118.7a");
    ruling!(
        "Bedlam Reveler",
        "Bedlam Reveler’s first ability can’t reduce the {R}{R} in its cost."
    );
    compiles("Bedlam Reveler");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    for n in ["Lightning Bolt", "Opt", "Divination"] {
        t.graveyard(P0, n);
    }
    // Neither a creature card nor a card in another graveyard counts.
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P1, "Shock");
    let reveler = t.hand(P0, "Bedlam Reveler");
    let mountains = t.lands(P0, "Mountain", 5);
    // {6}{R}{R} less {3}.
    t.cast(P0, reveler).go();
    assert_eq!(tapped(&t, &mountains), 5);
    t.resolve();
    assert_eq!(t.named_on_battlefield("Bedlam Reveler").len(), 1);

    // Eight instants and sorceries: still {R}{R}.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    for _ in 0..8 {
        t.graveyard(P0, "Lightning Bolt");
    }
    let reveler = t.hand(P0, "Bedlam Reveler");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Island", 3);
    assert!(t.cast(P0, reveler).try_go().is_err());
    assert!(t.in_hand(P0, "Bedlam Reveler"));
    t.lands(P0, "Mountain", 1);
    t.cast(P0, reveler).go();
    assert_eq!(t.stack_len(), 1);
}

#[test]
fn costs_less_if_it_targets_a_tapped_creature() {
    cr!("601.2c", "601.2f");
    compiles("Fate of the Sun-Cryst");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let tapped_bears = t.battlefield(P1, "Grizzly Bears");
    t.g.objects[tapped_bears.0 as usize].tapped = true;
    let untapped = t.battlefield(P1, "Hill Giant");
    let fate = t.hand(P0, "Fate of the Sun-Cryst");
    t.lands(P0, "Plains", 3);
    // With three lands it can be cast, since its target isn't chosen yet.
    assert!(castable(&mut t, P0, fate));
    // {4}{W} for an untapped target: the cast is illegal and rewound.
    assert!(t.cast(P0, fate).target(untapped).try_go().is_err());
    assert!(t.in_hand(P0, "Fate of the Sun-Cryst"));
    // {2}{W} for a tapped one.
    t.clear_answers();
    t.cast(P0, fate).target(tapped_bears).go();
    t.resolve();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.on_battlefield(untapped));
}

#[test]
fn costs_less_if_you_control_a_wizard() {
    cr!("601.2f");
    compiles("Wizard's Lightning");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bolt = t.hand(P0, "Wizard's Lightning");
    t.lands(P0, "Mountain", 1);
    assert!(!castable(&mut t, P0, bolt));
    // An opponent's Wizard doesn't help.
    t.battlefield(P1, "Prodigal Sorcerer");
    assert!(!castable(&mut t, P0, bolt));
    t.battlefield(P0, "Prodigal Sorcerer");
    assert!(castable(&mut t, P0, bolt));
    t.cast(P0, bolt).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn a_leading_condition_on_a_cost_reduction() {
    cr!("601.2f");
    ruling!(
        "Avatar of Hope",
        "The mana value of this card is still 8, even if you only pay {W}{W} to cast it."
    );
    compiles("Avatar of Hope");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let avatar = t.hand(P0, "Avatar of Hope");
    t.lands(P0, "Plains", 2);
    t.g.player_mut(P0).life = 4;
    assert!(!castable(&mut t, P0, avatar));
    t.g.player_mut(P0).life = 3;
    assert!(castable(&mut t, P0, avatar));
    let s = t.cast(P0, avatar).go();
    assert_eq!(t.obj(s).chars.mana_value(), 8);
}

#[test]
fn costs_more_if_it_targets_a_dragon() {
    cr!("601.2f");
    compiles("Dragon's Prey");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let dragon = t.battlefield(P1, "Shivan Dragon");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let prey = t.hand(P0, "Dragon's Prey");
    let swamps = t.lands(P0, "Swamp", 3);
    assert!(castable(&mut t, P0, prey));
    // {2}{B} more for a Dragon: {4}{B}.
    assert!(t.cast(P0, prey).target(dragon).try_go().is_err());
    t.clear_answers();
    t.cast(P0, prey).target(bears).go();
    assert_eq!(tapped(&t, &swamps), 3);
    t.resolve();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    let prey2 = t.hand(P0, "Dragon's Prey");
    let more = t.lands(P0, "Swamp", 5);
    t.cast(P0, prey2).target(dragon).go();
    assert_eq!(tapped(&t, &more), 5);
    t.resolve();
    assert!(t.in_graveyard(P1, "Shivan Dragon"));
}

#[test]
fn increases_apply_before_reductions() {
    cr!("601.2f", "118.7a");
    ruling!(
        "Blasphemous Act",
        "start with the mana cost or alternative cost you're paying, add any cost increases, then apply any cost reductions (such as that of Blasphemous Act)"
    );
    ruling!(
        "Blasphemous Act",
        "Blasphemous Act's ability can't reduce the total cost to cast the spell below {R}."
    );
    compiles("Blasphemous Act");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    // Thalia makes it cost {1} more: {9}{R}, less {1} for each of nine creatures.
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    for _ in 0..4 {
        t.battlefield(P0, "Grizzly Bears");
        t.battlefield(P1, "Grizzly Bears");
    }
    let act = t.hand(P0, "Blasphemous Act");
    let mountains = t.lands(P0, "Mountain", 1);
    t.cast(P0, act).go();
    assert_eq!(tapped(&t, &mountains), 1);
    t.resolve();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 0);
    assert!(t.in_graveyard(P1, "Thalia, Guardian of Thraben"));
}

#[test]
fn a_colored_reduction_reduces_generic_mana_beyond_that_color() {
    cr!("601.2f", "118.7c");
    ruling!(
        "Khalni Hydra",
        "the Hydra's ability will reduce it too. It'll reduce the amount of green mana you need to spend first"
    );
    ruling!("Lodestone Golem", "effects that increase the cost are applied before effects that reduce the cost");
    compiles("Khalni Hydra");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    // {G}{G}{G}{G}{G}{G}{G}{G} plus {1}: eight green creatures leave {1}.
    t.battlefield(P1, "Lodestone Golem");
    for _ in 0..8 {
        t.battlefield(P0, "Grizzly Bears");
    }
    let hydra = t.hand(P0, "Khalni Hydra");
    assert!(!castable(&mut t, P0, hydra));
    t.lands(P0, "Wastes", 1);
    assert!(castable(&mut t, P0, hydra));
    // A ninth reduces the generic mana too: it's free.
    let mut t2 = TestGame::new(2);
    t2.set_step(P0, Step::PrecombatMain);
    t2.battlefield(P1, "Lodestone Golem");
    for _ in 0..9 {
        t2.battlefield(P0, "Grizzly Bears");
    }
    // A nongreen creature doesn't count.
    t2.battlefield(P0, "Hill Giant");
    let hydra = t2.hand(P0, "Khalni Hydra");
    t2.cast(P0, hydra).go();
    t2.resolve();
    assert_eq!(t2.named_on_battlefield("Khalni Hydra").len(), 1);
}

#[test]
fn domain_counts_basic_land_types_not_lands() {
    cr!("601.2f");
    ruling!(
        "Stratadon",
        "Domain abilities count the number of basic land types among lands you control, not how many lands you control"
    );
    compiles("Stratadon");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let s = t.hand(P0, "Stratadon");
    for n in ["Plains", "Island", "Swamp"] {
        t.lands(P0, n, 1);
    }
    t.lands(P0, "Mountain", 2);
    // Four types: {10} less {4} is more than five lands.
    assert!(!castable(&mut t, P0, s));
    t.lands(P0, "Mountain", 1);
    assert!(castable(&mut t, P0, s));
    t.cast(P0, s).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Stratadon").len(), 1);
}

#[test]
fn costs_x_less_where_x_is_devotion() {
    cr!("601.2f", "700.5");
    ruling!(
        "Drag to the Underworld",
        "The cost reduction ability reduces only the generic mana in Drag to the Underworld's cost."
    );
    compiles("Drag to the Underworld");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let drag = t.hand(P0, "Drag to the Underworld");
    let swamps = t.lands(P0, "Swamp", 2);
    assert!(!castable(&mut t, P0, drag));
    // Each Gray Merchant ({3}{B}{B}) adds 2 to devotion to black: X is 4, which removes
    // the {2} but not the {B}{B}.
    t.battlefield(P0, "Gray Merchant of Asphodel");
    t.battlefield(P0, "Gray Merchant of Asphodel");
    t.cast(P0, drag).target(bears).go();
    assert_eq!(tapped(&t, &swamps), 2);
    t.resolve();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn costs_less_if_you_cast_another_spell_this_turn() {
    cr!("601.2f", "601.2i");
    compiles("Focus the Mind");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    for _ in 0..5 {
        t.library_top(P0, "Island");
    }
    let focus = t.hand(P0, "Focus the Mind");
    t.lands(P0, "Island", 3);
    // The spell itself doesn't count: {4}{U}.
    assert!(!castable(&mut t, P0, focus));
    // An opponent's spell doesn't count either.
    let theirs = t.hand(P1, "Opt");
    t.lands(P1, "Island", 1);
    t.g.turn.priority = Some(P1);
    t.cast(P1, theirs).go();
    t.resolve();
    assert!(!castable(&mut t, P0, focus));
    let opt = t.hand(P0, "Opt");
    t.lands(P0, "Island", 1);
    t.cast(P0, opt).go();
    t.resolve();
    // After casting Opt: {2}{U}, paid with the three other Islands.
    assert!(castable(&mut t, P0, focus));
    t.cast(P0, focus).go();
    assert_eq!(t.stack_len(), 1);
}
