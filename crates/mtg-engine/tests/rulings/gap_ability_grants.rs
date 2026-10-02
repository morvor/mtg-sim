//! Rulings on objects that have or gain the abilities of other objects (gap-ability-grants,
//! CR 113.10, 613.1f, 201.5b, 607.5): Necrotic Ooze, Experiment Kraj, Quicksilver
//! Elemental, Agatha's Soul Cauldron, Koh, Steward of the Harvest, Manascape Refractor,
//! Skill Borrower, Myr Welder, Idris; and linked abilities across a created token (Ugin,
//! the Ineffable, CR 603.7).

use crate::r_s01_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::{Action, SpecialAction};
use mtg_engine::events::MoveCause;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// The texts of the activated abilities `id` has.
fn activated(t: &mut TestGame, id: ObjectId) -> Vec<String> {
    t.g.recompute();
    t.g.obj(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .map(|a| a.text.clone())
        .collect()
}

fn triggered(t: &mut TestGame, id: ObjectId) -> Vec<String> {
    t.g.recompute();
    t.g.obj(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Triggered(_)))
        .map(|a| a.text.clone())
        .collect()
}

/// The index among `id`'s activated abilities of the first one whose text contains `text`.
fn index_of(t: &mut TestGame, id: ObjectId, text: &str) -> usize {
    let all = activated(t, id);
    all.iter()
        .position(|a| a.contains(text))
        .unwrap_or_else(|| panic!("no activated ability {text:?} among {all:?}"))
}

fn has_keyword(t: &mut TestGame, id: ObjectId, k: KeywordKind) -> bool {
    t.g.recompute();
    t.g.obj(id).chars.has_keyword(k)
}

fn pool(t: &TestGame, p: PlayerId, ty: ManaType) -> usize {
    t.g.player(p).mana_pool.count(ty)
}

// ---------------------------------------------------------------------------
// Necrotic Ooze, Myr Welder: abilities of cards in graveyards or exiled with it
// ---------------------------------------------------------------------------

#[test]
fn necrotic_ooze_regenerates_itself_with_a_graveyard_cards_ability() {
    ruling!(
        "Necrotic Ooze",
        "as though it referenced Necrotic Ooze by name instead"
    );
    supported("Necrotic Ooze");
    let mut t = TestGame::new(2);
    let ooze = t.battlefield(P0, "Necrotic Ooze");
    let troll = t.graveyard(P1, "Cudgel Troll");
    t.lands(P0, "Forest", 1);
    let i = index_of(&mut t, ooze, "Regenerate");
    t.activate(P0, ooze, i, &[]).unwrap();
    t.resolve_all();
    t.g.destroy(ooze, None);
    t.settle();
    assert!(t.on_battlefield(ooze), "Necrotic Ooze has the regeneration shield");
    // Cudgel Troll's card isn't affected.
    assert_eq!(t.zone(troll), Zone::Graveyard(P1));
}

#[test]
fn necrotic_ooze_gains_only_activated_abilities() {
    ruling!("Necrotic Ooze", "Necrotic Ooze gains only activated abilities.");
    ruling!(
        "Necrotic Ooze",
        "Some keywords are activated abilities; they have colons in their reminder text."
    );
    let mut t = TestGame::new(2);
    let ooze = t.battlefield(P0, "Necrotic Ooze");
    // Shivan Dragon: flying (a static keyword) and a firebreathing activated ability.
    t.graveyard(P0, "Shivan Dragon");
    // Krosan Tusker: cycling (an activated keyword) and a triggered ability.
    t.graveyard(P0, "Krosan Tusker");
    // Lord of Atlantis: a static ability.
    t.graveyard(P1, "Lord of Atlantis");
    let merfolk = t.battlefield(P0, "Merfolk of the Pearl Trident");
    assert!(activated(&mut t, ooze).iter().any(|a| a.contains("+1/+0")));
    assert!(!has_keyword(&mut t, ooze, KeywordKind::Flying));
    assert!(has_keyword(&mut t, ooze, KeywordKind::Cycling));
    assert!(triggered(&mut t, ooze).is_empty());
    assert_eq!(t.pt(merfolk), (1, 1), "no lord's static ability");
}

#[test]
fn myr_welder_puts_charge_counters_on_itself() {
    ruling!(
        "Myr Welder",
        "treat Myr Welder's version of that ability as though it referenced Myr Welder by name instead"
    );
    ruling!(
        "Myr Welder",
        "It doesn't gain triggered abilities or static abilities."
    );
    supported("Myr Welder");
    let mut t = TestGame::new(2);
    let welder = t.battlefield(P0, "Myr Welder");
    let cannon = t.graveyard(P1, "Lux Cannon");
    let mine = t.graveyard(P1, "Howling Mine");
    for card in [cannon, mine] {
        let i = index_of(&mut t, welder, "Exile target artifact card");
        t.activate(P0, welder, i, &[Entity::Object(card)]).unwrap();
        t.resolve_all();
        t.g.untap(welder);
    }
    assert!(t.in_exile("Lux Cannon") && t.in_exile("Howling Mine"));
    // Lux Cannon's two activated abilities; not Howling Mine's triggered ability.
    assert_eq!(activated(&mut t, welder).len(), 3);
    assert!(triggered(&mut t, welder).is_empty());
    let i = index_of(&mut t, welder, "Put a charge counter");
    t.activate(P0, welder, i, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(welder, "charge"), 1);
}

// ---------------------------------------------------------------------------
// Experiment Kraj and Quicksilver Elemental
// ---------------------------------------------------------------------------

#[test]
fn experiment_kraj_pays_gained_costs_with_the_right_colors() {
    ruling!(
        "Experiment Kraj",
        "the costs of the activated abilities Experiment Kraj gains must be paid with the correct colors of mana"
    );
    supported("Experiment Kraj");
    supported("Quicksilver Elemental");
    let mut t = TestGame::new(2);
    let kraj = t.battlefield(P0, "Experiment Kraj");
    let troll = t.battlefield(P1, "Cudgel Troll");
    t.g.add_counters(Entity::Object(troll), "+1/+1", 1, None);
    t.lands(P0, "Island", 3);
    // Kraj has "{G}: Regenerate" but only blue mana to pay for it.
    let i = index_of(&mut t, kraj, "Regenerate");
    assert!(t.activate(P0, kraj, i, &[]).is_err());
    // Quicksilver Elemental may spend blue mana as though it were any color.
    let qe = t.battlefield(P0, "Quicksilver Elemental");
    let i = index_of(&mut t, qe, "gains all activated abilities");
    t.activate(P0, qe, i, &[Entity::Object(troll)]).unwrap();
    t.resolve_all();
    let i = index_of(&mut t, qe, "Regenerate");
    t.activate(P0, qe, i, &[]).unwrap();
    t.resolve_all();
    t.g.destroy(qe, None);
    t.settle();
    assert!(t.on_battlefield(qe));
}

#[test]
fn quicksilver_elemental_gains_an_opponents_creatures_abilities_as_its_own() {
    ruling!(
        "Quicksilver Elemental",
        "can gain the activated abilities of any creature on the battlefield that you can target with its ability, even if you don't control that creature"
    );
    ruling!(
        "Quicksilver Elemental",
        "so you treat the abilities as if they were printed on Quicksilver Elemental"
    );
    ruling!(
        "Quicksilver Elemental",
        "Quicksilver Elemental gains only activated abilities."
    );
    let mut t = TestGame::new(2);
    let qe = t.battlefield(P0, "Quicksilver Elemental");
    let husk = t.battlefield(P1, "Nantuko Husk");
    let dragon = t.battlefield(P1, "Shivan Dragon");
    let fodder = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    for c in [husk, dragon] {
        let i = index_of(&mut t, qe, "gains all activated abilities");
        t.activate(P0, qe, i, &[Entity::Object(c)]).unwrap();
        t.resolve_all();
    }
    // Firebreathing, but not flying.
    assert!(!has_keyword(&mut t, qe, KeywordKind::Flying));
    let base = t.pt(qe);
    t.answer_choose(P0, &[Entity::Object(fodder)]);
    let i = index_of(&mut t, qe, "Sacrifice a creature");
    t.activate(P0, qe, i, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(qe), (base.0 + 2, base.1 + 2), "Quicksilver Elemental gets +2/+2");
    assert_eq!(t.pt(husk), (2, 2), "not Nantuko Husk");
}

#[test]
fn quicksilver_elemental_collects_copies_of_a_once_each_turn_ability() {
    ruling!(
        "Quicksilver Elemental",
        "If you make two copies of an ability that can be activated once a turn, you can activate each of them once a turn."
    );
    ruling!(
        "Quicksilver Elemental",
        "collecting abilities from multiple creatures (or the same creature more than once)"
    );
    let mut t = TestGame::new(2);
    let qe = t.battlefield(P0, "Quicksilver Elemental");
    let rootwalla = t.battlefield(P1, "Rootwalla");
    t.lands(P0, "Island", 2);
    t.lands(P0, "Forest", 4);
    for _ in 0..2 {
        let i = index_of(&mut t, qe, "gains all activated abilities");
        t.activate(P0, qe, i, &[Entity::Object(rootwalla)]).unwrap();
        t.resolve_all();
    }
    let base = t.pt(qe);
    for k in 0..2 {
        let idx: Vec<usize> = activated(&mut t, qe)
            .iter()
            .enumerate()
            .filter(|(_, a)| a.contains("+2/+2"))
            .map(|(i, _)| i)
            .collect();
        assert_eq!(idx.len(), 2);
        t.activate(P0, qe, idx[k], &[]).unwrap();
        t.resolve_all();
    }
    assert_eq!(t.pt(qe), (base.0 + 4, base.1 + 4));
}

#[test]
fn quicksilver_elemental_turned_face_down_cant_be_turned_face_up() {
    ruling!(
        "Quicksilver Elemental",
        "you will not be able to turn this card face up because it will not have the Morph ability"
    );
    supported("Wall of Deceit");
    let mut t = TestGame::new(2);
    let qe = t.battlefield(P0, "Quicksilver Elemental");
    let wall = t.battlefield(P1, "Wall of Deceit");
    t.lands(P0, "Island", 5);
    let i = index_of(&mut t, qe, "gains all activated abilities");
    t.activate(P0, qe, i, &[Entity::Object(wall)]).unwrap();
    t.resolve_all();
    let i = index_of(&mut t, qe, "face down");
    t.activate(P0, qe, i, &[]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(qe).face_down);
    let actions = t.g.legal_actions(P0);
    assert!(!actions
        .iter()
        .any(|a| matches!(a, Action::Special(SpecialAction::TurnFaceUp { obj }) if *obj == qe)));
}

// ---------------------------------------------------------------------------
// Agatha's Soul Cauldron
// ---------------------------------------------------------------------------

/// Exiles `card` with the Cauldron, putting the reflexive +1/+1 counter on `creature`.
fn cauldron_exiles(t: &mut TestGame, cauldron: ObjectId, card: ObjectId, creature: ObjectId) {
    t.g.untap(cauldron);
    t.activate(P0, cauldron, 0, &[Entity::Object(card)]).unwrap();
    t.answer_targets(P0, &[Entity::Object(creature)]);
    t.resolve_all();
}

#[test]
fn agathas_soul_cauldron_grants_only_activated_abilities() {
    ruling!(
        "Agatha's Soul Cauldron",
        "Agatha's Soul Cauldron grants only activated abilities."
    );
    supported("Agatha's Soul Cauldron");
    let mut t = TestGame::new(2);
    let cauldron = t.battlefield(P0, "Agatha's Soul Cauldron");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Llanowar Elves");
    let dragon = t.graveyard(P1, "Shivan Dragon");
    cauldron_exiles(&mut t, cauldron, dragon, bears);
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    // The creature with a +1/+1 counter has firebreathing but not flying; the other
    // creature has neither.
    assert!(activated(&mut t, bears).iter().any(|a| a.contains("+1/+0")));
    assert!(!has_keyword(&mut t, bears, KeywordKind::Flying));
    assert!(!activated(&mut t, other).iter().any(|a| a.contains("+1/+0")));
    // A noncreature card exiled this way gives nothing and puts no counter.
    let mine = t.graveyard(P1, "Howling Mine");
    cauldron_exiles(&mut t, cauldron, mine, other);
    assert_eq!(t.counters(other, "+1/+1"), 0);
}

#[test]
fn agathas_soul_cauldron_abilities_refer_to_the_creature_that_has_them() {
    ruling!(
        "Agatha's Soul Cauldron",
        "so you treat the abilities as though they were printed on the creature that gained the ability"
    );
    let mut t = TestGame::new(2);
    let cauldron = t.battlefield(P0, "Agatha's Soul Cauldron");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let sprite = t.graveyard(P1, "Argothian Sprite");
    cauldron_exiles(&mut t, cauldron, sprite, bears);
    // "{7}: Put two +1/+1 counters on this creature", paid with any mana (the Cauldron
    // lets mana be spent as though it were any color).
    t.lands(P0, "Island", 7);
    let i = index_of(&mut t, bears, "Put two +1/+1 counters");
    t.activate(P0, bears, i, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 3);
}

// ---------------------------------------------------------------------------
// Koh, the Face Stealer
// ---------------------------------------------------------------------------

#[test]
fn koh_cant_exile_a_card_that_left_the_graveyard() {
    ruling!(
        "Koh, the Face Stealer",
        "If the card leaves the graveyard before the ability resolves, Koh will not exile the card"
    );
    supported("Koh, the Face Stealer");
    let mut t = TestGame::new(2);
    let koh = t.battlefield(P0, "Koh, the Face Stealer");
    let sorcerer = t.battlefield(P1, "Prodigal Sorcerer");
    t.g.destroy(sorcerer, None);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // In response, the card leaves the graveyard.
    let card = t.g.find_in_zone(Zone::Graveyard(P1), "Prodigal Sorcerer")[0];
    t.g.move_object(card, Zone::Hand(P1), MoveCause::Effect, None);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(!t.in_exile("Prodigal Sorcerer"));
    // There's no card to choose: Koh gains nothing.
    t.activate(P0, koh, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(activated(&mut t, koh).len(), 1);
}

#[test]
fn koh_gains_triggered_keyword_abilities_of_the_chosen_card() {
    ruling!(
        "Koh, the Face Stealer",
        "Some keyword abilities, such as prowess, are triggered abilities"
    );
    let mut t = TestGame::new(2);
    let koh = t.battlefield(P0, "Koh, the Face Stealer");
    let monk = t.battlefield(P1, "Monastery Swiftspear");
    t.g.destroy(monk, None);
    t.answer_yes(P0, true);
    t.resolve_all();
    let card = t.g.find_in_zone(Zone::Exile, "Monastery Swiftspear")[0];
    t.answer_choose(P0, &[Entity::Object(card)]);
    t.activate(P0, koh, 0, &[]).unwrap();
    t.resolve_all();
    // Prowess (a triggered keyword) but not haste (a static one).
    assert!(has_keyword(&mut t, koh, KeywordKind::Prowess));
    assert!(!has_keyword(&mut t, koh, KeywordKind::Haste));
    let base = t.pt(koh);
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.pt(koh), (base.0 + 1, base.1 + 1));
}

// ---------------------------------------------------------------------------
// Idris, Soul of the TARDIS
// ---------------------------------------------------------------------------

/// Casts Idris; its imprint ability exiles `artifact`. Returns Idris.
fn idris_exiling(t: &mut TestGame, artifact: ObjectId) -> ObjectId {
    let idris = t.hand(P0, "Idris, Soul of the TARDIS");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Mountain", 2);
    t.cast(P0, idris).go();
    t.resolve();
    t.answer_choose(P0, &[Entity::Object(artifact)]);
    t.resolve_all();
    t.g.current(idris)
}

#[test]
fn idris_counts_x_as_zero_in_the_exiled_cards_mana_value() {
    ruling!(
        "Idris, Soul of the TARDIS",
        "If the exiled card has an {X} in its mana cost, the value of X is 0 when determining that card's mana value."
    );
    supported("Idris, Soul of the TARDIS");
    let mut t = TestGame::new(2);
    let chalice = t.battlefield(P0, "Chalice of the Void");
    let idris = idris_exiling(&mut t, chalice);
    assert!(t.in_exile("Chalice of the Void"));
    assert_eq!(t.pt(idris), (3, 3));
}

#[test]
fn idris_gains_activated_keyword_abilities_like_equip() {
    ruling!(
        "Idris, Soul of the TARDIS",
        "Some keyword abilities, like equip, are activated abilities"
    );
    let mut t = TestGame::new(2);
    let splitter = t.battlefield(P0, "Bonesplitter");
    let idris = idris_exiling(&mut t, splitter);
    assert!(has_keyword(&mut t, idris, KeywordKind::Equip));
    // "Equipped creature gets +2/+0" is a static ability: Bonesplitter's mana value 1.
    assert_eq!(t.pt(idris), (4, 4));
}

// ---------------------------------------------------------------------------
// Steward of the Harvest and Manascape Refractor: lands' abilities
// ---------------------------------------------------------------------------

/// Steward of the Harvest enters and exiles `lands` from P0's graveyard.
fn steward_exiling(t: &mut TestGame, lands: &[ObjectId]) -> ObjectId {
    let targets: Vec<Entity> = lands.iter().map(|l| Entity::Object(*l)).collect();
    t.answer_targets(P0, &targets);
    let steward = t.enter(P0, "Steward of the Harvest");
    t.resolve_all();
    steward
}

#[test]
fn steward_of_the_harvest_gives_lands_intrinsic_and_named_abilities() {
    ruling!(
        "Steward of the Harvest",
        "Land cards with one or more basic land types have the intrinsic activated ability"
    );
    ruling!(
        "Steward of the Harvest",
        "treat the version of that ability on each creature you control as though it referenced the creature that has it by name instead"
    );
    supported("Steward of the Harvest");
    let mut t = TestGame::new(2);
    let forest = t.graveyard(P0, "Forest");
    let vault = t.graveyard(P0, "Mutavault");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let steward = steward_exiling(&mut t, &[forest, vault]);
    assert!(t.in_exile("Forest") && t.in_exile("Mutavault"));
    for c in [bears, steward] {
        assert!(activated(&mut t, c).iter().any(|a| a.contains("Add {G}")));
    }
    let i = index_of(&mut t, bears, "Add {G}");
    t.activate(P0, bears, i, &[]).unwrap();
    assert_eq!(pool(&t, P0, ManaType::G), 1);
    // Mutavault's "{1}: This land becomes a 2/2 creature with all creature types": the
    // Bears become one (paid with the {G} just added).
    let i = index_of(&mut t, bears, "becomes a 2/2");
    t.activate(P0, bears, i, &[]).unwrap();
    t.resolve_all();
    t.g.recompute();
    assert!(t.g.obj(bears).chars.all_creature_types);
}

#[test]
fn steward_of_the_harvest_gives_only_activated_abilities() {
    ruling!(
        "Steward of the Harvest",
        "Creatures you control gain only activated abilities of the exiled land cards."
    );
    ruling!(
        "Steward of the Harvest",
        "Some keywords are activated abilities; they have colons in their reminder text."
    );
    let mut t = TestGame::new(2);
    // Lotus Field: hexproof, an enters trigger, and a mana ability.
    let field = t.graveyard(P0, "Lotus Field");
    // Ash Barrens: basic landcycling (an activated keyword).
    let barrens = t.graveyard(P0, "Ash Barrens");
    let bears = t.battlefield(P0, "Grizzly Bears");
    steward_exiling(&mut t, &[field, barrens]);
    assert!(activated(&mut t, bears)
        .iter()
        .any(|a| a.contains("three mana of any one color")));
    assert!(has_keyword(&mut t, bears, KeywordKind::Cycling));
    assert!(!has_keyword(&mut t, bears, KeywordKind::Hexproof));
    assert!(triggered(&mut t, bears).is_empty());
}

#[test]
fn manascape_refractor_has_lands_intrinsic_mana_abilities_but_not_triggered_ones() {
    ruling!(
        "Manascape Refractor",
        "Lands with a basic land type have an intrinsic activated mana ability corresponding to their basic land type. Manascape Refractor has those abilities, too."
    );
    ruling!(
        "Manascape Refractor",
        "Manascape Refractor won't gain triggered abilities"
    );
    supported("Manascape Refractor");
    let mut t = TestGame::new(2);
    let refractor = t.battlefield(P0, "Manascape Refractor");
    t.battlefield(P1, "Island");
    t.battlefield(P1, "Lotus Field");
    let i = index_of(&mut t, refractor, "Add {U}");
    t.activate(P0, refractor, i, &[]).unwrap();
    assert_eq!(pool(&t, P0, ManaType::U), 1);
    assert!(triggered(&mut t, refractor).is_empty());
    assert!(!has_keyword(&mut t, refractor, KeywordKind::Hexproof));
}

#[test]
fn manascape_refractor_keeps_linked_activated_abilities_linked() {
    ruling!(
        "Manascape Refractor",
        "those two abilities Manascape Refractor gains are linked for as long as that card remains on the battlefield"
    );
    let mut t = TestGame::new(2);
    let refractor = t.battlefield(P0, "Manascape Refractor");
    // A land with two linked activated abilities (CR 607.2a).
    let land = custom_card(
        "Echo Vault",
        "Land",
        "",
        None,
        "{T}: Exile target card from a graveyard.\n{T}: You gain 1 life for each card exiled with Echo Vault.",
    );
    let vault = t.custom(P1, land, Zone::Battlefield);
    let bolt = t.graveyard(P1, "Lightning Bolt");
    let i = index_of(&mut t, refractor, "Exile target card");
    t.activate(P0, refractor, i, &[Entity::Object(bolt)]).unwrap();
    t.resolve_all();
    assert!(t.in_exile("Lightning Bolt"));
    t.g.untap(refractor);
    let i = index_of(&mut t, refractor, "for each card exiled");
    t.activate(P0, refractor, i, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 21, "the card the Refractor exiled");
    // The land's own ability isn't linked to the Refractor's exile.
    let i = index_of(&mut t, vault, "for each card exiled");
    t.activate(P1, vault, i, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    // Once the land leaves the battlefield, the Refractor loses both abilities.
    t.g.move_object(vault, Zone::Graveyard(P1), MoveCause::Effect, None);
    assert!(!activated(&mut t, refractor)
        .iter()
        .any(|a| a.contains("exiled with")));
}

#[test]
fn manascape_refractor_ability_linked_to_a_non_activated_ability_is_unlinked() {
    ruling!(
        "Manascape Refractor",
        "the ability Manascape Refractor has isn't linked to any ability"
    );
    let mut t = TestGame::new(2);
    let refractor = t.battlefield(P0, "Manascape Refractor");
    // "As this land enters, choose a color." / "{T}: Add one mana of the chosen color."
    t.answer(
        P1,
        DecisionKind::Option,
        mtg_engine::decision::Answer::Index(2),
    );
    let vale = t.enter(P1, "Shimmerdrift Vale");
    t.g.untap(vale);
    let i = index_of(&mut t, vale, "chosen color");
    t.activate(P1, vale, i, &[]).unwrap();
    assert_eq!(t.g.player(P1).mana_pool.total(), 1, "the Vale's own choice");
    // The Refractor's copy refers to an undefined choice: it adds nothing (CR 607.5a).
    let i = index_of(&mut t, refractor, "chosen color");
    t.activate(P0, refractor, i, &[]).unwrap();
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
}

// ---------------------------------------------------------------------------
// Skill Borrower: the top card of the library
// ---------------------------------------------------------------------------

#[test]
fn skill_borrower_has_cycling_it_cant_use() {
    ruling!(
        "Skill Borrower",
        "Skill Borrower may gain activated abilities that it can"
    );
    supported("Skill Borrower");
    let mut t = TestGame::new(2);
    let borrower = t.battlefield(P0, "Skill Borrower");
    t.library_top(P0, "Krosan Tusker");
    assert!(has_keyword(&mut t, borrower, KeywordKind::Cycling));
    t.lands(P0, "Forest", 3);
    let i = index_of(&mut t, borrower, "Cycling");
    assert!(t.activate(P0, borrower, i, &[]).is_err(), "cycling works only from a hand");
}

#[test]
fn skill_borrower_abilities_follow_the_top_card() {
    ruling!(
        "Skill Borrower",
        "will resolve normally even though Skill Borrower has lost that ability"
    );
    ruling!(
        "Skill Borrower",
        "Skill Borrower will momentarily gain the activated abilities of artifact and creature cards revealed this way"
    );
    let mut t = TestGame::new(2);
    let borrower = t.battlefield(P0, "Skill Borrower");
    // Library, top first: Prodigal Sorcerer, Grizzly Bears, Island.
    stack_library(&mut t, P0, &["Prodigal Sorcerer", "Grizzly Bears", "Island"]);
    let i = index_of(&mut t, borrower, "deals 1 damage");
    t.activate(P0, borrower, i, &[Entity::Player(P1)]).unwrap();
    // The top card changes while the ability is on the stack.
    t.g.draw_cards(P0, 1);
    assert!(!activated(&mut t, borrower)
        .iter()
        .any(|a| a.contains("deals 1 damage")));
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    // Grizzly Bears (a creature card without activated abilities) is on top, then an
    // Island: Skill Borrower has no other abilities after drawing them.
    t.g.draw_cards(P0, 2);
    assert!(activated(&mut t, borrower).is_empty());
}

// ---------------------------------------------------------------------------
// Ugin, the Ineffable: "When that token leaves the battlefield"
// ---------------------------------------------------------------------------

/// The Spirit tokens on the battlefield.
fn spirits(t: &TestGame) -> Vec<ObjectId> {
    t.g.battlefield
        .iter()
        .copied()
        .filter(|o| t.g.obj(*o).chars.has_subtype("Spirit"))
        .collect()
}

/// Activates Ugin's +1 ability once and resolves it; returns the Spirit token(s) created.
fn ugin_plus(t: &mut TestGame, ugin: ObjectId) -> Vec<ObjectId> {
    let before = spirits(t);
    let i = index_of(t, ugin, "Exile the top card");
    t.activate(P0, ugin, i, &[]).unwrap();
    t.resolve_all();
    spirits(t)
        .into_iter()
        .filter(|s| !before.contains(s))
        .collect()
}

#[test]
fn ugins_spirit_returns_the_card_even_after_ugin_left() {
    ruling!(
        "Ugin, the Ineffable",
        "If the Spirit leaves the battlefield for any reason, you'll put the exiled card into your hand, even if Ugin left the battlefield before that happened."
    );
    ruling!(
        "Ugin, the Ineffable",
        "Once you look at the exiled face-down card once, you may look at it again any time you wish."
    );
    supported("Ugin, the Ineffable");
    let mut t = TestGame::new(2);
    let ugin = t.battlefield(P0, "Ugin, the Ineffable");
    let lib = stack_library(&mut t, P0, &["Lightning Bolt", "Island"]);
    let spirits = ugin_plus(&mut t, ugin);
    assert_eq!(spirits.len(), 1);
    // The card is exiled face down; its owner may look at it, its opponent may not.
    let card = t.g.current(lib[0]);
    assert_eq!(t.zone(card), Zone::Exile);
    assert!(t.obj_now(card).face_down);
    assert!(mtg_engine::zones::may_look(&t.g, P0, card));
    assert!(!mtg_engine::zones::may_look(&t.g, P1, card));
    t.g.move_object(ugin, Zone::Graveyard(P0), MoveCause::Effect, None);
    t.g.destroy(spirits[0], None);
    t.resolve_all();
    assert!(t.in_hand(P0, "Lightning Bolt"));
}

#[test]
fn ugins_two_spirits_return_the_one_card_once() {
    ruling!(
        "Ugin, the Ineffable",
        "You'll put that card into your hand the first time that either of the tokens leaves the battlefield."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Doubling Season");
    let ugin = t.battlefield(P0, "Ugin, the Ineffable");
    let lib = stack_library(&mut t, P0, &["Lightning Bolt", "Shock", "Island"]);
    let spirits = ugin_plus(&mut t, ugin);
    assert_eq!(spirits.len(), 2);
    // One card exiled.
    assert_eq!(t.zone(t.g.current(lib[0])), Zone::Exile);
    assert_eq!(t.zone(t.g.current(lib[1])), Zone::Library(P0));
    t.g.destroy(spirits[0], None);
    t.resolve_all();
    assert!(t.in_hand(P0, "Lightning Bolt"));
    t.g.destroy(spirits[1], None);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn each_of_ugins_spirits_returns_its_own_card() {
    ruling!(
        "Ugin, the Ineffable",
        "you must keep track of which exiled card belongs to each Spirit token"
    );
    let mut t = TestGame::new(2);
    let ugin = t.battlefield(P0, "Ugin, the Ineffable");
    let lib = stack_library(&mut t, P0, &["Lightning Bolt", "Shock", "Island", "Island"]);
    let first = ugin_plus(&mut t, ugin);
    // Loyalty abilities: once each turn (CR 606.3).
    t.advance_to(P1, Step::PrecombatMain);
    t.advance_to(P0, Step::PrecombatMain);
    let second = ugin_plus(&mut t, ugin);
    assert_eq!((first.len(), second.len()), (1, 1));
    t.g.destroy(second[0], None);
    t.resolve_all();
    assert!(t.in_hand(P0, "Shock"));
    assert_eq!(t.zone(t.g.current(lib[0])), Zone::Exile);
    t.g.destroy(first[0], None);
    t.resolve_all();
    assert!(t.in_hand(P0, "Lightning Bolt"));
}

#[test]
fn ugins_minus_three_cant_target_a_colorless_land() {
    ruling!(
        "Ugin, the Ineffable",
        "A land normally has no color, even if it can produce one or more colors of mana."
    );
    let mut t = TestGame::new(2);
    let ugin = t.battlefield(P0, "Ugin, the Ineffable");
    t.battlefield(P1, "Island");
    t.battlefield(P1, "Ornithopter");
    // Only colorless permanents: the ability has no legal target.
    let i = index_of(&mut t, ugin, "Destroy target permanent");
    assert!(t.activate(P0, ugin, i, &[]).is_err());
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, ugin, i, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn ugin_reduces_colorless_spells_to_zero() {
    ruling!(
        "Ugin, the Ineffable",
        "A colorless spell whose mana cost is {2} or {1} will cost {0} to cast."
    );
    ruling!(
        "Ugin, the Ineffable",
        "The mana value of the spell remains unchanged, no matter what the total cost to cast it was."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ugin, the Ineffable");
    let mine = t.hand(P0, "Howling Mine");
    let spell = t.cast(P0, mine).go();
    assert_eq!(t.g.mana_value_of(spell), 2);
    t.resolve_all();
    assert!(!t.named_on_battlefield("Howling Mine").is_empty());
}
