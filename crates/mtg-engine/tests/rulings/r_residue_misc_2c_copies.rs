//! Item `rulings-residue-misc-2`, pass C — copies: Moritte of the Frost copying a permanent a
//! type-changing effect made a creature, Shelob's Food copies, a copy of a suspected
//! creature, Olag (Ludevic's back face) copying a transforming card, Dinosaur Headdress's
//! choice each time it's attached, and Locus of Enlightenment's gained activated abilities.

use crate::r_residue_misc_2c_common::*;
use mtg_engine::ability::AbilityKind;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::kwa::suspect_detain::is_suspected;
use mtg_engine::object::{FaceState, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// --- Moritte of the Frost ---------------------------------------------------------------------

#[test]
fn moritte_copying_a_crewed_vehicle_enters_as_a_noncreature_without_counters() {
    cr!("707.2", "613.1a", "614.1c", "702.122a");
    ruling!(
        "Moritte of the Frost",
        "If the copied permanent is affected by a type-changing effect, Moritte may enter the battlefield with different permanent types than the copied permanent currently has. Use the characteristics of Moritte as it enters the battlefield, not of the copied permanent, to determine whether it enters with an additional counter on it. Notably, if Moritte copies a permanent that's normally not a creature but has been made a creature by another effect, Moritte enters as a noncreature permanent that doesn't get +1/+1 counters or have changeling."
    );
    let mut t = TestGame::new(2);
    let copter = t.battlefield(P0, "Smuggler's Copter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(bears)]);
    activate_containing(&mut t, P0, copter, "Crew").expect("crew");
    t.resolve_all();
    assert!(is_creature(&mut t, copter));
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(copter)]);
    let m = t.enter(P0, "Moritte of the Frost");
    t.settle();
    let o = t.obj_now(m);
    assert_eq!(o.chars.name, "Smuggler's Copter", "{}", t.dump_log());
    assert!(!o.is(CardType::Creature));
    assert!(o.chars.supertypes.contains(Supertype::Legendary));
    assert!(o.chars.supertypes.contains(Supertype::Snow));
    assert_eq!(t.counters(m, counters::PLUS1), 0);
    assert!(!o.has_keyword(KeywordKind::Changeling));
    // Copying the Bears: a creature, with the counters and changeling.
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    let m2 = t.enter(P0, "Moritte of the Frost");
    t.settle();
    assert_eq!(t.counters(m2, counters::PLUS1), 2);
    assert!(t.obj_now(m2).has_keyword(KeywordKind::Changeling));
}

// --- Shelob, Child of Ungoliant ---------------------------------------------------------------

#[test]
fn shelob_food_copy_copies_only_the_copiable_values_and_cant_attack_or_block() {
    cr!("707.2", "707.9b", "111.10b", "508.1a", "509.1a");
    ruling!(
        "Shelob, Child of Ungoliant",
        "Except for being a Food artifact, having the listed activated ability, and losing all other card types and subtypes, the token copies only what was printed on the original creature (unless that creature is copying something else; see below). It doesn't copy whether that creature was tapped or untapped, whether it had any counters on it or Auras or Equipment attached to it, or any non-copy effects that had changed its power, toughness, types, color, or so on."
    );
    ruling!(
        "Shelob, Child of Ungoliant",
        "Since the token is a Food artifact and not a creature, it can't attack or block unless it becomes a creature somehow."
    );
    supported("Shelob, Child of Ungoliant");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Shelob, Child of Ungoliant");
    let spider = t.battlefield(P0, "Giant Spider");
    let giant = t.battlefield(P1, "Hill Giant");
    // Tapped, with counters and a pump that also makes it green and blue.
    crate::r_s26_common::dress_up(&mut t, giant);
    // And a Clone (P1's) copying a Serra Angel.
    let angel = t.battlefield(P1, "Serra Angel");
    let clone = clone_of(&mut t, P1, angel);
    for victim in [giant, clone] {
        t.g.deal_damage(spider, obj(t.g.current(victim)), 1, false);
        t.g.flush_events();
        kill(&mut t, victim);
        t.resolve_all();
    }
    let food_giant = named_by(&t, P0, "Hill Giant");
    assert_eq!(food_giant.len(), 1, "{}", t.dump_log());
    let f = food_giant[0];
    let o = t.obj_now(f);
    assert!(o.is_token() && o.is(CardType::Artifact) && !o.is(CardType::Creature));
    assert!(o.chars.has_subtype("Food") && !o.chars.has_subtype("Giant"));
    assert!(!o.tapped);
    assert_eq!(t.counters(f, counters::PLUS1), 0);
    assert_eq!(o.chars.colors, ColorSet::single(Color::Red));
    assert!(o
        .chars
        .abilities
        .iter()
        .any(|a| matches!(a.kind, AbilityKind::Activated(_)) && a.text.contains("3 life")));
    // The Clone's copy: a Serra Angel Food.
    let food_angel = named_by(&t, P0, "Serra Angel");
    assert_eq!(food_angel.len(), 1);
    assert!(!t.obj_now(food_angel[0]).is(CardType::Creature));
    // Neither can attack or block.
    assert!(!can_attack(&mut t, f));
    assert!(!t.g.can_block_at_all(f));
}

// --- Hot Pursuit: a copy of a suspected creature -----------------------------------------------

#[test]
fn a_copy_of_a_suspected_creature_isnt_suspected() {
    cr!("701.60c", "707.2");
    ruling!(
        "Hot Pursuit",
        "Being suspected isn’t a copiable value. If a permanent becomes a copy of a suspected creature, it won’t be suspected."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    t.enter(P0, "Hot Pursuit");
    t.resolve_all();
    assert!(is_suspected(&t.g, bears));
    let clone = clone_of(&mut t, P0, bears);
    assert!(!is_suspected(&t.g, clone));
    assert!(!t.obj_now(clone).has_keyword(KeywordKind::Menace));
    assert!(t.g.can_block_at_all(clone));
}

// --- Ludevic, Necrogenius // Olag, Ludevic's Hubris -------------------------------------------

const LUDEVIC: &str = "Ludevic, Necrogenius // Olag, Ludevic's Hubris";

/// P0's Ludevic transforms with X = the cards `exiled` (from P0's graveyard), Olag copying
/// the first one (if any is still exiled). With `remove_first`, the chosen card leaves exile
/// in response. Returns Olag.
fn olag(t: &mut TestGame, exiled: &[&str], remove: bool) -> ObjectId {
    supported(LUDEVIC);
    let ludevic = t.battlefield(P0, LUDEVIC);
    let cards: Vec<ObjectId> = exiled.iter().map(|n| t.graveyard(P0, n)).collect();
    t.lands(P0, "Island", 2);
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Wastes", cards.len());
    t.answer(
        P0,
        DecisionKind::X,
        mtg_engine::decision::Answer::Number(cards.len() as i64),
    );
    let es: Vec<Entity> = cards.iter().map(|c| obj(*c)).collect();
    t.answer_choose(P0, &es);
    t.answer_choose_moved(P0, &es[..1]);
    t.activate(P0, ludevic, 0, &[]).expect("Ludevic");
    if remove {
        for c in &cards {
            mv(t, *c, Zone::Graveyard(P0));
        }
    }
    t.resolve_all();
    let o = t.g.current(ludevic);
    assert_eq!(
        t.obj(o).chars.name,
        "Olag, Ludevic's Hubris",
        "{}",
        t.dump_log()
    );
    o
}

#[test]
fn olag_with_no_exiled_creature_cards_is_a_plain_4_4() {
    cr!("712.20", "707.9b", "608.2b");
    ruling!(
        "Ludevic, Necrogenius // Olag, Ludevic's Hubris",
        "If there are no creature cards exiled with Ludevic as it transforms, perhaps because those cards were removed from exile in response to its activated ability or it transformed another way, Olag won't be a copy of anything. Olag will simply be its 4/4 self with no +1/+1 counters."
    );
    let mut t = TestGame::new(2);
    let o = olag(&mut t, &["Hill Giant"], true);
    assert_eq!(t.pt(o), (4, 4));
    assert_eq!(t.counters(o, counters::PLUS1), 0);
    assert!(!t.obj_now(o).chars.has_subtype("Giant"));
}

#[test]
fn olag_copying_a_daybound_card_doesnt_transform_at_night() {
    cr!("702.145b", "702.145c", "712.20");
    ruling!(
        "Ludevic, Necrogenius // Olag, Ludevic's Hubris",
        "It's possible for Olag to become a copy of a card with the daybound keyword ability as it transforms. If this happens, it doesn't transform as it becomes night."
    );
    let mut t = TestGame::new(2);
    let o = olag(&mut t, &["Brutal Cathar // Moonrage Brute"], false);
    assert!(t.obj_now(o).has_keyword(KeywordKind::DayboundAndNightbound));
    assert_eq!(t.obj_now(o).face, FaceState::Back);
    t.g.day = Some(false);
    t.g.recompute();
    t.settle();
    assert_eq!(t.obj_now(o).face, FaceState::Back, "{}", t.dump_log());
    assert_eq!(t.obj_now(o).chars.name, "Olag, Ludevic's Hubris");
}

#[test]
fn olag_copying_a_transforming_card_transforms_and_stays_the_copy() {
    cr!("712.20", "701.27a", "707.2");
    ruling!(
        "Ludevic, Necrogenius // Olag, Ludevic's Hubris",
        "If Olag becomes a copy of another transforming double-faced card that doesn't have nightbound, such as Delver of Secrets, abilities of it gains or other effects may cause it to transform. If it transforms this way, it does not become Ludevic again. The copy effect continues to apply and its characteristics don't change. If the creature it is a copy of has a triggered ability that occurs when it transforms, that ability will trigger."
    );
    // Brigid, Clachan's Heart: "Whenever this creature enters or transforms into Brigid,
    // Clachan's Heart, create a 1/1 green and white Kithkin creature token. At the beginning
    // of your first main phase, you may pay {G}. If you do, transform Brigid."
    let mut t = TestGame::new(2);
    let o = olag(
        &mut t,
        &["Brigid, Clachan's Heart // Brigid, Doun's Mind"],
        false,
    );
    assert_eq!(t.pt(o), (5, 5));
    let kithkin = tokens_with(&t, P0, "Kithkin").len();
    t.lands(P0, "Forest", 1);
    t.answer_yes(P0, true);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    let now = t.obj_now(o);
    assert_eq!(now.face, FaceState::Front, "{}", t.dump_log());
    assert_eq!(now.chars.name, "Olag, Ludevic's Hubris");
    assert!(now.chars.has_subtype("Zombie") && now.chars.has_subtype("Kithkin"));
    assert_eq!(t.pt(o), (5, 5));
    assert_eq!(tokens_with(&t, P0, "Kithkin").len(), kithkin + 1);
}

// --- Dinosaur Headdress -----------------------------------------------------------------------

#[test]
fn dinosaur_headdress_chooses_an_exiled_card_each_time_it_becomes_attached() {
    cr!("702.167c", "707.2", "301.5c");
    ruling!(
        "Paleontologist's Pick-Axe // Dinosaur Headdress",
        "Each time Dinosaur Headdress becomes attached to a creature, you choose one of the exiled creature cards used to craft it. You may choose the same card you chose the last time you had to choose."
    );
    const AXE: &str = "Paleontologist's Pick-Axe // Dinosaur Headdress";
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let axe = t.battlefield(P0, AXE);
    let giant = t.battlefield(P0, "Hill Giant");
    let lions = t.graveyard(P0, "Savannah Lions");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wurm = t.battlefield(P0, "Craw Wurm");
    t.lands(P0, "Wastes", 9);
    t.answer_choose(P0, &[obj(giant), obj(lions)]);
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_choose_moved(P0, &[obj(giant)]);
    activate_containing(&mut t, P0, axe, "Craft").expect("craft");
    t.resolve_all();
    assert_eq!(
        t.obj_now(bears).chars.name,
        "Hill Giant",
        "{}",
        t.dump_log()
    );
    let hd = t.g.current(axe);
    // Equipped to the Wurm: the Lions this time.
    t.answer_targets(P0, &[obj(wurm)]);
    t.answer_choose_moved(P0, &[obj(lions)]);
    activate_containing(&mut t, P0, hd, "Equip").expect("equip");
    t.resolve_all();
    assert_eq!(
        t.obj_now(wurm).chars.name,
        "Savannah Lions",
        "{}",
        t.dump_log()
    );
    assert_eq!(t.obj_now(bears).chars.name, "Grizzly Bears");
    // Back to the Bears: the Lions again (the same card as last time).
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_choose_moved(P0, &[obj(lions)]);
    activate_containing(&mut t, P0, hd, "Equip").expect("equip");
    t.resolve_all();
    assert_eq!(
        t.obj_now(bears).chars.name,
        "Savannah Lions",
        "{}",
        t.dump_log()
    );
}

// --- Locus of Enlightenment -------------------------------------------------------------------

const JEWEL: &str = "The Enigma Jewel // Locus of Enlightenment";

/// P0 crafts Locus of Enlightenment with the real cards `names` (on the battlefield).
fn locus_with(t: &mut TestGame, names: &[&str]) -> ObjectId {
    supported(JEWEL);
    t.set_step(P0, Step::PrecombatMain);
    let jewel = t.battlefield(P0, JEWEL);
    let mats: Vec<Entity> = names.iter().map(|n| obj(t.battlefield(P0, n))).collect();
    t.lands(P0, "Island", 9);
    t.answer_choose(P0, &mats);
    activate_containing(t, P0, jewel, "Craft").expect("craft");
    t.clear_answers();
    t.resolve_all();
    let locus = t.g.current(jewel);
    assert_eq!(
        t.obj(locus).chars.name,
        "Locus of Enlightenment",
        "{}",
        t.dump_log()
    );
    locus
}

/// The texts of the activated abilities of the object now.
fn activated_texts(t: &mut TestGame, id: ObjectId) -> Vec<String> {
    t.g.recompute();
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .map(|a| a.text.to_string())
        .collect()
}

#[test]
fn locus_of_enlightenment_has_only_the_activated_abilities_its_own_name_referenced() {
    cr!("702.167c", "113.7", "201.5");
    ruling!(
        "The Enigma Jewel // Locus of Enlightenment",
        "Locus of Enlightenment has only the activated abilities of the exiled cards used to craft it. It doesn't gain keyword abilities (unless those keyword abilities are activated), triggered abilities, or static abilities."
    );
    ruling!(
        "The Enigma Jewel // Locus of Enlightenment",
        "If an activated ability of an exiled card used to craft Locus of Enlightenment references the card it's printed on by name, treat Locus of Enlightenment's version of that ability as though it referenced Locus of Enlightenment instead."
    );
    // Birds of Paradise (flying, a mana ability), Prodigal Sorcerer ({T}: 1 damage),
    // Bonesplitter (a static +2/+0, equip), Shivan Dragon (flying, firebreathing).
    let mut t = TestGame::new(2);
    let locus = locus_with(
        &mut t,
        &[
            "Birds of Paradise",
            "Prodigal Sorcerer",
            "Bonesplitter",
            "Shivan Dragon",
        ],
    );
    let texts = activated_texts(&mut t, locus);
    assert!(texts.iter().any(|s| s.contains("Equip")), "{texts:?}");
    assert!(texts.iter().any(|s| s.contains("damage")), "{texts:?}");
    let o = t.obj_now(locus);
    assert!(!o.has_keyword(KeywordKind::Flying));
    assert!(!o.chars.abilities.iter().any(|a| matches!(
        a.kind,
        AbilityKind::Static(_) | AbilityKind::Triggered(_)
    ) && !a.text.contains("copy it")
        && !a.text.contains("each activated ability")));
    // The Sorcerer's ability on Locus: Locus deals the damage (and so does the copy Locus's
    // own ability makes).
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_yes(P0, false);
    activate_containing(&mut t, P0, locus, "damage").expect("ping");
    t.resolve_all();
    assert_eq!(t.life(P1), 18, "{}", t.dump_log());
    let sources: Vec<ObjectId> =
        t.g.turn_events
            .iter()
            .filter_map(|e| match e {
                mtg_engine::events::Event::Damage { source, .. } => Some(*source),
                _ => None,
            })
            .collect();
    assert_eq!(sources, vec![locus, locus]);
}

#[test]
fn locus_of_enlightenment_activates_each_instance_once_each_turn() {
    cr!("702.167c", "602.5b");
    ruling!(
        "The Enigma Jewel // Locus of Enlightenment",
        "The \"only once each turn\" restriction applies to each ability individually, even if multiples of the same card were exiled to craft Locus of Enlightenment. For example, if two of the same card were exiled to craft Locus of Enlightenment, it will have two instances of each of that card's activated abilities. Each of those abilities may be activated once each turn."
    );
    let mut t = TestGame::new(2);
    let locus = locus_with(
        &mut t,
        &[
            "Prodigal Sorcerer",
            "Prodigal Sorcerer",
            "Llanowar Elves",
            "Bonesplitter",
        ],
    );
    let pings: Vec<u64> = {
        t.g.recompute();
        t.obj_now(locus)
            .chars
            .abilities
            .iter()
            .filter(|a| matches!(a.kind, AbilityKind::Activated(_)) && a.text.contains("damage"))
            .map(|a| a.uid)
            .collect()
    };
    assert_eq!(pings.len(), 2);
    for uid in &pings {
        t.answer_targets(P0, &[pl(P1)]);
        t.answer_yes(P0, false);
        t.g.turn.priority = Some(P0);
        // ({T}: the Locus untaps between the two, as an effect would.)
        t.g.objects[locus.0 as usize].tapped = false;
        t.g.activate_ability(P0, locus, *uid).expect("each once");
        t.g.flush_events();
        t.resolve_all();
    }
    t.g.objects[locus.0 as usize].tapped = false;
    t.g.turn.priority = Some(P0);
    assert!(t.g.activate_ability(P0, locus, pings[0]).is_err());
    assert!(t.g.activate_ability(P0, locus, pings[1]).is_err());
}
