# Changelog

## Unreleased
- Add calc command #263
- Add website (#261)
- Fix VGA issues with real hardware (#258)
- Add rust binaries support (#255)
- Add dynamical disk information (#252)
- Add spawn syscall (#251)
- Add ELF loader (#248)
- Add basic userspace (#228)
- Bump acpi from 3.1.0 to 4.0.0 (#243)
- Bump sha2 from 0.9.6 to 0.9.8 (#244)
- Bump x86_64 from 0.14.4 to 0.14.5 (#245)
- Add file syscalls (#242)
- Bump aml from 0.15.0 to 0.16.0 (#241)
- Bump pic8259 from 0.10.1 to 0.10.2 (#235)
- Bump aml from 0.14.0 to 0.15.0 (#236)
- Bump pbkdf2 from 0.8.0 to 0.9.0 (#239)
- Bump sha2 from 0.9.5 to 0.9.6 (#240)

## 0.6.0 (2021-08-21)
- Add beep command ([#234](https://github.com/vinc/moros/pull/234))
- Add Lisp interpreter ([#207](https://github.com/vinc/moros/pull/207))
- Add VGA font loader ([#201](https://github.com/vinc/moros/pull/201))
- Add VGA palette loader ([#203](https://github.com/vinc/moros/pull/203))
- Add chess game ([#230](https://github.com/vinc/moros/pull/230))
- Add file offset ([#206](https://github.com/vinc/moros/pull/206))
- Add keyboard layout change at runtime ([#226](https://github.com/vinc/moros/pull/226))
- Add regular expression engine ([#222](https://github.com/vinc/moros/pull/222))
- Add syscalls ([#196](https://github.com/vinc/moros/pull/196))
- Add time to dir entry ([#215](https://github.com/vinc/moros/pull/215))
- Improve baremetal experience ([#232](https://github.com/vinc/moros/pull/232))
- Fix clippy warnings ([#214](https://github.com/vinc/moros/pull/2154))
- Move kernel code to api ([#204](https://github.com/vinc/moros/pull/204))
- Refactor editor ([#221](https://github.com/vinc/moros/pull/221))
- Refactor filesystem ([#225](https://github.com/vinc/moros/pull/225))
- Refactor line editing ([#212](https://github.com/vinc/moros/pull/212))
- Refactor print macros ([#208](https://github.com/vinc/moros/pull/208))
- Remove volatile crate ([#219](https://github.com/vinc/moros/pull/219))
- Update acpi crate from v2.3.1 to v3.1.0 ([#218](https://github.com/vinc/moros/pull/218))
- Update crypto crates ([#216](https://github.com/vinc/moros/pull/216))
- Update raw-cpuid from v9.0.0 to v10.0.0 ([#220](https://github.com/vinc/moros/pull/220))
- Use CSI for key events ([#210](https://github.com/vinc/moros/pull/210))
- Bump aml from 0.13.0 to 0.14.0 ([#227](https://github.com/vinc/moros/pull/227))
- Bump bootloader from 0.9.18 to 0.9.19 ([#233](https://github.com/vinc/moros/pull/233))
- Bump raw-cpuid from 10.0.0 to 10.2.0 ([#224](https://github.com/vinc/moros/pull/224))
- Bump spin from 0.9.1 to 0.9.2 ([#202](https://github.com/vinc/moros/pull/202))
- Bump x86_64 from 0.14.3 to 0.14.4 ([#209](https://github.com/vinc/moros/pull/209))

## 0.5.1 (2021-06-27)
- Add missing RX stats to PCNET driver ([#124](https://github.com/vinc/moros/pull/124))
- Disable `rand_chacha` with `debug_assertions` ([#120](https://github.com/vinc/moros/pull/120))
- Fix PCNET BCNT computation ([#122](https://github.com/vinc/moros/pull/122))
- Fix compilation errors ([#184](https://github.com/vinc/moros/pull/184))
- Migrate from TravisCI to GitHub Actions ([#131](https://github.com/vinc/moros/pull/131))
- Update aml crate ([#195](https://github.com/vinc/moros/pull/195))
- Update smoltcp crate ([#194](https://github.com/vinc/moros/pull/194))
- Bump acpi from 2.2.0 to 2.3.1 ([#180](https://github.com/vinc/moros/pull/180))
- Bump array-macro from 1.0.5 to 2.1.0 ([#188](https://github.com/vinc/moros/pull/188))
- Bump rand from 0.8.3 to 0.8.4 ([#176](https://github.com/vinc/moros/pull/176))
- Bump rand_core from 0.6.1 to 0.6.3 ([#185](https://github.com/vinc/moros/pull/185))
- Bump raw-cpuid from 8.1.2 to 9.0.0 ([#191](https://github.com/vinc/moros/pull/191))
- Bump spin from 0.7.1 to 0.9.1 ([#181](https://github.com/vinc/moros/pull/181))
- Bump time from 0.2.25 to 0.2.27 ([#186](https://github.com/vinc/moros/pull/186))
- Bump vte from 0.10.0 to 0.10.1 ([#174](https://github.com/vinc/moros/pull/174))

## 0.5.0 (2020-11-15)
- Add ACPI shutdown ([#111](https://github.com/vinc/moros/pull/111))
- Add a web server ([#114](https://github.com/vinc/moros/pull/114))
- Add nanowait busy loop with nanoseconds precision ([#78](https://github.com/vinc/moros/pull/78))
- Add new `date` and `env` commands ([#112](https://github.com/vinc/moros/pull/112))
- Add new `mem` command ([#113](https://github.com/vinc/moros/pull/113))
- Add pcnet driver ([#82](https://github.com/vinc/moros/pull/82))
- Add tests ([#118](https://github.com/vinc/moros/pull/118))
- Improve text editor ([#109](https://github.com/vinc/moros/pull/109))
- Remove cargo xbuild ([#83](https://github.com/vinc/moros/pull/83))
- Remove dependency on `rlibc` ([#115](https://github.com/vinc/moros/pull/115))
- Use ChaCha20 RNG ([#116](https://github.com/vinc/moros/pull/116))

## 0.4.0 (2020-07-29)
- Add ANSI Style type ([#76](https://github.com/vinc/moros/pull/76))
- Colorize user interface ([#69](https://github.com/vinc/moros/pull/69))
- Fix ATA busy loop hang
- Fix detection of magic superblock
- Handle RTC interrupts ([#71](https://github.com/vinc/moros/pull/71))
- Improve ATA reset
- Improve console ([#74](https://github.com/vinc/moros/pull/74))
- Improve editor ([#77](https://github.com/vinc/moros/pull/77))
- Improve installation and documentation ([#73](https://github.com/vinc/moros/pull/73))
- Optimize shell printing ([#75](https://github.com/vinc/moros/pull/75))
- Update dependencies ([#70](https://github.com/vinc/moros/pull/70))

## 0.3.1 (2020-04-13)
- Update ATA driver ([#41](https://github.com/vinc/moros/pull/41))
- Update dependencies ([#42](https://github.com/vinc/moros/pull/42))

## 0.3.0 (2020-02-16)
- Add PhysBuf for DMA ([#16](https://github.com/vinc/moros/pull/16))
- Add geotime command ([#14](https://github.com/vinc/moros/pull/14))
- Add process struct ([#19](https://github.com/vinc/moros/pull/19))
- Add tcp command ([#17](https://github.com/vinc/moros/pull/17))
- Improve filesystem ([#24](https://github.com/vinc/moros/pull/24))
- Improve shell history ([#18](https://github.com/vinc/moros/pull/18))
- Use VGA color palette ([#15](https://github.com/vinc/moros/pull/15))

## 0.2.0 (2020-02-02)
- Add autocompletion to shell
- Add heap allocation
- Add network stack

## 0.1.0 (2020-01-18)
- Add ATA PIO mode
- Add PCI enumeration
- Add RTC clock
- Add editor
- Add filesystem
- Add shell

## 0.0.0 (2019-12-28)
- Start MOROS project
