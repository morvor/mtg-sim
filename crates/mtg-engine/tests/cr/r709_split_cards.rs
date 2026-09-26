//! CR 709: split cards, including split permanent cards with a shared type line (Rooms,
//! CR 709.5).

use super::r709_common::*;
use mtg_engine::ability::*;
use mtg_engine::card::{card, Layout};
use mtg_engine::copy_rules::NamedCopy;
use mtg_engine::decision::{Action, SpecialAction};
use mtg_engine::eval::Ctx;
use mtg_engine::mana::{ManaSymbol, ManaType};
use mtg_engine::object::*;
use mtg_engine::rooms;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn value(t: &mut TestGame, p: PlayerId, v: Value) -> i64 {
    t.g.recompute();
    t.g.eval_value(&v, &Ctx::new(None, p))
}

#[test]
fn a_split_card_is_one_card_with_two_faces() {
    cr!("709.1", "709.2");
    ruling!("Fire // Ice", "Each split card is a single card");
    let def = card("Fire // Ice");
    assert_eq!(def.layout, Layout::Split);
    assert_eq!(def.faces.len(), 2);
    assert_eq!(def.faces[0].chars.name, "Fire");
    assert_eq!(def.faces[1].chars.name, "Ice");
    // It has a normal Magic card back: it isn't a double-faced card.
    assert!(!def.layout.is_double_faced());
    let mut t = TestGame::new(2);
    t.library_top(P0, "Fire // Ice");
    let before = t.library_size(P0);
    run_effect(
        &mut t,
        P0,
        None,
        &[],
        Effect::Draw {
            who: PlayerRef::You,
            n: Value::c(1),
        },
    );
    // Drawing it is drawing one card.
    assert_eq!(t.library_size(P0), before - 1);
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(value(&mut t, P0, Value::CardsDrawnThisTurn(PlayerRef::You)), 1);
    // In the graveyard it's one instant card, not two.
    let fi = t.g.player(P0).hand[0];
    t.g.discard(P0, fi, None);
    assert_eq!(t.graveyard_size(P0), 1);
    assert_eq!(
        value(
            &mut t,
            P0,
            Value::CardsInGraveyard(PlayerRef::You, Filter::Type(CardType::Instant))
        ),
        1
    );
}

#[test]
fn a_player_chooses_which_half_to_cast() {
    cr!("709.3", "709.3a", "709.3b");
    ruling!("Fire // Ice", "To cast a split card, choose one of its halves to cast");
    ruling!(
        "Fire // Ice",
        "The characteristics of the half you didn't cast are ignored while the spell is on the stack"
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let fi = t.hand(P0, "Fire // Ice");
    // The two ways to cast it are its two halves.
    let methods: Vec<CastMethod> = t
        .g
        .cast_options(P0, fi)
        .into_iter()
        .map(|o| o.method)
        .collect();
    assert_eq!(methods, vec![CastMethod::Half(0), CastMethod::Half(1)]);
    // With only {U}{U} available, only Ice ({1}{U}) can be cast: only the chosen half's
    // cost is evaluated, not the combined {1}{R}{1}{U}.
    t.lands(P0, "Island", 2);
    assert!(!can_cast_face(&mut t, P0, fi, FaceState::Half(0)));
    assert!(can_cast_face(&mut t, P0, fi, FaceState::Half(1)));
    assert!(t.cast(P0, fi).method(CastMethod::Half(0)).try_go().is_err());
    let fi = t.g.current(fi);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let spell = t
        .cast(P0, fi)
        .method(CastMethod::Half(1))
        .target(bears)
        .go();
    // On the stack it has only Ice's characteristics.
    let c = &t.obj(spell).chars;
    assert_eq!(c.name, "Ice");
    assert!(!c.has_name("Fire"));
    assert_eq!(c.colors, ColorSet::single(Color::Blue));
    assert_eq!(c.abilities.len(), card("Fire // Ice").faces[1].chars.abilities.len());
    assert_eq!(mv(&mut t, spell), 2);
    t.resolve_all();
    assert!(t.obj_now(bears).tapped);
    assert!(t.in_graveyard(P0, "Fire // Ice"));
}

#[test]
fn a_copy_of_a_split_card_can_be_cast_as_either_half() {
    cr!("709.3c");
    let mut t = TestGame::new(2);
    let fi = t.graveyard(P0, "Fire // Ice");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // "Copy target card in your graveyard. You may cast the copy without paying its mana
    // cost": the copy keeps both halves, and the player chooses to cast Ice.
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.answer_targets(P0, &[Entity::Object(bears)]);
    run_effect(
        &mut t,
        P0,
        None,
        &[Entity::Object(fi)],
        Effect::Seq(vec![
            Effect::CopyCard {
                what: Sel::Target(0),
                named: None,
            },
            Effect::CastCard {
                who: PlayerRef::You,
                what: Sel::Var(vars::CREATED),
                free: true,
                optional: true,
            },
        ]),
    );
    assert_eq!(t.stack_len(), 1);
    let copy = t.g.stack[0];
    assert_eq!(t.obj(copy).kind, ObjKind::CardCopy);
    assert_eq!(t.obj(copy).chars.name, "Ice");
    t.resolve_all();
    assert!(t.obj_now(bears).tapped);
}

#[test]
fn a_copy_of_a_split_spell_copies_the_same_half() {
    cr!("709.3b");
    ruling!("Fire // Ice", "If you copy a spell that's half of a split card, the copy copies that same half");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Island", 2);
    let fi = t.hand(P0, "Fire // Ice");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let spell = cast_half_targeting(&mut t, fi, 1, bears);
    run_effect(
        &mut t,
        P0,
        None,
        &[Entity::Object(spell)],
        Effect::CopySpell {
            what: Sel::Target(0),
            count: Value::c(1),
            new_targets: false,
        },
    );
    let copy = t.g.stack[1];
    assert_eq!(t.obj(copy).chars.name, "Ice");
    assert_eq!(mv(&mut t, copy), 2);
}

fn cast_half_targeting(t: &mut TestGame, card: ObjectId, half: u8, target: ObjectId) -> ObjectId {
    t.cast(P0, card)
        .method(CastMethod::Half(half))
        .target(target)
        .go()
}

#[test]
fn off_the_stack_a_split_card_has_both_halves_characteristics() {
    cr!("709.4", "709.4b", "709.4c");
    ruling!(
        "Fire // Ice",
        "A split card's characteristics are a combination of its two halves while it is not on the stack"
    );
    let mut t = TestGame::new(2);
    let fi = t.hand(P0, "Fire // Ice");
    t.g.recompute();
    let c = t.obj(fi).chars.clone();
    // Colors and mana value come from the combined mana cost {1}{R} + {1}{U}.
    assert_eq!(c.colors, ColorSet::single(Color::Red).union(ColorSet::single(Color::Blue)));
    assert_eq!(mv(&mut t, fi), 4);
    // "Search for a card with mana value 3 or less" can't find it.
    let ctx = Ctx::new(None, P0);
    assert!(!t
        .g
        .matches(fi, &Filter::ManaValue(Cmp::Le, Box::new(Value::c(3))), &ctx));
    // The symbols of each half are seen separately: two generic {1} symbols, not {2}.
    let syms = &c.mana_cost.as_ref().unwrap().symbols;
    assert_eq!(syms.len(), 4);
    assert_eq!(
        syms.iter()
            .filter(|s| matches!(s, ManaSymbol::Generic(1)))
            .count(),
        2
    );
    // Each ability of each half.
    let def = card("Fire // Ice");
    assert_eq!(
        c.abilities.len(),
        def.faces[0].chars.abilities.len() + def.faces[1].chars.abilities.len()
    );
    // Each card type of either half: Commit // Memory is an instant card and a sorcery
    // card while it isn't on the stack.
    let cm = t.graveyard(P0, "Commit // Memory");
    t.g.recompute();
    assert!(t.obj(cm).chars.is(CardType::Instant));
    assert!(t.obj(cm).chars.is(CardType::Sorcery));
    assert_eq!(mv(&mut t, cm), 10);
}

#[test]
fn a_split_card_has_two_names() {
    cr!("709.4a");
    ruling!("Fire // Ice", "you may choose one of those names, but not both");
    let mut t = TestGame::new(2);
    let fi = t.hand(P1, "Fire // Ice");
    t.g.recompute();
    // It has each half's name.
    let ctx = Ctx::new(None, P0);
    assert!(t.g.matches(fi, &Filter::Named("Fire".into()), &ctx));
    assert!(t.g.matches(fi, &Filter::Named("Ice".into()), &ctx));
    // Naming both isn't a legal choice.
    name_card(&mut t, P0, "Fire // Ice");
    let mage = t.enter(P0, "Meddling Mage");
    assert_eq!(t.obj_now(mage).choices.card_name.as_deref(), Some(""));
    // Naming one half: "Spells with the chosen name can't be cast." Ice can't be cast;
    // Fire, which doesn't have that name on the stack, can.
    name_card(&mut t, P0, "Ice");
    let mage = t.enter(P0, "Meddling Mage");
    assert_eq!(t.obj_now(mage).choices.card_name.as_deref(), Some("Ice"));
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 2);
    t.lands(P1, "Island", 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(t
        .cast(P1, fi)
        .method(CastMethod::Half(1))
        .target(bears)
        .try_go()
        .is_err());
    let fi = t.g.current(fi);
    let r = t.cast(P1, fi).method(CastMethod::Half(0)).target(bears).try_go();
    assert!(r.is_ok(), "{r:?}");
}

#[test]
fn a_fused_split_spell_has_both_halves_characteristics() {
    cr!("709.4d");
    ruling!(
        "Turn // Burn",
        "While the spell is on the stack, its mana value is the total amount of mana in both costs"
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Plains", 1);
    let wt = t.hand(P0, "Wear // Tear");
    let bauble = t.battlefield(P1, "Ornithopter");
    let aura = t.battlefield(P1, "Glorious Anthem");
    let spell = t
        .cast(P0, wt)
        .method(CastMethod::Keyword(mtg_engine::keywords::KeywordKind::Fuse))
        .target(bauble)
        .target(aura)
        .go();
    let c = t.obj(spell).chars.clone();
    assert!(c.has_name("Wear") && c.has_name("Tear"));
    assert_eq!(c.colors, ColorSet::single(Color::Red).union(ColorSet::single(Color::White)));
    assert_eq!(mv(&mut t, spell), 3);
    t.resolve_all();
    assert!(!t.on_battlefield(bauble));
    assert!(!t.on_battlefield(aura));
}

// ---------------------------------------------------------------------------
// Split cards with a shared type line: Rooms (CR 709.5)
// ---------------------------------------------------------------------------

fn unlock_door(half: usize, room: ObjectId) -> SpecialAction {
    SpecialAction::Other {
        name: format!("{}{half}", rooms::UNLOCK_ACTION),
        obj: Some(room),
    }
}

#[test]
fn each_door_has_the_shared_type_line() {
    cr!("709.5a");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let card_id = t.hand(P0, "Bottomless Pool // Locker Room");
    add_mana(&mut t, P0, ManaType::U, 5);
    // Locker Room, the right half, cast on its own: an Enchantment — Room spell.
    let spell = cast_half(&mut t, P0, card_id, 1);
    let c = t.obj(spell).chars.clone();
    assert_eq!(c.name, "Locker Room");
    assert!(c.is(CardType::Enchantment) && c.has_subtype("Room"));
    t.resolve_all();
    let room = the(&t, "Locker Room");
    let c = &t.obj(room).chars;
    assert!(c.is(CardType::Enchantment) && c.has_subtype("Room"));
    assert!(!c.has_name("Bottomless Pool"));
}

#[test]
fn a_copy_of_a_room_spell_enters_with_that_door_unlocked() {
    cr!("709.5b", "709.5d");
    ruling!(
        "Glassworks // Shattered Yard",
        "the copy retains the choice of which door was cast but also retains the full characteristics of the spell"
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let card_id = t.hand(P0, "Bottomless Pool // Locker Room");
    add_mana(&mut t, P0, ManaType::U, 5);
    let spell = cast_half(&mut t, P0, card_id, 1);
    run_effect(
        &mut t,
        P0,
        None,
        &[Entity::Object(spell)],
        Effect::CopySpell {
            what: Sel::Target(0),
            count: Value::c(1),
            new_targets: false,
        },
    );
    let copy = t.g.stack[1];
    assert_eq!(t.obj(copy).chars.name, "Locker Room");
    t.resolve_all();
    // The copy became a token Room with Locker Room unlocked, and it still has both
    // doors: Bottomless Pool can be unlocked.
    let token = t
        .g
        .battlefield
        .iter()
        .copied()
        .find(|id| t.obj(*id).kind == ObjKind::Token)
        .expect("token Room");
    assert_eq!(rooms::unlocked(&t.g, token), [false, true]);
    assert_eq!(t.obj(token).chars.name, "Locker Room");
    add_mana(&mut t, P0, ManaType::U, 1);
    t.g.turn.priority = Some(P0);
    t.g.perform_action(P0, Action::Special(unlock_door(0, token)))
        .unwrap();
    t.g.recompute();
    assert!(t.obj(token).chars.has_name("Bottomless Pool"));
    assert!(t.obj(token).chars.has_name("Locker Room"));
}

#[test]
fn a_permanent_copying_a_room_has_both_doors_and_its_own_designations() {
    cr!("709.5", "709.5b", "709.5c");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let card_id = t.hand(P0, "Bottomless Pool // Locker Room");
    add_mana(&mut t, P0, ManaType::U, 5);
    cast_half(&mut t, P0, card_id, 1);
    t.resolve_all();
    let room = the(&t, "Locker Room");
    // Copy Enchantment enters as a copy of the Room. It wasn't cast as a door, so both of
    // its doors are locked: it's a Room enchantment with no name and no abilities.
    t.answer(P0, DecisionKind::YesNo, Answer::Bool(true));
    t.answer_choose(P0, &[Entity::Object(room)]);
    let copy = t.enter(P0, "Copy Enchantment");
    t.g.recompute();
    let c = t.obj(copy).chars.clone();
    assert!(c.is(CardType::Enchantment) && c.has_subtype("Room"));
    assert_eq!(c.name, "");
    assert!(c.abilities.is_empty());
    assert_eq!(rooms::unlocked(&t.g, copy), [false, false]);
    // Its doors are the copied Room's: it can unlock Bottomless Pool for {U}.
    add_mana(&mut t, P0, ManaType::U, 1);
    t.g.turn.priority = Some(P0);
    t.g.perform_action(P0, Action::Special(unlock_door(0, copy)))
        .unwrap();
    t.g.recompute();
    assert_eq!(t.obj(copy).chars.name, "Bottomless Pool");
    assert_eq!(t.obj(copy).chars.abilities.len(), 1);
    // The original is unaffected.
    assert_eq!(t.obj(room).chars.name, "Locker Room");
}

#[test]
fn effects_can_lock_and_unlock_doors() {
    cr!("709.5f", "709.5g", "709.5h", "709.5j");
    ruling!(
        "Glassworks // Shattered Yard",
        "You can't choose to lock a door that's already locked with such an ability"
    );
    supported("Keys to the House");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let ogre = t.battlefield(P1, "Gray Ogre");
    let card_id = t.hand(P0, "Glassworks // Shattered Yard");
    add_mana(&mut t, P0, ManaType::R, 3);
    t.answer_targets(P0, &[Entity::Object(ogre)]);
    cast_half(&mut t, P0, card_id, 0);
    t.resolve_all();
    assert!(!t.on_battlefield(ogre));
    let room = the(&t, "Glassworks");
    // Keys to the House: "{3}, {T}, Sacrifice this artifact: Lock or unlock a door of
    // target Room you control." Locking: only an unlocked door can be chosen.
    let keys = t.battlefield(P0, "Keys to the House");
    add_mana(&mut t, P0, ManaType::C, 3);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    let ks = keys_index(&mut t, keys);
    t.activate(P0, keys, ks, &[Entity::Object(room)]).unwrap();
    t.resolve_all();
    assert_eq!(rooms::unlocked(&t.g, room), [false, false]);
    assert_eq!(t.obj(room).chars.name, "");
    // Unlocking a door with an effect is unlocking it: Glassworks's unlock ability
    // triggers again.
    let bears = t.battlefield(P1, "Grizzly Bears");
    let keys = t.battlefield(P0, "Keys to the House");
    add_mana(&mut t, P0, ManaType::C, 3);
    // Unlock Glassworks (the options are the two locked doors).
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.activate(P0, keys, ks, &[Entity::Object(room)]).unwrap();
    t.resolve_all();
    assert_eq!(rooms::unlocked(&t.g, room), [true, false]);
    assert!(!t.on_battlefield(bears));
}

fn keys_index(t: &mut TestGame, keys: ObjectId) -> usize {
    t.g.recompute();
    t.obj(keys)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .position(|a| a.text.contains("unlock"))
        .expect("lock or unlock ability")
}

#[test]
fn unlocking_a_locked_door_with_an_effect() {
    cr!("709.5f", "709.5i");
    ruling!(
        "Glassworks // Shattered Yard",
        "You can't choose to unlock a door that's already unlocked with such an ability"
    );
    supported("Ghostly Keybearer");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    // Entity Tracker draws a card whenever you fully unlock a Room.
    t.battlefield(P0, "Entity Tracker");
    for _ in 0..3 {
        t.library_top(P0, "Island");
    }
    let card_id = t.hand(P0, "Bottomless Pool // Locker Room");
    add_mana(&mut t, P0, ManaType::U, 5);
    cast_half(&mut t, P0, card_id, 1);
    t.resolve_all();
    let hand = t.hand_size(P0);
    let room = the(&t, "Locker Room");
    // Ghostly Keybearer: "Whenever this creature deals combat damage to a player, unlock
    // a locked door of up to one target Room you control." The only locked door is
    // Bottomless Pool.
    let kb = t.battlefield(P0, "Ghostly Keybearer");
    t.set_step(P0, Step::PrecombatMain);
    t.answer_targets(P0, &[Entity::Object(room)]);
    t.attack(&[(kb, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(rooms::unlocked(&t.g, room), [true, true]);
    // The Room became fully unlocked (Entity Tracker drew), and Locker Room's own
    // combat-damage trigger also drew a card.
    assert_eq!(t.hand_size(P0), hand + 2);
    assert!(t.obj_now(room).chars.has_name("Bottomless Pool"));
}
