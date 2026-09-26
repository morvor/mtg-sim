//! CR 730: merging with permanents (see also keywords/k702_140_mutate.rs).

use mtg_engine::events::{Event, MoveCause};
use mtg_engine::facedown;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::merge;
use mtg_engine::object::*;
use mtg_engine::replacement::TokenCreate;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;
use smol_str::SmolStr;
use std::sync::Arc;

const MUTATE: CastMethod = CastMethod::Keyword(KeywordKind::Mutate);

/// `p` casts Gemrazer ("Reach, trample; Mutate {1}{G}{G}; Whenever this creature mutates,
/// destroy target artifact or enchantment an opponent controls.") for its mutate cost
/// onto `target`, on top (`on_top`) or under, and it resolves.
fn mutate_gemrazer(t: &mut TestGame, p: PlayerId, target: ObjectId, on_top: bool) {
    mutate_with(t, p, "Gemrazer", target, on_top);
}

/// `p` casts a mutating creature card (Gemrazer or Dreamtail Heron: "Flying; Mutate
/// {3}{U}; Whenever this creature mutates, draw a card.") for its mutate cost onto
/// `target`, and it resolves.
fn mutate_with(t: &mut TestGame, p: PlayerId, name: &str, target: ObjectId, on_top: bool) {
    let gem = t.hand(p, name);
    let (colored, ty, generic) = match name {
        "Dreamtail Heron" => (1, ManaType::U, 3),
        _ => (2, ManaType::G, 1),
    };
    t.g.players[p.idx()].mana_pool.add_type(ty, colored);
    t.g.players[p.idx()].mana_pool.add_type(ManaType::C, generic);
    t.g.turn.priority = Some(p);
    t.cast(p, gem).method(MUTATE).target(target).go();
    t.answer(
        p,
        DecisionKind::Option,
        Answer::Index(if on_top { 0 } else { 1 }),
    );
    t.resolve();
    // (Its mutate trigger has no target.)
    t.resolve_all();
}

/// A 1/1 Soldier creature token for `p`.
fn soldier_token(t: &mut TestGame, p: PlayerId) -> ObjectId {
    let chars = Characteristics {
        name: SmolStr::new("Soldier"),
        card_types: CardTypeSet::single(CardType::Creature),
        subtypes: std::iter::once(SmolStr::new("Soldier").into()).collect(),
        power: Some(1),
        toughness: Some(1),
        rules_text: Arc::from(""),
        ..Default::default()
    };
    let spec = TokenCreate {
        chars,
        card: None,
        tapped: false,
        attacking: None,
        copy_of: None,
        copy_exceptions: vec![],
    };
    let made = t.g.create_tokens(p, spec, 1, None);
    t.g.objects[made[0].0 as usize].summoning_sick = false;
    made[0]
}

fn component_faces_down(t: &TestGame, id: ObjectId) -> Vec<bool> {
    merge::physical_components(&t.g, id)
        .iter()
        .map(|c| t.g.obj(*c).face_down)
        .collect()
}

#[test]
fn mutate_merges_an_object_with_a_permanent() {
    cr!("730.1", "730.2");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    mutate_gemrazer(&mut t, P0, bears, false);
    // One permanent, represented by both cards.
    assert_eq!(t.g.battlefield.iter().filter(|x| t.g.obj(**x).is_creature()).count(), 1);
    assert!(merge::is_merged(&t.g, bears));
    assert_eq!(merge::physical_components(&t.g, bears).len(), 2);
}

#[test]
fn a_merged_permanent_is_a_token_only_if_its_topmost_component_is() {
    cr!("730.2d");
    let mut t = TestGame::new(2);
    let under = soldier_token(&mut t, P0);
    mutate_gemrazer(&mut t, P0, under, true);
    assert!(!t.g.obj(under).is_token());
    assert_eq!(t.g.obj(under).chars.name, "Gemrazer");
    let over = soldier_token(&mut t, P0);
    mutate_gemrazer(&mut t, P0, over, false);
    assert!(t.g.obj(over).is_token());
    assert_eq!(t.g.obj(over).chars.name, "Soldier");
    // As it dies, the token component ceases to exist; the card is in the graveyard.
    t.g.move_object(over, Zone::Graveyard(P0), MoveCause::Destroy, None);
    t.settle();
    assert!(t.in_graveyard(P0, "Gemrazer"));
    assert_eq!(t.graveyard_size(P0), 1);
}

#[test]
fn a_mixed_merged_permanent_is_face_up_or_face_down_as_its_topmost_component() {
    cr!("730.2e");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(facedown::turn_face_down(&mut t.g, bears));
    let events = t.g.turn_events.len();
    // A face-up object merged on top: the permanent is face up, but it wasn't "turned
    // face up".
    mutate_gemrazer(&mut t, P0, bears, true);
    assert!(!t.g.obj(bears).face_down);
    assert_eq!(t.g.obj(bears).chars.name, "Gemrazer");
    assert_eq!(component_faces_down(&t, bears), vec![false, true]);
    assert!(!t.g.turn_events[events..]
        .iter()
        .any(|e| matches!(e, Event::TurnedFaceUp { .. })));
    // Merged under a face-down permanent, the permanent stays face down.
    let other = t.battlefield(P0, "Hill Giant");
    assert!(facedown::turn_face_down(&mut t.g, other));
    mutate_gemrazer(&mut t, P0, other, false);
    assert!(t.g.obj(other).face_down);
    assert_eq!(t.pt(other), (2, 2));
    assert_eq!(component_faces_down(&t, other), vec![true, false]);
}

#[test]
fn turning_a_merged_permanent_face_down_or_up_turns_each_component() {
    cr!("730.2f");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    mutate_gemrazer(&mut t, P0, bears, true);
    assert!(facedown::turn_face_down(&mut t.g, bears));
    t.g.recompute();
    assert_eq!(component_faces_down(&t, bears), vec![true, true]);
    assert_eq!(t.pt(bears), (2, 2));
    assert!(t.g.obj(bears).chars.name.is_empty());
    assert!(facedown::turn_face_up(&mut t.g, bears, false));
    t.g.recompute();
    assert_eq!(component_faces_down(&t, bears), vec![false, false]);
    assert_eq!(t.g.obj(bears).chars.name, "Gemrazer");
    // A face-down permanent with a face-up component under it: turning it face up turns
    // the face-down component face up.
    let giant = t.battlefield(P0, "Hill Giant");
    facedown::turn_face_down(&mut t.g, giant);
    mutate_gemrazer(&mut t, P0, giant, false);
    assert_eq!(component_faces_down(&t, giant), vec![true, false]);
    assert!(facedown::turn_face_up(&mut t.g, giant, false));
    t.g.recompute();
    assert_eq!(component_faces_down(&t, giant), vec![false, false]);
    assert_eq!(t.g.obj(giant).chars.name, "Hill Giant");
}

#[test]
fn a_face_down_merged_permanent_with_an_instant_card_cant_be_turned_face_up() {
    cr!("730.2g");
    let mut t = TestGame::new(2);
    // A manifested Lightning Bolt: a face-down 2/2.
    let bolt = t.battlefield(P0, "Lightning Bolt");
    t.g.objects[bolt.0 as usize].face_down = true;
    t.g.objects[bolt.0 as usize].choices.text = Some("Manifest".into());
    t.g.recompute();
    assert!(t.g.obj(bolt).is_creature());
    mutate_gemrazer(&mut t, P0, bolt, false);
    assert!(t.g.obj(bolt).face_down);
    let events = t.g.turn_events.len();
    // It would turn face up: it's revealed and stays face down; nothing that triggers on
    // turning face up triggers.
    assert!(!facedown::turn_face_up(&mut t.g, bolt, false));
    t.g.flush_events();
    assert!(t.g.obj(bolt).face_down);
    assert_eq!(component_faces_down(&t, bolt), vec![true, false]);
    let after = &t.g.turn_events[events..];
    assert!(!after.iter().any(|e| matches!(e, Event::TurnedFaceUp { .. })));
    assert!(after
        .iter()
        .any(|e| matches!(e, Event::Custom { name, .. } if name == facedown::REVEALED)));
}

#[test]
fn transforming_a_merged_permanent_turns_its_double_faced_components() {
    cr!("730.2i");
    let mut t = TestGame::new(2);
    // Thraben Gargoyle (Defender; "{6}: Transform this creature.") // Stonewing
    // Antagonizer (Flying).
    let gargoyle = t.battlefield(P0, "Thraben Gargoyle");
    mutate_gemrazer(&mut t, P0, gargoyle, true);
    assert!(t.g.obj(gargoyle).has_keyword(KeywordKind::Defender));
    assert!(!t.g.obj(gargoyle).has_keyword(KeywordKind::Flying));
    // The merged permanent isn't a transformed or double-faced permanent, but its
    // Gargoyle component transforms.
    t.lands(P0, "Forest", 6);
    let uid = t
        .g
        .obj(gargoyle)
        .chars
        .abilities
        .iter()
        .find(|a| matches!(a.kind, mtg_engine::ability::AbilityKind::Activated(_)) && a.text.contains("Transform"))
        .map(|a| a.uid)
        .expect("the Gargoyle's transform ability");
    t.g.turn.priority = Some(P0);
    t.g.activate_ability(P0, gargoyle, uid).expect("activate");
    t.resolve_all();
    assert_eq!(t.g.obj(gargoyle).chars.name, "Gemrazer");
    assert!(t.g.obj(gargoyle).has_keyword(KeywordKind::Flying));
    assert!(!t.g.obj(gargoyle).has_keyword(KeywordKind::Defender));
    assert!(t.g.obj(gargoyle).has_keyword(KeywordKind::Reach));
    let comps = merge::physical_components(&t.g, gargoyle);
    assert_eq!(t.g.obj(comps[1]).face, FaceState::Back);
    assert_eq!(t.g.obj(gargoyle).face, FaceState::Front);
}

#[test]
fn a_face_up_merged_permanent_with_a_double_faced_component_cant_be_turned_face_down() {
    cr!("730.2j");
    let mut t = TestGame::new(2);
    let gargoyle = t.battlefield(P0, "Thraben Gargoyle");
    mutate_gemrazer(&mut t, P0, gargoyle, true);
    assert!(!facedown::turn_face_down(&mut t.g, gargoyle));
    assert!(!t.g.obj(gargoyle).face_down);
    assert_eq!(t.g.obj(gargoyle).chars.name, "Gemrazer");
    assert_eq!(component_faces_down(&t, gargoyle), vec![false, false]);
}

#[test]
fn its_owner_arranges_the_cards_of_a_merged_permanent_put_into_a_graveyard_or_library() {
    cr!("730.3a");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    mutate_gemrazer(&mut t, P0, bears, true);
    // Gemrazer is on top; its owner puts the Bears into the graveyard first.
    t.answer(P0, DecisionKind::Order, Answer::Indices(vec![1, 0]));
    t.g.move_object(bears, Zone::Graveyard(P0), MoveCause::Destroy, None);
    let yard: Vec<String> = t
        .g
        .player(P0)
        .graveyard
        .iter()
        .map(|c| t.g.obj(*c).chars.name.to_string())
        .collect();
    assert_eq!(yard, vec!["Grizzly Bears", "Gemrazer"]);
    // On top of the library: the owner arranges them there too.
    let giant = t.battlefield(P0, "Hill Giant");
    mutate_gemrazer(&mut t, P0, giant, false);
    t.answer(P0, DecisionKind::Order, Answer::Indices(vec![0, 1]));
    t.g.move_object(giant, Zone::Library(P0), MoveCause::Effect, None);
    let lib = &t.g.player(P0).library;
    let top2: Vec<String> = lib[lib.len() - 2..]
        .iter()
        .map(|c| t.g.obj(*c).chars.name.to_string())
        .collect();
    assert_eq!(top2, vec!["Hill Giant", "Gemrazer"]);
}

#[test]
fn the_player_exiling_a_merged_permanent_orders_the_cards_timestamps() {
    cr!("730.3b");
    ruling!(
        "Duplicant",
        "If a melded permanent or a merged permanent is exiled by Duplicant's triggered ability, that ability's controller chooses the relative timestamp of the exiled cards."
    );
    let mut t = TestGame::new(2);
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    mutate_gemrazer(&mut t, P1, bears, true);
    // P0 exiles P1's merged permanent and chooses that Gemrazer gets the earlier
    // timestamp.
    t.lands(P0, "Plains", 1);
    let swords = t.hand(P0, "Swords to Plowshares");
    t.cast(P0, swords).target(bears).go();
    let asked_before = t.asked().len();
    t.answer(P0, DecisionKind::Order, Answer::Indices(vec![1, 0]));
    t.resolve_all();
    let order_asked: Vec<PlayerId> = t.asked()[asked_before..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::Order { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(order_asked, vec![P0]);
    let ts = |name: &str| t.g.obj(t.g.find_in_zone(Zone::Exile, name)[0]).timestamp;
    assert!(ts("Grizzly Bears") < ts("Gemrazer"));
    // The other order.
    let mut t2 = TestGame::new(2);
    t2.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    let bears = t2.battlefield(P1, "Grizzly Bears");
    mutate_gemrazer(&mut t2, P1, bears, true);
    t2.lands(P0, "Plains", 1);
    let swords = t2.hand(P0, "Swords to Plowshares");
    t2.cast(P0, swords).target(bears).go();
    t2.answer(P0, DecisionKind::Order, Answer::Indices(vec![0, 1]));
    t2.resolve_all();
    let ts = |name: &str| t2.g.obj(t2.g.find_in_zone(Zone::Exile, name)[0]).timestamp;
    assert!(ts("Gemrazer") < ts("Grizzly Bears"));
}

#[test]
fn an_effect_that_finds_the_new_object_finds_all_of_them() {
    cr!("730.3c");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    mutate_gemrazer(&mut t, P0, bears, true);
    // "Exile target creature you control, then return that card to the battlefield under
    // your control."
    t.lands(P0, "Plains", 1);
    let shift = t.hand(P0, "Cloudshift");
    t.cast(P0, shift).target(bears).go();
    t.resolve_all();
    // Both cards returned, as two permanents.
    assert_eq!(t.named_on_battlefield("Gemrazer").len(), 1);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert!(t.g.exile.is_empty());
}

#[test]
fn a_replacement_effect_applied_to_a_merged_permanent_applies_to_all_its_components() {
    cr!("730.3d");
    let mut t = TestGame::new(2);
    // "If Darksteel Colossus would be put into a graveyard from anywhere, reveal Darksteel
    // Colossus and shuffle it into its owner's library instead."
    let colossus = t.battlefield(P0, "Darksteel Colossus");
    mutate_gemrazer(&mut t, P0, colossus, true);
    let lib = t.library_size(P0);
    t.g.sacrifice(colossus, P0);
    t.settle();
    assert_eq!(t.graveyard_size(P0), 0);
    assert_eq!(t.library_size(P0), lib + 2);
    assert!(t.g.battlefield.iter().all(|x| t.g.obj(*x).owner != P0 || !t.g.obj(*x).is_creature()));
}

#[test]
fn replacement_effects_that_apply_only_to_cards_and_token_components() {
    cr!("730.3e");
    // A merged permanent that isn't a token: all its components, its token too, are
    // exiled by "If a card would be put into an opponent's graveyard from anywhere, exile
    // it instead."
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Leyline of the Void");
    let token = soldier_token(&mut t, P0);
    mutate_with(&mut t, P0, "Dreamtail Heron", token, true);
    t.g.move_object(token, Zone::Graveyard(P0), MoveCause::Destroy, None);
    t.settle();
    assert_eq!(t.graveyard_size(P0), 0);
    assert!(t.in_exile("Dreamtail Heron"));
    // A merged permanent that's a token: it and its token components go to the graveyard
    // (and cease to exist); its card components are exiled.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Leyline of the Void");
    let token = soldier_token(&mut t, P0);
    mutate_with(&mut t, P0, "Dreamtail Heron", token, false);
    assert!(t.g.obj(token).is_token());
    let entered_graveyard = t.g.turn_events.len();
    t.g.move_object(token, Zone::Graveyard(P0), MoveCause::Destroy, None);
    let to_graveyard: Vec<String> = t.g.turn_events[entered_graveyard..]
        .iter()
        .chain(t.g.events.iter())
        .filter_map(|e| match e {
            Event::ZoneChange {
                new,
                to: Zone::Graveyard(_),
                ..
            } => Some(t.g.obj(*new).chars.name.to_string()),
            _ => None,
        })
        .collect();
    assert_eq!(to_graveyard, vec!["Soldier"]);
    t.settle();
    assert!(t.in_exile("Dreamtail Heron"));
    assert_eq!(t.graveyard_size(P0), 0);
}
