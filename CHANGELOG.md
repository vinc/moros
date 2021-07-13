# Changelog

## Unreleased

- Add syscalls ([#196](https://github.com/vinc/moros/pull/196))
- Add VGA palette loader ([#203](https://github.com/vinc/moros/pull/203))
- Add VGA font loader ([#201](https://github.com/vinc/moros/pull/201))

## 0.5.1 (2021-06-27)

- Update aml crate ([#195](https://github.com/vinc/moros/pull/195))
- Update smoltcp crate ([#194](https://github.com/vinc/moros/pull/194))
- Fix compilation errors ([#184](https://github.com/vinc/moros/pull/184))
- Add missing RX stats to PCNET driver ([#124](https://github.com/vinc/moros/pull/124))
- Fix PCNET BCNT computation ([#122](https://github.com/vinc/moros/pull/122))
- Disable `rand_chacha` with `debug_assertions` ([#120](https://github.com/vinc/moros/pull/120))
- Migrate from TravisCI to GitHub Actions ([#131](https://github.com/vinc/moros/pull/131))
- Bump acpi from 2.2.0 to 2.3.1 ([#180](https://github.com/vinc/moros/pull/180))
- Bump array-macro from 1.0.5 to 2.1.0 ([#188](https://github.com/vinc/moros/pull/188))
- Bump rand from 0.8.3 to 0.8.4 ([#176](https://github.com/vinc/moros/pull/176))
- Bump rand_core from 0.6.1 to 0.6.3 ([#185](https://github.com/vinc/moros/pull/185))
- Bump raw-cpuid from 8.1.2 to 9.0.0 ([#191](https://github.com/vinc/moros/pull/191))
- Bump spin from 0.7.1 to 0.9.1 ([#181](https://github.com/vinc/moros/pull/181))
- Bump time from 0.2.25 to 0.2.27 ([#186](https://github.com/vinc/moros/pull/186))
- Bump vte from 0.10.0 to 0.10.1 ([#174](https://github.com/vinc/moros/pull/174))

## 0.5.0 (2020-11-15)
- Add a web server ([#114](https://github.com/vinc/moros/pull/114))
- Add tests ([#118](https://github.com/vinc/moros/pull/118))
- Use ChaCha20 RNG ([#116](https://github.com/vinc/moros/pull/116))
- Remove dependency on `rlibc` ([#115](https://github.com/vinc/moros/pull/115))
- Add new `mem` command ([#113](https://github.com/vinc/moros/pull/113))
- Add new `date` and `env` commands ([#112](https://github.com/vinc/moros/pull/112))
- Add ACPI shutdown ([#111](https://github.com/vinc/moros/pull/111))
- Improve text editor ([#109](https://github.com/vinc/moros/pull/109))
- Add pcnet driver ([#82](https://github.com/vinc/moros/pull/82))
- Remove cargo xbuild ([#83](https://github.com/vinc/moros/pull/83))
- Add nanowait busy loop with nanoseconds precision ([#78](https://github.com/vinc/moros/pull/78))

## 0.4.0 (2020-07-29)
- Improve editor ([#77](https://github.com/vinc/moros/pull/77))
- Add ANSI Style type ([#76](https://github.com/vinc/moros/pull/76))
- Fix detection of magic superblock
- Fix ATA busy loop hang
- Improve ATA reset
- Optimize shell printing ([#75](https://github.com/vinc/moros/pull/75))
- Improve console ([#74](https://github.com/vinc/moros/pull/74))
- Improve installation and documentation ([#73](https://github.com/vinc/moros/pull/73))
- Handle RTC interrupts ([#71](https://github.com/vinc/moros/pull/71))
- Colorize user interface ([#69](https://github.com/vinc/moros/pull/69))
- Update dependencies ([#70](https://github.com/vinc/moros/pull/70))

## 0.3.1 (2020-04-13)
- Update ATA driver ([#41](https://github.com/vinc/moros/pull/41))
- Update dependencies ([#42](https://github.com/vinc/moros/pull/42))

## 0.3.0 (2020-02-16)
- Add process struct ([#19](https://github.com/vinc/moros/pull/19))
- Add PhysBuf for DMA ([#16](https://github.com/vinc/moros/pull/16))
- Add tcp command ([#17](https://github.com/vinc/moros/pull/17))
- Add geotime command ([#14](https://github.com/vinc/moros/pull/14))
- Use VGA color palette ([#15](https://github.com/vinc/moros/pull/15))
- Improve filesystem ([#24](https://github.com/vinc/moros/pull/24))
- Improve shell history ([#18](https://github.com/vinc/moros/pull/18))

## 0.2.0 (2020-02-02)
- Add network stack
- Add heap allocation
- Add autocompletion to shell

## 0.1.0 (2020-01-18)
- Add shell
- Add editor
- Add filesystem
- Add ATA PIO mode
- Add PCI enumeration
- Add RTC clock

## 0.0.0 (2019-12-28)
- Start MOROS project
