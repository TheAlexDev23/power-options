---
layout: default
title: Defaults
nav_order: 2
---

# Default settings

By default, power-options will generate 5 profiles: Powersave++, Powersave, Balanced, Performance and Performance++.

# CPUFreq options

## `intel_psate` and `amd_pstate` CPUFreq drivers

|  	| Powersave++ 	| Powersave 	| Balanced 	| Performance 	| Performance++ 	|
|---	|---	|---	|---	|---	|---	|
| Operation Mode 	| active 	| active 	| active 	| active 	| active 	|
| Governor 	| powersave 	| powersave 	| powersave 	| powersave 	| performance 	|
| EPP/EPB (if supported) 	| power 	| balance_power 	| default 	| balance_performance 	| performance 	|
| Min/Max frequencies 	| None 	| None 	| None 	| None 	| None 	|
| Min/Max perf% (if supported) 	| 0-70 	| 0-100 	| 0-100 	| 0-100 	| 30-100 	|
| Boost (if supported) 	| off 	| off 	| on 	| on 	| on 	|
| HWP boost (if supported) 	| off 	| off 	| off 	| on 	| on 	|

## `acpi_cpufreq` and other CPUFreq drivers

|  	| Powersave++ 	| Powersave 	| Balanced 	| Performance 	| Performance++ 	|
|---	|---	|---	|---	|---	|---	|
| Governor 	| power 	| conservative 	| ondemand 	| performance 	| performance 	|
| EPP/EPB (if supported) 	| power 	| balance_power 	| default 	| balance_performance 	| performance 	|
| Min/Max frequencies 	| None 	| None 	| None 	| None 	| None 	|

# Radio options

| | Powersave++ | Powersave | Balanced | Performance | Performance++ |
|---	|---	|---	|---	|---	|---	|
| Block WiFi | off | off | off | off | off |
| Block Bluetooth | on | on | on | off | off |
| Block NFC | on | on | on | off | off |

# Network options

| | Powersave++ | Powersave | Balanced | Performance | Performance++ |
|---	|---	|---	|---	|---	|---	|
| Disable ethernet | true | true | off | off | off |
| Disable WiFi 7 | on | on | off | off | off |
| Disable WiFi 6 | on | off | off | off | off |
| Disable WiFi 5 | off | off | off | off | off |
| iwlwifi power_save | on | on | on | off | off |
| iwlwifi power_level | 0 | 1 | 3 | 5 | 5 |
| iwlwifi U-APSD | on | off | off | off | off |
| iwlmvm power_scheme | 3 | 3 | 2 | 1 | 1 |
| iwldvm force_cam | off | off | on | on | on |

# PCIe Active State Power Management

*Profile: mode*
- Powersave++: powersupersave
- Powersave: powersave
- Default: default
- Performance: performance
- Performance++: performance

# PCI Runtime Power Management

- Powersave++, Powersave, Balanced: **on**
- Performance, Performance++: **off**

# USB Runtime Power Management

- Powersave++, Powersave, Balanced: **on**
- Performance, Performance++: **off**

# SATA Active Link Power Management Policy

- Powersave++, Powersave, Balanced: **med_power_with_dipm**
- Performance, Performance++: **max_performance**

# Kernel Settings

|  	| Powersave++ 	| Powersave 	| Balanced 	| Performance 	| Performance++ 	|
|---	|---	|---	|---	|---	|---	|
| Disable NMI watchdog 	| true 	| true 	| true 	| true 	| true 	|
| VM Writeback 	| 60 	| 45 	| 30 	| 15 	| 15 	|
| Laptop Mode 	| 5 	| 5 	| 5 	| 5 	| 2 	|

# Sleep/timeout Settings

|  	| Powersave++ 	| Powersave 	| Balanced 	| Performance 	| Performance++ 	|
|---	|---	|---	|---	|---	|---	|
| Turn off screen 	| 10 minutes 	| 15 minutes 	| 20 minutes 	| 30 minutes 	| 45 minutes 	|
| Suspend 	| 15 minutes 	| 20 minutes 	| 30 minutes 	| 45 minutes 	| 60 minutes 	|

