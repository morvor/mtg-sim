//! Rulings batch P048 — "destroy/exile target ... Its controller creates a token": a spell
//! or ability whose only target is illegal as it resolves does nothing, so no one creates
//! the token (CR 608.2b); a legal target that isn't destroyed (indestructible, a shield
//! counter) still gives its controller the token; "for each permanent destroyed/put into
//! a graveyard this way" counts only the ones that were (CR 608.2c).

use crate::r_p048_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::*;

/// How a case's spell or ability is put on the stack targeting the given object.
#[derive(Clone, Copy)]
enum Start {
    /// The real card is cast by P0 (with lands for its mana cost).
    Cast,
    /// The second half of the split card is cast.
    CastHalf,
    /// The permanent (P0's) enters and its enters ability targets the object.
    Enter,
    /// Transmogrifying Wand's ability (three charge counters) is activated.
    Wand,
    /// The card is cast for its mutate cost onto P0's Grizzly Bears and its "whenever this
    /// creature mutates" ability targets the object.
    Mutate,
}

/// Puts the case's spell or ability on the stack, targeting `target` (P1's).
fn start(t: &mut TestGame, card: &str, how: Start, target: ObjectId) {
    match how {
        Start::Cast => {
            cast_new(t, P0, card, &[obj(target)]);
        }
        Start::CastHalf => {
            t.lands(P0, "Forest", 2);
            t.lands(P0, "Island", 2);
            let c = t.hand(P0, card);
            t.answer_targets(P0, &[obj(target)]);
            t.cast(P0, c).method(CastMethod::Half(1)).go();
        }
        Start::Enter => {
            t.answer_targets(P0, &[obj(target)]);
            t.enter(P0, card);
            t.settle();
        }
        Start::Wand => {
            let wand = t.battlefield(P0, card);
            put_counters(t, wand, "charge", 3);
            t.lands(P0, "Wastes", 1);
            activate_with(t, P0, wand, "Destroy target creature", &[obj(target)]);
        }
        Start::Mutate => {
            let bears = t.battlefield(P0, "Grizzly Bears");
            t.lands(P0, "Forest", 4);
            let c = t.hand(P0, card);
            t.cast(P0, c)
                .method(CastMethod::Keyword(KeywordKind::Mutate))
                .target(bears)
                .go();
            // The mutating spell resolves (merging on top); its ability targets the object.
            t.answer(
                P0,
                DecisionKind::Option,
                mtg_engine::decision::Answer::Index(0),
            );
            t.answer_targets(P0, &[obj(target)]);
            t.resolve();
            t.settle();
        }
    }
    assert!(t.stack_len() >= 1, "{card}: nothing on the stack");
}

/// The cases: card, how it starts, the permanent it targets (P1's), and the creature type
/// of the token its controller creates.
const CASES: &[(&str, Start, &str, &str)] = &[
    ("Rapid Hybridization", Start::Cast, "Grizzly Bears", "Frog"),
    ("Ravenform", Start::Cast, "Grizzly Bears", "Bird"),
    ("Bovine Intervention", Start::Cast, "Grizzly Bears", "Ox"),
    ("Harsh Annotation", Start::Cast, "Grizzly Bears", "Inkling"),
    ("Crib Swap", Start::Cast, "Grizzly Bears", "Shapeshifter"),
    (
        "Incubation // Incongruity",
        Start::CastHalf,
        "Grizzly Bears",
        "Frog",
    ),
    ("Pongify", Start::Cast, "Grizzly Bears", "Ape"),
    ("Transmogrifying Wand", Start::Wand, "Grizzly Bears", "Ox"),
    ("Sawtusk Demolisher", Start::Mutate, "Sol Ring", "Beast"),
    ("Secure the Scene", Start::Cast, "Grizzly Bears", "Soldier"),
    ("Angelic Ascension", Start::Cast, "Grizzly Bears", "Angel"),
    ("Stroke of Midnight", Start::Cast, "Grizzly Bears", "Human"),
    ("Cavalier of Dawn", Start::Enter, "Grizzly Bears", "Golem"),
    (
        "Soul of Emancipation",
        Start::Enter,
        "Grizzly Bears",
        "Angel",
    ),
];

#[test]
fn an_illegal_target_means_no_one_creates_the_token() {
    cr!("608.2b", "115.1");
    ruling!(
        "Rapid Hybridization",
        "If the creature is an illegal target when Rapid Hybridization tries to resolve, it won't resolve"
    );
    ruling!(
        "Ravenform",
        "If the target artifact or creature is an illegal target as Ravenform tries to resolve, it won't resolve"
    );
    ruling!(
        "Bovine Intervention",
        "If the target artifact or creature is an illegal target by the time Bovine Intervention tries to resolve, the spell doesn’t resolve. No player creates an Ox token."
    );
    ruling!(
        "Harsh Annotation",
        "If the target creature is an illegal target as Harsh Annotation tries to resolve, it won't resolve"
    );
    ruling!(
        "Crib Swap",
        "If the target creature is an illegal target by the time Crib Swap tries to resolve, the spell won't resolve."
    );
    ruling!(
        "Incubation // Incongruity",
        "If the target creature is an illegal target by the time Incongruity tries to resolve, the spell doesn't resolve."
    );
    ruling!(
        "Pongify",
        "If the target creature is an illegal target by the time Pongify tries to resolve, the spell won't resolve. No player creates an Ape token."
    );
    ruling!(
        "Transmogrifying Wand",
        "If the target creature is an illegal target by the time Transmogrifying Wand’s ability tries to resolve"
    );
    ruling!(
        "Sawtusk Demolisher",
        "If the target noncreature permanent is an illegal target by the time Sawtusk Demolisher’s last ability tries to resolve, the ability won’t resolve. No player will create a Beast token."
    );
    ruling!(
        "Secure the Scene",
        "If the target nonland permanent is an illegal target by the time Secure the Scene tries to resolve"
    );
    ruling!(
        "Angelic Ascension",
        "If the target permanent is an illegal target by the time Angelic Ascension tries to resolve"
    );
    ruling!(
        "Stroke of Midnight",
        "If the target permanent is an illegal target by the time Stroke of Midnight tries to resolve, the spell doesn't resolve. No player creates a Human token."
    );
    ruling!(
        "Cavalier of Dawn",
        "However, if you do choose a target and the target permanent is an illegal target by the time that ability tries to resolve, the ability won’t resolve. No player will create a Golem token."
    );
    ruling!(
        "Soul of Emancipation",
        "If a permanent is no longer a legal target as Soul of Emancipation's triggered ability resolves, that target's controller will not create an Angel for that permanent."
    );
    for &(card, how, target, token) in CASES {
        supported(card);
        let mut t = TestGame::new(2);
        let victim = t.battlefield(P1, target);
        start(&mut t, card, how, victim);
        // The target leaves the battlefield (a new object, CR 400.7) in response.
        bounce(&mut t, victim);
        t.resolve_all();
        assert_eq!(tokens_by_player(&t, token), vec![0, 0], "{card}: no token");
        assert!(t.in_hand(P1, target), "{card}");
    }
}

#[test]
fn soul_of_emancipation_gives_angels_only_for_its_legal_targets() {
    cr!("608.2b", "608.2c");
    ruling!(
        "Soul of Emancipation",
        "If a permanent is no longer a legal target as Soul of Emancipation's triggered ability resolves, that target's controller will not create an Angel for that permanent."
    );
    supported("Soul of Emancipation");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Sol Ring");
    t.answer_targets(P0, &[obj(a), obj(b)]);
    t.enter(P0, "Soul of Emancipation");
    t.settle();
    bounce(&mut t, a);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Sol Ring"));
    assert_eq!(tokens_by_player(&t, "Angel"), vec![0, 1]);
}

#[test]
fn a_legal_target_that_isnt_destroyed_still_gives_its_controller_the_token() {
    cr!("608.2b", "702.12b");
    ruling!(
        "Rapid Hybridization",
        "If Rapid Hybridization resolves and the creature isn't destroyed (perhaps because it has indestructible), its controller will still get the Frog Lizard token."
    );
    ruling!(
        "Bovine Intervention",
        "If the target is legal but not destroyed (most likely because it has indestructible), its controller does create an Ox token."
    );
    ruling!(
        "Transmogrifying Wand",
        "If the target creature is a legal target but can’t be destroyed, most likely because it has indestructible, its controller gets an Ox token and gets to keep that creature."
    );
    ruling!(
        "Pongify",
        "If the target is legal but not destroyed (most likely because it has indestructible), its controller does create an Ape token."
    );
    ruling!(
        "Sawtusk Demolisher",
        "If the target is legal but not destroyed (most likely because it has indestructible), its controller does create a Beast token."
    );
    ruling!(
        "Stroke of Midnight",
        "If the target is legal but not destroyed (most likely because it has indestructible), its controller does create a Human token."
    );
    ruling!(
        "Cavalier of Dawn",
        "If the target is legal but not destroyed (most likely because it has indestructible), its controller will create a Golem token."
    );
    let cases: &[(&str, Start, &str, &str)] = &[
        ("Rapid Hybridization", Start::Cast, "Darksteel Myr", "Frog"),
        ("Bovine Intervention", Start::Cast, "Darksteel Myr", "Ox"),
        ("Transmogrifying Wand", Start::Wand, "Darksteel Myr", "Ox"),
        ("Pongify", Start::Cast, "Darksteel Myr", "Ape"),
        (
            "Sawtusk Demolisher",
            Start::Mutate,
            "Darksteel Ingot",
            "Beast",
        ),
        ("Stroke of Midnight", Start::Cast, "Darksteel Myr", "Human"),
        ("Cavalier of Dawn", Start::Enter, "Darksteel Myr", "Golem"),
    ];
    for &(card, how, target, token) in cases {
        supported(card);
        let mut t = TestGame::new(2);
        let victim = t.battlefield(P1, target);
        start(&mut t, card, how, victim);
        t.resolve_all();
        assert!(t.on_battlefield(victim), "{card}: not destroyed");
        assert_eq!(t.obj_now(victim).controller, P1, "{card}: P1 keeps it");
        assert_eq!(
            tokens_by_player(&t, token),
            vec![0, 1],
            "{card}: P1's token"
        );
    }
}

#[test]
fn soul_of_emancipation_gives_an_angel_for_a_target_a_shield_counter_saves() {
    cr!("608.2b", "122.1c");
    ruling!(
        "Soul of Emancipation",
        "If a permanent is still a legal target as Soul of Emancipation's triggered ability resolves but it isn't destroyed, perhaps because it had a shield counter, its controller will create an Angel token for that permanent."
    );
    supported("Soul of Emancipation");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    put_counters(&mut t, a, "shield", 1);
    t.answer_targets(P0, &[obj(a)]);
    t.enter(P0, "Soul of Emancipation");
    t.resolve_all();
    assert!(t.on_battlefield(a));
    assert_eq!(t.counters(a, "shield"), 0);
    assert_eq!(tokens_by_player(&t, "Angel"), vec![0, 1]);
}

#[test]
fn cavalier_of_dawns_ability_without_a_target_does_nothing() {
    cr!("603.3d", "115.1");
    ruling!(
        "Cavalier of Dawn",
        "You may put this creature’s enters ability onto the stack without choosing any target. It will resolve with no effect."
    );
    supported("Cavalier of Dawn");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[]);
    t.enter(P0, "Cavalier of Dawn");
    t.settle();
    // On the stack with no target ("up to one").
    assert_eq!(t.stack_len(), 1);
    let ab = *t.g.stack.last().unwrap();
    assert!(crate::r_s25_common::targets_of(&t, ab).is_empty());
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    // No one creates a Golem.
    assert_eq!(tokens_by_player(&t, "Golem"), vec![0, 0]);
}

#[test]
fn for_each_permanent_destroyed_or_put_into_a_graveyard_counts_only_those() {
    cr!("608.2c", "702.12b", "701.19a", "614.1a");
    ruling!(
        "Descent of the Dragons",
        "If a creature is targeted but not destroyed (for example, if it regenerates or has indestructible), it won’t count toward the number of Dragon tokens put onto the battlefield."
    );
    ruling!(
        "Terastodon",
        "If a targeted permanent has indestructible or regenerates, its controller won't get an Elephant token for it."
    );
    ruling!(
        "Terastodon",
        "Similarly, if the targeted permanent is destroyed but a replacement effect moves it to a different zone instead of its owner's graveyard, its controller won't get an Elephant token for it."
    );
    // Descent of the Dragons: Grizzly Bears, Darksteel Myr and a Drudge Skeletons with a
    // regeneration shield ("{B}: Regenerate this creature.") — one Dragon.
    supported("Descent of the Dragons");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let myr = t.battlefield(P1, "Darksteel Myr");
    let skeletons = t.battlefield(P1, "Drudge Skeletons");
    mana(&mut t, P1, mtg_engine::mana::ManaType::B, 1);
    activate_containing(&mut t, P1, skeletons, "Regenerate").expect("regenerate");
    t.resolve_all();
    lands_for_cost(&mut t, P0, "Descent of the Dragons");
    let c = t.hand(P0, "Descent of the Dragons");
    t.answer_targets(P0, &[obj(bears), obj(myr), obj(skeletons)]);
    t.cast(P0, c).go();
    t.resolve_all();
    assert!(t.on_battlefield(myr) && t.on_battlefield(skeletons));
    assert!(t.obj_now(skeletons).tapped, "regenerated");
    assert_eq!(tokens_by_player(&t, "Dragon"), vec![0, 1]);
    // Terastodon: Sol Ring, Darksteel Ingot (indestructible) — one Elephant.
    supported("Terastodon");
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P1, "Sol Ring");
    let ingot = t.battlefield(P1, "Darksteel Ingot");
    t.answer(
        P0,
        DecisionKind::YesNo,
        mtg_engine::decision::Answer::Bool(true),
    );
    t.answer_targets(P0, &[obj(ring), obj(ingot)]);
    t.enter(P0, "Terastodon");
    t.resolve_all();
    assert!(t.on_battlefield(ingot));
    assert!(t.in_graveyard(P1, "Sol Ring"));
    assert_eq!(tokens_by_player(&t, "Elephant"), vec![0, 1]);
    // ... and with Rest in Peace, the destroyed Sol Ring is exiled instead: no Elephant.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Rest in Peace");
    let ring = t.battlefield(P1, "Sol Ring");
    t.answer(
        P0,
        DecisionKind::YesNo,
        mtg_engine::decision::Answer::Bool(true),
    );
    t.answer_targets(P0, &[obj(ring)]);
    t.enter(P0, "Terastodon");
    t.resolve_all();
    assert!(t.in_exile("Sol Ring"));
    assert_eq!(tokens_by_player(&t, "Elephant"), vec![0, 0]);
}

#[test]
fn terastodon_can_target_permanents_of_different_players() {
    cr!("115.1", "608.2c");
    ruling!(
        "Terastodon",
        "The targeted permanents may be controlled by different players."
    );
    let mut t = TestGame::new(3);
    let a = t.battlefield(P1, "Sol Ring");
    let b = t.battlefield(P2, "Sol Ring");
    let c = t.battlefield(P0, "Sol Ring");
    t.answer(
        P0,
        DecisionKind::YesNo,
        mtg_engine::decision::Answer::Bool(true),
    );
    t.answer_targets(P0, &[obj(a), obj(b), obj(c)]);
    t.enter(P0, "Terastodon");
    t.resolve_all();
    assert_eq!(tokens_by_player(&t, "Elephant"), vec![1, 1, 1]);
}

#[test]
fn death_by_dragons_targets_one_player_and_does_nothing_if_that_player_is_illegal() {
    cr!("608.2b", "702.11c");
    ruling!(
        "Death by Dragons",
        "If Death by Dragons resolves, the player who’s targeted is the only one who won’t put a Dragon token onto the battlefield."
    );
    ruling!(
        "Death by Dragons",
        "If the player is an illegal target when Death by Dragons tries to resolve, it won’t resolve"
    );
    supported("Death by Dragons");
    let mut t = TestGame::new(4);
    cast_new(&mut t, P0, "Death by Dragons", &[Entity::Player(P2)]);
    t.resolve_all();
    assert_eq!(tokens_by_player(&t, "Dragon"), vec![1, 1, 0, 1]);
    // P2 gets hexproof in response (Leyline of Sanctity: "You have hexproof."): nothing.
    let mut t = TestGame::new(4);
    cast_new(&mut t, P0, "Death by Dragons", &[Entity::Player(P2)]);
    t.battlefield(P2, "Leyline of Sanctity");
    t.resolve_all();
    assert_eq!(tokens_by_player(&t, "Dragon"), vec![0, 0, 0, 0]);
}

#[test]
fn declaration_in_stone_with_an_illegal_target_exiles_nothing() {
    cr!("608.2b");
    ruling!(
        "Declaration in Stone",
        "If the target creature is an illegal target when Declaration in Stone tries to resolve, it won't resolve and none of its effects will happen. No creatures will be exiled, including those with the same name as the target."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Declaration in Stone", &[obj(a)]);
    bounce(&mut t, a);
    t.resolve_all();
    assert!(t.on_battlefield(b));
    assert!(named_by(&t, P1, "Clue").is_empty());
}

#[test]
fn ovinomancer_targeting_itself_has_an_illegal_target() {
    cr!("608.2b", "602.2b");
    ruling!(
        "Ovinomancer",
        "Ovinomancer is a legal target for its own ability. Activating that ability will cause Ovinomancer to return to its owner’s hand, so the ability will have an illegal target. It won’t resolve, and no one will create a Sheep token."
    );
    supported("Ovinomancer");
    let mut t = TestGame::new(2);
    let ovi = t.battlefield(P0, "Ovinomancer");
    let ab = activate_with(&mut t, P0, ovi, "Destroy target creature", &[obj(ovi)]);
    assert!(ab.is_some());
    assert!(t.in_hand(P0, "Ovinomancer"));
    t.resolve_all();
    assert_eq!(tokens_by_player(&t, "Sheep"), vec![0, 0]);
}
