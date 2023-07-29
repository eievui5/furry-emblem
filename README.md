# Furry Emblem

This is a repository of projects for the Furry Emblem engine, a game engine for creating tactical RPGs.

It consists of 3 main projects:
- fe-engine
	- The game engine, targetting the GBA.
- fe-editor
	- Editor for engine's data files.
- fe-data
	- Definitions for the engine's data file formats.
	- Allows formats to be shared between fe-editor and fe-engine's build script.

Ideally, fe-engine should not hard-code any features or make assumptions about the data formats.
Multiple projects and engines should be supported.
