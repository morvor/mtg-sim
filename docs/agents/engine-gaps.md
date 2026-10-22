# Engine gap inventory (mtg-sim)

Base: `85f020d` (branch claude/mtg-rules-engine-lkr27h). Sources: 787 `remaining_gaps` lines from
implementer/reviewer journals (`journal-gaps.txt`, a superset of the 508 lines in `reported-gaps.txt`),
engine source comments, `docs/*-exemptions/*.tsv`, the owner's list of known gaps, and spot reads of the
code. Every gap below was checked against the current code (file:line evidence). Card support was
checked with `mtg-tools unsupported --card` (binary built from main+S30, i.e. a few commits behind HEAD;
where it mattered the code was read directly). "UNS" = the card's line is reported unsupported today. Note:
that command silently skips cards that are legal in no format (ante cards, Un-cards, planes) and needs the
full "A // B" name of split/flip/double-faced cards; such cards were checked by reading the code instead.

Out of scope by owner decision: Contraptions, host/augment. Parse-only gaps (phrases the compiler doesn't
recognise but the engine could execute) are not listed as gaps; see "Dropped" at the end.

Size: S = < 1 day, M = 1-3 days, L = > 3 days for one engineer-agent.

Summary: 33 gap entries (G01-G33) covering about 75 distinct confirmed engine gaps, grouped into 14 work
items (`gap-items.json`, briefs in `items/<slug>.md`). Sections below are in work-item priority order.

---

## Index: gaps by work item

| Work item (priority order) | Gaps |
|---|---|
| 1. `gap-effect-language` | G29 Conditional and open-ended repetition ("repeat this process"), G22 Tokens whose power/toughness is a value ("create an X/X ... token, where X is ...") |
| 2. `gap-activation` | G03 Activated-ability cost modification is incomplete, G04 Permissions to activate abilities outside their normal timing or limits |
| 3. `gap-cast-permissions` | G18 Play/cast grants from resolved effects are too coarse, G31 Choosing which permission a cast or land play uses; per-type permissions, G21 Foretell from exile only offers the front face |
| 4. `gap-trigger-timing` | G09 Trigger conditions are checked after the whole resolution, not immediately after each event, G13 Keyword parameters that vary ("bushido X") are frozen when the trigger triggers, G30 "Except the first one they draw in each of their draw steps" |
| 5. `gap-targeting` | G06 Target constraints across several targets (group requirements), G07 Exact target counts that depend on choices (multikicker, "X target creatures") |
| 6. `gap-event-causes` | G08 Who put counters, and counters put as a cost vs. by an effect, G16 Events don't record what caused a destruction or a counter |
| 7. `gap-spell-costs` | G01 Alternative costs offered by other objects ("rather than pay the mana cost for spells you cast"), G02 Optional/choice additional costs and flash-for-cost granted to classes of spells by other objects, G20 Cost parts that look only at your own zones |
| 8. `gap-zone-moves` | G28 Auras attached to (or targeting) cards outside the battlefield, G23 "Instead" zone-change replacements drop all destination details, G24 Fabricate ignores counters being prevented |
| 9. `gap-simultaneity` | G10 "Each player ..." effects run one player at a time instead of choices-then-simultaneous, G25 "Shuffle ... into their library" doesn't shuffle when there's nothing to shuffle in |
| 10. `gap-mana-payment` | G05 Paying costs: X restrictions and alternative ways to pay mana symbols, G19 Mana: spend triggers for abilities, symbol-type restrictions, player mana abilities |
| 11. `gap-ability-grants` | G11 Gaining the activated (and triggered) abilities of other objects, G12 Losing or targeting by part of a keyword: one protection/landwalk instance; "hexproof from" abilities, G15 Linked abilities that refer to the player or object a previous ability affected (CR 607.1) |
| 12. `gap-rule-statics` | G14 Rule-modifying statics the engine has no representation for |
| 13. `gap-combat` | G17 Combat declarations and as-though permissions |
| 14. `gap-variants-misc` | G26 Grand Melee and range-of-influence fidelity, G27 Commander eligibility ignores abilities that work outside the game, G33 Ante: anteing a permanent and exchanging ownership, G32 Small rules-fidelity bugs |

## 1. gap-effect-language: Effect language: open-ended repetition and tokens with value-defined power/toughness

### G29. Conditional and open-ended repetition ("repeat this process")
- Rules: CR 608.2c (instructions in order), the cards' own loop text; rulings below.
- Engine now: `Effect::Repeat { times: Value, effect }` (ability.rs:2661-2664) repeats a number of times
  fixed before the first iteration (resolve.rs:152-157); "Repeat this process once" has a follow-up
  pattern (oracle/patterns/repeat_process.rs). Nothing repeats while a condition holds, until a player
  stops, or "any number of times" with a choice each time. 40 unsupported lines contain "repeat this
  process".
- Cards (UNS): Ad Nauseam, Primal Surge, Grindstone, Sphinx's Tutelage, Tainted Pact, Cultivator Colossus, Eureka, Hypergenesis, Countryside Crusher, Hoarder's Greed,
  Insatiable Frugivore, Tyrant of Discord, Grist, the Hunger Tide (+1), Kindle the Carnage, Trade Secrets,
  Helm of Obedience ("repeats this process until ...").
- Rulings: Primal Surge "Repeating the process includes the instruction to repeat the process", "If putting
  any of those permanent cards onto the battlefield causes abilities to trigger, those abilities will wait to
  go on the stack until Primal Surge has finished resolving"; Ad Nauseam "Each time you put the revealed card
  into your hand and lose the appropriate amount of life, you decide whether to continue", "even if your
  life total has been reduced to 0 or less".
- Approach: `Effect::RepeatWhile { body, cond: Condition, may: Option<PlayerRef>, max_iterations }`
  (condition re-evaluated after each pass, using the pass's "this way" results; a safety cap for loops the
  rules leave to the players, CR 104.4b/732).
- Size: S-M

### G22. Tokens whose power/toughness is a value ("create an X/X ... token, where X is ...")
- Rules: CR 111.3/111.4 (the effect that creates a token defines its characteristics), 107.3 (X fixed
  as the effect is applied, 608.2h).
- Engine now: `TokenSpec { power: Option<i32>, toughness: Option<i32>, .. }` (ability.rs:1645-1656) and
  `Effect::CreateToken { spec: TokenSpec, .. }` (ability.rs:2806-2812) can only describe fixed P/T;
  there is no oracle pattern or engine path for X/X tokens (no hits for "X/X" under src/oracle). 83
  unsupported lines contain "X/X".
- Cards (UNS): Gelatinous Genesis ("Create X X/X green Ooze creature tokens"), Spoils of Blood, Mystic
  Genesis, Promise of Power, Corpse Cobble, Kin-Tree Invocation, Tumbleweed Rising, Formless Genesis,
  Sylvan Offering, Nissa, Ascended Animist, Saint Elenda, Dance of the Tumbleweeds (spree), Pest
  Infestation; token copies with value exceptions (Soul Separator, S27 #67).
- Approach: let `CreateToken`/`TokenSpec` carry `Value` P/T (or a `pt: Option<(Value, Value)>` override
  evaluated once at creation into the token's copiable values), keeping `TokenSpec` usable in
  replacements (`PlusTokens`) and predefined tokens.
- Size: M (core AST change touching every `TokenSpec` construction)

## 2. gap-activation: Activated abilities: complete cost modification and activation timing permissions/restrictions

### G03. Activated-ability cost modification is incomplete
- Rules: CR 602.2b (applies 601.2b-h to abilities), 601.2f (total cost: increases, then reductions;
  target-dependent changes are known because targets are chosen first), 118.7, 118.9 (alternative costs
  exist for activated abilities too: "rather than pay the equip cost").
- Engine now: `Game::ability_total_cost` (casting.rs:1846-1920) applies modifiers in static order (a
  reduction can be applied before a later increase), ignores `ReduceColored` and `AlternativeCost`
  (`_ => {}` at casting.rs:1900), and matches only the source (`self.matches(src, f)`), never the
  ability's targets — it isn't even given the stack object, although targets are chosen before costs
  (casting.rs:2063-2069).
  There is also no "This effect can't reduce the mana in that cost to less than one mana" floor anywhere
  in the engine (no hits outside oracle/).
- Cards (UNS): Heartstone, Zirda, the Dawnwaker, Training Grounds, Biomancer's Familiar, Power Artifact
  ("cost {2} less ... can't reduce the mana in that cost to less than one mana"); Kíli the Resourceful
  ("pay {0} rather than pay the equip cost of the first equip ability you activate each turn"), Highway
  Reaver (unearth), New Perspectives (cycling); Dwarven Mauler and 3 others ("Equip abilities you activate
  that target ~ cost {N} less"), Kopala, Warden of Waves ("Abilities your opponents activate that target a
  Merfolk you control cost {2} more").
- Rulings: Training Grounds "takes the total cost to activate ... into account, not just the cost printed
  on it" (Suppression Field example: increases first), "If an activated ability ... costs no generic mana
  ... it won't increase the cost to include a mana payment of {1}"; Power Artifact "Only affects the generic
  mana part"; Zirda: same floor wording.
- Approach: give `ability_total_cost` the stack id (targets) and the 601.2f ordering used for spells
  (collect increases, then reductions), support `ReduceColored` and a `floor_one_mana` flag on
  reductions (applies only if the cost has mana; never adds mana), and add alternative activation costs
  as an announced choice (with "first ... each turn" tracking).
- Size: M

### G04. Permissions to activate abilities outside their normal timing or limits
- Rules: CR 602.5d (sorcery-timing activation), 606.3 (loyalty abilities once per turn, sorcery timing),
  302.6 / 609.4 ("as though it had haste" for {T} abilities).
- Engine now: `can_activate` (casting.rs:1698-1790) hard-codes the sorcery check
  (`act.timing == Sorcery || act.is_loyalty`, casting.rs:1729-1732) and loyalty's once-per-turn limit
  (casting.rs:1761-1767); no static or effect can relax them (`kw::activation_allowed` can only forbid). The {T}/{Q} payability check accepts only the Haste keyword
  (casting.rs:2316-2317); there is no "as though it had haste" permission (as_though.rs implements only
  spend-as-any-color and other-graveyards).
- Cards (UNS): Leonin Shikari, Forge Anew (equip at instant speed), The Wandering Emperor, Teferi,
  Temporal Archmage (emblem), Teferi, Master of Time, Jace's Machinations (loyalty at instant speed on any
  turn), Oath of Teferi, Urza, Planeswalker (loyalty twice each turn), Thousand-Year Elixir ("activate
  abilities of creatures you control as though those creatures had haste").
- Rulings: Leonin Shikari "You're still subject to any other restrictions on activating equip abilities";
  Oath of Teferi "you may activate the same ability of a planeswalker twice", "If you somehow control more
  than one Oath of Teferi, you won't be able to activate ... more than twice"; Thousand-Year Elixir "doesn't
  actually grant haste ... nor does it let you attack with them as though they had haste".
- The opposite restriction is missing too: "Activate only as an instant" on a mana ability (CR 602.5e) must
  stop it being activated while a spell is being cast or paid for (CR 605.3a would otherwise allow it);
  `ActivationTiming::Instant` is the unrestricted default (ability.rs:209-225), so there's no way to say
  it. Cards (UNS): Lion's Eye Diamond, Diamond Lion, Rhystic Cave (ruling LED: "it can only be activated at
  times when you can cast an instant. Yes, this is a bit weird").
- Approach: a `PlayerModification`/static "activation permission" (filter on ability kind/source:
  equip, loyalty of filter, all abilities of filter) with fields instant_timing / any_turn / per_turn_limit
  / ignore_summoning_sickness, consulted in `can_activate` and the {T} payability check; an
  `ActivationTiming::AsInstant` that requires priority with no payment in progress.
- Size: M

## 3. gap-cast-permissions: Casting permissions: richer play grants, choosing which permission is used, foretell faces

### G18. Play/cast grants from resolved effects are too coarse
- Rules: CR 601.3, 305.9 / 305.1 ("you may cast that card" doesn't let you play a land), 118.9 (an
  alternative cost attached to a permission), 601.3e, 702.8 (flash timing given by the permission).
- Engine now: `PlayGrant { player, object, duration, free, source, turn }` (casting.rs:53-60) is created
  by `Effect::GrantPlayPermission { who, what, duration, free }` (ability.rs:3216-3221) and always permits
  both playing a land and casting (`permission_allows_with` ignores `land` for grants, casting.rs:384-390).
  A grant can't be cast-only, can't carry an alternative cost ("pay life equal to its mana value rather than
  pay its mana cost") or an extra cost ("A spell cast this way costs {2} more"), and can't give flash timing
  ("you may cast them as though they had flash").
- Cards (UNS; 126 unsupported lines contain "you may cast that card/it/those cards", part of them for
  this reason): Ragavan, Nimble Pilferer ("exile the top card of that player's library. Until end of turn, you
  may cast that card"); Hurl Through Hell; Xander's Pact, Inside Information ("If you cast a spell this way,
  pay life equal to its mana value rather than pay its mana cost": an alternative cost on the grant); Invasion of Gobakhan ("its owner may play it. A spell cast this way costs {2}
  more"); Azula, Cunning Usurper ("cast them as though they had flash"). Patterns currently accept only
  "you may play" (or texts whose card can't be a land) because a grant worded "you may cast" would let a
  land be played (oracle-card_flow implementer note).
- Approach: extend `PlayGrant`/`GrantPlayPermission` with `spells_only`, `alt_cost: Option<Cost>`,
  `extra_cost`, `flash` (mirroring `PlayPermission`, ability.rs:2268-2280) and honour them in
  `permission_allows_with` and `cast_options`.
- Size: S-M

### G31. Choosing which permission a cast or land play uses; per-type permissions
- Rules: CR 601.2/601.3 (a player announces how a card is cast, including which permission allows casting
  it from a zone), 305.1/305.9; Muldrotha rulings.
- Engine now: kw/once_each_turn_cast.rs (header, lines 9-14): a spell cast from the graveyard "uses the
  permission of the first such object that allows it"; there is no decision for which permission is used,
  and once-each-turn usage is recorded per object (`TurnHistory::once_permissions_used: Vec<ObjectId>`,
  game.rs:446), so "a land and a permanent spell of each permanent type" can't be tracked.
- Cards (UNS): Muldrotha, the Gravetide ("During each of your turns, you may play a land and cast a
  permanent spell of each permanent type from your graveyard"); Karador + Yawgmoth's Will / Lurrus combos
  consume the wrong permission.
- Rulings: Muldrotha "If multiple effects allow you to play a card from your graveyard, you must announce
  which permission you're using as you begin to play the card", "you may cast an artifact creature spell as
  your artifact spell and cast another artifact creature spell as your creature spell", "Use the type of the
  card as it's played or cast to determine which permanent type to count it as".
- Approach: make the permission part of the cast/land-play action (a `CastOption` field listing the
  permission source and, for typed permissions, the type slot), consume exactly that one, and key usage by
  (source, slot).
- Size: M

### G21. Foretell from exile only offers the front face
- Rules: CR 702.143 (foretell), 712.11b (modal DFC cast as either face), 601.3e.
- Engine now: kw/foretell.rs:225-233 builds every foretell option with `FaceState::Front`; foretell costs
  "equal to its mana cost reduced by {2}" use the front face's cost.
- Cards: Ethereal Valkyrie / any MDFC foretold (Kolvori, God of Kinship // The Ringhart Crest example).
- Rulings: Ethereal Valkyrie "If you foretell a modal double-faced card, the foretell cost will be based on
  the mana cost of the face you cast from exile ... you can't play that card as a land".
- Approach: iterate `castable_faces` as escape/flashback already do (kw/escape.rs:54).
- Size: S

## 4. gap-trigger-timing: Trigger detection at event time; resolution-time keyword values; draw-step draw tracking

### G09. Trigger conditions are checked after the whole resolution, not immediately after each event
- Rules: CR 603.2 (an ability triggers when its trigger event occurs), 603.10 ("objects that exist
  immediately after an event are checked ... and continuous effects that exist at that time are used to
  determine ... what the objects involved in the event look like"; leaves-the-battlefield looks back),
  603.6a (enters triggers checked each time an event puts permanents onto the battlefield), 608.2c
  (later instructions of the same resolution happen after that check).
- Engine now: events are queued in `self.events` and trigger detection runs in `flush_events`
  (triggers.rs:124-161), which recomputes the *current* state first (triggers.rs:130-132). A resolving
  spell or ability flushes only at its end (stack.rs:802) or before creating a delayed trigger
  (resolve.rs:1522, 1566). So a later instruction of the same resolution changes whether earlier events
  match: an enters trigger with a power/type/controller condition sees the token after "Put two +1/+1
  counters on it"; `EnteredPermanent::as_entered` (game.rs:383-385) is likewise taken at flush time
  (reviewer note on wf_b2f642ec). The amass action works around it locally; living weapon, job select and
  For Mirrodin! needed their own "check triggers between creating and attaching" fixes.
- Cards: any "Create a token ... Put N +1/+1 counters on it"/"It gains ..." effect combined with
  conditional enters triggers (e.g. Garruk's Uprising "Whenever a creature with power 4 or greater enters",
  Elemental Bond, Guardian Project), "The token enters with N counters" built as create-then-put (reviewer
  note wf_a01a7158: anything checking the token's P/T as it enters sees a 0/0).
- Approach: snapshot what triggers need at event time — evaluate `detect_triggers` per event right after
  each primitive action (or record each event with the characteristics of the objects involved, as
  `as_entered` does, at emission rather than flush), keeping batch boundaries for "one or more" triggers.
  Then remove the local workarounds.
- Size: L

### G13. Keyword parameters that vary ("bushido X") are frozen when the trigger triggers
- Rules: CR 107.3c/608.2h (X defined by text is evaluated as the ability resolves), Fumiko ruling.
- Engine now: keyword-derived triggered abilities embed the keyword's current `n` as a constant
  (kw/bushido.rs: `Value::c(n)` in `derived`), and `AddKeywordX` (layers.rs:1060, 1630) evaluates X when
  characteristics are computed, so the bonus is fixed at trigger time.
- Cards: Fumiko the Lowblood (supported; wrong amount if attackers change before resolution).
- Rulings (UNCOVERED): Fumiko "The bushido bonus is calculated each time Fumiko's bushido trigger
  resolves, based on the number of attackers at that time."
- Approach: when a keyword's N comes from `AddKeywordX`, give the derived ability a `Value` that
  re-reads the keyword's value from the source on resolution (or carry the `Value` in `Keyword`).
- Size: S

### G30. "Except the first one they draw in each of their draw steps"
- Rules: CR 504.1 (draw step draw), 121.2; the cards' wording counts draws made during that player's draw
  step.
- Engine now: `Event::Drew { player, card, nth }` (events.rs:121-126) counts draws per turn; nothing records
  which draws happened in the player's current draw step, and the draw replacement events
  (`ReplacementEvent::Draw`, ability.rs:1730) can't be qualified by it.
- Cards (UNS, 10): Orcish Bowmasters, Notion Thief, Hullbreacher, Alhammarret's Archive, Teferi's Ageless
  Insight, Chains of Mephistopheles, Magus of the Chains, Xyris, the Writhing Storm, Leela, Sevateem
  Warrior, Bard, King of Dale.
- Rulings: Notion Thief "If two or more players each control a Notion Thief and a player would draw a card
  other than the first one in their draw step, that player chooses one of the applicable Notion Thief effects
  to apply"; "If an opponent is instructed to draw a card then discard a card, and Notion Thief causes you to
  draw a card instead, that opponent still discards a card".
- Approach: track draws per (player, step) in `TurnHistory`, expose a filter/condition "not the first card
  drawn in their draw step" usable by draw triggers and draw replacements.
- Size: S

## 5. gap-targeting: Target and choice constraints across several objects; exact target counts

### G06. Target constraints across several targets (group requirements)
- Rules: CR 115.1, 115.3, 601.2c (targets chosen with all their requirements), 700.2 (modes), 115.10
  ("each mode must target a different player").
- Engine now: `TargetSpec` (ability.rs:584-601) has only `what`, `min`, `max`, `distinct_from`, `divide`,
  `chosen_by_opponent`, `condition`. Nothing can require targets to be in the same zone/graveyard, to have
  different or the same controllers, to share a creature/card type, to have equal toughness, to have a total
  mana value/power at most N, or to be one per opponent ("for each opponent, up to one target creature that
  player controls"). No code in the engine checks such relations (no hits for "single graveyard",
  "different controller", "same controller" outside oracle/).
- Cards (UNS): Decompose, Arashin Sunshield, Famished Ghoul, Unlicensed Hearse, Pestilent Cauldron, Martyr
  of Bones ("from a single graveyard"); Return from Extinction, Unbury, Raise the Draugr, Secret Tunnel
  ("that share a creature type"); The Trickster-God's Heist ("share a card type"); Simic Guildmage ("another
  target creature with the same controller"); Cloud, Midgar Mercenary/Limit Break ("with different
  controllers"); V.A.T.S. ("with equal toughness"); The Super Hero Civil War, Tocasia, Dig Site Mentor and
  Joshua's chapter III ("total mana value N or less"); Mass Mutiny, Molten Primordial, Hideous Taskmaster,
  In the Darkness Bind Them, Bronzebeak Foragers, Diluvian Primordial, Sheoldred // The True Scriptures,
  Age of Ultron, Battle at the Helvault, Vault 13, The Parting of the Ways ("for each opponent/player, ...
  up to one target ... that player controls"); Vindictive Lich, Balor, Shadrix Silverquill, Splinter & Leo
  ("Each mode must target a different player").
- The same relations are missing for untargeted choices of several cards (searches, "choose", costs):
  "with different names" (Saheeli Rai -X, Transmutation Font, Ormos, Archive Keeper, Battle for Bretagard),
  "for each card type, ... a card of that type" (Atraxa, Grand Unifier), "total mana value X or less" (Rod of
  Absorption); `library::search` and `ask_objects` validate only count and filter per card
  (library.rs:106-135).
- Rulings (UNCOVERED): Return from Extinction / Unbury / Raise the Draugr "the cards must share at least one
  creature type"; Vindictive Lich "You can't choose more modes ... than you have opponents".
- Approach: add a `group: Vec<TargetGroupRule>` to the ability's targeting (SameZone, SameController,
  DifferentControllers, ShareCreatureType, ShareCardType, EqualToughness, TotalAtMost(Value,
  stat), OnePerPlayer(PlayerFilter)) checked in target_rules.rs when choosing (601.2c) and, where the rules
  say so, on resolution (608.2b only checks each target individually, so most are choice-time only); a
  per-player slot expansion for "for each opponent ... target"; a cross-mode "different player" rule.
- Size: L

### G07. Exact target counts that depend on choices (multikicker, "X target creatures")
- Rules: CR 601.2c, 702.33d (multikicker), 115.1 ("X target creatures" means exactly X).
- Engine now: `TargetSpec.min` is a fixed `u32` (ability.rs:586) while `max` is a `Value`; "X target
  creatures"/"one more than the number of times it was kicked" compile as min 1 (or 0), max X, so fewer
  targets may be chosen and more kicks than targets are allowed.
- Cards: Comet Storm, Strength of the Tajuru (supported, but approximate), Thrive, Rot-Curse Rakshasa
  ("each of X target creatures").
- Rulings (UNCOVERED): Comet Storm / Strength of the Tajuru "The number of targets you choose for ... is one
  more than the number of times it's kicked".
- Approach: make `min` a `Value` (about 36 constructions; `TargetSpec::one/up_to` helpers keep most call
  sites unchanged) and evaluate both bounds after X/kicks are announced (601.2b before 601.2c).
- Size: S-M

## 6. gap-event-causes: Event attribution: who put counters (and whether as a cost), what destroyed or countered something

### G08. Who put counters, and counters put as a cost vs. by an effect
- Rules: CR 122.6, 122.6a (the player who puts counters: the controller of the spell/ability, or the player
  the effect names, e.g. tribute's opponent); 118.3 / 602.2 (counters put as a cost aren't put by an
  effect: Doubling Season "loyalty ... put on as a cost, not as an effect").
- Engine now: `Event::CountersAdded { target, kind, n }` (events.rs:140-144) records no player and no
  source; `TriggerCond::CountersPut { filter, kind }` (ability.rs:2404) can't say "you put";
  `TurnHistory::counters_put` is a bare count (game.rs:431). `counter_rules::who_puts_counters`
  (counter_rules.rs:314-320) infers the putter from the source's controller, which is wrong for tribute
  (the chosen opponent puts them, kw/tribute.rs) and has no notion of a cost. Loyalty and "put a counter
  on ~" costs go through the same `add_counters` as effects (casting.rs:2595-2598, 2761-2764), so an
  "If an effect would put one or more counters" replacement can't exclude costs.
- Cards (UNS): All Will Be One, Stocking the Pantry and other "Whenever you put one or more counters on"
  triggers; Doubling Season and Selesnya Loft Gardens ("If an effect would put one or more counters ...";
  Doubling Season's counter half is UNS) need the cost/effect split, while "If you would put" replacements
  (Vorinclex, Monstrous Raider: supported) correctly apply to costs too.
- Rulings: tribute (11 cards, e.g. Nessian Wilds Ravager) "For effects that check which player put counters
  on the entering creature, the player chosen to pay tribute puts those counters on it" (UNCOVERED); Doubling
  Season "if you activate an ability whose cost has you put loyalty counters on a planeswalker, the number you
  put on isn't doubled"; All Will Be One "toxic ... those counters are placed as a single event, and the
  ability triggers one time".
- Approach: add `by: Option<PlayerId>` and `as_cost: bool` to `ReplEvent::AddCounters`/`Event::CountersAdded`
  (and to enters-with-counters), set by every caller (costs pass `as_cost`, tribute passes the opponent);
  add a `by: PlayerRel` to `TriggerCond::CountersPut` and `ReplacementEvent::PutCounters`; an
  `effect_only` flag for "If an effect would put".
- Size: M

### G16. Events don't record what caused a destruction or a counter
- Rules: CR 701.8 (destroy), 701.6 (counter); trigger and condition wording "destroyed/countered by a
  spell or ability an opponent controlled" (Cobra Trap ruling: "A spell or ability destroys a permanent
  only if that spell or ability specifically contains the word 'destroy'"), umbra armor (702.89a: the
  spell or ability that tried to destroy the creature is what destroys the Aura).
- Engine now: `Event::Destroyed { obj }` (events.rs:244-246) and `Event::Countered { what }`
  (events.rs:89-91) carry no cause; `ReplEvent::Destroy` has a `source` (replacement.rs:129-132) but it's
  dropped from the event. `ZoneChange.by` is a player only. Umbra armor's replacement runs
  `Effect::Destroy { what: Sel::This }` in the Aura's own context (kw/umbra_armor.rs:34-42), so the Aura is
  destroyed by itself rather than by the original spell or ability.
- Cards (UNS): Karmic Justice ("Whenever a spell or ability an opponent controls destroys a noncreature
  permanent you control"), Cobra Trap, Summoning Trap (history conditions on that cause).
- Rulings: Karmic Justice "Spells and abilities that cause you to sacrifice permanents will not cause
  Karmic Justice's ability to trigger"; Cobra Trap "destroys a permanent only if that spell or ability
  specifically contains the word 'destroy'"; Hyena Umbra "that spell or ability is what causes the Aura to
  be destroyed instead ... if a spell or ability deals lethal damage ..., the game rules regarding lethal
  damage cause the Aura to be destroyed".
- Approach: add `cause: Option<ObjectId>` (the resolving spell/ability, None for SBAs) to `Destroyed` and
  `Countered`, record it in `TurnHistory`, add trigger/condition forms keyed on its controller, and make
  `ReplacementAction::Instead` effects inherit the replaced event's cause.
- Size: S-M

## 7. gap-spell-costs: Spell costs from other objects: external alternative costs, optional additional costs, minimum costs, any-graveyard costs

### G01. Alternative costs offered by other objects ("rather than pay the mana cost for spells you cast")
- Rules: CR 118.9, 118.9a (only one alternative cost), 601.2b, 601.2f; jump-start (702.133) is not an
  alternative cost, so it combines with one (Radical Idea ruling).
- Engine now: `Game::cast_options` (crates/mtg-engine/src/casting.rs:610-637) builds alternative-cost
  options only from the card's own `CostTarget::ThisSpell` statics (plus Omniscience-style "free from hand",
  casting.rs:638-650). `CostModifier { applies_to: CostTarget::Spells(_), change: AlternativeCost(_) }` from
  other permanents is explicitly ignored when totalling costs (casting.rs:1634-1637) and never produces a
  cast option. A cast action is identified by card + `CastMethod`, so "jump-start/permission X *and* an
  external alternative cost" can't be expressed.
- Cards (UNS): Fist of Suns, Jodah, Archmage Eternal, Rooftop Storm, Runeforge Champion, Darksteel
  Monolith, As Foretold, Grenzo, Crooked Jailer, Charred Foyer, Tlincalli Hunter, Demon of Fate's Design,
  Cramped Vents ("once each turn/once during each of your turns" variants need per-source use tracking,
  cf. `TurnHistory::once_permissions_used`, game.rs:446). (Alternative costs attached to a resolved
  effect's permission are G18; static `PlayPermission`s already carry a `cost`, ability.rs:2278-2279.)
- Rulings to cite once fixed: Radical Idea "you may pay that alternative cost when you jump-start a spell";
  Fist of Suns "You can't combine this with other alternative costs, such as flashback", "the only legal
  choice for X is 0".
- Approach: in `cast_options`, also scan `self.statics.cost_modifiers` for `Spells(f)` +
  `AlternativeCost` matching the would-be spell (reuse `spells_change_applies_as`), producing an option
  per (face, method) that has no alternative cost yet (normal cast, jump-start, permission casts, not
  flashback/escape/other alt costs). Add an optional "once each turn" use key.
- Size: M

### G02. Optional/choice additional costs and flash-for-cost granted to classes of spells by other objects
- Rules: CR 601.2b, 601.2f, 118.8; 601.3c.
- Engine now: from other permanents' statics only `AdditionalCost`, increases and reductions apply;
  `OptionalAdditionalCost`, `AdditionalCostChoice` and `FlashForAdditionalCost` with
  `CostTarget::Spells(_)` are dropped (casting.rs:1634-1637); `cost_choices.rs` only reads the spell's own.
- Cards (UNS): Defiler of Vigor/Faith/Dreams/Flesh/Instinct ("As an additional cost to cast green permanent
  spells, you may pay 2 life. Those spells cost {G} less ... if you paid life this way"), Chorus of the
  Conclave ("you may pay any amount of mana").
- Rulings: Defiler of Vigor "You may only pay the additional cost once per permanent spell."; "this ability
  does not cause the spells to have Phyrexian mana symbols".
- Also missing: a minimum total cost applied after all increases and reductions ("each spell that would
  cost less than three mana to cast costs three mana", Trinisphere, UNS; `CostChange` has no such kind,
  ability.rs:2092-2119). Ruling: Trinisphere "start with the mana cost or alternative cost you're paying,
  add any cost increases, then apply any cost reductions. Finally, apply Trinisphere's effect".
- Approach: collect optional additional costs from matching `Spells(f)` modifiers in `cost_choices.rs`
  (announced at 601.2b, recorded under a per-source name in `CastInfo::paid`); allow a cost modifier to
  be conditional on that name; add `CostChange::MinimumMana(n)` applied last.
- Size: S-M

### G20. Cost parts that look only at your own zones
- Rules: CR 118 / 602.2b: "Exile a Fungus card from a graveyard" may use any graveyard.
- Engine now: `cost_zone_cards` (casting.rs:2494-2508) returns only `p`'s graveyard/hand/library for
  `CostPart::Exile { zone, .. }`; "from a single graveyard" needs G06's group rule.
- Cards: Thelon of Havenwood ("{B}{G}, Exile a Fungus card from a graveyard": compiles, but can only use
  your own graveyard), Night Soil (UNS: "Exile two creature cards from a single graveyard").
- Approach: carry an owner qualifier on `CostPart::Exile` (yours / any / single graveyard) and choose
  across all graveyards.
- Size: S

## 8. gap-zone-moves: Zone changes: Auras attached to cards in graveyards, full 'instead' destinations, fabricate fallback

### G28. Auras attached to (or targeting) cards outside the battlefield
- Rules: CR 303.4a (an Aura spell's target is defined by its enchant ability, e.g. "Enchant creature card in
  a graveyard"), 303.4b-c, 702.5a, 704.5m; 607.2c / 607.1 ("enchant creature put onto the battlefield with
  ~").
- Engine now: `aura_target_spec` (attach.rs:74-84) lets the Aura spell target a card in a graveyard, but
  `legal_attachment_as` (attach.rs:107-140) returns false for any object not on the battlefield
  (attach.rs:138), so such an Aura can't enter attached and would be put into the graveyard by 704.5m
  before its enters trigger resolves. There's also no "loses enchant A and gains enchant B" handling of
  the enchant keyword's filter beyond generic keyword add/remove, nor a linked "put onto the battlefield
  with ~" enchant quality.
- Cards (UNS): Animate Dead, Dance of the Dead, Necromancy ("becomes an Aura with 'enchant creature put onto
  the battlefield with ~'"), Spellweaver Volute ("Enchant instant card in a graveyard").
- Rulings: Animate Dead "You target a creature card in a graveyard when you cast it. It enters the battlefield
  attached to that card. Then it returns that card to the battlefield, and attaches itself to the card
  again", "If the creature put onto the battlefield has protection from black ... Animate Dead won't be able
  to attach to it. It will be put into the graveyard as a state-based action", "A creature card with shroud
  may be targeted by Animate Dead".
- Approach: allow attachment to objects in the zone the enchant filter names (graveyard), keep
  `attached_to` across that zone, have 704.5m use the enchant filter's zone; support swapping the enchant
  keyword in layer 6 and a linked enchant quality (607.2c).
- Size: M

### G23. "Instead" zone-change replacements drop all destination details
- Rules: CR 614.1a, 614.6 (the modified event happens instead), 701.6 (counter destinations: Hinder,
  Desertion, Delay), 702.62 (suspend granted in exile), 111/708 (face down), 110.2 (under whose control).
- Engine now: `ReplacementAction::MoveInstead(dest)` (replacement.rs:1158-1181) keeps only `dest.zone`
  and `dest.position`; `Destination`'s `tapped`, `controller`, `face_down`, `with_counters`, `attacking`,
  `transformed`, `with_mods`, `attached_to` (ability.rs:504-529) are ignored, and a choice between two
  positions ("top or bottom") can't be expressed.
- Cards (UNS): Delay, Gandalf of the Secret Fire ("exile it with three time counters ... If it doesn't have
  suspend, it gains suspend"; S29 #37: about 12-17 cards share the phrase), Hinder ("your choice of the top
  or bottom"), Desertion ("onto the battlefield under your control").
- Rulings: Delay "If the target spell was cast with flashback, Delay's effect will exile it, not the
  flashback effect. The card will get time counters and gain suspend"; Desertion "The card is put onto the
  battlefield ... not ... cast from your hand"; Hinder "Hinder's controller, not necessarily the controller
  of the countered spell, chooses where the countered spell goes".
- Approach: build the replacement `MoveEv` from the whole `Destination` (as `move_to_destination` does,
  resolve.rs:2195-2240: EtbInfo tapped/controller/counters/face_down/attacking), add a
  `Destination::choice` of positions asked of the replacement's controller, and a "gains suspend" effect for
  the exiled card (a keyword grant that follows the card in exile, CR 400.7 exception like 400.7g).
- Size: M

### G24. Fabricate ignores counters being prevented
- Rules: CR 702.123a + Angel of Invention ruling ("If you can't put +1/+1 counters on the creature for any
  reason as fabricate resolves ..., you just create Servo tokens").
- Engine now: kw/fabricate.rs `derived` (lines 41-70) asks "may put counters" whenever the source is on the
  battlefield and creates Servos only if the player declined (`Not(PrevHappened)`); if a replacement
  effect prevents the counters ("can't have counters put on it", Solemnity), the player gets neither.
- Approach: decide "can't put" by asking the replacement pipeline whether an `AddCounters` event would be
  prevented (as kw/riot.rs `counters_prevented` already does), and treat "put 0" as not putting.
- Size: S

## 9. gap-simultaneity: Simultaneous multi-player and multi-object actions; shuffling empty sets

### G10. "Each player ..." effects run one player at a time instead of choices-then-simultaneous
- Rules: CR 101.4 (choices in APNAP order, then actions simultaneously), 608.2e, 608.2f.
- Engine now: `Effect::ForEachPlayer` (resolve.rs:135-143) executes the whole body for one player before the
  next, so player 2 chooses after player 1's action has already happened, the events of each player land in
  different batches, and a card put onto the battlefield by one player can be copied by another's Clone.
  Only `Effect::Sacrifice` and `Effect::Discard` with plural players use `apnap_round` (resolve.rs:237-260,
  1121-1140) and act simultaneously.
- Cards: Show and Tell, Tempting Wurm, Hunted Wumpus, Kynaios and Tiro ("Each player may put ... from their
  hand onto the battlefield", UNS for this reason); Exhume (supported, moves cards one player at a time,
  r110 pattern); tempting offers (Tempt with Discovery etc.: accepted opponents' effects happen one by one,
  so "one or more" triggers fire per player and Iwamori-style choices see earlier results); "each player
  shuffles ... then draws that many" (done per player).
- Rulings: Show and Tell "After all choices are made, the cards are put onto the battlefield simultaneously"
  and "those choices are made after all players choose their card" (UNCOVERED); tempting offer "the effect
  happens simultaneously for each one who accepted the offer" (cited, but only the order is tested).
- Approach: a two-phase executor for ForEachPlayer bodies whose actions are simultaneous-capable (moves,
  token creation, draws, life changes): collect each player's choices in APNAP order (`apnap_round`), then
  perform one combined action (one `move_objects` call / one event batch). Keep per-player sequencing for
  bodies that the rules process separately (608.2f).
- Size: M-L

### G25. "Shuffle ... into their library" doesn't shuffle when there's nothing to shuffle in
- Rules: CR 701.24c, 701.24d (the library is shuffled even if the set of objects is empty or they aren't
  where they're expected), 701.24e/f (shuffle triggers).
- Engine now: `Effect::ShuffleInto` (resolve.rs:1346-1367) shuffles only the libraries of owners of the
  objects actually moved; with an empty hand and graveyard nothing is shuffled. `ShuffleIntoLibrary`
  (resolve.rs:1368-) already follows 701.24c/d. The Timetwister family compiles to `ShuffleInto`
  (oracle/patterns/r400_zones.rs:143).
- Cards: Timetwister, Time Reversal, Echo of Eons, Commit-style "shuffle your hand and graveyard".
- Approach: give `ShuffleInto` the player(s) whose library is shuffled (or compile these to
  `ShuffleIntoLibrary`) and always shuffle.
- Size: S

## 10. gap-mana-payment: Mana and payment: X limits and colours, alternative ways to pay symbols, spend triggers for abilities, player mana abilities

### G05. Paying costs: X restrictions and alternative ways to pay mana symbols
- Rules: CR 107.3a/107.3f (X chosen as announced, within what the text allows: "X can't be 0"); 107.3a and
  601.2b with an alternative cost whose X is defined by what is exiled (Shoals); 609.4b / 118 payment
  modifications ("Spend only black mana on X"; K'rrik: "For each {B} in a cost, you may pay 2 life rather
  than pay that mana" changes only how the cost is paid).
- Engine now: `Decision::ChooseX { source, max }` (decision.rs:73) has no minimum; every caller
  (casting.rs:1341, 1992; mana_abilities.rs:594; kw/suspend.rs:203; kw/morph_face_up.rs:151) accepts any
  n >= 0. `SpendContext` (mana.rs:423-443) has no way to say which mana may pay the X part. The only
  pay-life-for-mana mechanism is Phyrexian symbols (cost_rules.rs); there's no player modification that
  makes {B} payable with life. The alternative-cost compiler refuses X (costs_casting_alt.rs plain_cost),
  and the casting flow has no "X defined by the cost paid" path.
- Cards (UNS): Hogaak, Arisen Necropolis ("You can't spend mana to cast ~": only convoke/delve may pay);
  Thieving Skydiver, Expansive Reapplication, Marath, Will of the Wild ("X can't be 0");
  Consume Spirit, Drain Life, Soul Burn, Crypt Rats ("Spend only black mana on X"); K'rrik, Son of Yawgmoth;
  Disrupting Shoal, Nourishing Shoal and the other Shoals ("exile a blue card with mana value X from your
  hand rather than pay this spell's mana cost").
- Rulings (UNCOVERED): Disrupting Shoal "include the value chosen for that X when determining the mana value
  of that spell, even if it was cast for an alternative cost and no mana was spent on X"; K'rrik "doesn't
  modify or reduce costs you pay. It changes only how you may pay those costs", "You can't pay 2 life to pay
  for generic mana"; familiars/Helm of Awakening "the generic X cost is still considered generic" (S27 #45/#73).
- Approach: add `min` to `ChooseX` (and validate); add a per-payment restriction "X paid only with mana of
  color C" in `SpendContext` honoured by the solver; a player modification "may pay N life for each {C}
  symbol" handled where Phyrexian symbols are announced (`cost_rules::announce_phyrexian`); an alternative
  cost part "exile a card with MV X" that fixes X from the exiled card.
- Size: M

### G19. Mana: spend triggers for abilities, symbol-type restrictions, player mana abilities
- Rules: CR 106.6 (spending restrictions), 106.6a (mana that triggers when spent), 605.1a/605.3 (a player
  may be granted "any time you could activate a mana ability, you may pay 1 life: add {C}"), 107.4/118.
- Engine now: `Effect::AddManaWithSpentTrigger { add, spell_filter, body }` (ability.rs:3025-3029) only
  fires for spells; `ManaRestriction` (mana.rs) has no "can't be spent to pay generic mana" or "only on
  X" kinds; there's no player-level mana ability usable during payment (mana_abilities.rs only activates
  abilities of objects).
- Cards (UNS): Sunken Palace ("When you spend this mana to cast a spell or activate an ability, copy that
  spell or ability"), Jegantha, the Wellspring and The Rebellious Intelligence ("This mana can't be spent to
  pay generic mana costs"; ruling "You can spend mana from Jegantha's mana ability to pay for a hybrid
  symbol such as {2/W}, but only if you choose to pay the colored mana component"), Channel ("Until end of
  turn, any time you could activate a mana ability, you may pay 1 life. If you do, add {C}"; ruling "Once
  your life total is 0, you can't pay any more life").
- Approach: generalise the spent trigger to `SpendContext.is_ability`; add `ManaRestriction::NotGeneric`
  and an X-only/colour-for-X restriction (see G05) in the solver's per-symbol matching; a
  `PlayerModification::ManaAbility(Cost, ManaProduction)` offered by the payment solver like an
  object's mana ability.
- Size: M

## 11. gap-ability-grants: Layer 6 and linked abilities: gaining others' abilities, partial keyword loss, ability-quality hexproof, linked targets

### G11. Gaining the activated (and triggered) abilities of other objects
- Rules: CR 613.1f (layer 6 ability-adding effects), 613.8 (dependency: what the effect copies depends on
  other layer-6 effects), 607.5 (linked pairs gained together), 201.5b (a gained ability that names its
  original object uses the new object's name; Necrotic Ooze ruling).
- Engine now: `Modification` (ability.rs ~1520-1560) has `AddAbility(Ability)`, `AddKeyword`,
  `AddKeywordsOf { kinds, from }` (keywords only) and `AddThisAbility`, but nothing that copies the
  activated/triggered abilities of the objects a filter/selection names, re-evaluated continuously.
- Cards (UNS): Necrotic Ooze, Experiment Kraj, Quicksilver Elemental ("gains all activated abilities of
  target creature until end of turn"), Agatha's Soul Cauldron, Koh, the Face Stealer (activated and
  triggered), Steward of the Harvest, Manascape Refractor, Skill Borrower, Myr Welder, Idris, Soul of the
  TARDIS.
- Rulings: Necrotic Ooze "gains only activated abilities. It doesn't gain keyword abilities (unless those
  keyword abilities are activated)", "references the card it's printed on by name ... as though it
  referenced Necrotic Ooze"; Experiment Kraj "costs of the activated abilities ... must be paid with the
  correct colors"; S27 #63/#66 colon rulings (Steward of the Harvest/Skill Borrower, Koh/Idris).
- Approach: `Modification::AddAbilitiesOf { from: Sel|Filter, kinds: {Activated, Triggered}, zone }` in
  layer 6, computed after the source objects' own layer-6 state (dependency per 613.8 since the copied set
  depends on other layer-6 effects), with `~` in the copied text rebound to the gaining object and
  link ids preserved (607.5).
- Size: M

### G12. Losing or targeting by part of a keyword: one protection/landwalk instance; "hexproof from" abilities
- Rules: CR 702.14 (landwalk variants are separate abilities), 702.16 (each "protection from X" is a
  separate ability; losing "protection from red" leaves others), 613.1f; 702.11d ("hexproof from
  [quality]" where the quality can describe the spell or ability itself).
- Engine now: `Modification::RemoveKeyword(KeywordKind)` (ability.rs:1551) removes every instance of a
  kind; there's no way to remove "protection from red" or "islandwalk" only. Hexproof-from qualities are
  matched only against the ability's *source* (`object_untargetable`, stack.rs:457-475), so "hexproof from
  activated and triggered abilities" can't be expressed.
- Cards (UNS): Mystic Decree ("All creatures lose flying and islandwalk"), Shelkin Brownie ("loses all
  'bands with other' abilities", which must not remove plain banding), Volatile Stormdrake ("hexproof
  from activated and triggered abilities"; ruling "can't be the target of any activated or triggered abilities your opponents control").
- Approach: `RemoveKeyword` with an optional parameter/filter match (`RemoveKeywordInstance(Keyword)`);
  let hexproof/protection quality filters also be matched against the targeting stack object (kind:
  spell/activated/triggered).
- Size: S

### G15. Linked abilities that refer to the player or object a previous ability affected (CR 607.1)
- Rules: CR 607.1 (one ability affects objects or *players*, the other refers to them), 607.1d (linked
  across a created token), 607.3.
- Engine now: linked data exists for exiled cards (`GameObject::linked`, `Sel::Linked`, eval.rs:907) and
  for choices (`linked_choices` with a `player` field, eval.rs:197-210, 707-710), but no effect records
  *the target* of one ability for its linked partner, and `Effect::Choose` only records choices made with
  a `ChoiceKind`.
- Cards (UNS): Laquatus's Champion, Soul Scourge ("When ~ leaves the battlefield, that player gains 6
  life"), Ugin, the Ineffable ("When that token leaves the battlefield, put the exiled card into your
  hand", linked across the token, 607.1d).
- Rulings: Laquatus's Champion "the player who gains 6 life is the player who was the target of the first
  ability (or, if that ability is still on the stack, the player who is its target)".
- Approach: an `Effect::NoteLinked { what: Sel }` (or a flag on targets) that stores entities in the
  source's linked data under the ability's link id, plus `Sel::LinkedPlayer`; for a still-pending
  first ability, read the target from the stack object (the ruling's parenthetical).
- Size: S-M

## 12. gap-rule-statics: Missing rule-modifying statics (can't be copied, counters persist, damage not removed, sacrifice/payment restrictions, face-up restrictions, turns taken, as-though infect)

### G14. Rule-modifying statics the engine has no representation for
- Rules / engine now (none of these exist anywhere outside oracle/, checked by grep):
  - "[This] can't be copied" (CR 113.6g, 707.10): `copy::copy_spell` (copy.rs) never checks it.
    Cards (UNS): See Double, Display of Power, Choreographed Sparks, Gogo, Master of Mimicry, Ulalek.
  - "Counters remain on ~ as it moves to any zone other than a player's hand or library" (exception to
    CR 122.2 / 400.7): Skullbriar, the Walking Grave, Me, the Immortal. Rulings: Skullbriar "The counters
    that remain ... aren't 'placed'", "Counters that adjust power and/or toughness affect Skullbriar's power
    and/or toughness in zones other than the battlefield".
  - "Damage isn't removed from ~ during cleanup steps" (modifies CR 514.2): Ancient Adamantoise, Uthgardt
    Fury, Switchgrass Grazer, Patient Zero, Victory of the Pyrohammer, Case of the Market Melee. Ruling:
    Ancient Adamantoise "Effects that remove all damage from a permanent (such as regeneration) will still
    remove damage".
  - "Spells and abilities your opponents control can't cause you to sacrifice permanents" (CR 701.21):
    `Restriction::CantBeSacrificed(Filter)` (ability.rs:2019, actions.rs:1168) has no notion of what
    causes the sacrifice. Cards: Sigarda, Host of Herons, Tajuru Preserver, The Master, Multiplied. Rulings:
    Sigarda "if it would force you to sacrifice a permanent, you just don't", "It won't stop ... the legend
    rule", "you can't choose to sacrifice a permanent" for "unless you sacrifice".
  - "can't be turned face up" restrictions (CR 708, 702.37e): only merged permanents are checked
    (merge.rs:217, facedown.rs:271). Cards: Unable to Scream, Karlov Watchdog (ruling: "opponents can't
    attempt to turn face-down creatures face up either by paying a disguise or morph cost or by paying the
    mana cost of a cloaked or manifested creature").
  - Turns a player has taken ("your first, second, or third turns of the game"): `Player` has no turn
    counter (only `turn.number`, game-wide). Cards: Serra Avenger, Jace Reawakened, Spider-Man 2099, The
    Keeper of the Yellow Hat, Captivating Crossroads. Rulings: Serra Avenger "cares about how many turns
    you have taken, not ... how many turns the game has had", "If the game is restarted ... you can't cast".
  - "Players can't pay life [or sacrifice X] to cast spells or activate abilities [that aren't mana
    abilities]" (CR 118.3, 119.4): `can_pay_life` (actions.rs:1386-1388) only checks "can't lose life".
    Cards (UNS): Karn's Sylex, Yasharn, Implacable Earth, Angel of Jubilation.
  - "All damage is dealt to you as though its source had infect" (CR 702.90, 120.3b): only the wither
    variant exists (kw/wither.rs:16); damage to players checks the source's own infect (actions.rs:1547-1556).
    Card (UNS): Phyrexian Unlife.
- Approach: one small hook each — a `Restriction::CantBeCopied(Filter)` checked in `copy_spell` and token
  copy creation; a zone-change hook keeping counters (counters not "put"); a cleanup-step check in
  turn.rs; a `cause_controller` on sacrifice (pass the resolving object's controller to
  `Game::sacrifice`); `Restriction::CantTurnFaceUp(Filter, PlayerRel)` checked by facedown/morph/disguise
  turn-up paths; `Player::turns_taken` incremented in turn begin and a `Value`; a cost-payment restriction checked in
  `can_pay_cost`/`pay_cost_part` for life and sacrifices; an "as though infect" player modification.
- Size: M (eight S pieces)

## 13. gap-combat: Combat: who declares attackers, as-though-haste attacks and tapped blockers, per-creature attack targets, trample assignment order

### G17. Combat declarations and as-though permissions
- Rules: CR 508.1 (active player declares attackers), 509.1; effects that make another player choose
  attackers (Master Warcraft); 508.4 (each creature put onto the battlefield attacking: its controller
  chooses what it attacks); 302.6 / 508.1c ("can attack as though it had haste"); 509.1a ("can block as
  though untapped"); 702.19b (lethal damage counts damage from *other creatures being assigned in the same
  step*, in whatever order the player assigns).
- Engine now:
  - `declare_attackers_step` always asks the active player (combat.rs:964-990); only blocks have a chooser
    override (`block_choice::block_decider`, combat.rs:1616-1618). Master Warcraft (UNS) needs "you choose
    which creatures attack", the active player then choosing what each attacks (ruling).
  - Attack legality checks `summoning_sick && !Haste` only (combat.rs:267); blockers must be untapped
    (`can_block_at_all`, combat.rs:337-345).
    No "as though it had haste" for attacking (Instill Energy, Frenzied Saddlebrute: UNS) or "tapped
    creatures can block as though untapped" (Masako the Humorless: UNS; ruling "allows tapped creatures to
    block only if they could otherwise block").
  - `move_to_destination` picks one attack target for all moved objects, chosen by the effect's controller
    (resolve.rs:2212-2216 -> `attack_target_for_new_attacker`, resolve.rs:2302-2306), not per object by
    each object's controller (tokens are per token, resolve.rs:2275-2296). No current card moves several
    nontoken cards attacking at once, but owner-control returns are affected.
  - Trample: attackers' assignments are made in declaration order and each sees only earlier ones
    (`combat_damage_step`, combat.rs:1818-1825; `kw::trample::lethal_damage` uses `pending`), so a
    trampler declared before another attacker blocked by the same creature can't count that damage.
- Rulings: Master Warcraft "the person who cast Master Warcraft first chooses the complete group of
  creatures that are going to attack. Then, for each of those creatures, the active player chooses who or
  what it's going to attack"; "You choose attackers ... regardless of whether it's your turn".
- Approach: an attack-chooser hook mirroring `block_decider`; a `Restriction`/static permission "may attack
  as though it had haste" and "may block while tapped" consulted in `attack_options`/`block_options`;
  per-object attack target in `move_to_destination`; assign trample damage after all non-trample
  assignments to shared blockers (or let the player order assignments).
- Size: M

## 14. gap-variants-misc: Grand Melee and range-of-influence fidelity, commander eligibility, ante primitives, small fidelity bugs

### G26. Grand Melee and range-of-influence fidelity
- Rules: CR 807.4 (several turns at once; each turn has its own cleanup step, CR 514.2, and its own
  "that turn" effects), 807.4i/807.4j (extra turns), 807.5b (a player with priority for several stacks
  chooses the stack for a spell or ability), 801.7 (triggers outside the range of influence don't trigger),
  614.10 / "skip that turn" effects for extra turns.
- Engine now (multiplayer/grand_melee.rs documents the first two as "known simplifications", lines 15-17):
  "until end of turn" effects end at the cleanup step of whichever marker's turn reaches it first, and mana
  pools empty whenever any marker's step ends; extra turns re-queued to `extra_before_next` or taken by the
  holder lose their "that turn" actions (grand_melee.rs:382-407 drops them); `begin_marker_turn`
  (grand_melee.rs:243-) never calls `skip::extra_turn_skipped` (only turn.rs:609 does), so Stranglehold /
  Gerrard's Hourglass Pendant don't apply on that path; spells and abilities cast with priority for several
  stacks always go on the stack of the turn being played (no 807.5b choice); one-shot delayed triggers
  (`once_matches`, triggers.rs:589-598) skip the 801.7 `trigger_in_range` filter that ordinary and
  repeating delayed triggers get.
- Approach: key "until end of turn"/"this turn" durations and mana-pool emptying to the marker whose turn
  created them; keep extra-turn actions with queue entries by id; call `extra_turn_skipped` when a marker
  begins an extra turn; ask for the stack per 807.5b; apply `trigger_in_range` to once-delayed matches.
- Size: M

### G27. Commander eligibility ignores abilities that work outside the game
- Rules: CR 903.3 (the commander must be a legendary creature card, or have text letting it be one),
  113.6c (an ability that says which zones it doesn't function in works everywhere else, "even outside the
  game and before the game begins").
- Engine now: `kw::partner::can_be_commander` (kw/partner.rs:210-224) looks only at the front face's printed
  characteristics, so a card that is a creature outside the battlefield through its own static ability
  is rejected; that static itself is also unsupported.
- Cards: Grist, the Hunger Tide ("As long as Grist isn't on the battlefield, it's a 1/1 Insect creature in
  addition to its other types", UNS). Ruling: "Grist, the Hunger Tide can be your commander as its first
  ability works before the game begins during deck construction."
- Approach: compute the card's characteristics with its own abilities that function outside the game
  (layers on a hypothetical object in the command zone) before testing eligibility; support the static.
- Size: S

### G33. Ante: anteing a permanent and exchanging ownership
- Rules: CR 407.3 (ante cards may add or remove cards from the ante zone or change a card's owner), 407.4
  (to ante an object is to put it into the ante zone from whichever zone it's in; only its owner can),
  108.3 (owner).
- Engine now: ante.rs provides the pregame ante (407.2), `ANTE_TOP` (ante the top card of a library),
  `GAIN_OWNERSHIP` ("You own target card in the ante") and `EXCHANGE_WITH_TOP`; there is no effect that antes
  a permanent or other object from its current zone, and none that exchanges the ownership of two objects
  (no hits for "exchange ownership" in src/).
- Cards (legal nowhere, so `mtg-tools unsupported` skips them; implementer reports listed them unsupported and
  no test uses them): Jeweled Bird ("Ante this artifact. If you do, put all other cards you own from the ante
  into your graveyard"), Tempest Efreet, Timmerian Fiends, Bronze Tablet ("Exchange ownership of the revealed
  card and ~"), Rebirth, Amulet of Quoz ("may ante the top card of their library. If they don't, ...").
- Rulings: Jeweled Bird "The card is exchanged for your entire contribution to the ante".
- Approach: an `ante` move cause usable on any object (owner check per 407.4) and an "exchange ownership"
  effect for two objects (owners swap; controllers follow from the move instructions on the cards); add tests
  with `GameConfig { ante: true }` (tests/cr/r407_ante.rs has the harness).
- Size: S

### G32. Small rules-fidelity bugs
- Graveyard "cards" counts include tokens: `Value::GraveyardSize` and `PlayerFilter::GraveyardSize`
  (eval.rs:1017-1019, 249) count every object in the graveyard zone; a token there until the next
  state-based action check isn't a card (CR 108.2b, 111.7). Affects threshold-style "seven or more cards
  in your graveyard" checks made mid-resolution (e.g. "Sacrifice a creature token. Then if you have seven or
  more cards in your graveyard ..."). Fix: count only cards. Size S.
- Several spell copies created at once are stacked in a fixed order: `Effect::CopySpell` (resolve.rs:854-865)
  loops `copy_spell` per object and per count without letting the controller order them (CR 405.3; with new
  targets the order matters). Cards: Display of Power, Thousand-Year Storm (copies with different targets),
  Gogo, Mister Fantastic. Fix: create all copies, then ask for their relative order (the copy-for-each-target
  path already does). Size S.
- A meld trigger on a copy of a meld card does nothing: `MELD_PAIR_CONDITION` (merge.rs:806-809) is false
  unless a real meld pair resolves, but the instruction "exile them, then meld them" should still exile both
  and leave them in exile (CR 701.42b-c: objects that can't be melded stay where they are, i.e. exiled).
  Size S.

---

## Known gaps from the owner's list: status

- Copying activated abilities (CR 707.10): works. Rings of Brighthearth, Strionic Resonator, Lithoform Engine
  and Illusionist's Bracers are supported; `copy::copy_spell` (copy.rs) copies stack abilities, also from last
  known information. Jaya's Phoenix / Leori stay unsupported only because "copy the next loyalty ability you
  activate this turn" isn't parsed (a delayed `AbilityActivated` trigger + copy is expressible).
- Gaining control of a spell on the stack: works (Aethersnatch, Commandeer supported; tests
  tests/cr/r112_spells.rs, tests/rulings/r_s22_misc.rs). No card gains control of an ability.
- Which player put counters: confirmed gap, G08.
- Per-object controller for multi-object effects (Rakdos Charm): expressible today (`Effect::ForEach` +
  `DealDamage { source: Sel::Var, to: ControllerOf(Var) }`), so the card itself is a parsing gap; the
  remaining engine part (each iteration is a separate event batch instead of simultaneous damage) is in G10.
- Linked abilities (CR 607): exile links (`Sel::Linked`, `CreatorLinked`), chosen values, noted information,
  craft/devour/champion links all exist; the missing case is a linked *target/player* (G15) and the linked
  enchant quality (G28).

## Dropped items (for audit)

### Stale: fixed since they were reported (checked in code or by card/ruling status)
- Rebound (kw/rebound.rs; rebound vs Leyline choice: kw/mod.rs `resolved_destinations`, Staggershock ruling
  cited); persist, undying, bestow, umbra armor, crew, reconfigure, saddle, teamwork, living metal, convert,
  explore, incubate, manifest/cloak/manifest dread, venture/Undercity/initiative, job select (trigger timing
  fixed), companion setup (start.rs:225, 323), hidden/double agenda, Attraction junkyard
  (attraction_cards.rs:61), Infinity/harness.
- Turning permanents face down (Backslide, Master of the Veil supported) and CR 712.16 (facedown.rs:76-83);
  flip cards (flip.rs; Bushi Tenderfoot supported; Kitsune Mystic and Nezumi Graverobber fail only on their
  flip conditions, a parsing matter).
- Mana value of back faces and melded permanents (mana_value.rs:45-80); merged/melded permanents leaving the
  battlefield (merge.rs:516, `found_all`), mutate 730.2j; Rooms lock/unlock/copy (rooms.rs).
- Cloudshift/Momentary Blink/Ephemerate return; statics in several zones (Teferi, Mage of Zhalfir supported);
  flash with "sacrifice if cast at instant speed" (`CastInfo::instant_timing`); "Tap an untapped creature you
  control" ignoring summoning sickness (casting.rs:2395); search may find nothing (library.rs:101-135); CR
  701.20d (reveal.rs), 701.24g (actions.rs:141), 205.3d (object.rs:44), 205.1a (layers.rs:1517); suspend's
  haste duration (kw/suspend.rs:268); granted blitz/escape/flashback and per-face graveyard casting
  (next_spell::cast_grant_keywords, kw/escape.rs:54, kw/flashback.rs:67); commander damage per card
  (commander_rules.rs:311); creatures that crewed it (kw/crew.rs `CrewRecord`); assist mana recorded
  (kw/assist.rs:123); 901.10 plane owner leaving (planechase.rs:651-); Emperor deploy
  (multiplayer/deploy.rs) and adjacency (multiplayer/attack.rs); shared team turns 805.4d/805.7
  (teams.rs, triggers.rs:500, 1913); kicker and other additional costs vs reductions; dual-land payment;
  temporary cost changes (oracle/patterns/costs_for_a_duration.rs); "if the {2}{U} cost was paid"
  (spell_costs.rs `alternative_cost_name`); redirection shields and "a source of your choice" (recent
  commits); Adventure exile on resolution (adventure.rs); the ante zone and Contract from Below
  (tests/cr/r407_ante.rs; the remaining ante cards are G33); wishes; Alms
  Collector / draw replacements (draw_rules.rs); hybrid announcement vs reductions (cost_rules.rs:74-99);
  split second (casting.rs:906-910); trample/assign-as-though-unblocked hook (kw/assign_as_though_unblocked.rs).

### Parse-only (the engine can execute it; the compiler doesn't recognise the phrase)
- Rakdos Charm (see above); Admonition Angel / Parallax Wave / Wormfang Behemoth (linked exile via
  `MoveEv.source`, `Sel::Linked`); Jaya's Phoenix / Leori; Siege Behemoth / Ruxa / Invasion of Ikoria (grant
  of `MAY_ASSIGN_UNBLOCKED`); Teferi, Time Raveler-style temporary flash permissions (player
  `FlashPermission`); Academy Manufactor (replacements already return several token events, so the 3^n
  ruling holds); Blossombind ("can't have counters" = `PutCounters` + `Prevent`); Stifle-style "counter target
  activated ability from an artifact source" filters; "spend mana as though it were mana of any type" for
  exiled cards (`Effect::SpendAnyTypeMana`); Mox Diamond / Lotus Vale "If ~ would enter, ... instead";
  Lich-style "draw that many instead"; group "if a creature would die this turn, exile it instead"
  (`Effect::AddReplacement`); "top or bottom of their library" as an effect (Vanish from Sight, Run Behind
  supported); Cultivate/Kodama's Reach split destinations; protection from "the chosen player";
  becomes-a-copy exception varieties; trap and Case "To solve" history conditions (`turn_events` +
  `Condition::AllTriggerConditionsThisTurn`, per-turn `Value`s); pronoun/antecedent tracking ("they", "those
  creatures", "that creature"), "where X is" ordering, reflexive/ability-word paragraph joins; equip, ward
  and cumulative-upkeep cost phrasings; X-less "you may pay {X}" (`bind_x_for_payment` only matters for
  Martyr of Frost, whose X is defined).
- Sticker-sheet cards ("{TK}{TK} — 1/5", about 100 unsupported lines): the sticker mechanic is implemented
  (stickers.rs, `StickerSheet`, `choose_sheets`), but nothing compiles Scryfall sticker-sheet cards into
  `StickerSheet`s; tests build them by hand. A data/compiler task, not an engine gap.
- Imminent Doom's "that much" compiling to the cast event's amount (a compile error, not an engine gap).

### Out of scope (owner decision)
- Contraptions (rulings S24 #5-7, S25 #1, S28 #7-8, S35 #13-17; 45 cards) and host/augment (S01 #89-101, S19
  host ruling; `oracle_hardening_mechanics.rs` already reports them unsupported).

### Not required by the Comprehensive Rules (owner may opt in)
- Un-set/playtest mechanics outside the CR (CR 100.7): last strike (Extremely Slow Zombie, Garbage Elemental),
  "cast ... as though they were the card Colossal Dreadmaw" (The Colossal Dreadmaw, The Heron Moon, Mystery
  Booster 2 playtest), choosing people outside the game, Sly Spy, number-word modification (Truss, More or
  Less), flavor-text counting (Hardy/Fluros of Myra's Marvels).
- Alchemy digital-only mechanics not in the CR: perpetually (~239 unsupported lines), seek (~130),
  spellbook/draft (~100), intensify, specialize, double team, boon. (Conjure is already implemented.)

### Not engine rules gaps (tooling, agents, heuristics, or rulings the CR supersedes)
- Hidden information (what a player may see: `facedown::can_look_at`, `LookAtHand` only logs, forecast and
  ninjutsu reveal queries): owned by docs/agents/items/agent-api.md.
- `legal_actions` over-offering (check-only payment ignores CR 106.6 restrictions; convoke/improvise counted
  twice; sacrifice-for-mana planner conflicts): the real cast/activation is validated and rolled back;
  exactness is required by the agent-api item.
- Mana planner heuristics (AnyOneColor(n) planned as separate units, no symmetry breaking, replacement order
  during planning): actual payments are validated.
- Loop detection (104.4b) and fragmented loops (732.3) heuristics, shortcuts as priority actions only.
- CR 733.1: reversing every mana ability of an illegal action is allowed ("may"); the library exception has
  no current mana ability that moves, reveals or shuffles library cards.
- Deck legality (deck.rs, attraction deck checks) is an API callers use; interchangeable names (no Scryfall
  data).
- Rulings the current CR supersedes: Wort, the Raidmother (CR 610.5 makes cast grants one-shot), Mossfire Egg
  (CR 605.1a), MDFC "can't transform" rulings (CR 701.27g, 712.3).
- Ambiguous/unsettled readings: ward on a copy that keeps its targets, deathtouch in 120.10 excess damage,
  granted changeling vs layer 4, Panharmonicon on non-ETB triggers, Torpor Orb on leave-graveyard triggers,
  tempting offer order when cast on another player's turn (engine follows CR 101.4), Grand Melee
  planeswalking readings of 701.31a/b.
- No current card affected: keyword-derived activated/triggered abilities added after layer 6, "can't have
  [keyword]" at the end of layer 6, `Filter::zone()` returning None for `Or`, `Value::DistinctNames` greed,
  legend rule with chained interchangeable names, copies made from LKI losing saved context/splices
  (stack.rs:1042 drops `saved_ctx`; no storm/epic/casualty card references it), bestowed Aura status ending at
  the next SBA check, Word of Command's resolution window.
- UI-only: "discard up to N" asking twice, per-card dredge prompt, ordering identical tokens.
