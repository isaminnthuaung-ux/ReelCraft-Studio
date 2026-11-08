# Big Sur UI reference

The UI uses the supplied **Big Sur UI Kit.sketch** as a visual reference. React,
Tauri, and the Rust processing engine remain the application stack.

Measurements read directly from the Sketch document:

| Kit style or symbol | Source value | Application |
| --- | --- | --- |
| Large Button / Primary | #017AFF, 6 px radius, 13 px SF Pro Text Medium | Primary buttons and focus accents |
| Large Button / Secondary | White fill, 6 px radius | Toolbar buttons and secondary actions |
| Switches / On | 37 × 22 px, 20 px white thumb | Smart Cut and text switches |
| Segmented Control / Selected | White fill, 4 px radius, 12 px medium text | Inspector and text-layer tabs |
| Descriptor / Left / Gray / 12 | #797979, 12 px regular | Help text and secondary labels |
| Title / Left / Black / 13 | #232526, 13 px bold | Panel headings |
| Blur / Background #01 | #F5F5F5 at 80% opacity | Sidebar material |
| Shadow #04 | y .5 px, blur 1 px, spread .5 px, black 10% | Controls |
| Shadow #03 | y 2 px, blur 10 px, black 10% | Popovers |

Controls retain larger hit areas than the kit's 20–28 px visual samples for
desktop usability. The encoding selector stays centered in the export bar.
Outline icons use the existing Lucide dependency and inherit control colors;
they substitute for the kit's icon appearance rather than extracting Apple symbols.
SF Pro is requested only when installed; Segoe UI and Myanmar Text remain font
fallbacks. No font files, wallpapers, or binary design files are redistributed.
Native operating-system window controls are retained.
