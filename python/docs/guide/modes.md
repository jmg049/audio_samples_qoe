# Scoring modes

{func}`~audio_samples_qoe.visqol` supports two modes, selected with the
`mode` keyword argument (default `"audio"`).

---

## Audio mode

```python
score = visqol(ref, deg)               # default
score = visqol(ref, deg, mode="audio") # explicit
```

| Property | Value |
|---|---|
| Operating rate | 48 kHz (both signals resampled) |
| Gammatone bands | 32, up to Nyquist |
| MOS mapping | Support vector regression (libSVM) |
| Patch selection | All analysis windows |
| Identical score | ~4.73 |

Use audio mode for music, broadcast content, or any full-bandwidth
signal. The SVR model is calibrated on codec and bandwidth-limiting
degradations at 48 kHz.

---

## Speech mode

```python
score = visqol(ref, deg, mode="speech")
```

| Property | Value |
|---|---|
| Operating rate | Reference's native rate (degraded resampled to match) |
| Gammatone bands | 21, capped at 8 kHz |
| MOS mapping | Exponential NSIM → MOS fit |
| Patch selection | Voice-activity gated (silent frames excluded) |
| Identical score | 5.0 (exact) |

Use speech mode for narrowband or wideband telephony, VoIP, or voice
codec evaluation. Upstream recommends **16 kHz** input.

The exponential fit maps a perfect NSIM similarity of 1.0 to exactly 5.0,
so identical signals always score 5.0 regardless of content.

---

## Choosing a mode

| Scenario | Recommended mode |
|---|---|
| Music quality (streaming, codec, mastering) | `"audio"` |
| Speech codec / VoIP / ASR pre-processing | `"speech"` |
| Mixed content | `"audio"` (more conservative) |
| Narrowband telephony (≤8 kHz) | `"speech"` |

When in doubt, use `"audio"`. The SVR model generalises better across
content types; speech mode's VAD gating can produce surprising results on
non-speech signals with long silence.

---

## Conformance

Both modes are conformance-tested against Google's C++ reference. Maximum
deviation across the full test corpus:

| Mode | Max Δ MOS | Notes |
|---|---|---|
| Audio | 0.025 | FMA instruction scheduling |
| Speech (short clips) | 0.08 | Short-duration exponential fit sensitivity |
