//! Rulings batch P168, historic, instant, Island and kicker synergies: what's historic
//! (CR 700.6) for each way cards word it, "whenever you cast a historic spell" triggers
//! (cast only, CR 601.2i; never for lands, CR 305.1; resolving even if the spell is
//! countered), Lunar Mystic's payment on resolution, Kumena's Speaker and Port Town's land
//! types (CR 305.6), Archway of Innovation's improvise (CR 702.126) and kicked-spell
//! triggers (CR 702.33d).

use crate::r_s01_common::*;
use crate::r_s04_common::{ability_targets, untapped_lands};
use crate::r_s05_common::move_to;
use crate::r_s06_common::{attach_new, has_kw};
use crate::r_s20_common::tap_for_mana;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

fn pl(p: PlayerId) -> Entity {
    Entity::Player(p)
}

/// Casts the real card `name` from `p`'s hand, giving `p` the lands for its mana cost.
fn cast_new(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) -> ObjectId {
    give_mana_for(t, p, name);
    let c = t.hand(p, name);
    for e in targets {
        t.answer_targets(p, &[*e]);
    }
    t.g.turn.priority = Some(p);
    t.cast(p, c).go()
}

/// The names of the objects among `es`.
fn names(t: &TestGame, es: &[Entity]) -> Vec<String> {
    let mut v: Vec<String> = es
        .iter()
        .filter_map(|e| match e {
            Entity::Object(o) => Some(t.g.obj(*o).chars.name.to_string()),
            _ => None,
        })
        .collect();
    v.sort();
    v
}

/// The candidates of the target and entity choices asked since decision `from`.
fn choices_since(t: &TestGame, from: usize) -> Vec<Vec<Entity>> {
    t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseTargets { candidates, .. }
            | Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .collect()
}

/// The cards with "whenever you cast a historic spell".
const CAST_TRIGGERS: [&str; 8] = [
    "Artificer's Assistant",
    "Basim Ibn Ishaq",
    "Cabal Paladin",
    "D'Avenant Trapper",
    "Daring Archaeologist",
    "Diligent Excavator",
    "Mishra's Self-Replicator",
    "Serra Disciple",
];

/// What a cast trigger's effect changes, read before and after it resolves.
fn effect_state(t: &TestGame, name: &str, source: ObjectId, victim: ObjectId) -> i64 {
    match name {
        "Artificer's Assistant" => t
            .asked()
            .iter()
            .filter(|(_, d)| matches!(d, Decision::Scry { .. }))
            .count() as i64,
        "Basim Ibn Ishaq" => t.hand_size(P0) as i64,
        "Cabal Paladin" => -(t.life(P1) as i64),
        "D'Avenant Trapper" => t.obj_now(victim).tapped as i64,
        "Daring Archaeologist" => t.counters(source, "+1/+1") as i64,
        "Diligent Excavator" => -(t.library_size(P1) as i64),
        "Mishra's Self-Replicator" => t.named_on_battlefield(name).len() as i64,
        "Serra Disciple" => t.pt(source).0 as i64,
        _ => unreachable!(),
    }
}

/// How much one resolution of each cast trigger changes [`effect_state`].
fn effect_size(name: &str) -> i64 {
    match name {
        "Cabal Paladin" => 2,
        "Diligent Excavator" => 2,
        _ => 1,
    }
}

/// A game with `name` on P0's battlefield, a creature for P1 (for D'Avenant Trapper) and
/// answers for the trigger's choices queued.
fn cast_trigger_game(name: &str) -> (TestGame, ObjectId, ObjectId) {
    let mut t = TestGame::new(2);
    let source = t.battlefield(P0, name);
    let victim = t.battlefield(P1, "Grizzly Bears");
    match name {
        "D'Avenant Trapper" => {
            t.answer_targets(P0, &[obj(victim)]);
        }
        "Diligent Excavator" => {
            t.answer_targets(P0, &[pl(P1)]);
        }
        "Mishra's Self-Replicator" => {
            t.answer_yes(P0, true);
            t.lands(P0, "Wastes", 1);
        }
        _ => {}
    }
    (t, source, victim)
}

#[test]
fn historic_cast_triggers_count_a_legendary_artifact_once_and_resolve_if_countered() {
    cr!("700.6", "603.2", "603.3");
    ruling!(
        "Artificer's Assistant",
        "A card, spell, or permanent is historic if it has the legendary supertype, the artifact card type, or the Saga subtype. Having two of those qualities doesn’t make an object more historic than another or provide an additional bonus—an object either is historic or it isn’t."
    );
    ruling!(
        "Basim Ibn Ishaq",
        "A card, spell, or permanent is historic if it has the legendary supertype, the artifact card type, or the Saga subtype. Having two of those qualities doesn’t make an object more historic than another or provide an additional bonus—an object either is historic or it isn’t."
    );
    ruling!(
        "Cabal Paladin",
        "A card, spell, or permanent is historic if it has the legendary supertype, the artifact card type, or the Saga subtype. Having two of those qualities doesn’t make an object more historic than another or provide an additional bonus—an object either is historic or it isn’t."
    );
    ruling!(
        "D'Avenant Trapper",
        "A card, spell, or permanent is historic if it has the legendary supertype, the artifact card type, or the Saga subtype. Having two of those qualities doesn’t make an object more historic than another or provide an additional bonus—an object either is historic or it isn’t."
    );
    ruling!(
        "Daring Archaeologist",
        "A card, spell, or permanent is historic if it has the legendary supertype, the artifact card type, or the Saga subtype. Having two of those qualities doesn’t make an object more historic than another or provide an additional bonus—an object either is historic or it isn’t."
    );
    ruling!(
        "Diligent Excavator",
        "A card, spell, or permanent is historic if it has the legendary supertype, the artifact card type, or the Saga subtype. Having two of those qualities doesn’t make an object more historic than another or provide an additional bonus—an object either is historic or it isn’t."
    );
    ruling!(
        "Mishra's Self-Replicator",
        "A card, spell, or permanent is historic if it has the legendary supertype, the artifact card type, or the Saga subtype. Having two of those qualities doesn’t make an object more historic than another or provide an additional bonus—an object either is historic or it isn’t."
    );
    ruling!(
        "Serra Disciple",
        "A card, spell, or permanent is historic if it has the legendary supertype, the artifact card type, or the Saga subtype. Having two of those qualities doesn’t make an object more historic than another or provide an additional bonus—an object either is historic or it isn’t."
    );
    ruling!(
        "Artificer's Assistant",
        "Some abilities trigger “whenever you cast a historic spell.” Such an ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    ruling!(
        "Basim Ibn Ishaq",
        "Some abilities trigger “whenever you cast a historic spell.” Such an ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    ruling!(
        "Cabal Paladin",
        "Some abilities trigger “whenever you cast a historic spell.” Such an ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    ruling!(
        "D'Avenant Trapper",
        "Some abilities trigger “whenever you cast a historic spell.” Such an ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    ruling!(
        "Daring Archaeologist",
        "Some abilities trigger “whenever you cast a historic spell.” Such an ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    ruling!(
        "Diligent Excavator",
        "Some abilities trigger “whenever you cast a historic spell.” Such an ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    ruling!(
        "Mishra's Self-Replicator",
        "Some abilities trigger “whenever you cast a historic spell.” Such an ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    ruling!(
        "Serra Disciple",
        "Some abilities trigger “whenever you cast a historic spell.” Such an ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    for name in CAST_TRIGGERS {
        supported(name);
        // Shadowspear, a legendary artifact: one trigger, above the spell; P1 counters
        // the spell and the trigger still resolves, once.
        let (mut t, source, victim) = cast_trigger_game(name);
        let spear = cast_new(&mut t, P0, "Shadowspear", &[]);
        t.settle();
        assert_eq!(t.stack_len(), 2, "{name}");
        assert_eq!(t.g.stack[0], spear, "{name}");
        cast_new(&mut t, P1, "Counterspell", &[obj(spear)]);
        t.resolve();
        assert_eq!(t.zone(t.g.current(spear)), Zone::Graveyard(P0), "{name}");
        let before = effect_state(&t, name, source, victim);
        t.resolve_all();
        let after = effect_state(&t, name, source, victim);
        assert_eq!(after - before, effect_size(name), "{name}");
        // Grizzly Bears isn't historic.
        let (mut t, _, _) = cast_trigger_game(name);
        cast_new(&mut t, P0, "Grizzly Bears", &[]);
        t.settle();
        assert_eq!(t.stack_len(), 1, "{name}");
    }
}

#[test]
fn daring_archaeologist_legendary_artifact_and_saga_spells_are_historic() {
    cr!("700.6");
    ruling!(
        "Daring Archaeologist",
        "A card, spell, or permanent is historic if it has the legendary supertype, the artifact card type, or the Saga subtype. Having two of those qualities doesn’t make an object more historic than another or provide an additional bonus—an object either is historic or it isn’t."
    );
    for (name, historic) in [
        ("Isamaru, Hound of Konda", true),
        ("Sol Ring", true),
        ("History of Benalia", true),
        ("Shadowspear", true),
        ("Grizzly Bears", false),
        ("Lightning Bolt", false),
    ] {
        let mut t = TestGame::new(2);
        let arch = t.battlefield(P0, "Daring Archaeologist");
        let targets = if name == "Lightning Bolt" {
            vec![pl(P1)]
        } else {
            vec![]
        };
        cast_new(&mut t, P0, name, &targets);
        t.resolve_all();
        assert_eq!(t.counters(arch, "+1/+1"), historic as u32, "{name}");
    }
}

#[test]
fn historic_in_each_wording_counts_a_legendary_artifact_once() {
    cr!("700.6");
    let spear = "Shadowspear";
    let bears = "Grizzly Bears";

    // Arbaaz Mir: "Whenever ~ or another nontoken historic permanent you control enters".
    ruling!(
        "Arbaaz Mir",
        "A card, spell, or permanent is historic if it has the legendary supertype, the artifact card type, or the Saga subtype. Having two of those qualities doesn’t make an object more historic than another or provide an additional bonus—an object either is historic or it isn’t."
    );
    supported("Arbaaz Mir");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Arbaaz Mir");
    t.enter(P0, spear);
    t.enter(P0, bears);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);

    // Aya of Alexandria: "Whenever a historic creature you control deals combat damage to
    // a player". The Peregrine Dynamo is a legendary artifact creature.
    ruling!(
        "Aya of Alexandria",
        "A card, spell, or permanent is historic if it has the legendary supertype, the artifact card type, or the Saga subtype. Having two of those qualities doesn’t make an object more historic than another or provide an additional bonus—an object either is historic or it isn’t."
    );
    supported("Aya of Alexandria");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Aya of Alexandria");
    let dynamo = t.battlefield(P0, "The Peregrine Dynamo");
    let b = t.battlefield(P0, bears);
    t.attack(&[(dynamo, pl(P1)), (b, pl(P1))], &[]);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Assassin").len(), 2); // Aya and one token
    assert_eq!(tokens(&t, P0).len(), 1);

    // Bayek of Siwa: "other historic creatures you control have double strike".
    ruling!(
        "Bayek of Siwa",
        "A card, spell, or permanent is historic if it has the legendary supertype, the artifact card type, or the Saga subtype. Having two of those qualities doesn’t make an object more historic than another or provide an additional bonus—an object either is historic or it isn’t."
    );
    supported("Bayek of Siwa");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bayek of Siwa");
    let dynamo = t.battlefield(P0, "The Peregrine Dynamo");
    let b = t.battlefield(P0, bears);
    t.g.recompute();
    assert!(has_kw(&t, dynamo, KeywordKind::DoubleStrike));
    assert!(!has_kw(&t, b, KeywordKind::DoubleStrike));

    // Abstergo Entertainment: "Return up to one target historic card from your graveyard".
    ruling!(
        "Abstergo Entertainment",
        "A card, spell, or permanent is historic if it has the legendary supertype, the artifact card type, or the Saga subtype. Having two of those qualities doesn’t make an object more historic than another or provide an additional bonus—an object either is historic or it isn’t."
    );
    supported("Abstergo Entertainment");
    let mut t = TestGame::new(2);
    let abstergo = t.battlefield(P0, "Abstergo Entertainment");
    t.graveyard(P0, spear);
    t.graveyard(P0, bears);
    let c = ability_targets(&mut t, abstergo, 2);
    assert_eq!(names(&t, &c), vec![spear]);

    // Ballad of the Black Flag: "You may put a historic card from among them into your
    // hand."
    ruling!(
        "Ballad of the Black Flag",
        "A card, spell, or permanent is historic if it has the legendary supertype, the artifact card type, or the Saga subtype. Having two of those qualities doesn’t make an object more historic than another or provide an additional bonus—an object either is historic or it isn’t."
    );
    supported("Ballad of the Black Flag");
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &[spear, bears, bears]);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.enter(P0, "Ballad of the Black Flag");
    t.resolve_all();
    let c = choices_since(&t, from);
    assert!(!c.is_empty());
    assert!(c.iter().all(|c| names(&t, c) == vec![spear]), "{c:?}");

    // Board the Weatherlight: "You may reveal a historic card from among them".
    ruling!(
        "Board the Weatherlight",
        "A card, spell, or permanent is historic if it has the legendary supertype, the artifact card type, or the Saga subtype. Having two of those qualities doesn’t make an object more historic than another or provide an additional bonus—an object either is historic or it isn’t."
    );
    supported("Board the Weatherlight");
    let mut t = TestGame::new(2);
    let lib = stack_library(&mut t, P0, &[bears, spear, bears, bears, bears]);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(lib[1])]);
    cast_new(&mut t, P0, "Board the Weatherlight", &[]);
    t.resolve_all();
    let c = choices_since(&t, from);
    assert!(!c.is_empty());
    assert!(c.iter().all(|c| names(&t, c) == vec![spear]), "{c:?}");
    assert!(t.in_hand(P0, spear));

    // Crystal Skull, Isu Spyglass: "cast historic spells from the top of your library".
    ruling!(
        "Crystal Skull, Isu Spyglass",
        "A card, spell, or permanent is historic if it has the legendary supertype, the artifact card type, or the Saga subtype. Having two of those qualities doesn’t make an object more historic than another or provide an additional bonus—an object either is historic or it isn’t."
    );
    supported("Crystal Skull, Isu Spyglass");
    for (top, ok) in [(spear, true), (bears, false)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Crystal Skull, Isu Spyglass");
        t.lands(P0, "Forest", 2);
        let c = t.library_top(P0, top);
        t.g.turn.priority = Some(P0);
        assert_eq!(t.cast(P0, c).try_go().is_ok(), ok, "{top}");
    }

    // Curator's Ward: "When enchanted permanent leaves the battlefield, if it was historic,
    // draw two cards."
    ruling!(
        "Curator's Ward",
        "A card, spell, or permanent is historic if it has the legendary supertype, the artifact card type, or the Saga subtype. Having two of those qualities doesn’t make an object more historic than another or provide an additional bonus—an object either is historic or it isn’t."
    );
    for (what, drawn) in [(spear, 2), (bears, 0)] {
        let mut t = TestGame::new(2);
        let p = t.battlefield(P0, what);
        attach_new(&mut t, P0, "Curator's Ward", p);
        let hand = t.hand_size(P0);
        t.g.destroy(p, None);
        t.settle();
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + drawn, "{what}");
    }

    // Desynchronization: "Return each nonland permanent that's not historic".
    ruling!(
        "Desynchronization",
        "A card, spell, or permanent is historic if it has the legendary supertype, the artifact card type, or the Saga subtype. Having two of those qualities doesn’t make an object more historic than another or provide an additional bonus—an object either is historic or it isn’t."
    );
    supported("Desynchronization");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, spear);
    let b = t.battlefield(P0, bears);
    cast_new(&mut t, P0, "Desynchronization", &[]);
    t.resolve_all();
    assert!(t.on_battlefield(s));
    assert_eq!(t.zone(t.g.current(b)), Zone::Hand(P0));

    // Excalibur, Sword of Eden: "costs {X} less ..., where X is the total mana value of
    // historic permanents you control" (Shadowspear's 1 counts once; the Bears not).
    ruling!(
        "Excalibur, Sword of Eden",
        "A card, spell, or permanent is historic if it has the legendary supertype, the artifact card type, or the Saga subtype. Having two of those qualities doesn’t make an object more historic than another or provide an additional bonus—an object either is historic or it isn’t."
    );
    supported("Excalibur, Sword of Eden");
    for (mana, ok) in [(10, false), (11, true)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, spear);
        t.battlefield(P0, bears);
        t.lands(P0, "Wastes", mana);
        let x = t.hand(P0, "Excalibur, Sword of Eden");
        assert_eq!(t.cast(P0, x).try_go().is_ok(), ok, "{mana}");
    }

    // Guardians of Koilos: "return another target historic permanent you control".
    ruling!(
        "Guardians of Koilos",
        "A card, spell, or permanent is historic if it has the legendary supertype, the artifact card type, or the Saga subtype. Having two of those qualities doesn’t make an object more historic than another or provide an additional bonus—an object either is historic or it isn’t."
    );
    supported("Guardians of Koilos");
    let mut t = TestGame::new(2);
    t.battlefield(P0, spear);
    t.battlefield(P0, bears);
    let from = t.asked().len();
    t.answer_yes(P0, false);
    t.enter(P0, "Guardians of Koilos");
    t.settle();
    let c = choices_since(&t, from);
    assert_eq!(c.len(), 1);
    assert_eq!(names(&t, &c[0]), vec![spear]);

    // Layla Hassan: "return target historic card from your graveyard".
    ruling!(
        "Layla Hassan",
        "A card, spell, or permanent is historic if it has the legendary supertype, the artifact card type, or the Saga subtype. Having two of those qualities doesn’t make an object more historic than another or provide an additional bonus—an object either is historic or it isn’t."
    );
    supported("Layla Hassan");
    let mut t = TestGame::new(2);
    t.graveyard(P0, spear);
    t.graveyard(P0, bears);
    let from = t.asked().len();
    t.enter(P0, "Layla Hassan");
    t.settle();
    let c = choices_since(&t, from);
    assert_eq!(c.len(), 1);
    assert_eq!(names(&t, &c[0]), vec![spear]);

    // The Capitoline Triad: "costs {1} less ... for each historic card in your graveyard".
    ruling!(
        "The Capitoline Triad",
        "A card, spell, or permanent is historic if it has the legendary supertype, the artifact card type, or the Saga subtype. Having two of those qualities doesn’t make an object more historic than another or provide an additional bonus—an object either is historic or it isn’t."
    );
    for (mana, ok) in [(8, false), (9, true)] {
        let mut t = TestGame::new(2);
        t.graveyard(P0, spear);
        t.graveyard(P0, bears);
        t.lands(P0, "Wastes", mana);
        let x = t.hand(P0, "The Capitoline Triad");
        assert_eq!(t.cast(P0, x).try_go().is_ok(), ok, "{mana}");
    }

    // Thran Temporal Gateway: "put a historic permanent card from your hand onto the
    // battlefield".
    ruling!(
        "Thran Temporal Gateway",
        "A card, spell, or permanent is historic if it has the legendary supertype, the artifact card type, or the Saga subtype. Having two of those qualities doesn’t make an object more historic than another or provide an additional bonus—an object either is historic or it isn’t."
    );
    supported("Thran Temporal Gateway");
    let mut t = TestGame::new(2);
    let gate = t.battlefield(P0, "Thran Temporal Gateway");
    let s = t.hand(P0, spear);
    t.hand(P0, bears);
    t.lands(P0, "Wastes", 4);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(s)]);
    t.activate(P0, gate, 0, &[]).unwrap();
    t.resolve_all();
    let c = choices_since(&t, from);
    assert!(c.iter().all(|c| names(&t, c) == vec![spear]), "{c:?}");
    assert_eq!(t.named_on_battlefield(spear).len(), 1);

    // Urza's Tome: "discard a card unless you exile a historic card from your graveyard".
    ruling!(
        "Urza's Tome",
        "A card, spell, or permanent is historic if it has the legendary supertype, the artifact card type, or the Saga subtype. Having two of those qualities doesn’t make an object more historic than another or provide an additional bonus—an object either is historic or it isn’t."
    );
    for (in_gy, exiled) in [(spear, true), (bears, false)] {
        let mut t = TestGame::new(2);
        let tome = t.battlefield(P0, "Urza's Tome");
        let g = t.graveyard(P0, in_gy);
        t.hand(P0, "Island");
        t.lands(P0, "Wastes", 3);
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(g)]);
        let hand = t.hand_size(P0);
        t.activate(P0, tome, 0, &[]).unwrap();
        t.resolve_all();
        assert_eq!(t.zone(t.g.current(g)) == Zone::Exile, exiled, "{in_gy}");
        assert_eq!(t.hand_size(P0), if exiled { hand + 1 } else { hand }, "{in_gy}");
    }

    // What Must Be Done: "Return target historic permanent card from your graveyard".
    ruling!(
        "What Must Be Done",
        "A card, spell, or permanent is historic if it has the legendary supertype, the artifact card type, or the Saga subtype. Having two of those qualities doesn’t make an object more historic than another or provide an additional bonus—an object either is historic or it isn’t."
    );
    supported("What Must Be Done");
    let mut t = TestGame::new(2);
    let s = t.graveyard(P0, spear);
    t.graveyard(P0, bears);
    give_mana_for(&mut t, P0, "What Must Be Done");
    let wmbd = t.hand(P0, "What Must Be Done");
    let from = t.asked().len();
    t.cast(P0, wmbd).modes(&[1]).target(s).go();
    let c = choices_since(&t, from);
    assert_eq!(c.len(), 1);
    assert_eq!(names(&t, &c[0]), vec![spear]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield(spear).len(), 1);
}

#[test]
fn historic_cast_triggers_dont_trigger_without_a_cast() {
    cr!("601.2i", "603.2");
    ruling!(
        "Artificer's Assistant",
        "An ability that triggers “whenever you cast a historic spell” doesn’t trigger if a historic card is put onto the battlefield without being cast."
    );
    ruling!(
        "Basim Ibn Ishaq",
        "An ability that triggers “whenever you cast a historic spell” doesn’t trigger if a historic card is put onto the battlefield without being cast."
    );
    ruling!(
        "Cabal Paladin",
        "An ability that triggers “whenever you cast a historic spell” doesn’t trigger if a historic card is put onto the battlefield without being cast."
    );
    ruling!(
        "D'Avenant Trapper",
        "An ability that triggers “whenever you cast a historic spell” doesn’t trigger if a historic card is put onto the battlefield without being cast."
    );
    ruling!(
        "Daring Archaeologist",
        "An ability that triggers “whenever you cast a historic spell” doesn’t trigger if a historic card is put onto the battlefield without being cast."
    );
    ruling!(
        "Diligent Excavator",
        "An ability that triggers “whenever you cast a historic spell” doesn’t trigger if a historic card is put onto the battlefield without being cast."
    );
    ruling!(
        "Mishra's Self-Replicator",
        "An ability that triggers “whenever you cast a historic spell” doesn’t trigger if a historic card is put onto the battlefield without being cast."
    );
    ruling!(
        "Serra Disciple",
        "An ability that triggers “whenever you cast a historic spell” doesn’t trigger if a historic card is put onto the battlefield without being cast."
    );
    for name in CAST_TRIGGERS {
        let (mut t, _, _) = cast_trigger_game(name);
        t.enter(P0, "Shadowspear");
        t.enter(P0, "Isamaru, Hound of Konda");
        t.settle();
        assert_eq!(t.stack_len(), 0, "{name}");
    }
}

#[test]
fn historic_cast_triggers_ignore_legendary_lands() {
    cr!("305.1", "601.2i", "701.27a");
    ruling!(
        "Artificer's Assistant",
        "Lands are never cast, so abilities that trigger “whenever you cast a historic spell” won’t trigger if you play a legendary land. They also won’t trigger if a card on the battlefield transforms into a legendary land, as the Ixalan and Rivals of Ixalan double-faced cards do."
    );
    ruling!(
        "Cabal Paladin",
        "Lands are never cast, so abilities that trigger “whenever you cast a historic spell” won’t trigger if you play a legendary land. They also won’t trigger if a card on the battlefield transforms into a legendary land, as the Ixalan and Rivals of Ixalan double-faced cards do."
    );
    ruling!(
        "D'Avenant Trapper",
        "Lands are never cast, so abilities that trigger “whenever you cast a historic spell” won’t trigger if you play a legendary land. They also won’t trigger if a card on the battlefield transforms into a legendary land, as the Ixalan and Rivals of Ixalan double-faced cards do."
    );
    ruling!(
        "Daring Archaeologist",
        "Lands are never cast, so abilities that trigger “whenever you cast a historic spell” won’t trigger if you play a legendary land. They also won’t trigger if a card on the battlefield transforms into a legendary land, as the Ixalan and Rivals of Ixalan double-faced cards do."
    );
    ruling!(
        "Diligent Excavator",
        "Lands are never cast, so abilities that trigger “whenever you cast a historic spell” won’t trigger if you play a legendary land. They also won’t trigger if a card on the battlefield transforms into a legendary land, as the Ixalan and Rivals of Ixalan double-faced cards do."
    );
    ruling!(
        "Mishra's Self-Replicator",
        "Lands are never cast, so abilities that trigger “whenever you cast a historic spell” won’t trigger if you play a legendary land. They also won’t trigger if a card on the battlefield transforms into a legendary land, as the Ixalan and Rivals of Ixalan double-faced cards do."
    );
    ruling!(
        "Serra Disciple",
        "Lands are never cast, so abilities that trigger “whenever you cast a historic spell” won’t trigger if you play a legendary land. They also won’t trigger if a card on the battlefield transforms into a legendary land, as the Ixalan and Rivals of Ixalan double-faced cards do."
    );
    for name in CAST_TRIGGERS {
        if name == "Basim Ibn Ishaq" {
            continue;
        }
        let (mut t, _, _) = cast_trigger_game(name);
        let urborg = t.hand(P0, "Urborg, Tomb of Yawgmoth");
        t.play_land(P0, urborg).unwrap();
        t.settle();
        assert_eq!(t.stack_len(), 0, "{name}");
        assert_eq!(t.named_on_battlefield("Urborg, Tomb of Yawgmoth").len(), 1);
        // Search for Azcanta transforms into Azcanta, the Sunken Ruin, a legendary land.
        let search = t.battlefield(P0, "Search for Azcanta // Azcanta, the Sunken Ruin");
        assert!(mtg_engine::dfc::transform(&mut t.g, search));
        t.g.recompute();
        t.g.flush_events();
        t.settle();
        let ruin = t.g.current(search);
        assert!(t.obj_now(ruin).is(CardType::Land), "{name}");
        assert!(t.obj_now(ruin).chars.supertypes.contains(Supertype::Legendary));
        assert_eq!(t.stack_len(), 0, "{name}");
    }
}

#[test]
fn banish_to_another_universe_gone_before_its_trigger_resolves_exiles_nothing() {
    cr!("610.3b", "603.3");
    ruling!(
        "Banish to Another Universe",
        "If Banish to another Universe leaves the battlefield before its triggered ability resolves, the target permanent will not be exiled."
    );
    supported("Banish to Another Universe");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    let banish = t.enter(P0, "Banish to Another Universe");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    move_to(&mut t, banish, Zone::Graveyard(P0));
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    // Otherwise it's exiled.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    t.enter(P0, "Banish to Another Universe");
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(bears)), Zone::Exile);
}

#[test]
fn lunar_mystic_trigger_resolves_first_and_pays_on_resolution() {
    cr!("603.3", "608.2");
    ruling!(
        "Lunar Mystic",
        "The decision to pay {1} is made when the triggered ability resolves. It’s not an additional cost of the instant spell."
    );
    ruling!(
        "Lunar Mystic",
        "The triggered ability will go on the stack on top of the instant spell and resolve before it."
    );
    supported("Lunar Mystic");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lunar Mystic");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Exactly the mana for Giant Growth: nothing is asked for {1} as it's cast.
    let from = t.asked().len();
    let growth = cast_new(&mut t, P0, "Giant Growth", &[obj(bears)]);
    t.settle();
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::OptionalCost { .. })));
    assert_eq!(t.stack_len(), 2);
    assert_eq!(t.g.stack[0], growth);
    // The {1} is paid as the ability resolves, with mana P0 got after casting.
    t.lands(P0, "Wastes", 1);
    t.answer_yes(P0, true);
    let hand = t.hand_size(P0);
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 1);
    // The instant is still on the stack.
    assert_eq!(t.g.stack, vec![growth]);
    assert_eq!(t.pt(bears), (2, 2));
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 5));
}

#[test]
fn kumenas_speaker_bonus_and_marked_damage() {
    cr!("704.5g", "613.4c", "120.6");
    ruling!(
        "Kumena's Speaker",
        "Because damage remains marked on a creature until it’s removed as the turn ends, the damage Kumena’s Speaker takes during combat may become lethal if you no longer control a Merfolk or Island later in the turn."
    );
    ruling!(
        "Kumena's Speaker",
        "Kumena’s Speaker gets only +1/+1 from its ability, no matter how many Merfolk and Islands you control."
    );
    ruling!(
        "Kumena's Speaker",
        "Most nonbasic lands that produce blue mana aren’t Islands. For example, Glacial Fortress isn’t an Island. Some nonbasic lands (such as Irrigated Farmland from the Amonkhet set) do have basic land types printed on the type line and may be Islands."
    );
    // Two Islands and two other Merfolk: still only +1/+1.
    let mut t = TestGame::new(2);
    let speaker = t.battlefield(P0, "Kumena's Speaker");
    let islands = t.lands(P0, "Island", 2);
    t.battlefield(P0, "Merfolk of the Pearl Trident");
    t.battlefield(P0, "Merfolk of the Pearl Trident");
    t.g.recompute();
    assert_eq!(t.pt(speaker), (2, 2));
    // One damage, then it loses its other Merfolk and Islands: lethal.
    let bolt_src = t.battlefield(P1, "Raging Goblin");
    crate::r_s06_common::damage(&mut t, bolt_src, 1, speaker);
    assert!(t.on_battlefield(speaker));
    for id in islands {
        move_to(&mut t, id, Zone::Hand(P0));
    }
    for m in t.named_on_battlefield("Merfolk of the Pearl Trident") {
        move_to(&mut t, m, Zone::Hand(P0));
    }
    t.settle();
    assert!(!t.on_battlefield(speaker));
    // Glacial Fortress isn't an Island; Irrigated Farmland is.
    for (land, pt) in [("Glacial Fortress", (1, 1)), ("Irrigated Farmland", (2, 2))] {
        let mut t = TestGame::new(2);
        let speaker = t.battlefield(P0, "Kumena's Speaker");
        t.battlefield(P0, land);
        t.g.recompute();
        assert_eq!(t.pt(speaker), pt, "{land}");
    }
}

/// Plays `land` from P0's hand, revealing `reveal` (if any) for its "As this land enters,
/// you may reveal ..." choice, and returns it on the battlefield.
fn play_revealing(t: &mut TestGame, land: &str, reveal: Option<ObjectId>) -> ObjectId {
    let id = t.hand(P0, land);
    match reveal {
        Some(r) => {
            t.answer_yes(P0, true);
            t.answer_choose(P0, &[obj(r)]);
        }
        None => {
            t.answer_yes(P0, false);
            t.answer_choose(P0, &[]);
        }
    }
    t.play_land(P0, id).unwrap();
    t.settle();
    t.g.current(id)
}

#[test]
fn port_town_reveals_by_subtype_not_by_mana() {
    cr!("305.6", "614.12");
    ruling!(
        "Port Town",
        "You may reveal any land card with either or both of the appropriate subtypes. It doesn't have to be a basic land. For example, you could reveal Canopy Vista from the Battle for Zendikar set to satisfy the ability of Port Town."
    );
    ruling!(
        "Port Town",
        "Lands don't have a subtype just because they can produce mana of the corresponding color. Port Town itself is neither a Plains nor an Island, even though it produces white and blue mana, so you can't reveal one to satisfy the ability of another."
    );
    let mut t = TestGame::new(2);
    let vista = t.hand(P0, "Canopy Vista");
    let town = play_revealing(&mut t, "Port Town", Some(vista));
    assert!(!t.obj_now(town).tapped);
    // Another Port Town can't be revealed.
    let mut t = TestGame::new(2);
    let other = t.hand(P0, "Port Town");
    let from = t.asked().len();
    let town = play_revealing(&mut t, "Port Town", Some(other));
    assert!(t.obj_now(town).tapped);
    assert!(choices_since(&t, from)
        .iter()
        .all(|c| !c.contains(&obj(other))));
}

#[test]
fn port_town_put_onto_the_battlefield_tapped_stays_tapped() {
    cr!("614.12", "614.1c");
    ruling!(
        "Port Town",
        "If an effect instructs you to put Port Town onto the battlefield tapped, it will still enter the battlefield tapped even if you reveal a land card from your hand."
    );
    supported("Arboreal Grazer");
    let mut t = TestGame::new(2);
    let town = t.hand(P0, "Port Town");
    let island = t.hand(P0, "Island");
    // Arboreal Grazer: "you may put a land card from your hand onto the battlefield tapped."
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(town)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(island)]);
    t.enter(P0, "Arboreal Grazer");
    t.resolve_all();
    let town = t.g.current(town);
    assert!(t.on_battlefield(town));
    assert!(t.obj_now(town).tapped);
}

/// P1's Tempting Wurm lets P0 put `lands` from hand onto the battlefield at the same time;
/// `reveal` is P0's answer for Port Town's reveal.
fn enter_together(t: &mut TestGame, lands: &[ObjectId], reveal: Option<ObjectId>) {
    t.answer_yes(P0, true);
    t.answer_choose(P0, &lands.iter().map(|l| obj(*l)).collect::<Vec<_>>());
    if let Some(r) = reveal {
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(r)]);
    }
    t.enter(P1, "Tempting Wurm");
    t.resolve_all();
}

#[test]
fn port_town_may_reveal_a_land_entering_with_it() {
    cr!("614.12", "614.1c");
    ruling!(
        "Port Town",
        "If a Plains or Island is entering the battlefield from your hand at the same time as Port Town, you may reveal the other land to have Port Town enter untapped."
    );
    supported("Tempting Wurm");
    let mut t = TestGame::new(2);
    let town = t.hand(P0, "Port Town");
    let island = t.hand(P0, "Island");
    enter_together(&mut t, &[town, island], Some(island));
    let town = t.g.current(town);
    assert!(t.on_battlefield(town));
    assert!(t.on_battlefield(t.g.current(island)));
    assert!(!t.obj_now(town).tapped);
}

#[test]
fn archway_of_innovation_needs_an_island_already() {
    cr!("614.12", "614.1c");
    ruling!(
        "Archway of Innovation",
        "You must already control an Island as Archway of Innovation enters the battlefield for it to enter untapped. If it enters the battlefield at the same time as an Island when you control no other Islands, it will enter tapped."
    );
    supported("Archway of Innovation");
    let mut t = TestGame::new(2);
    let arch = t.hand(P0, "Archway of Innovation");
    let island = t.hand(P0, "Island");
    enter_together(&mut t, &[arch, island], None);
    let arch = t.g.current(arch);
    assert!(t.on_battlefield(arch));
    assert!(t.obj_now(arch).tapped);
    // With an Island already there, it enters untapped.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Island");
    let arch = t.hand(P0, "Archway of Innovation");
    t.play_land(P0, arch).unwrap();
    assert!(!t.obj_now(t.g.current(arch)).tapped);
}

#[test]
fn nimbus_maze_checks_land_types_not_names() {
    cr!("305.6", "602.5b");
    ruling!(
        "Nimbus Maze",
        "Nimbus Maze's last two abilities check the subtypes of lands you control, not their names. You can activate the appropriate abilities as long as you control a land with the corresponding subtype, Plains or Island. Most nonbasic lands don't have basic land types, even if they produce colored mana. For example, Nimbus Maze itself is neither a Plains nor an Island, while Hallowed Fountain is both."
    );
    supported("Nimbus Maze");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Nimbus Maze");
    t.battlefield(P0, "Nimbus Maze");
    t.battlefield(P0, "Port Town");
    for color in ["{W}", "{U}"] {
        t.g.recompute();
        t.g.turn.priority = Some(P0);
        let usable = t
            .g
            .activatable_abilities(P0)
            .iter()
            .any(|(s, ab)| *s == a && ab.text.contains(color));
        assert!(!usable, "{color}");
    }
    t.battlefield(P0, "Hallowed Fountain");
    assert!(tap_for_mana(&mut t, P0, a, "Add {W}"));
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::W), 1);
    let b = t.named_on_battlefield("Nimbus Maze")[1];
    assert!(tap_for_mana(&mut t, P0, b, "Add {U}"));
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::U), 1);
}

#[test]
fn tempest_djinn_is_0_4_off_the_battlefield() {
    cr!("611.3a", "113.6");
    ruling!(
        "Tempest Djinn",
        "Tempest Djinn's ability that modifies its power applies only while it's on the battlefield. In all other zones, it's a 0/4 creature card."
    );
    supported("Tempest Djinn");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    let d = t.hand(P0, "Tempest Djinn");
    t.g.recompute();
    assert_eq!(t.pt(d), (0, 4));
    let spell = t.cast(P0, d).go();
    t.g.recompute();
    assert_eq!(t.pt(spell), (0, 4));
    t.resolve_all();
    let d = t.named_on_battlefield("Tempest Djinn")[0];
    assert_eq!(t.pt(d), (3, 4));
}

/// A game where P0 activated Archway of Innovation's last ability (the next spell has
/// improvise), with Plains for other mana.
fn archway_game(plains: usize) -> TestGame {
    let mut t = TestGame::new(2);
    let arch = t.battlefield(P0, "Archway of Innovation");
    t.lands(P0, "Island", 1);
    t.activate(P0, arch, 1, &[]).unwrap();
    t.resolve_all();
    t.lands(P0, "Plains", plains);
    t
}

fn improvise_choices(t: &TestGame, from: usize) -> Vec<(Vec<Entity>, u32)> {
    t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities {
                prompt,
                candidates,
                max,
                ..
            } if prompt.contains("mprovise") => Some((candidates.clone(), *max)),
            _ => None,
        })
        .collect()
}

#[test]
fn archway_improvise_cant_tap_artifacts_used_for_mana() {
    cr!("702.126a", "601.2g", "601.2h");
    ruling!(
        "Archway of Innovation",
        "If an artifact you control has a mana ability with {T} in the cost, activating that ability while casting a spell with improvise will result in the artifact being tapped before you pay the spell's costs. You won't be able to tap it again for improvise. Similarly, if you sacrifice an artifact to activate a mana ability while casting a spell with improvise, that artifact won't be on the battlefield when you pay the spell's costs, so you won't be able to tap it for improvise."
    );
    supported("Lotus Petal");
    // Divination ({2}{U}): Mind Stone's {C} and Lotus Petal's {U} are made first.
    let mut t = archway_game(0);
    let stone = t.battlefield(P0, "Mind Stone");
    let petal = t.battlefield(P0, "Lotus Petal");
    let orn = t.battlefield(P0, "Ornithopter");
    assert!(tap_for_mana(&mut t, P0, stone, "Add {C}"));
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    assert!(tap_for_mana(&mut t, P0, petal, "Add"));
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::U), 1);
    assert!(!t.on_battlefield(petal));
    let div = t.hand(P0, "Divination");
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(orn)]);
    t.cast(P0, div).go();
    let c = improvise_choices(&t, from);
    assert_eq!(c.len(), 1);
    assert_eq!(c[0].0, vec![obj(orn)]);
    assert!(t.obj_now(orn).tapped);
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
}

#[test]
fn archway_improvise_applies_to_the_total_cost_after_x() {
    cr!("702.126b", "601.2f", "107.3a");
    ruling!(
        "Archway of Innovation",
        "When calculating a spell's total cost, include any alternative costs, additional costs, or anything else that increases or reduces the cost to cast the spell. Improvise applies after the total cost is calculated. Improvise doesn't change a spell's mana cost or mana value."
    );
    ruling!(
        "Archway of Innovation",
        "When using improvise to cast a spell with {X} in its cost, first choose the value for X. That choice, plus any cost increases or decreases, will determine the spell's total cost. Then you can tap artifacts you control to help pay that cost. For example, if you activate the last ability of Archway of Innovation and then cast Wrath of the Skies (a spell with mana cost {X}{W}{W}), choosing 3 as the value of X, the total cost is {3}{W}{W}. If you tap two artifacts, you'll have to pay {1}{W}{W}."
    );
    supported("Wrath of the Skies");
    // Wrath of the Skies with X = 3: two artifacts and three Plains.
    let mut t = archway_game(3);
    let a = t.battlefield(P0, "Ornithopter");
    let b = t.battlefield(P0, "Ornithopter");
    let c = t.battlefield(P0, "Ornithopter");
    let wrath = t.hand(P0, "Wrath of the Skies");
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(a), obj(b)]);
    let spell = t.cast(P0, wrath).x(3).go();
    let ch = improvise_choices(&t, from);
    assert_eq!(ch.len(), 1);
    assert_eq!(ch[0].1, 3);
    assert!(t.obj_now(a).tapped && t.obj_now(b).tapped && !t.obj_now(c).tapped);
    assert_eq!(untapped_lands(&t, P0), 0);
    // The mana value is X + 2 whatever was tapped.
    assert_eq!(t.g.mana_value_of(spell), 5);
    // Thalia's {1} increase is part of the total cost improvise can pay; Divination's
    // mana value stays 3.
    let mut t = archway_game(0);
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    t.lands(P0, "Island", 1);
    let arts: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P0, "Ornithopter")).collect();
    let div = t.hand(P0, "Divination");
    t.answer_choose(P0, &arts.iter().map(|a| obj(*a)).collect::<Vec<_>>());
    let spell = t.cast(P0, div).go();
    assert!(arts.iter().all(|a| t.obj_now(*a).tapped));
    assert_eq!(t.g.mana_value_of(spell), 3);
}

#[test]
fn kicked_spell_triggers_resolve_first_and_need_no_particular_mana() {
    cr!("702.33d", "603.3");
    ruling!(
        "Rumbling Aftershocks",
        "If you cast a kicked spell, Rumbling Aftershocks’s ability triggers and goes on the stack on top of it. The ability will resolve before the spell does."
    );
    ruling!(
        "Lullmage's Familiar",
        "The last ability of Lullmage's Familiar triggers when you cast a kicked spell even if you didn't spend mana from its first ability to cast that spell."
    );
    // Rumbling Aftershocks: Kavu Titan kicked; the 1 damage resolves first.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rumbling Aftershocks");
    t.lands(P0, "Forest", 5);
    let kavu = t.hand(P0, "Kavu Titan");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[pl(P1)]);
    let spell = t.cast(P0, kavu).kicked(true).go();
    t.settle();
    assert_eq!(t.g.stack[0], spell);
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.g.stack, vec![spell]);
    // Lullmage's Familiar, untapped: Forests pay for the kicked Kavu Titan.
    let mut t = TestGame::new(2);
    let fam = t.battlefield(P0, "Lullmage's Familiar");
    t.lands(P0, "Forest", 5);
    let kavu = t.hand(P0, "Kavu Titan");
    t.cast(P0, kavu).kicked(true).go();
    assert!(!t.obj_now(fam).tapped);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
}
