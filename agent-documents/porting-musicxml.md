# Porting `musicxml` — design report

Status: in progress (first commit ports only the domain-independent pieces; see
"Strategy (revised)" below).

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
way around. Layering (for now):

```
common
  ↑
core         unbiased common denominator: pitch systems, pitch, interval,
             scale, tuning, and generic *structure* (Container/Cursor/…)
  ↑
species-counterpoint   domain model (Note, Measure, Voice, Score, Chord,
                       Harmony, Clef, NonHarmonicType) + solver, for now
  ↑
musicxml     concrete adapter: `impl ToMxl for Score` / `to_mxl(&Score)`
```

A standalone general/`tonal` domain crate is a *postponed* design problem (see
`species-counterpoint-approach.md` §6); `musicxml` imports
`species-counterpoint` for now.

Consequences:

- `musicxml` becomes concrete functions over the domain's concrete types; the
  `...Like` traits are deleted (they existed only to decouple the TS package
  from a specific score implementation).
- `Clef` and `NonHarmonicType` are domain concepts and move to
  `species-counterpoint` (for now `Clef` still lives in `musicxml` to match the
  TS source; see §6).
- `pitch()` is bound to `std_hept::Pitch`, not generic over a pitch system:
  MusicXML is Western staff notation and cannot represent, say, a 17-tone
  system. Don't pretend otherwise.

### When traits come back

Only if a *second* consumer needs to render a *different* score shape (or a
second renderer needs the same shape). Then the trait belongs in the **domain
crate** (`species-counterpoint` for now, later a general domain crate), named
after the domain (`Score`, `Voice`, `Measure`, …), and `musicxml` becomes
generic over it. Not needed now.

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

## 5. Strategy (revised)

> Supersedes the earlier "adapters live in `species-counterpoint`" idea — see
> `porting-species-counterpoint.md`.

The `musicxml` adapters live in `musicxml` and are added **directly to the
domain containers** via a `ToMxl` trait (`impl ToMxl for Score`, …), rather than
being generic over `...Like` traits. This requires the domain types
(`Note`/`Measure`/`Voice`/`Score`/`Chord`/`Harmony`/`Clef`) to live in a crate
`musicxml` can depend on. For now that crate is `species-counterpoint` (the
domain types are local to it; a standalone general/`tonal` crate is a postponed
design problem — see `species-counterpoint-approach.md`).

For now `musicxml` contains only `Clef`/`ClefType` and the `pitch()` element
writer; the `note`/`measure`/`part`/`score` adapters are implemented against
`species-counterpoint` once it exists (and `Clef` moves there, since `musicxml`
will import it).

## 6. Deferred / open items

- Move `Clef`, `NonHarmonicType` into `species-counterpoint` when it is created
  (they live there for now; a standalone domain crate is postponed).
- Write the `note`/`measure`/`part`/`score` adapters in `musicxml` (as
  `impl ToMxl for species_counterpoint::*`), porting the `divisions = 2` scale
  (`duration.mul(2)` → `duration() * 2`) and the suspension/neighbor/passing
  lyric mapping from `Note.non_harmonic`.
- Decide enum-vs-unified types for heterogeneous voices/measures.
- XML details: `alter` from `rational_value(acci)`; `octave` from `period`;
  `<step>` from a `CDEFGAB` index table.

## 7. Serialization library

`quick-xml` (crate `quick-xml`, currently 0.42) replaces `xmlbuilder2`. It
provides a streaming `Writer` with an ergonomic `create_element(...)`
/ `with_attribute(..)` / `write_inner_content(..)` API and
`new_with_indent(..)` for pretty-printing (the TS code used
`prettyPrint: true`).
