# Quilt
A <em>Kirby's Epic Yarn</em> modding tool.

## Features
- [X] Level editor
- [X] Level graphics editor
- [X] GfArch utility

### Roadmap
- Collision editor
- MNEB rendering
- Save editor
- Message editor
- Potential <em>Kirby's Extra Epic Yarn</em> support

## Capabilities
### Level Editor
- Open and save levels
- Edit level collisions, gimmicks, enemies, and more
- Support for images for gimmicks
- Render backgrounds for ease of editing and alignment

### Level Graphics Editor
- Open `.bgst3` files
- Render a grid
- Export, replace, and remove tiles

### GfArch utility
- Extract and save `.gfa` files within Quilt

## Screenshots
### Level Editor
![le_preview_1](assets/screenshots/LE_SS_1.png "Level Editor view of Fountain Gardens")
![le_preview_2](assets/screenshots/LE_SS_2.png "Level Editor view of Patch Castle with **experimental** BGST preview.")

### Level Graphics Editor
![be_preview_1](assets/screenshots/BE_SS_1.png "BGST Editor view of Fountain Gardens")
![be_preview_2](assets/screenshots/BE_SS_2.png "BGST Editor view of Meta Melon Isle")


## Setup
### Game Files
1. Dump your copy of <em>Kirby's Epic Yarn</em>.
2. Extract the game's contents. You can use Dolphin Emulator to do this.

### Quilt Setup
1. Download the latest release of Quilt or compile from source.
2. Create a folder called `quilt_res` next to the Quilt app.
3. Download the latest gimmick images for Quilt from [the Quilt image collection](https://github.com/Swiftshine/key-quilt-image). Put the `tex` folder within the `quilt_res` folder. To update your images, redownload the repository.
4. Download the latest `objectdata.json` from [the object database](https://github.com/Swiftshine/key-objectdb). Put this in the `quilt_res` folder. Alternatively, you can open the level editor and go to `Object Data > Update` to download it from within Quilt.

In the end, your folder structure should look like this:
```
[folder Quilt is in]
    ├── quilt_res/
    │    ├── tex/
    │    │   └── [texture folders]
    │    └── objectdata.json
    └── [Quilt executable (Quilt.exe, etc.)]
```

Note that Quilt will autogenerate the `quilt_res` folder in order to download `objectdata.json`.
