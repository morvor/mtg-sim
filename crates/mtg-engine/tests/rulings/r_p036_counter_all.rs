//! Rulings batch P036 — what counts as a mana ability for counters of abilities (CR 605.1a,
//! 605.1b: no target, not loyalty; mana abilities don't use the stack, CR 605.3b), counters
//! of every ability (Kadena's Silencer, Summary Dismissal: CR 701.6b) and what they leave
//! alone, copying and countering prowess, and Voracious Greatshark without a target.

use crate::r_p036_common::*;
use crate::r_s01_common::supported;
use crate::r_s04_common::top_of_stack;
use crate::r_s06_common::activate_containing;
use crate::r_s20_common::tap_for_mana;
use crate::r_s24_common::pool;
use crate::r_s25_common::{cast_new, lands_for_cost};
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// The abilities P1 has on the stack in its main phase, with mana abilities resolved:
/// Chandra, Torch of Defiance's loyalty ability "+1: Add {R}{R}." (on the stack: a
/// loyalty ability isn't a mana ability), Deathrite Shaman's "{T}: Exile target land card
/// from a graveyard. Add one mana of any color." (on the stack: it has a target), and
/// Llanowar Elves's "{T}: Add {G}." and a Forest enchanted with Overgrowth ("Whenever
/// enchanted land is tapped for mana, its controller adds an additional {G}{G}.") tapped
/// for mana (mana abilities: they resolved at once, adding {G}{G}{G}{G}).
struct ManaScene {
    t: TestGame,
    loyalty: ObjectId,
    targeted: ObjectId,
}

fn mana_scene() -> ManaScene {
    supported("Chandra, Torch of Defiance");
    supported("Deathrite Shaman");
    supported("Overgrowth");
    let mut t = TestGame::new(2);
    turn_of(&mut t, P1);
    let chandra = t.battlefield(P1, "Chandra, Torch of Defiance");
    let shaman = t.battlefield(P1, "Deathrite Shaman");
    let elves = t.battlefield(P1, "Llanowar Elves");
    let forest = t.battlefield(P1, "Forest");
    crate::r_s06_common::attach_new(&mut t, P1, "Overgrowth", forest);
    let land = t.graveyard(P0, "Forest");
    let loyalty = activate_containing(&mut t, P1, chandra, "Add {R}{R}")
        .unwrap()
        .expect("a loyalty ability uses the stack");
    t.answer_targets(P1, &[obj(land)]);
    let targeted = activate_containing(&mut t, P1, shaman, "Exile target land")
        .unwrap()
        .expect("an ability with a target uses the stack");
    let stack = t.stack_len();
    assert!(tap_for_mana(&mut t, P1, elves, "Add {G}"));
    assert!(tap_for_mana(&mut t, P1, forest, "Add {G}"));
    t.settle();
    // The mana abilities (and Overgrowth's triggered mana ability) didn't use the stack.
    assert_eq!(t.stack_len(), stack);
    assert_eq!(pool(&t, P1, ManaType::G), 4);
    ManaScene {
        t,
        loyalty,
        targeted,
    }
}

#[test]
fn counters_of_abilities_cant_target_mana_abilities() {
    cr!("605.1a", "605.1b", "605.3b", "115.1");
    ruling!(
        "Tale's End",
        "Tale's End can't target an activated or triggered mana ability. An activated mana ability is one that could add mana as it resolves, doesn't have a target, and isn't a loyalty ability. A triggered mana ability is one that could add mana as it resolves and triggers on an activated mana ability."
    );
    ruling!(
        "Disallow",
        "An activated mana ability is one that could add mana as it resolves, doesn't have a target, and isn't a loyalty ability. A triggered mana ability is one that could add mana and triggers on an activated mana ability."
    );
    ruling!(
        "Squelch",
        "Squelch can’t target a mana ability because mana abilities don’t use the stack."
    );
    ruling!(
        "Nimble Obstructionist",
        "Mana abilities can't be targeted. An activated mana ability is one that could add mana as it resolves, doesn't have a target, and isn't a loyalty ability. A triggered mana ability is one that could add mana as it resolves and triggers on an activated mana ability."
    );
    for name in ["Tale's End", "Disallow", "Squelch"] {
        supported(name);
        let mut s = mana_scene();
        lands_for_cost(&mut s.t, P0, name);
        let card = s.t.hand(P0, name);
        let from = s.t.asked().len();
        s.t.answer_targets(P0, &[obj(s.loyalty)]);
        s.t.cast_with(P0, card, &[]).unwrap();
        let mut c = offered(&s.t, P0, from).remove(0);
        c.sort();
        let mut want = vec![obj(s.loyalty), obj(s.targeted)];
        want.sort();
        assert_eq!(c, want, "{name}");
        s.t.resolve();
        assert!(!on_stack(&s.t, s.loyalty), "{name}");
        s.t.resolve();
        // Chandra's {R}{R} was never added.
        assert_eq!(pool(&s.t, P1, ManaType::R), 0, "{name}");
    }
    // Nimble Obstructionist's cycling trigger.
    supported("Nimble Obstructionist");
    let mut s = mana_scene();
    let card = s.t.hand(P0, "Nimble Obstructionist");
    s.t.lands(P0, "Island", 1);
    s.t.lands(P0, "Wastes", 2);
    let from = s.t.asked().len();
    s.t.answer_targets(P0, &[obj(s.targeted)]);
    crate::r_s04_common::cycle(&mut s.t, P0, card, 0).unwrap();
    s.t.settle();
    let mut c = offered(&s.t, P0, from).remove(0);
    c.sort();
    let mut want = vec![obj(s.loyalty), obj(s.targeted)];
    want.sort();
    assert_eq!(c, want);
}

#[test]
fn kadenas_silencer_counters_loyalty_and_targeted_mana_producers_but_not_mana_abilities() {
    cr!("605.1a", "605.1b", "605.3b", "701.6a", "702.37e");
    ruling!(
        "Kadena's Silencer",
        "Kadena's Silencer can't counter an activated or triggered mana ability. An activated mana ability is one that could add mana as it resolves, doesn't have a target, and isn't a loyalty ability. A triggered mana ability is one that could add mana as it resolves and triggers on an activated mana ability."
    );
    supported("Kadena's Silencer");
    let mut s = mana_scene();
    let silencer = silencer_face_down(&mut s.t);
    turn_up(&mut s.t, silencer);
    assert!(!on_stack(&s.t, s.loyalty) && !on_stack(&s.t, s.targeted));
    // The mana from the mana abilities stays; Chandra's and the Shaman's never comes.
    assert_eq!(pool(&s.t, P1, ManaType::G), 4);
    assert_eq!(pool(&s.t, P1, ManaType::R), 0);
    assert_eq!(s.t.g.player(P1).mana_pool.total(), 4);
    assert!(s.t.g.stack.is_empty());
}

/// P0's Kadena's Silencer face down on the battlefield (cast face down earlier), with
/// {1}{U} available to turn it face up for its megamorph cost.
fn silencer_face_down(t: &mut TestGame) -> ObjectId {
    let card = crate::r_s01_common::place(t, P0, "Kadena's Silencer", Zone::Hand(P0));
    let saved = (t.g.turn.active, t.g.turn.step);
    t.set_step(P0, mtg_engine::turn::Step::PrecombatMain);
    let stack: Vec<_> = std::mem::take(&mut t.g.stack);
    t.lands(P0, "Wastes", 3);
    let spell = t
        .cast(P0, card)
        .method(mtg_engine::object::CastMethod::FaceDown(
            mtg_engine::keywords::KeywordKind::Morph,
        ))
        .go();
    t.resolve();
    let fd = t.g.current(spell);
    assert!(t.obj(fd).face_down && t.on_battlefield(fd));
    t.g.stack = stack;
    t.set_step(saved.0, saved.1);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 1);
    fd
}

/// P0 turns the Silencer face up (a special action) and its "When this creature is turned
/// face up, counter all abilities your opponents control." trigger resolves.
fn turn_up(t: &mut TestGame, silencer: ObjectId) {
    assert!(crate::r_s11_common::turn_face_up(t, P0, silencer));
    t.settle();
    let trig = top_of_stack(t);
    assert_eq!(t.obj(trig).controller, P0);
    t.resolve();
    assert!(!on_stack(t, trig));
}

#[test]
fn kadenas_silencer_counters_abilities_on_the_stack_only() {
    cr!("701.6a", "702.6a", "702.35a", "116.2b", "702.37e", "604.1");
    ruling!(
        "Kadena's Silencer",
        "Kadena's Silencer counters only activated and triggered abilities already on the stack. It doesn't prevent abilities from being activated or from triggering later in the turn, and it has no effect on static abilities."
    );
    ruling!(
        "Kadena's Silencer",
        "Activated abilities are written in the form \"[Cost]: [Effect].\" Some keyword abilities (such as equip) are activated abilities and will have colons in their reminder texts. Notably, turning a creature face up is not an activated ability."
    );
    ruling!(
        "Kadena's Silencer",
        "Triggered abilities use the word \"when,\" \"whenever,\" or \"at.\" They're often written as \"[Trigger condition], [effect].\" Some keyword abilities (such as madness's ability to cast the exiled card) are triggered abilities and will have \"when,\" \"whenever,\" or \"at\" in their reminder text."
    );
    supported("Fiery Temper");
    supported("Hooded Hydra");
    let mut t = TestGame::new(2);
    let silencer = silencer_face_down(&mut t);
    turn_of(&mut t, P1);
    let hydra = crate::r_s12_common::morph(&mut t, P1, "Hooded Hydra");
    t.battlefield(P1, "Glorious Anthem");
    t.battlefield(P1, "Soul Warden");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let dragon = t.battlefield(P1, "Shivan Dragon");
    let bs = t.battlefield(P1, "Bonesplitter");
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    // P1: equip (activated keyword ability), firebreathing, Soul Warden's trigger, and a
    // madness trigger; P0: Prodigal Pyromancer's ping at P1.
    t.lands(P1, "Wastes", 1);
    t.answer_targets(P1, &[obj(bears)]);
    let equip = activate_containing(&mut t, P1, bs, "Equip").unwrap().unwrap();
    t.lands(P1, "Mountain", 2);
    let fire = activate_containing(&mut t, P1, dragon, "gets +1/+0").unwrap().unwrap();
    t.enter(P1, "Hill Giant");
    t.settle();
    let warden_trig = top_of_stack(&t);
    let temper = t.hand(P1, "Fiery Temper");
    t.g.discard(P1, temper, None);
    t.g.flush_events();
    t.settle();
    let madness = top_of_stack(&t);
    assert_ne!(madness, warden_trig);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let ping = activate_containing(&mut t, P0, pyro, "damage").unwrap().unwrap();
    // P1 turns Hooded Hydra face up: a special action, not an activated ability.
    t.lands(P1, "Forest", 2);
    t.lands(P1, "Wastes", 3);
    let before = t.stack_len();
    assert!(crate::r_s11_common::turn_face_up(&mut t, P1, hydra));
    assert_eq!(t.stack_len(), before);
    turn_up(&mut t, silencer);
    for id in [equip, fire, warden_trig, madness] {
        assert!(!on_stack(&t, id));
    }
    assert!(on_stack(&t, ping));
    // Nothing of P1's happened; the Hydra is face up with its counters; static abilities
    // (Glorious Anthem) still apply.
    assert_eq!(crate::r_s06_common::attached_to(&t, bs), None);
    assert_eq!(t.pt(dragon), (6, 6));
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.zone(t.g.current(temper)), Zone::Exile);
    assert!(!t.obj_now(hydra).face_down);
    assert_eq!(t.counters(hydra, "+1/+1"), 5);
    assert_eq!(t.pt(bears), (3, 3));
    t.resolve();
    assert_eq!(t.life(P1), 19);
    // Later in the turn, P1's abilities can be activated and trigger again.
    t.lands(P1, "Mountain", 1);
    activate_containing(&mut t, P1, dragon, "gets +1/+0").unwrap();
    t.resolve();
    assert_eq!(t.pt(dragon), (7, 6));
    t.enter(P1, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
}

#[test]
fn summary_dismissal_leaves_static_abilities_and_resolved_effects() {
    cr!("701.6a", "604.1", "608.2c");
    ruling!(
        "Summary Dismissal",
        "Only activated and triggered abilities on the stack are countered. Static abilities of objects remain unaffected, and activated and triggered abilities of objects may be activated or may trigger again later in the turn. Spells and abilities that have already resolved aren’t affected."
    );
    supported("Summary Dismissal");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Glorious Anthem");
    t.battlefield(P1, "Soul Warden");
    let dragon = t.battlefield(P1, "Shivan Dragon");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // A Giant Growth that already resolved.
    cast_new(&mut t, P0, "Giant Growth", &[obj(bears)]);
    t.resolve();
    assert_eq!(t.pt(bears), (5, 5));
    // On the stack: P1's firebreathing, Soul Warden's trigger, and a Lightning Bolt.
    t.lands(P1, "Mountain", 1);
    let fire = activate_containing(&mut t, P1, dragon, "gets +1/+0").unwrap().unwrap();
    t.enter(P1, "Hill Giant");
    t.settle();
    let trig = top_of_stack(&t);
    let bolt = cast_new(&mut t, P1, "Lightning Bolt", &[obj(bears)]);
    cast_new(&mut t, P0, "Summary Dismissal", &[]);
    t.resolve();
    assert!(t.g.stack.is_empty());
    assert!(!on_stack(&t, fire) && !on_stack(&t, trig) && !on_stack(&t, bolt));
    assert_eq!(t.zone(t.g.current(bolt)), Zone::Exile);
    assert_eq!(t.life(P1), 20);
    // Static: Glorious Anthem; resolved: Giant Growth.
    assert_eq!(t.pt(dragon), (6, 6));
    assert_eq!(t.pt(bears), (5, 5));
    // Again later in the turn.
    t.lands(P1, "Mountain", 1);
    activate_containing(&mut t, P1, dragon, "gets +1/+0").unwrap();
    t.resolve();
    assert_eq!(t.pt(dragon), (7, 6));
    t.enter(P1, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.life(P1), 21);
}

#[test]
fn prowess_is_a_triggered_ability_to_copy_or_counter() {
    cr!("702.108a", "707.10", "701.6a");
    ruling!(
        "Peter Parker's Camera",
        "Triggered abilities use the words \"when,\" \"whenever,\" or \"at.\" They're often written as \"[Trigger condition], [effect].\" Some keyword abilities (such as prowess) are triggered abilities and will have \"when,\" \"whenever,\" or \"at the beginning of\" in their reminder text."
    );
    ruling!(
        "Spider-Sense",
        "Triggered abilities use the words \"when,\" \"whenever,\" or \"at.\" They're often written as \"[Trigger condition], [effect].\" Some keyword abilities (such as prowess) are triggered abilities and will have \"when,\" \"whenever,\" or \"at the beginning of\" in their reminder text."
    );
    supported("Peter Parker's Camera");
    supported("Spider-Sense");
    // Camera: "{2}, {T}, Remove a film counter from this artifact: Copy target activated
    // or triggered ability you control."
    let mut t = TestGame::new(2);
    let camera = t.enter(P0, "Peter Parker's Camera");
    assert_eq!(t.counters(camera, "film"), 3);
    let spear = t.battlefield(P0, "Monastery Swiftspear");
    cast_new(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    t.settle();
    let trig = top_of_stack(&t);
    t.lands(P0, "Wastes", 2);
    t.answer_targets(P0, &[obj(trig)]);
    activate_containing(&mut t, P0, camera, "Copy target").unwrap();
    t.resolve_all();
    assert_eq!(t.counters(camera, "film"), 2);
    assert_eq!(t.pt(spear), (3, 4));
    // Spider-Sense counters P1's prowess trigger.
    let mut t = TestGame::new(2);
    turn_of(&mut t, P1);
    let spear = t.battlefield(P1, "Monastery Swiftspear");
    cast_new(&mut t, P1, "Shock", &[Entity::Player(P0)]);
    t.settle();
    let trig = top_of_stack(&t);
    cast_new(&mut t, P0, "Spider-Sense", &[obj(trig)]);
    t.resolve_all();
    assert_eq!(t.pt(spear), (1, 2));
    assert_eq!(t.life(P0), 18);
}

#[test]
fn voracious_greatshark_can_be_cast_with_nothing_to_counter() {
    cr!("603.3d");
    ruling!(
        "Voracious Greatshark",
        "You can cast Voracious Greatshark without a target for its enters-the-battlefield triggered ability. That ability will simply not happen. It won't cause Voracious Greatshark to counter itself. It's not that hungry."
    );
    supported("Voracious Greatshark");
    let mut t = TestGame::new(2);
    let shark = cast_new(&mut t, P0, "Voracious Greatshark", &[]);
    t.resolve();
    let shark = t.g.current(shark);
    assert!(t.on_battlefield(shark));
    assert!(t.g.stack.is_empty());
    // Control: with P1's creature spell below it, the trigger counters that spell.
    let mut t = TestGame::new(2);
    let bears = bears_spell(&mut t, P1);
    let shark = cast_new(&mut t, P0, "Voracious Greatshark", &[]);
    t.answer_targets(P0, &[obj(bears)]);
    t.resolve();
    assert!(t.on_battlefield(t.g.current(shark)));
    t.resolve();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}
