//! CR 601.2g–h: mana abilities are activated before the total cost is paid, so what they
//! sacrifice isn't there for the cost's other parts ("As an additional cost to cast this
//! spell, sacrifice a creature."), and what convoke or improvise taps as the cost is paid
//! can't have been sacrificed for mana first. The automatic payment
//! (`automatic-mana-payment`) keeps what the other parts need, and neither offers nor
//! attempts a payment that would take it.

use mtg_engine::decision::Action;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn main_phase() -> TestGame {
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t
}

/// Whether casting `card` is among `p`'s legal actions.
fn offered(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    let card = t.g.current(card);
    t.g.legal_actions(p).contains(&Action::Cast {
        card,
        method: CastMethod::Normal,
    })
}

fn on_stack(t: &TestGame, name: &str) -> bool {
    t.g.stack
        .iter()
        .any(|id| t.g.obj(*id).chars.name.as_str() == name)
}

fn tapped(t: &TestGame, id: ObjectId) -> bool {
    t.g.obj(t.g.current(id)).tapped
}

#[test]
fn bone_splinters_sacrifice_isnt_also_sacrificed_to_phyrexian_altar() {
    cr!("601.2g", "601.2h", "118.3");
    interp!("automatic-mana-payment");
    interp!("mana-abilities-planned-with-the-objects-their-costs-take");
    ruling!(
        "Bone Splinters",
        "You must sacrifice exactly one creature to cast this spell; you can’t cast it without sacrificing a creature"
    );
    // Bone Splinters ({B}; "As an additional cost to cast this spell, sacrifice a
    // creature.") with Phyrexian Altar ("Sacrifice a creature: Add one mana of any color.")
    // and one creature: the creature pays only one of the two costs.
    let mut t = main_phase();
    t.battlefield(P0, "Phyrexian Altar");
    let a = t.battlefield(P0, "Grizzly Bears");
    let target = t.battlefield(P1, "Grizzly Bears");
    let splinters = t.hand(P0, "Bone Splinters");
    assert!(!offered(&mut t, P0, splinters));
    assert!(t.cast(P0, splinters).target(target).try_go().is_err());
    assert!(t.on_battlefield(a));
    // With a second creature, one pays the {B} and the other is the spell's sacrifice.
    let b = t.battlefield(P0, "Grizzly Bears");
    assert!(offered(&mut t, P0, splinters));
    t.cast(P0, splinters).target(target).go();
    assert!(on_stack(&t, "Bone Splinters"));
    assert!(!t.on_battlefield(a) && !t.on_battlefield(b));
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
}

#[test]
fn fling_with_phyrexian_altar_needs_a_creature_for_each_cost() {
    cr!("601.2g", "601.2h", "118.3");
    interp!("automatic-mana-payment");
    interp!("mana-abilities-planned-with-the-objects-their-costs-take");
    ruling!(
        "Fling",
        "You must sacrifice exactly one creature to cast this spell; you can't cast it without sacrificing a creature"
    );
    // Fling ({1}{R}, "As an additional cost to cast this spell, sacrifice a creature."):
    // two activations of Phyrexian Altar and the spell's sacrifice take three creatures.
    let mut t = main_phase();
    t.battlefield(P0, "Phyrexian Altar");
    let giant = t.battlefield(P0, "Hill Giant");
    let a = t.battlefield(P0, "Grizzly Bears");
    let fling = t.hand(P0, "Fling");
    assert!(!offered(&mut t, P0, fling));
    assert!(t.cast(P0, fling).target(P1).try_go().is_err());
    assert!(t.on_battlefield(a) && t.on_battlefield(giant));
    let b = t.battlefield(P0, "Grizzly Bears");
    assert!(offered(&mut t, P0, fling));
    // The Altar's two activations take the Bears (each chosen as it's activated), and the
    // player sacrifices Hill Giant to Fling.
    t.clear_answers();
    for c in [a, b, giant] {
        t.answer_choose(P0, &[Entity::Object(c)]);
    }
    t.cast(P0, fling).target(P1).go();
    assert!(on_stack(&t, "Fling"));
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    assert!(!t.on_battlefield(giant));
    t.resolve();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn a_spell_in_hand_isnt_exiled_for_its_own_mana() {
    cr!("601.2a", "601.2g", "118.3");
    interp!("mana-abilities-planned-with-the-objects-their-costs-take");
    // Cadaverous Bloom ("Exile a card from your hand: Add {B}{B} or {G}{G}.") can't exile
    // the card being cast: it's put on the stack first (CR 601.2a).
    let mut t = main_phase();
    t.battlefield(P0, "Cadaverous Bloom");
    let sign = t.hand(P0, "Sign in Blood");
    assert!(!offered(&mut t, P0, sign));
    assert!(t.cast(P0, sign).target(P1).try_go().is_err());
    assert!(t.in_hand(P0, "Sign in Blood"));
}

#[test]
fn a_convoked_creature_isnt_sacrificed_for_mana() {
    cr!("702.51a", "601.2g", "601.2h");
    interp!("automatic-mana-payment");
    interp!("mana-abilities-planned-with-the-objects-their-costs-take");
    // Stoke the Flames ({2}{R}{R}, convoke) with Raging Goblin convoking {R} and two
    // Savannah Lions {2}: the remaining {R} can't come from Phyrexian Altar sacrificing
    // one of them (they're tapped as the cost is paid, after mana abilities), so Shivan
    // Reef pays it, dealing 1 damage.
    let mut t = main_phase();
    t.battlefield(P0, "Phyrexian Altar");
    let reef = t.battlefield(P0, "Shivan Reef");
    let goblin = t.battlefield(P0, "Raging Goblin");
    let a = t.battlefield(P0, "Savannah Lions");
    let b = t.battlefield(P0, "Savannah Lions");
    let stoke = t.hand(P0, "Stoke the Flames");
    t.answer_choose(
        P0,
        &[Entity::Object(goblin), Entity::Object(a), Entity::Object(b)],
    );
    t.cast(P0, stoke).target(P1).go();
    assert!(on_stack(&t, "Stoke the Flames"));
    for c in [goblin, a, b] {
        assert!(t.on_battlefield(c) && tapped(&t, c));
    }
    assert!(tapped(&t, reef));
    assert_eq!(t.life(P0), 19);
}

#[test]
fn an_improvised_artifact_isnt_sacrificed_for_mana() {
    cr!("702.126a", "601.2g", "601.2h");
    interp!("automatic-mana-payment");
    interp!("mana-abilities-planned-with-the-objects-their-costs-take");
    // Sweatworks Brawler ({3}{R}, improvise) with three Ornithopters improvising {3}: the
    // {R} comes from Shivan Reef, not from Phyrexian Altar sacrificing an Ornithopter.
    let mut t = main_phase();
    t.battlefield(P0, "Phyrexian Altar");
    let reef = t.battlefield(P0, "Shivan Reef");
    let birds: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P0, "Ornithopter")).collect();
    let brawler = t.hand(P0, "Sweatworks Brawler");
    let picks: Vec<Entity> = birds.iter().map(|o| Entity::Object(*o)).collect();
    t.answer_choose(P0, &picks);
    t.cast(P0, brawler).go();
    assert!(on_stack(&t, "Sweatworks Brawler"));
    for c in &birds {
        assert!(t.on_battlefield(*c) && tapped(&t, *c));
    }
    assert!(tapped(&t, reef));
    assert_eq!(t.life(P0), 19);
}

#[test]
fn convokes_default_leaves_out_the_creature_the_mana_needs() {
    cr!("702.51a", "601.2g", "601.2h");
    interp!("default-cost-helpers");
    interp!("mana-abilities-planned-with-the-objects-their-costs-take");
    // Stoke the Flames ({2}{R}{R}) with Ashnod's Altar ("Sacrifice a creature: Add
    // {C}{C}.") and Savannah Lions and two Raging Goblins: the Goblins convoke {R}{R} and
    // the Altar sacrifices the Lions for {2}. Tapping the Lions too would leave nothing for
    // the Altar (CR 601.2g–h), so the default doesn't.
    let mut t = main_phase();
    t.battlefield(P0, "Ashnod's Altar");
    let lions = t.battlefield(P0, "Savannah Lions");
    let a = t.battlefield(P0, "Raging Goblin");
    let b = t.battlefield(P0, "Raging Goblin");
    let stoke = t.hand(P0, "Stoke the Flames");
    t.cast(P0, stoke).target(P1).go();
    assert!(on_stack(&t, "Stoke the Flames"));
    assert!(!t.on_battlefield(lions));
    assert!(tapped(&t, a) && tapped(&t, b));
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
}
