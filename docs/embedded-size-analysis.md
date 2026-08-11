# Hotaru embedded target-size analysis

This analysis estimates Hotaru's target-side flash and RAM cost on devices
with at most 4 MiB of flash, including ESP8266-class hardware.

The name “Rusat” is ambiguous. Because it may mean **Rust**, this report
separately measures the Rust/no_std baseline. It also includes a conditional
assessment of the public embedded `hotaru_mqtt` client branch in case that
protocol work was intended. A different Rusat crate needs its repository or
path before it can be measured exactly.

The important rule is that Cargo package, source-tree, and `rlib` sizes are not
firmware overhead. Only code and data retained by the final linker count. The
useful number is therefore the delta between otherwise identical linked
firmware, with and without the component under test.

## Executive summary

The current checkout's crate manifests report Hotaru `0.8.4`. Linked release
probes using the normal public API produced these target-side deltas:

| Target/profile | Linked flash delta | Static RAM delta |
| --- | ---: | ---: |
| Cortex-M4, one `APP.lit_url(...)` endpoint, `opt-level = "z"` | **30.0 KiB** | **1 byte**, excluding heap |
| Cortex-M4, one `APP.lit_url(...)` endpoint, `opt-level = "s"` | **38.1 KiB** | **1 byte**, excluding heap |
| RV32IMC no-atomic, one literal `endpoint!` | **34.4 KiB** | **1 byte**, excluding heap |
| Cortex-M4, `lite_regex` + one typed endpoint | **239.4 KiB** | **1 byte**, excluding heap |
| Cortex-M4, `full_regex` + one typed endpoint | **539.2 KiB** | **1 byte**, excluding heap |

The recommended planning figures are:

- Hotaru plus one normal public endpoint: **about 30 KiB** with
  `opt-level = "z"` or **38 KiB** with `opt-level = "s"`.
- Eight literal macro endpoints: **about 32 KiB flash** on Cortex-M4.
- Sixteen literal macro endpoints: **about 34 KiB flash**.
- Thirty-two literal macro endpoints: **about 39 KiB flash**.
- Hotaru plus the bounded HCR no_std HTTP protocol with a reachable connection
  path: **about 40 KiB with `opt-level = "z"` or 48 KiB with
  `opt-level = "s"`**, before the real runtime, board I/O, and SDK.
- With the HCR firmware's current `opt-level = "s"` profile, reserve roughly
  **50–70 KiB flash** for Hotaru + bounded HTTP + runtime/I/O integration,
  excluding the ESP8266 SDK, application logic, static frontend assets,
  allocator, bootloader, and filesystem/OTA partitions.

The no-regex one-endpoint result is under 1% of a raw 4 MiB flash chip. Even
the full Unicode regex profile is about 13.2% of 4 MiB, but raw chip capacity is
the wrong denominator for the Arduino ESP8266 build. The Arduino core documents
a **1 MiB maximum main-sketch space** and divides the rest among OTA,
filesystem, EEPROM, and SDK areas. Against a 1 MiB sketch ceiling, these probes
are approximately:

- public literal endpoint: **2.9%**;
- `lite_regex` typed endpoint: **23.4%**;
- `full_regex` typed endpoint: **52.7%**.

Those percentages are scale comparisons only. The regex binaries were built
for Cortex-M and do not prove that the same dependency profile builds for
Xtensa ESP8266.

The ESP8266EX has 160 KiB of physical RAM, but the Espressif datasheet reports
only about **50 KiB for programmable Heap + Data** when working in Station mode
and connected to a router. The HCR firmware uses AP+STA, so its actual free
heap must be measured on hardware and may be lower. Data RAM, IRAM placement,
allocation peaks, network buffers, and SDK reservations are therefore more
likely to be the first limit.

If “Rusat” meant the embedded MQTT client, its exact number is not yet
measured. The public `hotaru_mqtt` embedded branch found during this work is a
large, general MQTT 3.1.1/5.0 client with QoS 0/1/2, properties, dynamic
queues, and session maps. It does not currently link against this checkout
without fetching missing branch dependencies and adapting its older Hotaru
API. Until a rebased client is linked, **do not quote a numeric MQTT flash or
base-heap overhead**; the result is too dependent on retained protocol
features and configured capacities.

- The packet limit is a configured maximum, not preallocated RAM.
- Queue, inflight, QoS-2, alias, topic, and payload ownership determine the
  live-session heap.
- The branch's desktop-oriented default capacities are not suitable for
  ESP8266 and must be reduced before deployment.

## What contributes to overhead

### Flash or ROM

1. `hotaru_core`
   - application builder and protocol registry;
   - protocol type erasure and dispatch;
   - URL tree, parser, registration, and walker;
   - handler and middleware execution chains;
   - runtime and transport abstraction glue;
   - retained error, panic, and formatting paths.
2. Optional route matching
   - no regex dependency;
   - `lite_regex` without Unicode tables;
   - `full_regex` with Unicode tables.
3. Facade and DSL
   - `hotaru` re-exports normally add no code by themselves;
   - `hotaru_trans` runs as a host-side procedural macro;
   - its generated endpoint constructor and definition/bind code do ship.
4. Runtime
   - Embassy executor integration and Hotaru job queue;
   - worker tasks, timer/timeout/select support, mutexes, once-cells, and join
     signals.
5. I/O and transport
   - `embedded-io-async` or board-specific TCP adapters;
   - buffering and partial read/write loops;
   - ESP8266 platform and FFI shims.
6. Protocol
   - HTTP or MQTT parser and encoder;
   - channel, context, and session state;
   - TLS, compression, authentication, broker, and retained-message code if
     selected.
7. Application and platform
   - handlers, route/topic strings, JSON/form logic, assets, certificates;
   - Rust `core`/`alloc`, panic handler, allocator, HAL/Arduino core, Wi-Fi/TCP
     stack, boot metadata, linker padding, and architecture helpers.

### Static and dynamic RAM

1. `.data` and `.bss` globals.
2. Allocator bookkeeping and configured heap.
3. Hotaru registry, route nodes, strings, maps, and handler references.
4. Runtime queue, worker futures, timer queue, and executor task storage.
5. Per-connection read/write buffers and protocol contexts.
6. MQTT writer queue, ack waiters, session maps, topic aliases,
   subscriptions, payloads, and inflight messages.
7. Networking stack buffers and vendor SDK reservations.
8. Task/interrupt stacks and temporary parsing/encoding allocation peaks.

## Measurement method

The headline Hotaru and regex probes use this size-minimizing release profile:

```toml
[profile.release]
codegen-units = 1
lto = true
opt-level = "z"
panic = "abort"
strip = "symbols"
```

They were built from Hotaru revision `b06aadf` with Rust `1.88.0`.
They are `no_std + alloc` and use a fixed 64 KiB bump heap so real allocation
paths remain link-reachable. The same allocator is present in the baseline and
Hotaru images, so its `.bss` cancels out of the delta.

The HCR HTTP measurements additionally use the sibling
`hcr_hardware/hcr-http` source snapshot present on August 11, 2026. That
sibling workspace has no Git revision recorded here, so preserve or version
the probe inputs before treating these figures as a release-regression gate.

The primary target is `thumbv7em-none-eabihf`, a stable 32-bit bare-metal
target that can exercise every regex profile. The no-atomic comparison uses
`riscv32imc-unknown-none-elf` only to exercise Hotaru's
`spawn_local_no_atomic` feature mode on another stable 32-bit target. It is
not an Xtensa or ESP8266 code-generation proxy.

The bounded HCR HTTP path was also rebuilt with `opt-level = "s"`, matching
the sibling ESP8266 firmware workspace's release profile.

Flash is the sum of loadable code/read-only-data/unwind sections plus the load
image of initialized `.data`; static RAM is runtime `.data + .bss`. The custom
probe link scripts place `.text`, `.rodata`, and generated unwind-index output
in flash and `.data`/`.bss` in RAM. The Cortex-M baseline and component images
both contain the same 16-byte `.ARM.exidx`, so it cancels in every delta below.

These are framework probes, not complete board images: no HAL, Wi-Fi stack,
bootloader, filesystem, or final ESP8266 Arduino link is included.

## Measured results

### Rust/no_std baseline

Rust does not impose one fixed firmware-size tax. With `no_std`, LTO, and
section garbage collection, only used monomorphized code is retained.

The linked probes showed:

| Rust-only Cortex-M4 probe | Flash above empty entry point |
| --- | ---: |
| Panic handler + fixed bump allocator path | **about 42 B** |
| `Box` + `Vec` + `String` + `Rc` + `BTreeMap` operations | **about 4.9 KiB** |

The corresponding collection probe on RV32IMC added about **4.5 KiB**. These
figures exclude the configured heap capacity: the probe's 64 KiB heap is a
chosen `.bss` reservation, not mandatory Rust overhead. A real ESP8266
allocator can instead use the board/Arduino heap through the platform bridge.

All Hotaru deltas below subtract the otherwise identical Rust allocator
baseline. They therefore describe Hotaru/framework overhead on top of Rust,
not the total firmware size.

### Public endpoint path

Cortex-M4 linked sections:

| Probe | `.text` | `.rodata` | Flash delta |
| --- | ---: | ---: | ---: |
| Allocator-only baseline | 48 B | 0 B | — |
| One protocol, no route | 15,668 B | 100 B | **15,720 B** |
| One low-level literal route | 20,720 B | 260 B | **20,932 B** |
| One public `APP.lit_url("/health")` endpoint + manual `bind` | 30,228 B | 528 B | **30,708 B** |

The public result includes the server builder, protocol registry entry,
macro-generated endpoint constructor, `App::bind`, definition preparation, URL
tree creation, final handler, and route registration.

A direct low-level route built through `hotaru_core` and the same low-level
route built through the `hotaru` facade linked identically. Re-exports alone do
not add target code. The normal public definition/bind funnel retains about
9.5 KiB more code than the low-level probe; that delta includes macro-generated
constructor, middleware/config builder, and canonical bind machinery rather
than being a pure routing-backend comparison.

The RV32IMC no-atomic macro-endpoint probe linked at 35,264 B after subtracting
its 52 B baseline. Architecture code generation explains the difference, so
30–35 KiB is the safer one-endpoint cross-target planning range.

A separate low-level Cortex-M probe retaining the Embassy adapter's
initialization and spawn paths added approximately 176 B of flash and 100 B of
static RAM over its matching low-level route baseline. This is only the Hotaru
adapter/storage. Board executor task storage, timer driver/queue,
critical-section bridge, and application futures still belong in the full
firmware measurement; do not add 176 B directly to the public-endpoint table.

### Route scaling

These use `trans`-style `endpoint!` constructors, explicit manual binding, and
three-segment literal paths sharing the same endpoint body:

| Literal endpoints | Cortex-M4 flash delta |
| ---: | ---: |
| 1 | 30,716 B |
| 4 | 31,500 B |
| 8 | 32,688 B |
| 11 | 33,608 B |
| 16 | 35,104 B |
| 32 | 39,840 B |

After the first endpoint pulls in registration machinery, each additional
similar macro endpoint costs roughly 250–320 B of flash in this probe. Runtime
heap growth is separate: each route constructs strings, nodes, map entries, and
handler references during registration.

### Regex profiles

The typed probes register `/sensor/<uint:id>` through the public macro/bind
path so the parser, regex compiler, and matcher remain linked:

| Cortex-M4 profile | `.text` | `.rodata` | Flash delta |
| --- | ---: | ---: | ---: |
| No regex, public `APP.lit_url(...)` endpoint | 30,228 B | 528 B | **30,708 B** |
| `lite_regex`, public typed endpoint | 230,436 B | 14,788 B | **245,176 B** |
| `full_regex`, public typed endpoint | 234,012 B | 318,172 B | **552,136 B** |

Relative to the no-regex endpoint:

- `lite_regex` adds about **209 KiB**.
- `full_regex` adds about **509 KiB**.
- Most of the full-profile increase is Unicode-table `.rodata`.

These regex figures are **Cortex-M measurements, not proof that the dependency
builds on ESP8266**. A stable no-pointer-atomic target failed while compiling
`regex-automata` because that dependency uses `alloc::sync::Arc` and atomic
compare/exchange paths. The actual Xtensa toolchain was unavailable, so treat
both Hotaru regex profiles as unsupported on ESP8266 until a final
`xtensa-esp8266-none-elf` build proves otherwise.

For ESP8266-class firmware, use literal routes and `Any`/`AnyPath`. Even on
targets where it compiles, enable `lite_regex` only if typed/regex segments are
required. Avoid `full_regex` unless Unicode regex semantics are genuinely
needed and the final partition budget has been checked.

### Bounded embedded HTTP probe

The existing sibling HCR hardware workspace contains a no_std HTTP protocol
with bounded request/response sizes. A one-endpoint Cortex-M4 probe with a
synthetic valid request, Hotaru dispatch, HCR parsing/encoding, endpoint
execution, and response writing reachable linked at approximately:

- **39.6 KiB** above the matching allocator baseline with `opt-level = "z"`;
- **48.0 KiB** above the matching allocator baseline with `opt-level = "s"`.

Compared with the corresponding public Hotaru endpoint, the bounded HTTP
protocol path adds roughly **9.6–9.9 KiB** in these probes.

This is useful evidence that a compact embedded HTTP protocol does not require
hundreds of KiB. It is still not the final ESP8266 result because its test wire
and single-poll probe runtime do not retain the board-specific TCP adapter, the
full Embassy executor/timer implementation, the Arduino SDK, or networking
buffers.

## RAM interpretation

The linked framework probes differ from the allocator-only baseline by one
byte of `.bss`; most Hotaru state is allocated dynamically during server
construction and route registration. This does **not** mean Hotaru consumes one
byte of RAM.

A realistic measurement must calculate:

```text
peak Hotaru heap =
    minimum free heap before Hotaru initialization
  - minimum free heap after server construction, route registration,
    and representative traffic
```

Record at least:

1. free heap before server construction;
2. free heap after protocol registry construction;
3. free heap after all routes are registered;
4. minimum free heap with one connection;
5. minimum free heap at maximum configured concurrency;
6. minimum free heap during maximum request/response or MQTT payload;
7. executor/task storage and stack high-water marks.

The HCR ESP8266 HTTP consumer currently chooses:

- 1,024 B buffered read half;
- request head up to 1,024 B;
- request body up to 512 B;
- response buffer up to 2,048 B;
- three Hotaru/Embassy workers;
- job queue capacity eight.

Some buffers coexist, so budget several KiB per active request before
application state, Wi-Fi SDK memory, and heap fragmentation.

## Conditional MQTT client assessment

This section applies only if “Rusat” meant the current embedded MQTT client
work. It is not part of the measured Hotaru core result above.

The public `hotaru_mqtt` embedded branch inspected here was revision
`5f3ec5f` from August 1, 2026. It is split so target firmware only needs the
MQTT client/core crate. The broker is a separate std-only crate and must not be
linked into sensor firmware.

The target client/core contains:

- MQTT 3.1.1 and MQTT 5 packet codec;
- CONNECT, PUBLISH, SUBSCRIBE, UNSUBSCRIBE, ping, and acknowledgment flows;
- QoS 0/1/2 state and packet-ID tracking;
- MQTT 5 properties and topic aliases;
- dynamic `Bytes`, `Vec`, `Arc<str>`, maps, oneshot channels, event listeners,
  and a bounded async writer queue;
- Hotaru protocol/channel/context integration and runtime-neutral sync.

Its source is approximately 6,260 Rust lines / 238 KiB. Source size is not
binary size, but the feature set explains why it should be materially larger
than the compact HTTP implementation.

### Current limitation

The no_std branch is based on an older Hotaru API and needs dependencies that
are absent from this checkout's offline cache. It therefore did not produce a
trustworthy final linked binary in this analysis. Any numeric overhead must
come from a linked delta after that branch is rebased.

### Resource model and required limits

No numeric MQTT flash or base-heap range is reported until the no_std branch is
rebased and linked. The live RAM model is still clear: it grows with queued
commands and payload ownership, inflight QoS operations, inbound QoS-2 stashes,
topic aliases, subscriptions, and active codec buffers. The configured maximum
packet size is a rejection threshold; it is not allocated in advance.

The branch's current defaults include:

- maximum packet size: 1 MiB;
- writer queue: 1,000 messages;
- inbound QoS-2 receive maximum: 64;
- topic alias maximum: 16;
- outbound inflight maximum: 20.

Those are unsuitable for ESP8266. As initial engineering limits to validate,
not guaranteed final values, start approximately with:

- maximum MQTT packet: **1–4 KiB**, based on the real payload;
- writer queue: **4–8**;
- inflight QoS operations: **2–4**;
- inbound QoS-2 maximum: **1–4**, or omit QoS 2 if unnecessary;
- topic aliases: **0–4**;
- a fixed small subscription set.

Test every limit using worst-case owned topic and payload sizes, not empty
packets.

## ESP8266-specific status

The current CI has an ESP8266 Xtensa compile probe, but explicitly does not
claim HAL, timer driver, Wi-Fi, linker, flashing, or hardware coverage. A
sibling HCR consumer has a complete Arduino-link script, but this host currently
lacks:

- the custom `esp` Rust toolchain;
- `arduino-cli`;
- ESP8266 Arduino core 3.1.2 and its `xtensa-lx106-elf-*` tools.

No exact ESP8266 `.elf`/`.bin`, IRAM, or DRAM total was therefore generated.
Cortex-M/RISC-V results establish the retained feature cost and a rough
planning band, but architecture code generation can differ materially. They
are not a substitute for the final Xtensa link.

ESP8266 reporting must separate:

- flash code/data (`.irom0.text`, flash literals/rodata);
- IRAM code (`.iram.text` and linker catch-all);
- initialized DRAM;
- `.bss`;
- minimum free heap at runtime.

The HCR linker flow already relocates ordinary Rust code out of the 32 KiB IRAM
catch-all into flash. That post-processing is mandatory before deciding whether
Hotaru fits. Do not treat the physical SRAM total—or the linker's nominal DRAM
region—as usable application heap: the Espressif datasheet's Station-mode
figure is about 50 KiB for Heap + Data, before application-specific
high-water-mark analysis. Minimum-free-heap and peak-allocation measurements
are required in addition to flash size.

## Reproducible final-device procedure

Build three otherwise identical images with the same toolchain, board options,
SDK, allocator, stacks, assets, and release profile.

1. **Baseline:** board support, networking, allocator, and application skeleton
   without Hotaru/MQTT behavior.
2. **Hotaru:** `default-features = false`, `embedded`, the correct local spawn
   mode, no regex initially, selected runtime/transport/protocol, and all real
   routes.
3. **MQTT/Rusat:** rebased no_std client/core only, with constrained safety
   limits; never include the broker on the device.

For an ELF target:

```sh
llvm-size -A target/<target>/release/<firmware>
llvm-objdump -h target/<target>/release/<firmware>
```

For ESP8266, use the pinned Arduino core's matching
`xtensa-lx106-elf-size`/`objdump` and retain the linker map.

Calculate:

```text
flash overhead = flash(component image) - flash(baseline)
static RAM overhead = data+bss(component image) - data+bss(baseline)
peak heap overhead = baseline minimum free heap - component minimum free heap
```

Then register all routes/topics, open maximum concurrency, process maximum
packet sizes, fill queues/inflight tables to configured bounds, reconnect
repeatedly, and record heap and stack high-water marks.

## Recommendations

1. Use `default-features = false`, `embedded`, and the correct local spawn
   mode.
2. Start with **no regex** and literal/`Any`/`AnyPath` routes.
3. Do not use the std-only `hotaru_http`; use a bounded no_std protocol.
4. Keep `hotaru_trans` if desired: the macro runs on the build host. Disable
   auto-registration on bare metal and bind explicitly.
5. Treat RAM as the primary ESP8266 constraint. Bound every buffer, queue, map,
   payload, and connection count.
6. Compile only the MQTT client/core, never the broker.
7. Add an embedded MQTT feature profile that removes MQTT 5, QoS 2, large
   dynamic queues, or other unused general functionality.
8. Accept releases using the processed Xtensa ELF/map plus runtime heap/stack
   telemetry, not crate archive sizes.

## Hardware references

- [ESP8266 Arduino Core: configuration and the 1 MB main-sketch
  limit](https://arduino-esp8266.readthedocs.io/en/latest/ideoptions.html)
- [ESP8266 Arduino Core 3.1.2: flash/filesystem
  layout](https://arduino-esp8266.readthedocs.io/en/stable/filesystem.html)
- [Espressif ESP8266EX datasheet: available Heap + Data in Station
  mode](https://documentation.espressif.com/0a-esp8266ex_datasheet_en.html)
