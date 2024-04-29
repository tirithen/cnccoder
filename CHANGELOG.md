# Changelog

All notable changes to this project will be documented in this file. See [standard-version](https://github.com/conventional-changelog/standard-version) for commit guidelines.

## [0.2.0](///compare/v0.1.1...v0.2.0) (2024-04-29)


### ⚠ BREAKING CHANGES

* write_project no longer accepts the first name argument
as the name will now instead be read from the program struct. The name
can now be set by the Program::set_name method. Also setting z_safe and
z_tool_change values that are smaller than the max z value from the cuts
will now result in an error when the Program::to_instructions method
runs.

### Features

* add Context::operations() to get operations 0d0d77c
* add gcode meta data, imp. rel. tests e8ed51d
* add meta data to gcode files, check z_safe 873d471
* add MIN, MAX const, splat method on Vector2/3 de22f48
* **vector:** Add Vector3::ZERO/Vector2::ZERO const 56ab125
* wait for tool spin up after tool change 5d9002c


### Bug Fixes

* Context::merge() signature f061036

### [0.1.1](///compare/v0.1.0...v0.1.1) (2023-10-31)


### Features

* add Program::context method 613fabc

## 0.1.0 (2023-07-04)


### ⚠ BREAKING CHANGES

* Rename Plane to Area
* Circle::hole was renamed to Circle::drill. Adds
mandatory ToolPathCompensation to Cut::circle (Circle struct) that is
used when calculating cut radius. All to_instructions and
Program::to_gcode now returns Result wrapped values for error handling.
* When using program.extend the action method passed in
now must return a Result.
* Prefer &str over String for function arguments. Use
program units for move instructions.

### Features

* add camotics project file generator 923c1c3
* add circle tool compensation, return Result 2036d22
* add line segments path fc6acdf
* add plane and circle cuts 0f11957
* add tool context handling 571df29
* add wasm setup 091f04e
* adds tool comp., unit conv., arc improv. dce2af0
* **frame:** adds frame cut 8c5e831
* improve public api, document crate 9705fe8
* **path:** add point segments 91a2fa1
* **planing:** adds planing program d8eb646
* progr meta, fix tool ordering 2009cd6
* **program:** add merge method e33269c
* **program:** add Program::new_empty_from method ea0f97c
* **program:** end at tool change height 262814b
* **program:** expose z_safe/tool_change getters 89cd73b
* return result from program extensions c322a95
* set G21/G20 units after each tool change f3bdac2
* **to_gcode:** add gcode serialization 417dfc8
* **to_instructions:** add gcode instructions conv 292e171
* **tool:** implement Default trait 66cc423
* **units:** add measurement conversions 40dece0
* **vector:** add with_x/y/z to change one axis e64c67d
* **vector:** impl glam and nalgebra conversions 4f8043a
* write project to disk, calculate bounds 3ee2e1d


### Bug Fixes

* add ; suffix to comments f434444
* **cuts:** fixes mixup of circle arguments bf45ef1
* tool ordering when serializing 6807a89


* prefer &str over String in signatures 792c3db
