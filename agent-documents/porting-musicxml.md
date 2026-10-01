# Porting `musicxml` — design report

Status: in progress (first commit ports only the domain-independent pieces; see
"Temporary strategy" below).

## 1. Goal and current shape of the source

The TS package `packages/musicxml` is a MusicXML serializer. It has three
kinds of content:

- **Data** — `Clef`, `ClefType` (pure data, no duck typing).
- **Duck-typed structural interfaces** — `NoteLike`, `ChordLike`,
  `HarmonyLike`, `MeasureLike`, `VoiceLike`, `ScoreLike` (TypeScript
  intersection types describing *the shape the serializer needs*).
- **Serializer functions** — `pitch`, `note`, `measure`, `part`, `score`
  (generic over the structural interfaces, using `xmlbuilder2`).

The TS dependency direction is:

```
common → core → musicxml → species-counterpoint
```

`species-counterpoint` imports `Clef` and `NoteLike` *from* `musicxml`, i.e.
the domain depends on the serializer. That direction is the problem this
report addresses.

## 2. Dependency inversion

`musicxml` should be an **adapter** that depends on the domain, not the other
way around. Recommended layering:

```
common
  ↑
core         unbiased common denominator: pitch systems, pitch, interval,
             scale, tuning, and generic *structure* (Container/Cursor/…)
  ↑
tonal        Western/common-practice domain model: Note, Measure, Voice,
             Score, Chord, Harmony, Clef, NonHarmonicType
  ↑
musicxml     concrete adapter: `to_mxl(&tonal::Score) -> String`
  ↑
species-counterpoint / application
```

Consequences:

- `musicxml` becomes concrete functions over `tonal`'s concrete types; the
  `...Like` traits are deleted (they existed only to decouple the TS package
  from a specific score implementation).
- `Clef` and `NonHarmonicType` are domain concepts and move to `tonal` (for
  now they stay in `musicxml` to match the TS source; see §6).
- `pitch()` is bound to `std_hept::Pitch`, not generic over a pitch system:
  MusicXML is Western staff notation and cannot represent, say, a 17-tone
  system. Don't pretend otherwise.

### When traits come back

Only if a *second* consumer needs to render a *different* score shape (or a
second renderer needs the same shape). Then the trait belongs in the **domain
crate** (`tonal`), named after the domain (`Score`, `Voice`, `Measure`, …),
and `musicxml` becomes generic over it. Not needed now.

## 3. `core::structure` decisions

The TS `SequentialContainer<T>` class conflates two things; the Rust port has
already split them:

1. The abstraction — "an ordered sequence of children with start time + span,
   plus cursor navigation" → the `Container` trait.
2. The layout policy — "children packed back-to-back" → the concrete
   `SequentialContainer<T>` struct.

This split is correct and idiomatic (`SequentialContainer<T>` is essentially a
`Vec<T>` with a precomputed offset table). Consumers should be generic over
`Container`, never over the struct.

**`Measure` is not a pure sequential container.** In TS it `extends
SequentialContainer<Note>` *and* `implements DurationalElement` with its own
`duration`, which is *not* the sum of its notes (a partially written measure).
So in Rust `Measure` composes a `SequentialContainer<Note>` (note placement)
and stores `duration` separately (its extent as a child of a `Voice`):

```rust
struct Measure {
    notes: SequentialContainer<Note>,   // delegation: impl Container { Item = Note }
    duration: Rational,                  // impl DurationalElement
    writable_position: Option<Rational>,
}
```

`Voice` and `HarmonyBackground` are genuinely back-to-back and either contain a
`SequentialContainer` or hold a `Vec` and compute `start` inline.

**Heterogeneous collections.** TS relies on structural subtyping + array
covariance so `Score.voices: Voice[]` can hold `FixedVoice` and
`CounterpointVoice` together. Rust has neither, and `Container` is *not*
object-safe (`type Item` + `fn item(&self) -> &Self::Item`), so
`Box<dyn VoiceLike>` is unavailable. Resolution: model heterogeneity with
concrete enums (`enum Voice { Fixed(..), Counterpoint(..) }`) or unify the
types; keep `core::Container` statically typed. Decide this when porting
`species-counterpoint`, not before.

## 4. Why `Container`/`Cursor` covers the serializer

The TS `Containers.ts` traversal maps 1:1 onto the existing `core` API:

| TS | Rust (`core::structure`) |
| --- | --- |
| `v.entries()` | `v.iter_cursors()` |
| `m.value.first()?.withParent(m)` | `m.first_child()` |
| `n.next()` | `Cursor::next()` |
| `n.nextGlobal()?.value.isTied` | `n.next_global().is_some_and(\|nn\| nn.is_tied())` |
| `m.index` | `m.index()` |
| `s.harmony?.at(m.index)?.value.toString()` | `s.harmony().and_then(\|h\| h.cursor(m.index())).map(\|c\| c.to_string())` |

So the serializer needs no new abstraction: it uses `Container`/`Cursor` for
traversal and binds to concrete domain types for the leaf data.

## 5. Temporary strategy (current phase)

The score structures in `species-counterpoint` are entangled with solver logic
and will be disentangled into `tonal` only *after* the rest is ported.
Therefore, for now:

- **`musicxml` crate** contains only the domain-independent pieces:
  - `Clef` / `ClefType` (pure data).
  - `pitch()` — `<pitch>` serialization of a `std_hept::Pitch`.
  - XML emission helpers over `quick-xml`.
- **Adapters** (`note`, `measure`, `part`, `score`) — which need the
  `Note`/`Measure`/`Voice`/`Score` types — are deferred to
  `species-counterpoint` (and will later move to `musicxml` once `tonal`
  exists).
- **`species-counterpoint` / `tonal` crates are not created yet.**

## 6. Deferred / open items

- Move `Clef`, `NonHarmonicType` into `tonal` when it is extracted.
- Write the `note`/`measure`/`part`/`score` adapters (in `species-counterpoint`
  for now, later `musicxml`), porting the `divisions = 2` scale
  (`duration.mul(2)` → `duration() * 2`) and the suspension/neighbor/passing
  lyric mapping from `Note.type`.
- Decide enum-vs-unified types for heterogeneous voices/measures.
- XML details: `alter` from `rational_value(acci)`; `octave` from `period`;
  `<step>` from a `CDEFGAB` index table.

## 7. Serialization library

`quick-xml` (crate `quick-xml`, currently 0.42) replaces `xmlbuilder2`. It
provides a streaming `Writer` with an ergonomic `create_element(...)`
/ `with_attribute(..)` / `write_inner_content(..)` API and
`new_with_indent(..)` for pretty-printing (the TS code used
`prettyPrint: true`).
