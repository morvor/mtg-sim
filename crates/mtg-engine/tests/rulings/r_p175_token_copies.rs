//! Rulings batch P175 — effects that create temporary token copies of permanents and cards
//! (Saheeli, the Gifted's −7, Red Sun's Twilight, Cogwork Assembler, Firion, Esper Terra,
//! Séance): what the copy copies (CR 707.2–707.3, 707.9b), {X} as 0 (CR 107.3m), the
//! copied "enters" abilities (CR 603.6a, 614.12), and copiable values the copy effect sets.

use crate::r_p175_common::*;
use crate::r_s06_common::attach_new;
use crate::r_s26_common::{dress_up, fresh, new_tokens};
use mtg_engine::ability::{Effect, PlayerRef, TokenSpec, Value};
use mtg_engine::decision::Decision;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{counters, CardType, Color, ColorSet};
use mtg_engine::*;

/// `p` creates a token (as a resolving effect would) and settles. Returns the token.
fn make_token(t: &mut TestGame, p: PlayerId, spec: TokenSpec) -> ObjectId {
    let before = t.g.battlefield.clone();
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
    let tok = t
        .g
        .battlefield
        .iter()
        .copied()
        .find(|id| !before.contains(id))
        .expect("no token");
    t.g.flush_events();
    t.settle();
    tok
}

/// A 1/1 colorless Thopter artifact creature token (enchantment creature: a white
/// Spirit), as the effect that created it states.
fn simple_token(t: &mut TestGame, enchantment: bool) -> ObjectId {
    let (name, ty, colors) = if enchantment {
        ("Spirit", CardType::Enchantment, ColorSet::single(Color::White))
    } else {
        ("Thopter", CardType::Artifact, ColorSet::default())
    };
    make_token(
        t,
        P0,
        TokenSpec {
            name: name.into(),
            colors,
            supertypes: vec![],
            card_types: vec![ty, CardType::Creature],
            subtypes: vec![name.into()],
            power: Some(1),
            toughness: Some(1),
            abilities: vec![],
            scryfall_name: None,
            pt_values: None,
        },
    )
}

/// How P0 makes token copies in a test: `copy(t, original)` returns the new tokens.
type Maker = fn(&mut TestGame, ObjectId) -> Vec<ObjectId>;

/// Saheeli, the Gifted's −7 ("For each artifact you control, create a token that's a copy
/// of it."), with `original` P0's only artifact.
fn saheeli(t: &mut TestGame, _original: ObjectId) -> Vec<ObjectId> {
    let s = t.battlefield(P0, "Saheeli, the Gifted");
    t.g.add_counters(Entity::Object(s), counters::LOYALTY, 3, None);
    let before = t.g.battlefield.clone();
    act(t, P0, s, 2, &[]);
    new_tokens(t, P0, &before)
}

/// Cogwork Assembler: "{7}: Create a token that's a copy of target artifact."
fn cogwork(t: &mut TestGame, original: ObjectId) -> Vec<ObjectId> {
    let c = t.battlefield(P0, "Cogwork Assembler");
    let before = t.g.battlefield.clone();
    act(t, P0, c, 0, &[obj(original)]);
    new_tokens(t, P0, &before)
}

/// Red Sun's Twilight with X = 5, destroying `original`.
fn red_sun(t: &mut TestGame, original: ObjectId) -> Vec<ObjectId> {
    lots(t, P0);
    let spell = t.hand(P0, "Red Sun's Twilight");
    let before = t.g.battlefield.clone();
    t.cast(P0, spell).x(5).targets(&[obj(original)]).go();
    t.resolve_all();
    new_tokens(t, P0, &before)
}

/// P0's Terra, transformed into Esper Terra; a lore counter triggers chapter I ("Create a
/// token that's a copy of target nonlegendary enchantment you control.") on `original`.
fn esper_terra(t: &mut TestGame, original: ObjectId) -> Vec<ObjectId> {
    let terra = t.battlefield(P0, "Terra, Magical Adept // Esper Terra");
    mtg_engine::dfc::transform(&mut t.g, terra);
    t.g.recompute();
    let terra = t.g.current(terra);
    assert_eq!(t.obj_now(terra).chars.name.as_str(), "Esper Terra");
    let before = t.g.battlefield.clone();
    t.answer_targets(P0, &[obj(t.g.current(original))]);
    t.g.add_counters(Entity::Object(terra), counters::LORE, 1, None);
    t.g.flush_events();
    t.resolve_all();
    new_tokens(t, P0, &before)
        .into_iter()
        .filter(|id| *id != terra)
        .collect()
}

/// Firion, Wild Rose Warrior on the battlefield; then `name` (a nontoken Equipment)
/// enters under P0's control, made by `enter`. Returns (the Equipment, Firion's copies).
fn firion(t: &mut TestGame, enter: impl FnOnce(&mut TestGame) -> ObjectId) -> (ObjectId, Vec<ObjectId>) {
    t.battlefield(P0, "Firion, Wild Rose Warrior");
    let before = t.g.battlefield.clone();
    let e = enter(t);
    t.resolve_all();
    let toks = new_tokens(t, P0, &before)
        .into_iter()
        .filter(|id| t.obj(*id).chars.has_subtype("Equipment"))
        .collect();
    (e, toks)
}

fn one(v: Vec<ObjectId>) -> ObjectId {
    assert_eq!(v.len(), 1, "expected one token: {v:?}");
    v[0]
}

fn name(t: &TestGame, id: ObjectId) -> String {
    t.obj_now(id).chars.name.to_string()
}

/// The number of "choose an option" (color) decisions P0 was asked since `from`.
fn options_asked(t: &TestGame, from: usize) -> usize {
    t.asked()[from..]
        .iter()
        .filter(|(p, d)| *p == P0 && matches!(d, Decision::ChooseOption { .. }))
        .count()
}

#[test]
fn copied_artifact_enters_abilities_work() {
    cr!("707.5", "603.6a", "614.1c", "614.12");
    ruling!(
        "Saheeli, the Gifted",
        "Any enters-the-battlefield abilities of a copied artifact will trigger when the token enters the battlefield. Any \"as [this artifact] enters the battlefield\" or \"[this artifact] enters the battlefield with\" abilities of the artifact will also work."
    );
    ruling!(
        "Red Sun's Twilight",
        "Any enters-the-battlefield abilities of the copied artifact will trigger when the token enters the battlefield. Any \"as [this artifact] enters the battlefield\" or \"[this artifact] enters the battlefield with\" abilities of the artifact will also work."
    );
    supported("Saheeli, the Gifted");
    supported("Red Sun's Twilight");
    for (who, maker) in [("Saheeli", saheeli as Maker), ("Red Sun's Twilight", red_sun)] {
        // Prophetic Prism: "When this artifact enters, draw a card."
        let mut t = TestGame::new(2);
        let prism = t.battlefield(P0, "Prophetic Prism");
        let hand = t.hand_size(P0);
        let tok = one(maker(&mut t, prism));
        assert_eq!(name(&t, tok), "Prophetic Prism", "{who}");
        assert_eq!(t.hand_size(P0), hand + 1, "{who}: the token's trigger drew");
        // Triskelion: "enters with three +1/+1 counters".
        let mut t = TestGame::new(2);
        let trisk = t.battlefield(P0, "Triskelion");
        let tok = one(maker(&mut t, trisk));
        assert_eq!(t.counters(tok, counters::PLUS1), 3, "{who}");
        // Caged Sun: "As this artifact enters, choose a color." — chosen for the token.
        let mut t = TestGame::new(2);
        let sun = t.battlefield(P0, "Caged Sun");
        let from = t.asked().len();
        one(maker(&mut t, sun));
        assert_eq!(options_asked(&t, from), 1, "{who}: color chosen for the token");
    }
}

#[test]
fn copied_equipment_and_enchantment_enters_abilities_work() {
    cr!("707.5", "603.6a", "614.1c", "614.12");
    ruling!(
        "Firion, Wild Rose Warrior",
        "If the Equipment copied by the token had any \"when [this permanent] enters\" abilities, the token also has those abilities, and they'll trigger when it's created. Similarly, any \"as [this permanent] enters\" or \"[this permanent] enters with\" abilities that the token has copied will also work."
    );
    ruling!(
        "Terra, Magical Adept // Esper Terra",
        "If the enchantment copied by the token had any \"when [this permanent] enters\" abilities, the token also has those abilities, and they'll trigger when it's created. Similarly, any \"as [this permanent] enters\" or \"[this permanent] enters with\" abilities that the token has copied will also work."
    );
    supported("Firion, Wild Rose Warrior");
    supported("Terra, Magical Adept // Esper Terra");
    // Firion: Batterskull's living weapon (a Germ for the token too); Malefic Scythe
    // enters with a soul counter (the token too).
    let mut t = TestGame::new(2);
    let (_, toks) = firion(&mut t, |t| t.enter(P0, "Batterskull"));
    one(toks);
    assert_eq!(with_subtype(&t, P0, "Germ").len(), 2);
    let mut t = TestGame::new(2);
    let (_, toks) = firion(&mut t, |t| t.enter(P0, "Malefic Scythe"));
    let tok = one(toks);
    assert_eq!(t.counters(tok, "soul"), 1);
    // Esper Terra: a token Banishing Light exiles a second creature; a token Story
    // Circle has its color chosen.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(a)]);
    let light = t.enter(P0, "Banishing Light");
    t.resolve_all();
    assert_eq!(t.zone(a), Zone::Exile);
    t.answer_targets(P0, &[obj(b)]);
    let tok = one(esper_terra(&mut t, light));
    assert_eq!(name(&t, tok), "Banishing Light");
    assert_eq!(t.zone(b), Zone::Exile);
    let mut t = TestGame::new(2);
    let circle = t.battlefield(P0, "Story Circle");
    let from = t.asked().len();
    one(esper_terra(&mut t, circle));
    assert_eq!(options_asked(&t, from), 1);
}

#[test]
fn copied_x_costs_are_zero() {
    cr!("707.2", "107.3m", "202.3e");
    ruling!(
        "Saheeli, the Gifted",
        "If a copied artifact has {X} in its mana cost, X is considered to be 0."
    );
    ruling!(
        "Firion, Wild Rose Warrior",
        "If the copied Equipment has {X} in its mana cost, X is 0."
    );
    let mut t = TestGame::new(2);
    let chalice = crate::r_s27_common::chalice_x2(&mut t, P0);
    let tok = one(saheeli(&mut t, chalice));
    crate::r_s27_common::chalice_copy_has_x_0(&t, tok);
    // Sigil of Distinction ({X}, "enters with X charge counters") cast with X = 2.
    let mut t = TestGame::new(2);
    let (sigil, toks) = firion(&mut t, |t| {
        lots(t, P0);
        let c = t.hand(P0, "Sigil of Distinction");
        t.cast(P0, c).x(2).go();
        c
    });
    assert_eq!(t.counters(t.g.current(sigil), "charge"), 2);
    let tok = one(toks);
    assert_eq!(t.counters(tok, "charge"), 0);
    assert_eq!(t.obj_now(tok).chars.mana_value(), 0);
}

#[test]
fn copies_of_tokens_use_the_original_token_characteristics() {
    cr!("707.2", "111.3", "707.3");
    ruling!(
        "Saheeli, the Gifted",
        "If a copied artifact is a token, the token that's created copies the original characteristics of that token as stated by the effect that created the token."
    );
    ruling!(
        "Cogwork Assembler",
        "If the copied artifact is a token, the token created with Cogwork Assembler copies the original characteristics of that token as stated by the effect that that created that token."
    );
    ruling!(
        "Terra, Magical Adept // Esper Terra",
        "If the copied enchantment is a token, the new token that's created copies the original characteristics of that token as stated by the effect that created the token."
    );
    supported("Cogwork Assembler");
    for (who, maker, ench) in [
        ("Saheeli", saheeli as Maker, false),
        ("Cogwork Assembler", cogwork, false),
        ("Esper Terra", esper_terra, true),
    ] {
        let mut t = TestGame::new(2);
        let tok = simple_token(&mut t, ench);
        dress_up(&mut t, tok);
        let copy = one(maker(&mut t, tok));
        let o = t.obj_now(copy);
        assert_eq!(name(&t, copy), if ench { "Spirit" } else { "Thopter" }, "{who}");
        assert_eq!(t.pt(copy), (1, 1), "{who}");
        let colors = if ench {
            ColorSet::single(Color::White)
        } else {
            ColorSet::default()
        };
        assert_eq!(o.chars.colors, colors, "{who}");
        assert!(fresh(&t, copy), "{who}");
    }
}

#[test]
fn copies_of_copies_enter_as_what_was_copied() {
    cr!("707.3", "706.2");
    ruling!(
        "Saheeli, the Gifted",
        "If a copied artifact is copying something else (for example, if the copied artifact is a Sculpting Steel), then the token enters the battlefield as whatever that artifact copied."
    );
    ruling!(
        "Firion, Wild Rose Warrior",
        "If the copied Equipment is copying something else, then the token enters as whatever that Equipment copied, with the listed exception."
    );
    ruling!(
        "Terra, Magical Adept // Esper Terra",
        "If the copied enchantment is copying something else, then the token enters as whatever that enchantment copied."
    );
    supported("Sculpting Steel");
    supported("Copy Artifact");
    supported("Copy Enchantment");
    // Saheeli: Sculpting Steel copying Ornithopter. Saheeli copies P0's artifacts — the
    // Ornithopter is P1's, so P0's only artifact is the Steel.
    let mut t = TestGame::new(2);
    let orn = t.battlefield(P1, "Ornithopter");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(orn)]);
    let steel = t.enter(P0, "Sculpting Steel");
    t.resolve_all();
    assert_eq!(name(&t, steel), "Ornithopter");
    let tok = one(saheeli(&mut t, steel));
    assert_eq!(name(&t, tok), "Ornithopter");
    assert_eq!(t.pt(tok), (0, 2));
    // Firion: Copy Artifact entering as a copy of Bonesplitter (an Equipment enchantment):
    // the token is a Bonesplitter (an enchantment too) whose equip costs {2} less.
    let mut t = TestGame::new(2);
    let splitter = t.battlefield(P1, "Bonesplitter");
    let (copy, toks) = firion(&mut t, |t| {
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(splitter)]);
        t.enter(P0, "Copy Artifact")
    });
    assert_eq!(name(&t, copy), "Bonesplitter");
    let tok = one(toks);
    assert_eq!(name(&t, tok), "Bonesplitter");
    assert!(t.obj_now(tok).is(CardType::Enchantment));
    assert!(crate::r_p198_common::has_ability(&t, tok, "{2} less"));
    // Esper Terra: Copy Enchantment copying Glorious Anthem.
    let mut t = TestGame::new(2);
    let anthem = t.battlefield(P1, "Glorious Anthem");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(anthem)]);
    let copy = t.enter(P0, "Copy Enchantment");
    t.resolve_all();
    assert_eq!(name(&t, copy), "Glorious Anthem");
    let tok = one(esper_terra(&mut t, copy));
    assert_eq!(name(&t, tok), "Glorious Anthem");
}

#[test]
fn token_copies_copy_only_the_printed_permanent() {
    cr!("707.2", "707.9b");
    ruling!(
        "Red Sun's Twilight",
        "Each token copies exactly what was printed on the original artifact and nothing else (unless that artifact is copying something else or is a token; see below). It doesn't copy whether that artifact is tapped or untapped, whether it has any counters on it or Auras and Equipment attached to it, or any non-copy effects that have changed its power, toughness, types, color, or so on."
    );
    ruling!(
        "Cogwork Assembler",
        "The token copies exactly what was printed on the original artifact and nothing else (unless that artifact is copying something else or is a token; see below). It doesn't copy whether that artifact is tapped or untapped, whether it has any counters on it or Auras and Equipment attached to it, or any non-copy effects that have changed its power, toughness, types, color, or so on."
    );
    ruling!(
        "Saheeli, the Gifted",
        "While resolving Saheeli's third ability, each token copies exactly what was printed on the original artifact and nothing else (unless that artifact is copying something else or is a token; see below). It doesn't copy whether that artifact is tapped or untapped, whether it has any counters on it or Auras attached to it, or any non-copy effects that have changed its types, color, or so on."
    );
    ruling!(
        "Terra, Magical Adept // Esper Terra",
        "The token copies exactly what was printed on the original enchantment (unless that enchantment is copying something else or is a token; see below). It doesn't copy whether that enchantment is tapped or untapped, whether it has any counters on it or any Auras or Equipment attached to it, or any non-copy effects that have changed its types, color, power and toughness, and so on."
    );
    ruling!(
        "Firion, Wild Rose Warrior",
        "The token copies exactly what was printed on the original Equipment, with the listed exception (unless that Equipment is copying something else; see below). It doesn't copy whether that Equipment is tapped or untapped, whether it has any counters on it or any Auras or Equipment attached to it, or any non-copy effects that have changed its types, color, power and toughness, and so on."
    );
    // An Ornithopter (0/2 colorless artifact creature), tapped, with two +1/+1 counters,
    // +3/+3 and green-blue, wearing Holy Strength and a Bonesplitter; Leafcrown Dryad (2/2
    // green enchantment creature) the same for Esper Terra.
    for (who, maker, card, pt, colors) in [
        ("Red Sun's Twilight", red_sun as Maker, "Ornithopter", (0, 2), ColorSet::default()),
        ("Cogwork Assembler", cogwork, "Ornithopter", (0, 2), ColorSet::default()),
        ("Saheeli", saheeli, "Ornithopter", (0, 2), ColorSet::default()),
        ("Esper Terra", esper_terra, "Leafcrown Dryad", (2, 2), ColorSet::single(Color::Green)),
    ] {
        let mut t = TestGame::new(2);
        let orig = t.battlefield(P0, card);
        attach_new(&mut t, P0, "Holy Strength", orig);
        if who != "Saheeli" {
            // (Saheeli would copy an Equipment of P0's too.)
            attach_new(&mut t, P1, "Bonesplitter", orig);
        }
        dress_up(&mut t, orig);
        let tok = one(maker(&mut t, orig));
        assert_eq!(name(&t, tok), card, "{who}");
        assert_eq!(t.pt(tok), pt, "{who}");
        assert_eq!(t.obj_now(tok).chars.colors, colors, "{who}");
        assert!(fresh(&t, tok), "{who}");
    }
    // Firion: Bonesplitter tapped, with counters and recolored before the trigger resolves.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Firion, Wild Rose Warrior");
    let before = t.g.battlefield.clone();
    let b = t.enter(P0, "Bonesplitter");
    t.settle();
    dress_up(&mut t, b);
    t.resolve_all();
    let tok = one(
        new_tokens(&t, P0, &before)
            .into_iter()
            .filter(|id| *id != b)
            .collect(),
    );
    assert_eq!(name(&t, tok), "Bonesplitter");
    assert_eq!(t.obj_now(tok).chars.colors, ColorSet::default());
    assert!(fresh(&t, tok));
}

#[test]
fn red_suns_twilight_copies_only_legal_targets() {
    cr!("608.2b", "608.2c");
    ruling!(
        "Red Sun's Twilight",
        "If X is 5 or more and any of the target artifacts become illegal, you create a token only for each of the artifacts that is still a legal target."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Ornithopter");
    let b = t.battlefield(P1, "Prophetic Prism");
    lots(&mut t, P0);
    let spell = t.hand(P0, "Red Sun's Twilight");
    let before = t.g.battlefield.clone();
    t.cast(P0, spell).x(5).targets(&[obj(a), obj(b)]).go();
    bounce(&mut t, b);
    t.resolve_all();
    let toks = new_tokens(&t, P0, &before);
    assert_eq!(toks.len(), 1);
    assert_eq!(name(&t, toks[0]), "Ornithopter");
    assert!(t.in_hand(P1, "Prophetic Prism"));
}

/// P0's Séance (entering in P1's turn) turns the creature card `card` in P0's graveyard
/// into a token copy at the beginning of P0's next upkeep. Returns the token.
fn seance(t: &mut TestGame, card: ObjectId) -> ObjectId {
    pass_turn(t);
    t.battlefield(P0, "Séance");
    t.answer_targets(P0, &[obj(card)]);
    t.answer_yes(P0, true);
    let before = t.g.battlefield.clone();
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    one(new_tokens(t, P0, &before))
}

#[test]
fn seance_token_copies_the_card_as_a_spirit() {
    cr!("707.2", "707.9b", "302.6");
    ruling!(
        "Séance",
        "The token copies exactly what was printed on the original card and nothing else. It doesn’t copy any information about the object the card was before it was put into your graveyard."
    );
    ruling!(
        "Séance",
        "The token doesn’t have haste (unless the card it’s copying has haste) and usually won’t be able to attack."
    );
    supported("Séance");
    // Grizzly Bears that died tapped with counters, pumped and recolored: the token is a
    // fresh 2/2 green Bear Spirit, summoning sick.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    dress_up(&mut t, bears);
    kill(&mut t, bears);
    let card = t.g.current(bears);
    let tok = seance(&mut t, card);
    assert_eq!(name(&t, tok), "Grizzly Bears");
    assert!(t.obj_now(tok).chars.has_subtype("Spirit"));
    assert_eq!(t.pt(tok), (2, 2));
    assert_eq!(t.obj_now(tok).chars.colors, ColorSet::single(Color::Green));
    assert!(fresh(&t, tok));
    t.advance_to(P0, Step::BeginningOfCombat);
    assert!(!crate::r_s09_common::legal_attack(&mut t, &[(tok, pl(P1))]));
    // A copy of Raging Goblin (haste) can attack.
    let mut t = TestGame::new(2);
    let card = t.graveyard(P0, "Raging Goblin");
    let tok = seance(&mut t, card);
    t.advance_to(P0, Step::BeginningOfCombat);
    assert!(crate::r_s09_common::legal_attack(&mut t, &[(tok, pl(P1))]));
}

#[test]
fn a_populated_seance_token_is_a_spirit_and_isnt_exiled() {
    cr!("701.36a", "707.2", "707.9b", "603.7c");
    ruling!(
        "Séance",
        "The delayed triggered ability that exiles the token isn’t a characteristic of the token. If an effect such as populate copies the token, the new copy won’t be exiled."
    );
    ruling!(
        "Séance",
        "The token is a Spirit in addition to its other types. This is a copiable value of the token that other effects may copy."
    );
    supported("Rootborn Defenses");
    let mut t = TestGame::new(2);
    let card = t.graveyard(P0, "Grizzly Bears");
    let tok = seance(&mut t, card);
    t.set_step(P0, Step::PrecombatMain);
    let before = t.g.battlefield.clone();
    cast_at(&mut t, P0, "Rootborn Defenses", &[]);
    t.answer_choose(P0, &[obj(tok)]);
    t.resolve_all();
    let copy = one(new_tokens(&t, P0, &before));
    assert!(t.obj_now(copy).chars.has_subtype("Spirit"));
    assert!(t.obj_now(copy).chars.has_subtype("Bear"));
    end_step(&mut t);
    assert!(!t.on_battlefield(tok), "the Séance token is exiled");
    assert!(t.on_battlefield(copy), "the populated copy isn't");
}
